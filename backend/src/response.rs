use std::convert::TryInto;
use std::fmt::Display;
use std::ops::{Deref, FromResidual, Try};

use actix_session::storage::LoadError;
use actix_web::body::{BodySize, MessageBody};
use actix_web::error::{
  ErrorBadRequest, ErrorForbidden, ErrorInternalServerError, ErrorNotFound, ErrorUnauthorized,
  PayloadError,
};
use actix_web::{HttpResponse, Responder};
use bytecheck::StructCheckError;
use bytes::{BufMut, Bytes, BytesMut};
use rkyv::de::deserializers::{SharedDeserializeMap, SharedDeserializeMapError};
use rkyv::ser::serializers::{
  AlignedSerializer, AllocScratch, AllocScratchError, CompositeSerializer,
  CompositeSerializerError, FallbackScratch, HeapScratch, SharedSerializeMap,
  SharedSerializeMapError,
};
use rkyv::validation::validators::{CheckDeserializeError, DefaultValidatorError};
use rkyv::validation::CheckArchiveError;
use rkyv::{AlignedVec, Archive, Deserialize, Serialize};
use sled::IVec;
use zerocopy::AsBytes;

use crate::api::user::PasswordValidationError;

pub struct Response(pub Result<ResponseBody, ResponseError>);

impl Response {
  pub fn empty() -> Self {
    Self(Ok(ResponseBody::None))
  }
}

impl From<ResponseError> for Response {
  fn from(e: ResponseError) -> Self {
    Self(Err(e))
  }
}

impl<Data> From<Data> for Response
where
  Data: TryInto<ResponseBody, Error: Into<ResponseError>>,
{
  fn from(data: Data) -> Self {
    match data.try_into() {
      Ok(val) => Self(Ok(val)),
      Err(e) => Self(Err(e.into())),
    }
  }
}

impl Deref for Response {
  type Target = Result<ResponseBody, ResponseError>;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl AsRef<Result<ResponseBody, ResponseError>> for Response {
  fn as_ref(&self) -> &Result<ResponseBody, ResponseError> {
    &self.0
  }
}

// blanket implementation for ResponseErrors
impl<B, E> FromResidual<std::result::Result<B, E>> for Response
where
  B: Into<ResponseBody>,
  E: Into<ResponseError>,
{
  fn from_residual(residual: std::result::Result<B, E>) -> Self {
    match residual {
      Ok(body) => Self(Ok(body.into())),
      Err(error) => Self(Err(error.into())),
    }
  }
}

impl Try for Response {
  type Output = ResponseBody;

  type Residual = Result<std::convert::Infallible, ResponseError>;

  fn from_output(output: Self::Output) -> Self {
    Self(Ok(output))
  }

  fn branch(self) -> std::ops::ControlFlow<Self::Residual, Self::Output> {
    match self {
      Self(Ok(val)) => std::ops::ControlFlow::Continue(val),
      Self(Err(e)) => std::ops::ControlFlow::Break(Err(e)),
    }
  }
}

impl Responder for Response {
  type Body = ResponseBody;

  fn respond_to(self, _req: &actix_web::HttpRequest) -> HttpResponse<Self::Body> {
    match self.0 {
      Ok(body) => HttpResponse::Ok().message_body(body).unwrap(),
      Err(ResponseError::ErrorInternalServerError) => HttpResponse::InternalServerError()
        .message_body(ResponseBody::None)
        .unwrap(),
      Err(ResponseError::ErrorBadRequest) => HttpResponse::BadRequest()
        .message_body(ResponseBody::None)
        .unwrap(),
      Err(ResponseError::ErrorUnauthorized) => HttpResponse::Forbidden()
        .message_body(ResponseBody::None)
        .unwrap(),
      Err(ResponseError::ErrorUnauthenticated) => HttpResponse::Unauthorized()
        .message_body(ResponseBody::None)
        .unwrap(),
      Err(ResponseError::ErrorNotFound) => HttpResponse::NotFound()
        .message_body(ResponseBody::None)
        .unwrap(),
      Err(ResponseError::Infallible) => unreachable!(),
    }
  }
}

/// The return type for all Database-Webserver related API functions
pub enum ResponseBody {
  Archive(Vec<u8>),
  Numeric(u64),
  None,
}

impl From<std::convert::Infallible> for ResponseBody {
  fn from(_: std::convert::Infallible) -> Self {
    Self::None
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

/// Implements the DB-to-result conversion for sleds return-values. Semantics depend on @vec being
/// a value returned by `sled::Tree::get`
impl From<sled::Result<Option<IVec>>> for Response {
  fn from(vec: sled::Result<Option<IVec>>) -> Self {
    match vec {
      Ok(Some(val)) => Self(Ok(ResponseBody::from(val))),
      Ok(None) => Self(Err(ResponseError::ErrorNotFound)),
      Err(_e) => Self(Err(ResponseError::ErrorInternalServerError)),
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

impl MessageBody for ResponseBody {
  type Error = ResponseError;

  fn size(&self) -> actix_web::body::BodySize {
    match &self {
      ResponseBody::Archive(vec) => BodySize::Sized(vec.len() as u64),
      ResponseBody::None => actix_web::body::BodySize::None,
      ResponseBody::Numeric(_) => BodySize::Sized(std::mem::size_of::<u64>() as u64),
    }
  }

  fn poll_next(
    self: std::pin::Pin<&mut Self>,
    _cx: &mut std::task::Context<'_>,
  ) -> std::task::Poll<Option<Result<bytes::Bytes, Self::Error>>> {
    let body = self.get_mut();
    let val = std::task::Poll::Ready(match body {
      ResponseBody::Archive(val) => Some({
        // serve data in a single go and do not return anything else
        let val = std::mem::take(val);

        Ok(val.into())
      }),
      ResponseBody::None => None,
      ResponseBody::Numeric(num) => Some(Ok({
        let mut bytes = BytesMut::with_capacity(std::mem::size_of::<u64>());
        bytes.put_u64(*num);
        bytes.into()
      })),
    });
    drop(std::mem::replace(body, ResponseBody::None));

    val
  }

  fn try_into_bytes(self) -> Result<bytes::Bytes, Self>
  where
    Self: Sized,
  {
    match self {
      ResponseBody::Archive(vec) => Ok(vec.into()),
      ResponseBody::Numeric(num) => Ok(Bytes::from(num.to_be().as_bytes().to_owned())),
      ResponseBody::None => Ok(Bytes::new()),
    }
  }
}

impl<Data> From<&Data> for ResponseBody
where
  Data: Archive
    + Serialize<
      CompositeSerializer<
        AlignedSerializer<AlignedVec>,
        FallbackScratch<HeapScratch<4096>, AllocScratch>,
        SharedSerializeMap,
      >,
    >,
  <Data as Archive>::Archived: Deserialize<Data, SharedDeserializeMap>,
{
  fn from(data: &Data) -> Self {
    Self::Archive(rkyv::to_bytes(data).unwrap().to_vec())
  }
}

impl From<IVec> for ResponseBody {
  fn from(val: IVec) -> Self {
    Self::Archive(val.to_vec())
  }
}

impl From<u64> for ResponseBody {
  fn from(num: u64) -> Self {
    Self::Numeric(num)
  }
}

impl From<AlignedVec> for ResponseBody {
  fn from(vec: AlignedVec) -> Self {
    Self::Archive(vec.into())
  }
}
