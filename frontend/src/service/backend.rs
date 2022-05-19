use std::fmt::Display;
use std::fs::File;
use std::io::{BufReader, ErrorKind, Read};
use std::path::Path;

use bytes::Buf;
use einkaufsliste::model::list::{FlatItemsList, List};
use einkaufsliste::model::requests::StoreItemAttached;
use einkaufsliste::model::shop::Shop;
use reqwest::{tls, Certificate, Client, StatusCode};
use rkyv::AlignedVec;
use tokio::sync::Mutex;

use crate::TransmissionError;

#[derive(Debug)]
pub enum APIServiceInitializationError {
  ErrorReadingFile(std::io::Error),
  ErrorParsingCertificate,
  ErrorBuildingClient,
}

impl Display for APIServiceInitializationError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let error_message = match self {
      APIServiceInitializationError::ErrorReadingFile(e) => format!("Error reading certificate file: {}", e),
      APIServiceInitializationError::ErrorParsingCertificate => "Error parsing certificate data".to_owned(),
      APIServiceInitializationError::ErrorBuildingClient => "Error building reqwest client".to_owned(),
    };

    write!(f, "{}", error_message)
  }
}

pub struct APIService<'a> {
  http_client: Mutex<Client>,
  base_url: &'a str,
  base_url_index: usize,
}

impl<'a> APIService<'a> {
  pub fn new(base_url: &'a str, cert_path: &Path) -> Result<APIService<'a>, APIServiceInitializationError> {
    let mut buffer = Vec::new();
    BufReader::new(File::open(cert_path).map_err(APIServiceInitializationError::ErrorReadingFile)?)
      .read_to_end(&mut buffer)
      .map_err(APIServiceInitializationError::ErrorReadingFile)?;

    let client = reqwest::ClientBuilder::new()
      .https_only(true)
      .min_tls_version(tls::Version::TLS_1_2)
      .add_root_certificate(
        Certificate::from_pem(&buffer).map_err(|_| APIServiceInitializationError::ErrorParsingCertificate)?,
      )
      .build()
      .map_err(|_| APIServiceInitializationError::ErrorBuildingClient)?;

    Ok(APIService {
      http_client: Mutex::new(client),
      base_url,
      base_url_index: base_url.len(),
    })
  }

  pub(crate) async fn store_shop(&self, shop: Shop) -> Result<u64, TransmissionError> {
    // TODO: evaluate thread-local scratcb space
    let mut url = String::with_capacity(256);
    const URI: &str = "/shop";
    url.push_str(self.base_url);
    url.push_str(URI);

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
      .map_err(|e| TransmissionError::InvalidResponseError(e.into()))?;

    // i hate this api...
    if new_id_bytes.len() < 8 {
      Err(TransmissionError::InvalidResponseError("Answer was too short.".into()))
    } else {
      Ok(new_id_bytes.get_u64())
    }
  }

  pub(crate) async fn get_shop(&self, id: u64) -> Result<Shop, TransmissionError> {
    let mut url = String::with_capacity(256);
    url.push_str(self.base_url);
    url.push_str("/shop/");
    url.push_str(&id.to_string());

    let response = self
      .http_client
      .lock()
      .await
      .get(url)
      .send()
      .await
      .map_err(TransmissionError::NetworkError)?;

    let response_bytes = match response.status() {
      StatusCode::OK => response.bytes().await.map_err(|_| TransmissionError::FailedRequest)?,
      _ => return Err(TransmissionError::FailedRequest),
    };

    // the alignment is apparently lost along the way so we need to reallocate + realign (by copying)
    let mut buffer = AlignedVec::with_capacity(response_bytes.len() - (response_bytes.len() % 64) + 64);
    buffer.extend_from_slice(&response_bytes);

    let shop = rkyv::from_bytes::<Shop>(&buffer).map_err(|e| TransmissionError::InvalidResponseError(e.into()))?;

    Ok(shop)
  }

  pub(crate) async fn push_new_item_list(list: List) -> Result<u64, TransmissionError> {
    const base_url: &str = "http://127.0.0.1:8080";
    const uri: &str = "/itemList";

    let mut url = String::from(base_url);
    url.push_str(uri);
    let bytes = rkyv::to_bytes::<_, 1024>(&list).map_err(|_| TransmissionError::SerializationError)?;
    let client = reqwest::Client::new();
    let response = client
      .post(url)
      .body::<Vec<u8>>(bytes.into())
      .send()
      .await
      .map_err(TransmissionError::NetworkError)?;

    let mut new_id_bytes = response
      .bytes()
      .await
      .map_err(|e| TransmissionError::InvalidResponseError(e.into()))?;

    // i hate this api...
    if new_id_bytes.len() < 8 {
      Err(TransmissionError::InvalidResponseError("Answer was too short.".into()))
    } else {
      Ok(new_id_bytes.get_u64_le()) //FIXME: Endianness
    }
  }

  pub(crate) async fn get_flat_items_list(id: u64) -> Result<FlatItemsList, TransmissionError> {
    let response = reqwest::get(format!("http://127.0.0.1:8080/itemList/{}/flat", id))
      .await
      .map_err(TransmissionError::NetworkError)?;

    let response_bytes = match response.status() {
      StatusCode::OK => response.bytes().await.map_err(|_| TransmissionError::FailedRequest)?,
      _ => return Err(TransmissionError::FailedRequest),
    };

    // the alignment is apparently lost along the way so we need to reallocate + realign (by copying)
    let mut buffer = AlignedVec::with_capacity(response_bytes.len() - (response_bytes.len() % 64) + 64);
    buffer.extend_from_slice(&response_bytes);

    let item_list =
      rkyv::from_bytes::<FlatItemsList>(&buffer).map_err(|e| TransmissionError::InvalidResponseError(e.into()))?;

    Ok(item_list)
  }

  async fn push_item_attached(command: StoreItemAttached) -> Result<(), TransmissionError> {
    const base_url: &str = "http://127.0.0.1:8080";
    const uri: &str = "/item/attached";

    let mut url = String::from(base_url);
    url.push_str(uri);
    let bytes = rkyv::to_bytes::<_, 128>(&command).map_err(|_| TransmissionError::SerializationError)?;
    let client = reqwest::Client::new();
    let response = client
      .post(url)
      .body::<Vec<u8>>(bytes.into())
      .send()
      .await
      .map_err(TransmissionError::NetworkError)?;

    match response.status() {
      StatusCode::CREATED => Ok(()),
      _ => Err(TransmissionError::FailedRequest),
    }
  }
}
