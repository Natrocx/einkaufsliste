use std::fmt::Display;
use std::marker::PhantomData;
use std::ops::Deref;
use std::sync::Arc;

use bytes::Buf;
use einkaufsliste::model::item::Item;
use einkaufsliste::model::list::{FlatItemsList, List};
use einkaufsliste::model::requests::{LoginUserV1, RegisterUserV1, StoreItemAttached};
use einkaufsliste::model::shop::Shop;
use einkaufsliste::model::user::{ObjectList, User};
use einkaufsliste::model::Identifiable;
use futures::lock::Mutex;
use reqwest::{Client, StatusCode};
use rkyv::AlignedVec;
use zerocopy::AsBytes;

use crate::TransmissionError;

pub trait ApplicationState {}

pub struct Uninitialized;
impl ApplicationState for Uninitialized {}

pub struct LoggedIn;
impl ApplicationState for LoggedIn {}

#[derive(Clone)]
pub struct APIServiceII<T: ApplicationState> {
  service: Arc<APIService>,
  state: PhantomData<T>,
}

impl APIServiceII<Uninitialized> {
  pub fn new(api_service: APIService) -> Self {
    Self {
      service: Arc::new(api_service),
      state: PhantomData {},
    }
  }

  pub async fn login(self, command: &LoginUserV1) -> Result<APIServiceII<LoggedIn>, APIServiceII<Uninitialized>> {
    match self.service.login_v1(command).await {
      Ok(_) => Ok(APIServiceII {
        service: self.service,
        state: PhantomData {},
      }),
      Err(_) => Err(self),
    }
  }

  pub async fn register(self, command: &RegisterUserV1) -> Result<APIServiceII<LoggedIn>, APIServiceII<Uninitialized>> {
    match self.service.register_v1(command).await {
      Ok(_) => Ok(APIServiceII {
        service: self.service,
        state: PhantomData {},
      }),
      Err(_) => Err(self),
    }
  }
}

impl Deref for APIServiceII<LoggedIn> {
  type Target = APIService;

  fn deref(&self) -> &Self::Target {
    &self.service
  }
}

#[derive(Debug)]
pub enum APIServiceInitializationError {
  ReadingFile(std::io::Error),
  ParsingCertificate,
  BuildingClient,
}

impl Display for APIServiceInitializationError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let error_message = match self {
      APIServiceInitializationError::ReadingFile(e) => format!("Error reading certificate file: {}", e),
      APIServiceInitializationError::ParsingCertificate => "Error parsing certificate data".to_owned(),
      APIServiceInitializationError::BuildingClient => "Error building reqwest client".to_owned(),
    };

    write!(f, "{}", error_message)
  }
}
pub struct APIService {
  http_client: Mutex<Client>,
  pub base_url: &'static str,
}

impl APIService {
  pub fn new(base_url: &'static str) -> Result<APIService, APIServiceInitializationError> {
    let client = reqwest::ClientBuilder::new()
      .build()
      .map_err(|_| APIServiceInitializationError::BuildingClient)?;

    Ok(APIService {
      http_client: Mutex::new(client),
      base_url,
    })
  }

  #[cfg(not(target_arch = "wasm32"))]
  pub fn insecure() -> Result<APIService, APIServiceInitializationError> {
    let client = reqwest::ClientBuilder::new()
      .cookie_store(true)
      .danger_accept_invalid_certs(true)
      .https_only(true)
      .http2_prior_knowledge()
      .build()
      .map_err(|_| APIServiceInitializationError::BuildingClient)?;

    Ok(APIService {
      http_client: Mutex::new(client),
      base_url: "https://localhost:8443",
    })
  }

  // TODO: evaluate ThreadLocal storage for url
  fn build_url(&self, uri: &str) -> String {
    let mut url = String::with_capacity(256);
    url.push_str(self.base_url);
    url.push_str(uri);

    url
  }

  /// builds a url from base, uri and id. the [uri] does not require a trailing /
  fn build_url_with_id(&self, uri: &str, id: u64) -> String {
    format!("{}{}/{}", self.base_url, uri, id)
  }

  pub(crate) async fn store_shop(&self, shop: Shop) -> Result<u64, TransmissionError> {
    let url = self.build_url("/shop");

    let bytes = rkyv::to_bytes::<_, 128>(&shop).map_err(|_| TransmissionError::SerializationError)?;
    let response = self
      .http_client
      .lock()
      .await
      .post(url)
      .body::<Vec<u8>>(bytes.into())
      .send()
      .await
      .map_err(TransmissionError::NetworkError)?;

    let mut new_id_bytes = response
      .bytes()
      .await
      .map_err(|e| TransmissionError::InvalidResponseError(e.to_string()))?;

    // i hate this api...
    if new_id_bytes.len() < 8 {
      Err(TransmissionError::InvalidResponseError("Answer was too short.".into()))
    } else {
      Ok(new_id_bytes.get_u64())
    }
  }

  pub async fn get_item(&self, id: u64) -> Result<Item, TransmissionError> {
    let url = self.build_url_with_id("/item", id);

    let response = self.http_client.lock().await.get(url).send().await?;

    Ok(rkyv::from_bytes::<Item>(&response.bytes().await?)?)
  }

  pub(crate) async fn get_shop(&self, id: u64) -> Result<Shop, TransmissionError> {
    let url = self.build_url_with_id("/shop", id);

    let response = self
      .http_client
      .lock()
      .await
      .get(url)
      .send()
      .await
      .map_err(TransmissionError::NetworkError)?;

    let response_bytes = match response.status() {
      StatusCode::OK => response
        .bytes()
        .await
        .map_err(|e| TransmissionError::InvalidResponseError(format!("Empty Response. Expected data. {e}")))?,
      status => return Err(TransmissionError::FailedRequest(status)),
    };

    // the alignment is apparently lost along the way so we need to reallocate + realign (by copying)
    let mut buffer = AlignedVec::with_capacity(response_bytes.len() - (response_bytes.len() % 64) + 64);
    buffer.extend_from_slice(&response_bytes);

    let shop = rkyv::from_bytes::<Shop>(&buffer).map_err(|e| TransmissionError::InvalidResponseError(e.to_string()))?;

    Ok(shop)
  }

  pub(crate) async fn push_new_item_list(&self, list: List) -> Result<u64, TransmissionError> {
    let bytes = rkyv::to_bytes::<_, 1024>(&list).map_err(|_| TransmissionError::SerializationError)?;
    let response = self
      .http_client
      .lock()
      .await
      .post(self.build_url("/itemList"))
      .fetch_mode_no_cors()
      .body::<Vec<u8>>(bytes.into())
      .send()
      .await
      .map_err(TransmissionError::NetworkError)?;

    let mut new_id_bytes = response
      .bytes()
      .await
      .map_err(|e| TransmissionError::InvalidResponseError(e.to_string()))?;

    if new_id_bytes.len() < 8 {
      Err(TransmissionError::InvalidResponseError("Answer was too short.".into()))
    } else {
      Ok(new_id_bytes.get_u64())
    }
  }

  pub(crate) async fn get_flat_items_list(&self, id: u64) -> Result<FlatItemsList, TransmissionError> {
    let response = self
      .http_client
      .lock()
      .await
      .get(format!("{}/itemList/{}/flat", self.base_url, id))
      .send()
      .await
      .map_err(TransmissionError::NetworkError)?;

    let response_bytes = match response.status() {
      StatusCode::OK => response
        .bytes()
        .await
        .map_err(|e| TransmissionError::InvalidResponseError(format!("Empty Response. Expected data: {e}")))?,
      status => return Err(TransmissionError::FailedRequest(status)),
    };

    // the alignment is apparently lost along the way so we need to reallocate + realign (by copying)
    let mut buffer = AlignedVec::with_capacity(response_bytes.len() - (response_bytes.len() % 64) + 64);
    buffer.extend_from_slice(&response_bytes);

    let item_list =
      rkyv::from_bytes::<FlatItemsList>(&buffer).map_err(|e| TransmissionError::InvalidResponseError(e.to_string()))?;

    Ok(item_list)
  }

  pub(crate) async fn push_item_attached(&self, command: StoreItemAttached) -> Result<u64, TransmissionError> {
    let url = self.build_url("/item/attached");

    let bytes = rkyv::to_bytes::<_, 256>(&command).map_err(|_| TransmissionError::SerializationError)?;
    let response = self
      .http_client
      .lock()
      .await
      .post(url)
      .body::<Vec<u8>>(bytes.into())
      .send()
      .await?;

    let mut response_bytes = response.bytes().await?;
    if response_bytes.len() >= std::mem::size_of::<u64>() {
      Ok(response_bytes.get_u64())
    } else {
      Err(TransmissionError::InvalidResponseError("Response was too short".into()))
    }
  }

  pub(crate) async fn register_v1(&self, command: &RegisterUserV1) -> Result<u64, TransmissionError> {
    let url = self.build_url("/register/v1");

    let bytes = rkyv::to_bytes::<_, 256>(command).map_err(|_| TransmissionError::SerializationError)?;
    let response = self
      .http_client
      .lock()
      .await
      .post(url)
      .body(bytes.to_vec())
      .send()
      .await
      .map_err(TransmissionError::NetworkError)?;
    // juggle ownership
    let status = response.status();

    let mut id_bytes = response
      .bytes()
      .await
      .map_err(|e| TransmissionError::InvalidResponseError(e.to_string()))?;

    match status {
      StatusCode::OK => {
        if id_bytes.len() < 8 {
          Err(TransmissionError::InvalidResponseError(
            "Incomplete response from server".to_string(),
          ))
        } else {
          Ok(id_bytes.get_u64())
        }
      }
      _ => Err(TransmissionError::FailedRequest(status)),
    }
  }

  pub async fn login_v1(&self, command: &LoginUserV1) -> Result<<User as Identifiable>::Id, TransmissionError> {
    let url = self.build_url("/login/v1");

    let bytes = rkyv::to_bytes::<_, 128>(command).map_err(|_| TransmissionError::SerializationError)?;
    let response = self
      .http_client
      .lock()
      .await
      .post(url)
      .body(bytes.into_vec())
      .send()
      .await
      .map_err(TransmissionError::NetworkError)?;

    match response.status() {
      StatusCode::OK => {
        let mut response_bytes = response.bytes().await.unwrap();
        if response_bytes.len() < 8 {
          Err(TransmissionError::InvalidResponseError(
            "Incomplete response from server.".to_string(),
          ))
        } else {
          Ok(response_bytes.get_u64())
        }
      }
      status => Err(TransmissionError::FailedRequest(status)),
    }
  }

  pub async fn update_item(&self, item: &Item) -> Result<(), TransmissionError> {
    let url = self.build_url("/item/v1");

    let bytes = rkyv::to_bytes::<_, 256>(item).map_err(|_| TransmissionError::SerializationError)?;
    let response = self
      .http_client
      .lock()
      .await
      .post(url)
      .body(bytes.into_vec())
      .send()
      .await
      .map_err(TransmissionError::NetworkError)?;

    match response.status() {
      StatusCode::OK => Ok(()),
      status => Err(TransmissionError::FailedRequest(status)),
    }
  }

  pub async fn get_users_lists(&self) -> Result<ObjectList, TransmissionError> {
    let url = format!("{}/user/lists", self.base_url);

    let response = self.http_client.lock().await.get(&url).send().await?;

    let lists = rkyv::from_bytes::<ObjectList>(response.bytes().await?.as_bytes())?;

    Ok(lists)
  }
}
