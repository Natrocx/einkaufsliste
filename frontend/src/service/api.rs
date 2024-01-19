use std::array::TryFromSliceError;
use std::cell::Ref;
use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::{Arc, RwLock};

use bytes::Bytes;
use einkaufsliste::model::item::Item;
use einkaufsliste::model::list::{FlatItemsList, List};
use einkaufsliste::model::requests::{DeleteItem, LoginUserV1, RegisterUserV1, StoreItemAttached};
use einkaufsliste::model::user::User;
use einkaufsliste::model::Identifiable;
use einkaufsliste::{ApiObject, Encoding};
use platform_dirs::AppDirs;
use reqwest::header::{HeaderValue, ACCEPT, CONTENT_TYPE};
use reqwest::Method;
#[cfg(not(target_arch = "wasm32"))]
use reqwest_cookie_store::CookieStoreRwLock;
use rkyv::de::deserializers::SharedDeserializeMap;
use rkyv::validation::validators::{CheckDeserializeError, DefaultValidator};
use rkyv::CheckBytes;

/*
 This file contains the API client and a reference counted Service for use in dioxus.

 It should not be used directly but rather through a cached service. (to be implemented)

 Maintainers notes:
   * remeber to call `.error_for_status()` on all responses to catch invalid requests and server errors
*/

// Avoid certificate errors on desktop
#[cfg(not(target_arch = "wasm32"))]
pub static DEVELOPMENT_CERTIFICATE: &[u8] = include_bytes!("./rootCA.pem");

static COOKIE_STORE_FILE_NAME: &str = "cookies.json";
// default configuration
#[cfg(not(target_arch = "wasm32"))]
lazy_static::lazy_static! {
  static ref APP_DIR: std::path::PathBuf = AppDirs::new(Some("einkaufsliste"), false).unwrap().state_dir;
  static ref COOKIE_STORE_PATH: std::path::PathBuf = APP_DIR.join(Path::new(COOKIE_STORE_FILE_NAME));
}

#[derive(Debug, Clone)]
pub struct ApiService {
  inner: Arc<ApiClient>,
}

impl ApiService {
  pub fn new(base_url: String) -> Result<Self, ApiError> {
    Ok(Self {
      inner: Arc::new(ApiClient::new(base_url)?),
    })
  }
}

impl Deref for ApiService {
  type Target = ApiClient;

  fn deref(&self) -> &Self::Target {
    &self.inner
  }
}

impl Drop for ApiClient {
  fn drop(&mut self) {
    #[cfg(not(target_arch = "wasm32"))]
    self.save_cookiestore();
    tracing::debug!("Dropped ApiClient - this should only happen on shutdown")
  }
}

#[derive(Debug)]
pub struct ApiClient {
  config: RwLock<ClientConfig>,
  #[cfg(not(target_arch = "wasm32"))]
  cookie_store: Arc<CookieStoreRwLock>,
  base_url: String,
  client: reqwest::Client,
}

#[derive(Debug)]
pub struct ClientConfig {
  pub encoding: Encoding,
  #[cfg(not(target_arch = "wasm32"))]
  pub cookie_store_base_path: PathBuf,
}

impl Default for ClientConfig {
  fn default() -> Self {
    Self {
      encoding: Encoding::default(),
      #[cfg(not(target_arch = "wasm32"))]
      cookie_store_base_path: APP_DIR.clone(),
    }
  }
}

impl ApiClient {
  #[cfg(target_arch = "wasm32")]
  fn build_client() -> Result<reqwest::Client, ApiError> {
    reqwest::Client::builder().build().map_err(Into::into)
  }

  #[cfg(feature = "dev-certificate")]
  #[cfg(not(target_arch = "wasm32"))]
  fn build_client(cookie_store: Arc<CookieStoreRwLock>) -> Result<reqwest::Client, ApiError> {
    let cert = reqwest::Certificate::from_pem(DEVELOPMENT_CERTIFICATE)?;
    reqwest::Client::builder()
      .add_root_certificate(cert)
      .http2_prior_knowledge()
      .cookie_store(true)
      .cookie_provider(cookie_store)
      .https_only(true)
      .build()
      .map_err(Into::into)
  }

  #[cfg(not(feature = "dev-certificate"))]
  #[cfg(not(target_arch = "wasm32"))]
  fn build_client(cookie_store: Arc<CookieStoreRwLock>) -> Result<reqwest::Client, ApiError> {
    reqwest::Client::builder()
      .cookie_store(true)
      .cookie_provider(cookie_store)
      .http2_prior_knowledge()
      .https_only(true)
      .build()
      .map_err(Into::into)
  }

  #[cfg(not(target_arch = "wasm32"))]
  fn setup_cookiestore(path: &Path) -> Result<Arc<reqwest_cookie_store::CookieStoreRwLock>, ApiError> {
    let cookie_store = {
      let actual_path = path.join(COOKIE_STORE_FILE_NAME);
      if let Ok(file) = std::fs::File::open(&actual_path).map(std::io::BufReader::new) {
        // use re-exported version of `CookieStore` for crate compatibility
        tracing::debug!("Loaded cookie store from path: {:?}", actual_path);
        reqwest_cookie_store::CookieStore::load_json(file).unwrap()
      } else {
        tracing::debug!(
          "Tried loading cookie store, but none found at {}. Creating new one.",
          actual_path.display()
        );
        reqwest_cookie_store::CookieStore::new(None)
      }
    };
    let cookie_store = reqwest_cookie_store::CookieStoreRwLock::new(cookie_store);
    let cookie_store = std::sync::Arc::new(cookie_store);

    Ok(cookie_store)
  }

  /**
  This function will panic if the CookieStore json file cannot be created.
  */
  #[cfg(not(target_arch = "wasm32"))]
  pub fn save_cookiestore(&self) {
    let read_lock = self.config.read().unwrap();

    if !read_lock.cookie_store_base_path.exists() {
      std::fs::create_dir_all(&read_lock.cookie_store_base_path).unwrap();
    }

    let mut writer = std::fs::File::create(read_lock.cookie_store_base_path.join(COOKIE_STORE_FILE_NAME))
      .map(std::io::BufWriter::new)
      .unwrap();
    let store = self.cookie_store.read().unwrap();
    store.save_json(&mut writer).unwrap();
  }

  pub fn set_encoding(&self, encoding: Encoding) {
    self.config.write().unwrap().encoding = encoding;
  }

  #[cfg(not(target_arch = "wasm32"))]
  pub fn new_with_config(base_url: String, config: ClientConfig) -> Result<Self, ApiError> {
    let cookie_store = Self::setup_cookiestore(&config.cookie_store_base_path)?;
    let client = Self::build_client(cookie_store.clone())?;

    Ok(Self {
      client,
      cookie_store,
      base_url,
      config: RwLock::new(config),
    })
  }

  #[cfg(not(target_arch = "wasm32"))]
  pub fn new(base_url: String) -> Result<Self, ApiError> {
    let cookie_store = Self::setup_cookiestore(&APP_DIR)?;
    let client = Self::build_client(cookie_store.clone())?;

    Ok(Self {
      client,
      cookie_store,
      base_url,
      config: RwLock::new(ClientConfig::default()),
    })
  }

  #[cfg(target_arch = "wasm32")]
  pub fn new(base_url: String) -> Result<Self, ApiError> {
    let client = Self::build_client()?;

    Ok(Self {
      client,
      base_url,
      config: RwLock::new(ClientConfig::default()),
    })
  }

  fn get_request_headers(&self) -> reqwest::header::HeaderMap {
    let mut headers = reqwest::header::HeaderMap::new();
    let lock = self.config.read().unwrap();
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

    tracing::trace!("Sending headers: {headers:?}");

    headers
  }

  async fn request<T: ApiObject<'static>>(
    &self,
    url: &str,
    method: reqwest::Method,
    body: &T,
  ) -> Result<Bytes, ApiError>
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
  pub async fn login(&self, credentials: LoginUserV1) -> Result<User, ApiError> {
    let url = format!("{}/login/v1", self.base_url);

    let response_body_bytes = self.request(&url, Method::POST, &credentials).await?;

    let user = self.decode(&response_body_bytes)?;

    Ok(user)
  }

  #[tracing::instrument]
  pub async fn register(&self, credentials: RegisterUserV1) -> Result<User, ApiError> {
    let url = format!("{}/register/v1", self.base_url);

    let body_bytes = self.request(&url, Method::POST, &credentials).await?;

    let user = self.decode(&body_bytes)?;

    Ok(user)
  }

  #[tracing::instrument]
  pub async fn fetch_all_lists(&self) -> Result<Vec<List>, ApiError> {
    let url = format!("{}/user/lists", self.base_url);

    let body = self.request(&url, Method::GET, &()).await?;

    let lists = self.decode(&body)?;

    Ok(lists)
  }

  #[tracing::instrument]
  pub async fn create_list(&self, mut list: List) -> Result<List, ApiError> {
    let url = format!("{}/itemList", self.base_url);

    let body = self.request(&url, Method::POST, &list).await?;

    list.id = self.decode(&body)?;

    Ok(list)
  }

  #[tracing::instrument(skip(self))]
  pub async fn new_item(&self, list_id: u64, item: Item) -> Result<u64, ApiError> {
    let url = format!("{}/item/attached", self.base_url);

    let body = self
      .request(&url, Method::POST, &StoreItemAttached { list_id, item })
      .await?;

    self.decode(&body)
  }

  pub async fn delete_item(&self, command: DeleteItem) -> Result<(), ApiError> {
    let url = format!("{}/item", self.base_url);

    self.request(&url, Method::DELETE, &command).await?;

    Ok(())
  }

  pub async fn fetch_list(&self, list_id: <List as Identifiable>::Id) -> Result<FlatItemsList, ApiError> {
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
  fn encode<T>(&self, data: &T) -> Result<Vec<u8>, ApiError>
  where
    T: serde::Serialize + rkyv::Archive + rkyv::Serialize<rkyv::ser::serializers::AllocSerializer<4096>>,
  {
    let encoding = self.config.read().unwrap().encoding;
    match encoding {
      Encoding::JSON => Ok(serde_json::to_vec(data)?),
      Encoding::Rkyv => Ok(rkyv::to_bytes(data)?.to_vec()),
    }
  }

  fn decode<'a, T>(&self, data: &'a [u8]) -> Result<T, ApiError>
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
pub enum ApiError {
  Network(reqwest::Error),
  InternalServer,
  Unauthorized,
  Unauthenticated,
  Encoding(String),
  Decoding(String),
  Unknown(String),
}

impl std::fmt::Display for ApiError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      ApiError::Network(e) => write!(f, "A network error occurred: {e}"),
      ApiError::InternalServer => write!(f, "An internal server error occured."),
      ApiError::Unauthorized => write!(f, "You are not authorized to access the requested resource."),
      ApiError::Unauthenticated => write!(f, "You must authenticate yourself to access the requested resource."),
      ApiError::Encoding(e) => write!(f, "An unexpected error occurred while encoding the request: {e}"),
      ApiError::Decoding(e) => write!(f, "An unexpected error occurred while decoding the response: {e}"),
      ApiError::Unknown(e) => write!(f, "Unknown error: {}", e),
    }
  }
}

impl From<serde_json::Error> for ApiError {
  fn from(e: serde_json::Error) -> Self {
    match e.is_io() {
      true => ApiError::Encoding(e.to_string()),
      false => ApiError::Decoding(e.to_string()),
    }
  }
}

impl From<reqwest::Error> for ApiError {
  fn from(e: reqwest::Error) -> Self {
    match e.status() {
      Some(status) => match status.as_u16() {
        401 => ApiError::Unauthenticated,
        403 => ApiError::Unauthorized,
        500 => ApiError::InternalServer,
        _ => ApiError::Unknown(format!("Unexpected status code: {} with message {e}", status,)),
      },
      None => ApiError::Network(e),
    }
  }
}

impl<S: std::error::Error, T: std::error::Error, H: std::error::Error>
  From<rkyv::ser::serializers::CompositeSerializerError<S, T, H>> for ApiError
{
  fn from(e: rkyv::ser::serializers::CompositeSerializerError<S, T, H>) -> Self {
    // i dont want to deal with lifetimes here so no stacktrace - deal with it :))))
    // can't stacktrace further than this anyway
    Self::Encoding(e.to_string().into())
  }
}

impl<C: std::error::Error, D: std::error::Error> From<CheckDeserializeError<C, D>> for ApiError {
  fn from(e: CheckDeserializeError<C, D>) -> Self {
    Self::Decoding(e.to_string().into())
  }
}
