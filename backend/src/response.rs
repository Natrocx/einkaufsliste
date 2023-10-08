use std::convert::{Infallible};
use std::fmt::Display;
use std::ops::{FromResidual, Try};

use actix_session::storage::LoadError;
use actix_web::body::{BoxBody};
use actix_web::error::{
  ErrorBadRequest, ErrorForbidden, ErrorInternalServerError, ErrorNotFound, ErrorUnauthorized,
  PayloadError,
};
use actix_web::http::header::ACCEPT;
use actix_web::{HttpResponse, Responder};
use bytecheck::StructCheckError;

use einkaufsliste::{ApiObject, Encoding};
use rkyv::de::deserializers::{SharedDeserializeMap, SharedDeserializeMapError};
use rkyv::ser::serializers::{
  AllocScratchError,
  CompositeSerializerError,
  SharedSerializeMapError,
};
use rkyv::validation::validators::{
  CheckDeserializeError, DefaultValidator, DefaultValidatorError,
};
use rkyv::validation::CheckArchiveError;
use rkyv::{Deserialize};



use crate::api::user::PasswordValidationError;

pub struct Response<T: ApiObject<'static>>(pub Result<T, ResponseError>)
where
  T::Archived: rkyv::CheckBytes<rkyv::validation::validators::DefaultValidator<'static>>
    + rkyv::Deserialize<T, rkyv::de::deserializers::SharedDeserializeMap>;

impl Response<()> {
  pub fn empty() -> Self {
    Self(Ok(()))
  }
}

impl<T: ApiObject<'static>> From<ResponseError> for Response<T>
where
  T::Archived: rkyv::CheckBytes<rkyv::validation::validators::DefaultValidator<'static>>
    + rkyv::Deserialize<T, rkyv::de::deserializers::SharedDeserializeMap>,
{
  fn from(e: ResponseError) -> Self {
    Self(Err(e))
  }
}

impl<T: ApiObject<'static>> From<T> for Response<T>
where
  <T as rkyv::Archive>::Archived:
    rkyv::Deserialize<T, rkyv::de::deserializers::SharedDeserializeMap>,
  <T as rkyv::Archive>::Archived:
    rkyv::CheckBytes<rkyv::validation::validators::DefaultValidator<'static>>,
{
  fn from(val: T) -> Self {
    Self(Ok(val))
  }
}

impl<T: ApiObject<'static>, E: Into<ResponseError>> FromResidual<Result<Infallible, E>>
  for Response<T>
where
  T::Archived: rkyv::CheckBytes<rkyv::validation::validators::DefaultValidator<'static>>
    + rkyv::Deserialize<T, rkyv::de::deserializers::SharedDeserializeMap>,
{
  fn from_residual(residual: Result<Infallible, E>) -> Self {
    match residual{
      Ok(_) => unreachable!(),
      Err(error) => Self(Err(error.into())),
    }
  }
}

impl<T: ApiObject<'static>> Try for Response<T>
where
  T::Archived: rkyv::CheckBytes<rkyv::validation::validators::DefaultValidator<'static>>
    + rkyv::Deserialize<T, rkyv::de::deserializers::SharedDeserializeMap>,
{
  type Output = T;

  type Residual = Result<Infallible, ResponseError>;

  fn from_output(output: Self::Output) -> Self {
    Self(Ok(output))
  }

  fn branch(self) -> std::ops::ControlFlow<Self::Residual, Self::Output> {
    match self.0 {
      Ok(val) => std::ops::ControlFlow::Continue(val),
      Err(e) => std::ops::ControlFlow::Break(Err(e)),
    }
  }
}

fn encode<'a, T: ApiObject<'a>>(encoding: Encoding, body: &T) -> Result<Vec<u8>, ResponseError>
where
  <T as rkyv::Archive>::Archived: rkyv::CheckBytes<rkyv::validation::validators::DefaultValidator<'a>>
    + rkyv::Deserialize<T, rkyv::de::deserializers::SharedDeserializeMap>,
{
  match encoding {
    Encoding::Rkyv => Ok(rkyv::to_bytes(body)?.to_vec()),
    Encoding::JSON => serde_json::to_vec(body).map_err(|_| ResponseError::ErrorInternalServerError),
  }
}

impl<T: ApiObject<'static>> Responder for Response<T>
where
  T::Archived:
    bytecheck::CheckBytes<DefaultValidator<'static>> + Deserialize<T, SharedDeserializeMap>,
{
  type Body = BoxBody;

  fn respond_to(self, req: &actix_web::HttpRequest) -> HttpResponse<Self::Body> {
    let encoding: einkaufsliste::Encoding = req
      .headers()
      .get(ACCEPT)
      .map(Into::into)
      .unwrap_or_default();
    tracing::debug!("Responding with content type: {:?}", encoding);

    match self.0 {
      Ok(body) => {
        let body = match encode(encoding, &body) {
          Ok(body) => body,
          Err(e) => {
            let message = e.to_string();
            return HttpResponse::InternalServerError().body(message);
          }
        };

        let body = BoxBody::new::<Vec<u8>>(body);
        HttpResponse::Ok().content_type(encoding).body(body)
      }
      Err(e) => {
        let message = e.to_string();
        let body = BoxBody::new::<String>(message);

        match e {
          ResponseError::ErrorBadRequest => HttpResponse::BadRequest().body(body),
          ResponseError::ErrorInternalServerError => HttpResponse::InternalServerError().body(body),
          ResponseError::ErrorNotFound => HttpResponse::NotFound().body(body),
          ResponseError::ErrorUnauthorized => HttpResponse::Unauthorized().body(body),
          ResponseError::ErrorUnauthenticated => HttpResponse::Unauthorized().body(body),
          ResponseError::Infallible => unreachable!(),
        }
      }
    }
  }
}

#[derive(Debug)]
pub enum ResponseError {
  ErrorBadRequest,
  ErrorUnauthenticated,
  ErrorUnauthorized,
  ErrorNotFound,
  ErrorInternalServerError,
  Infallible,
}

impl From<ResponseError> for actix_web::Error {
  fn from(val: ResponseError) -> Self {
    match val {
      ResponseError::ErrorUnauthorized => ErrorForbidden(val.to_string()),
      ResponseError::ErrorNotFound => ErrorNotFound(val.to_string()),
      ResponseError::ErrorInternalServerError => ErrorInternalServerError(val.to_string()),
      ResponseError::Infallible => unreachable!(),
      ResponseError::ErrorUnauthenticated => ErrorUnauthorized(val.to_string()),
      ResponseError::ErrorBadRequest => ErrorBadRequest(val.to_string()),
    }
  }
}

impl From<sled::Error> for ResponseError {
  fn from(_: sled::Error) -> Self {
    Self::ErrorInternalServerError
  }
}

impl From<std::io::Error> for ResponseError {
  fn from(_: std::io::Error) -> Self {
    Self::ErrorInternalServerError
  }
}

impl Display for ResponseError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let error: String = match self {
      ResponseError::ErrorUnauthorized => {
        "You are not authorized to access the requested Ressource".into()
      }
      ResponseError::ErrorInternalServerError => "An unknown internal Server error occurred".into(),
      ResponseError::Infallible => unreachable!(),
      ResponseError::ErrorNotFound => "Requested resource can not be found.".into(),
      ResponseError::ErrorUnauthenticated => {
        "You are not authenticated. You must authenticate yourself to use this endpoint.".into()
      }
      ResponseError::ErrorBadRequest => "Bad request: Submitted data was malformed.".into(),
    };

    write!(f, "{}", error)
  }
}
impl std::error::Error for ResponseError {}

impl
  std::convert::From<
    CompositeSerializerError<std::convert::Infallible, AllocScratchError, SharedSerializeMapError>,
  > for ResponseError
{
  fn from(
    _: CompositeSerializerError<
      std::convert::Infallible,
      AllocScratchError,
      SharedSerializeMapError,
    >,
  ) -> Self {
    Self::ErrorInternalServerError
  }
}

impl From<actix_web::Error> for ResponseError {
  fn from(_e: actix_web::Error) -> Self {
    Self::ErrorInternalServerError
  }
}

impl From<Option<std::convert::Infallible>> for ResponseError {
  fn from(opt: Option<std::convert::Infallible>) -> Self {
    match opt {
      Some(_) => Self::Infallible,
      None => Self::ErrorNotFound,
    }
  }
}

impl
  From<
    CheckDeserializeError<
      CheckArchiveError<StructCheckError, DefaultValidatorError>,
      SharedDeserializeMapError,
    >,
  > for ResponseError
{
  fn from(
    _e: CheckDeserializeError<
      CheckArchiveError<StructCheckError, DefaultValidatorError>,
      SharedDeserializeMapError,
    >,
  ) -> Self {
    Self::ErrorInternalServerError
  }
}

impl From<PasswordValidationError> for ResponseError {
  fn from(e: PasswordValidationError) -> Self {
    match e {
      PasswordValidationError::DbAccessError(_) | PasswordValidationError::RkyvValidationError => {
        Self::ErrorInternalServerError
      }
      PasswordValidationError::InvalidPassword | PasswordValidationError::NoSuchUserError => {
        Self::ErrorBadRequest
      }
    }
  }
}

impl From<rkyv::de::deserializers::SharedDeserializeMapError> for ResponseError {
  fn from(_: rkyv::de::deserializers::SharedDeserializeMapError) -> Self {
    Self::ErrorInternalServerError
  }
}

impl From<std::convert::Infallible> for ResponseError {
  fn from(_: std::convert::Infallible) -> Self {
    unreachable!()
  }
}

impl From<PayloadError> for ResponseError {
  fn from(err: PayloadError) -> Self {
    match err {
      PayloadError::Incomplete(_) => ResponseError::ErrorBadRequest,
      PayloadError::EncodingCorrupted => ResponseError::ErrorBadRequest,
      PayloadError::Overflow => ResponseError::ErrorBadRequest,
      PayloadError::UnknownLength => ResponseError::ErrorBadRequest,
      PayloadError::Http2Payload(_) => ResponseError::ErrorBadRequest,
      PayloadError::Io(_) => ResponseError::ErrorInternalServerError,
      _ => ResponseError::ErrorInternalServerError,
    }
  }
}

impl From<ResponseError> for LoadError {
  fn from(e: ResponseError) -> Self {
    match e {
      ResponseError::ErrorBadRequest |
      ResponseError::ErrorUnauthenticated |
      ResponseError::ErrorUnauthorized |
      ResponseError::ErrorNotFound |
      ResponseError::Infallible => LoadError::Other(e.into()),
      ResponseError::ErrorInternalServerError => LoadError::Deserialization(e.into()),
    }
  }
}
