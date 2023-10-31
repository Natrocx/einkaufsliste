use std::array::TryFromSliceError;
use std::cell::Cell;
use std::ops::Deref;
use std::path::Path;
use std::rc::Rc;
use std::sync::{Arc, RwLock};

use bytes::Bytes;
use dioxus::prelude::Scope;
use dioxus_desktop::wry::http::request;
use einkaufsliste::model::list::{FlatItemsList, List};
use einkaufsliste::model::requests::{LoginUserV1, RegisterUserV1};
use einkaufsliste::model::user::User;
use einkaufsliste::model::Identifiable;
use einkaufsliste::{ApiObject, Encoding};
use reqwest::header::{HeaderValue, ACCEPT, CONTENT_TYPE};
use reqwest::{Method, Response};
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

pub fn use_provide_api_service(cx: &Scope, base_url: String) {
  cx.use_hook(|| {
    let api_service = ApiService::new(base_url).unwrap();
    cx.provide_context(api_service)
  });
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
  config: RwLock<ClientConfig>,
  base_url: String,
  client: reqwest::Client,
}

#[derive(Debug, Default)]
pub struct ClientConfig {
  pub encoding: Encoding,
}

impl ApiClient {
  #[cfg(target_arch = "wasm32")]
  fn build_client() -> Result<reqwest::Client, APIError> {
    reqwest::Client::builder().build().map_err(Into::into)
  }

  #[cfg(feature = "dev-certificate")]
  #[cfg(not(target_arch = "wasm32"))]
  fn build_client() -> Result<reqwest::Client, APIError> {
    let cookie_store = Self::setup_cookiestore()?;
    let cert = reqwest::Certificate::from_pem(DEVELOPMENT_CERTIFICATE)?;
    reqwest::Client::builder()
      .add_root_certificate(cert)
      //.http2_prior_knowledge()
      .cookie_store(true)
      .cookie_provider(cookie_store)
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
  fn setup_cookiestore() -> Result<Arc<reqwest_cookie_store::CookieStoreRwLock>, APIError> {
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
    let cookie_store = reqwest_cookie_store::CookieStoreRwLock::new(cookie_store);
    let cookie_store = std::sync::Arc::new(cookie_store);

    Ok(cookie_store)
  }

  pub fn set_encoding(&self, encoding: Encoding) {
    self.config.write().unwrap().encoding = encoding;
  }

  pub fn new(base_url: String) -> Result<Self, APIError> {
    let client = Self::build_client()?;

    Ok(Self {
      client,
      base_url,
      config: RwLock::new(ClientConfig::default())
    })
  }

  fn get_request_headers(&self) -> reqwest::header::HeaderMap {
    let mut headers = reqwest::header::HeaderMap::new();
    let mut lock = self.config.read().unwrap();
    match lock.encoding {
      Encoding::JSON => {
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert(ACCEPT, HeaderValue::from_static("application/json"));
      }
      Encoding::Rkyv => {
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/rkyv"));
        headers.insert(ACCEPT, HeaderValue::from_static("application/rkyv"));
      }
    };

    tracing::debug!("Sending headers: {headers:?}");

    headers
  }

  async fn request<T: ApiObject<'static>>(
    &self,
    url: &str,
    method: reqwest::Method,
    body: &T,
  ) -> Result<Bytes, APIError>
  where
    <T as rkyv::Archive>::Archived: rkyv::Deserialize<T, rkyv::de::deserializers::SharedDeserializeMap>,
    <T as rkyv::Archive>::Archived: rkyv::CheckBytes<rkyv::validation::validators::DefaultValidator<'static>>,
  {
    let response = self
      .client
      .request(method, url)
      .body(self.encode(body)?)
      .headers(self.get_request_headers())
      .send()
      .await?;

    response.error_for_status()?.bytes().await.map_err(Into::into)
  }

  #[tracing::instrument]
  pub async fn login(&self, credentials: LoginUserV1) -> Result<User, APIError> {
    let url = format!("{}/login/v1", self.base_url);

    let response_body_bytes = self.request(&url, Method::POST, &credentials).await?;

    let user = self.decode(&response_body_bytes)?;

    Ok(user)
  }

  #[tracing::instrument]
  pub async fn register(&self, credentials: RegisterUserV1) -> Result<User, APIError> {
    let url = format!("{}/register/v1", self.base_url);

    let body_bytes = self.request(&url, Method::POST, &credentials).await?;

    let user = self.decode(&body_bytes)?;

    Ok(user)
  }

  #[tracing::instrument]
  pub async fn fetch_all_lists(&self) -> Result<Vec<List>, APIError> {
    let url = format!("{}/user/lists", self.base_url);

    let body = self.request(&url, Method::GET, &()).await?;

    let lists = self.decode(&body)?;

    Ok(lists)
  }

  #[tracing::instrument]
  pub async fn create_list(&self, list: &List) -> Result<u64, APIError> {
    let url = format!("{}/itemList", self.base_url);

    let body = self.request(&url, Method::POST, list).await?;

    Ok(u64::from_be_bytes(
      body
        .as_ref()
        .try_into()
        .map_err(|e: TryFromSliceError| APIError::Decoding(e.into()))?,
    ))
  }

  pub(crate) async fn update_list(&self, list: &List) -> Result<(), APIError> {
    let url = format!("{}/itemList", self.base_url);

    let _body = self.request(&url, Method::PUT, list).await?;
    // nothing to extract/check here except for the response status (handled above)

    Ok(())
  }

  pub async fn fetch_list(&self, list_id: &<List as Identifiable>::Id) -> Result<FlatItemsList, APIError> {
    let url = format!("{}/itemList/{}/flat", self.base_url, list_id);

    let body = self.request(&url, Method::GET, &()).await?;

    let list = self.decode(&body)?;

    Ok(list)
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
    let encoding = self.config.read().unwrap().encoding;
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
    let encoding = self.config.read().unwrap().encoding;
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
