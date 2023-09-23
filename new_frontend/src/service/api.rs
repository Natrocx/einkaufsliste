use std::cell::Cell;
use std::ops::Deref;
use std::rc::Rc;


use einkaufsliste::model::requests::LoginUserV1;

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

  #[cfg(not(target_arch = "wasm32"))]
  fn build_client() -> Result<reqwest::Client, APIError> {
    reqwest::Client::builder()
      .cookie_store(true)
      .http2_prior_knowledge()
      .redirect(reqwest::redirect::Policy::none())
      .build()
      .map_err(Into::into)
  }

  pub fn new(base_url: String) -> Result<Self, APIError> {
    let client = Self::build_client()?;

    Ok(Self {
      client,
      base_url,
      negotiated_encoding: Cell::new(Encoding::default()),
    })
  }

  pub async fn login(&self, credentials: LoginUserV1) -> Result<u64, APIError> {
    let response = self
      .client
      .post("/login/v1")
      .body(self.encode(&credentials)?)
      .send()
      .await?;

    let body = u64::from_be_bytes(
      response
        .bytes()
        .await?
        .as_ref()
        .try_into() // convert to array for u64::from_be_bytes
        .map_err(|e| APIError::Unknown(format!("Expected response from server not found. Got {e:?} instead.")))?, /* if conversion fails, then server must have returned garbage */
    );

    Ok(body)
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
}

#[derive(Debug, Default, Clone, Copy)]
pub enum Encoding {
  JSON,
  #[default]
  Rkyv,
}

#[derive(Debug)]
pub enum APIError {
  Network,
  InternalServer,
  Unauthorized,
  Unauthenticated,
  Encoding,
  Decoding,
  Unknown(String),
}

impl std::fmt::Display for APIError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      APIError::Network => write!(f, "Network error"),
      APIError::InternalServer => write!(f, "Internal server error"),
      APIError::Unauthorized => write!(f, "Unauthorized"),
      APIError::Unauthenticated => write!(f, "Unauthenticated"),
      APIError::Encoding => write!(f, "Encoding error"),
      APIError::Decoding => write!(f, "Decoding error"),
      APIError::Unknown(e) => write!(f, "Unknown error: {}", e),
    }
  }
}

impl From<serde_json::Error> for APIError {
  fn from(e: serde_json::Error) -> Self {
    match e.is_io() {
      true => APIError::Encoding,
      false => APIError::Decoding,
    }
  }
}

impl From<reqwest::Error> for APIError {
  fn from(e: reqwest::Error) -> Self {
    match e.status() {
      Some(status) => match status.as_u16() {
        401 => APIError::Unauthorized,
        403 => APIError::Unauthenticated,
        500 => APIError::InternalServer,
        _ => APIError::Unknown(format!("Unexpected status code: {} with message {e}", status,)),
      },
      None => APIError::Network,
    }
  }
}

impl<S, T, H> From<rkyv::ser::serializers::CompositeSerializerError<S, T, H>> for APIError {
  fn from(_value: rkyv::ser::serializers::CompositeSerializerError<S, T, H>) -> Self {
    Self::Encoding
  }
}
