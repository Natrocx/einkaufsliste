use std::array::TryFromSliceError;
use std::cell::Cell;
use std::ops::Deref;
use std::path::Path;
use std::rc::Rc;
use std::sync::Arc;

use einkaufsliste::model::list::List;
use einkaufsliste::model::requests::{LoginUserV1, RegisterUserV1};
use einkaufsliste::model::user::User;
use einkaufsliste::Encoding;
use reqwest::header::{HeaderValue, ACCEPT, CONTENT_TYPE};
#[cfg(not(target_arch = "wasm32"))]
use reqwest_cookie_store::CookieStoreMutex;
use rkyv::de::deserializers::SharedDeserializeMap;
use rkyv::validation::validators::{CheckDeserializeError, DefaultValidator};
use rkyv::CheckBytes;
use tracing::debug;

/*
 This file contains the API client and a reference counted Service for use in dioxus.

 It should not be used directly but rather through a cached service. (to be implemented)

 Maintainers notes:
   * remeber to call `.error_for_status()` on all responses to catch invalid requests and server errors
*/

// Avoid certificate errors on desktop
#[cfg(not(target_arch = "wasm32"))]
pub static DEVELOPMENT_CERTIFICATE: &[u8] = include_bytes!("./rootCA.pem");

#[derive(Clone)]
pub struct ApiService {
  inner: Rc<ApiClient>,
}

impl ApiService {
  pub fn new(base_url: String) -> Result<Self, APIError> {
    Ok(Self {
      inner: Rc::new(ApiClient::new(base_url)?),
    })
  }
}

impl Deref for ApiService {
  type Target = ApiClient;

  fn deref(&self) -> &Self::Target {
    &self.inner
  }
}

#[derive(Debug)]
pub struct ApiClient {
  // encoding that was negotiated to be used for the current session
  negotiated_encoding: Cell<Encoding>,
  base_url: String,
  client: reqwest::Client,
}

impl ApiClient {
  #[cfg(target_arch = "wasm32")]
  fn build_client() -> Result<reqwest::Client, APIError> {
    reqwest::Client::builder().build().map_err(Into::into)
  }

  #[cfg(feature = "dev-certificate")]
  #[cfg(not(target_arch = "wasm32"))]
  fn build_client() -> Result<reqwest::Client, APIError> {
    //let cookie_store = Self::setup_cookiestore()?;
    let cert = reqwest::Certificate::from_pem(DEVELOPMENT_CERTIFICATE)?;
    reqwest::Client::builder()
      .add_root_certificate(cert)
      //.http2_prior_knowledge()
      .cookie_store(true)
      //.cookie_provider(cookie_store)
      .https_only(true)
      .build()
      .map_err(Into::into)
  }

  #[cfg(not(feature = "dev-certificate"))]
  #[cfg(not(target_arch = "wasm32"))]
  fn build_client() -> Result<reqwest::Client, APIError> {
    let cookie_store = Self::setup_cookiestore()?;
    reqwest::Client::builder()
      .cookie_store(true)
      .cookie_provider(cookie_store)
      .http2_prior_knowledge()
      .https_only(true)
      .build()
      .map_err(Into::into)
  }

  #[cfg(not(target_arch = "wasm32"))]
  fn setup_cookiestore() -> Result<Arc<CookieStoreMutex>, APIError> {
    let app_dirs = platform_dirs::AppDirs::new(Some("einkaufsliste"), false).unwrap();
    let cookie_store_path = app_dirs.state_dir.join(Path::new("./cookies.json"));
    let cookie_store = {
      if let Ok(file) = std::fs::File::open(cookie_store_path).map(std::io::BufReader::new) {
        // use re-exported version of `CookieStore` for crate compatibility
        reqwest_cookie_store::CookieStore::load_json(file).unwrap()
      } else {
        reqwest_cookie_store::CookieStore::new(None)
      }
    };
    let cookie_store = reqwest_cookie_store::CookieStoreMutex::new(cookie_store);
    let cookie_store = std::sync::Arc::new(cookie_store);

    Ok(cookie_store)
  }

  pub fn set_encoding(&self, encoding: Encoding) {
    self.negotiated_encoding.set(encoding);
  }

  pub fn new(base_url: String) -> Result<Self, APIError> {
    let client = Self::build_client()?;

    Ok(Self {
      client,
      base_url,
      negotiated_encoding: Cell::new(Encoding::default()),
    })
  }

  fn get_request_headers(&self) -> reqwest::header::HeaderMap {
    let mut headers = reqwest::header::HeaderMap::new();
    match self.negotiated_encoding.get() {
      Encoding::JSON => {
        headers.insert(
          CONTENT_TYPE,
          HeaderValue::from_static("application/json"),
        );
        headers.insert(ACCEPT, HeaderValue::from_static("application/json"));
      }
      Encoding::Rkyv => {
        headers.insert(
          CONTENT_TYPE,
          HeaderValue::from_static("application/rkyv"),
        );
        headers.insert(ACCEPT, HeaderValue::from_static("application/rkyv"));
      }
    };

    headers
  }

  #[tracing::instrument]
  pub async fn login(&self, credentials: LoginUserV1) -> Result<User, APIError> {
    let response = self
      .client 
      .post(format!("{}/login/v1", self.base_url))
      .body(self.encode(&credentials)?)
      .headers(self.get_request_headers())
      .send()
      .await?;

    debug!("Login response: {response:?}");

    let body_bytes = response.error_for_status()?.bytes().await?;

    let user = self.decode(&body_bytes)?;

    Ok(user)
  }

  #[tracing::instrument]
  pub async fn register(&self, credentials: RegisterUserV1) -> Result<User, APIError> {
    let response = self
      .client
      .post(format!("{}/register/v1", self.base_url))
      .body(self.encode(&credentials)?)
      .headers(self.get_request_headers())
      .send()
      .await?;

    let body_bytes = response.error_for_status()?.bytes().await?;

    let user = self.decode(&body_bytes)?;

    Ok(user)
  }

  #[tracing::instrument]
  pub async fn fetch_all_lists(&self) -> Result<Vec<List>, APIError> {
    let response = self
      .client
      .get(format!("{}/user/lists", self.base_url))
      .headers(self.get_request_headers())
      .send()
      .await?;

    let body = response.error_for_status()?.bytes().await?;

    let lists = self.decode(&body)?;

    Ok(lists)
  }

  pub async fn create_list(&self, list: &List) -> Result<u64, APIError> {
    let response = self
      .client
      .post(format!("{}/itemList", self.base_url))
      .headers(self.get_request_headers())
      .body(self.encode(list)?)
      .send()
      .await?;

    Ok(u64::from_be_bytes(
      response
        .error_for_status()?
        .bytes()
        .await?
        .as_ref()
        .try_into()
        .map_err(|e: TryFromSliceError| APIError::Decoding(e.into()))?,
    ))
  }

  pub fn get_img_url(&self, image_id: u64) -> String {
    format!("{}/image/{}", self.base_url, image_id)
  }

  /**
  This function is used to encode data into the negotiated encoding.

  The data is required to implement all possible encodings for reasons of type solvability.
  */
  fn encode<T>(&self, data: &T) -> Result<Vec<u8>, APIError>
  where
    T: serde::Serialize + rkyv::Archive + rkyv::Serialize<rkyv::ser::serializers::AllocSerializer<4096>>,
  {
    let encoding = self.negotiated_encoding.get();
    match encoding {
      Encoding::JSON => Ok(serde_json::to_vec(data)?),
      Encoding::Rkyv => Ok(rkyv::to_bytes(data)?.to_vec()),
    }
  }

  fn decode<'a, T>(&self, data: &'a [u8]) -> Result<T, APIError>
  where
    T: serde::de::DeserializeOwned + rkyv::Archive,
    T::Archived: 'a + rkyv::Deserialize<T, SharedDeserializeMap> + CheckBytes<DefaultValidator<'a>>,
  {
    let encoding = self.negotiated_encoding.get();
    match encoding {
      Encoding::JSON => Ok(serde_json::from_slice(data)?),
      Encoding::Rkyv => Ok(rkyv::from_bytes(data)?),
    }
  }
}

#[derive(Debug)]
pub enum APIError {
  Network(reqwest::Error),
  InternalServer,
  Unauthorized,
  Unauthenticated,
  Encoding(Box<dyn std::error::Error>),
  Decoding(Box<dyn std::error::Error>),
  Unknown(String),
}

impl std::fmt::Display for APIError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      APIError::Network(e) => write!(f, "A network error occurred: {e}"),
      APIError::InternalServer => write!(f, "An internal server error occured."),
      APIError::Unauthorized => write!(f, "You are not authorized to access the requested resource."),
      APIError::Unauthenticated => write!(f, "You must authenticate yourself to access the requested resource."),
      APIError::Encoding(e) => write!(f, "An unexpected error occurred while encoding the request: {e}"),
      APIError::Decoding(e) => write!(f, "An unexpected error occurred while decoding the response: {e}"),
      APIError::Unknown(e) => write!(f, "Unknown error: {}", e),
    }
  }
}

impl From<serde_json::Error> for APIError {
  fn from(e: serde_json::Error) -> Self {
    match e.is_io() {
      true => APIError::Encoding(e.into()),
      false => APIError::Decoding(e.into()),
    }
  }
}

impl From<reqwest::Error> for APIError {
  fn from(e: reqwest::Error) -> Self {
    match e.status() {
      Some(status) => match status.as_u16() {
        401 => APIError::Unauthenticated,
        403 => APIError::Unauthorized,
        500 => APIError::InternalServer,
        _ => APIError::Unknown(format!("Unexpected status code: {} with message {e}", status,)),
      },
      None => APIError::Network(e),
    }
  }
}

impl<S: std::error::Error, T: std::error::Error, H: std::error::Error>
  From<rkyv::ser::serializers::CompositeSerializerError<S, T, H>> for APIError
{
  fn from(e: rkyv::ser::serializers::CompositeSerializerError<S, T, H>) -> Self {
    // i dont want to deal with lifetimes here so no stacktrace - deal with it :))))
    // can't stacktrace further than this anyway
    Self::Encoding(e.to_string().into())
  }
}

impl<C: std::error::Error, D: std::error::Error> From<CheckDeserializeError<C, D>> for APIError {
  fn from(e: CheckDeserializeError<C, D>) -> Self {
    Self::Decoding(e.to_string().into())
  }
}
