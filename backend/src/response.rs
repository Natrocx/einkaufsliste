use std::borrow::Borrow;

use std::fmt::Display;




use actix_web::body::{BodySize, MessageBody};
use actix_web::error::{ErrorInternalServerError, ErrorNotFound, ErrorUnauthorized};
use actix_web::{HttpResponse};
use bytes::{BufMut, BytesMut};
use rkyv::de::deserializers::SharedDeserializeMap;
use rkyv::ser::serializers::{
  AlignedSerializer, AllocScratch, AllocScratchError, CompositeSerializer, CompositeSerializerError, FallbackScratch,
  HeapScratch, SharedSerializeMap, SharedSerializeMapError,
};

use rkyv::{AlignedVec, Archive, Deserialize, Serialize};
use sled::IVec;

pub struct Response(Result<ResponseBody, ResponseError>);

/// The return type for all Database-Webserver related API functions
pub enum ResponseBody {
  Archive(Vec<u8>),
  Numeric(u64),
  None,
}

#[derive(Debug)]
pub enum ResponseError {
  ErrorUnauthorized,
  ErrorNotFound,
  ErrorInternalServerError,
  Infallible,
}

impl Into<actix_web::Error> for ResponseError {
  fn into(self) -> actix_web::Error {
    match self {
      ResponseError::ErrorUnauthorized => ErrorUnauthorized(self.to_string()),
      ResponseError::ErrorNotFound => ErrorNotFound(self.to_string()),
      ResponseError::ErrorInternalServerError => ErrorInternalServerError(self.to_string()),
      ResponseError::Infallible => unreachable!(),
    }
  }
}

impl Display for ResponseError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let error: String = match self {
      ResponseError::ErrorUnauthorized => "You are not authorized to access the requested Ressource".into(),
      ResponseError::ErrorInternalServerError => "An unknown internal Server error occurred".into(),
      ResponseError::Infallible => unreachable!(),
      ResponseError::ErrorNotFound => "Requested resource can not be found.".into(),
    };

    write!(f, "{}", error)
  }
}
impl std::error::Error for ResponseError {}

impl std::convert::From<CompositeSerializerError<std::convert::Infallible, AllocScratchError, SharedSerializeMapError>>
  for ResponseError
{
  fn from(_: CompositeSerializerError<std::convert::Infallible, AllocScratchError, SharedSerializeMapError>) -> Self {
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

pub fn sled_to_response(sled_val: sled::Result<Option<IVec>>) -> Result<HttpResponse, ResponseError> {
  match sled_val {
    Ok(Some(val)) => Ok(HttpResponse::Ok().body(ResponseBody::from(val))),
    Ok(None) => Err(ResponseError::ErrorNotFound),
    Err(_) => Err(ResponseError::ErrorInternalServerError),
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
    std::task::Poll::Ready(match self.get_mut() {
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
    })
  }
}

impl<Data> From<&Data> for ResponseBody
where
  Data: Archive
    + Serialize<
      CompositeSerializer<
        AlignedSerializer<AlignedVec>,
        FallbackScratch<HeapScratch<256>, AllocScratch>,
        SharedSerializeMap,
      >,
    >,
  <Data as Archive>::Archived: Deserialize<Data, SharedDeserializeMap>,
{
  fn from(data: &Data) -> Self {
    Self::Archive(rkyv::to_bytes(data.borrow()).unwrap().to_vec())
  }
}

impl From<IVec> for ResponseBody {
  fn from(val: IVec) -> Self {
    Self::Archive(val.to_vec())
  }
}
