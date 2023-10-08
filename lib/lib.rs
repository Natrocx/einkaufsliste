use std::cell::RefCell;

use model::list::List;
use rkyv::de::deserializers::SharedDeserializeMap;
use rkyv::ser::serializers::AllocSerializer;
use rkyv::validation::validators::DefaultValidator;

pub mod model;
#[cfg(feature = "backend")]
pub mod util;

pub trait ApiObject<'a>:
  rkyv::Archive + rkyv::Serialize<AllocSerializer<4096>> + serde::Serialize + serde::Deserialize<'a>
where
  Self::Archived: rkyv::Deserialize<Self, SharedDeserializeMap> + bytecheck::CheckBytes<DefaultValidator<'a>>,
{
}

impl<'a> ApiObject<'a> for () {}

#[derive(Debug, Default, Clone, Copy)]
pub enum Encoding {
  JSON,
  #[default]
  Rkyv,
}

#[cfg(feature = "backend")]
use actix_web::http::header::HeaderValue;

#[cfg(feature = "backend")]
impl From<Option<&HeaderValue>> for Encoding {
  fn from(value: Option<&HeaderValue>) -> Self {
    match value {
      Some(val) if val == "application/json" => Self::JSON,
      _ => Self::default(),
    }
  }
}

#[cfg(feature = "backend")]
impl From<&HeaderValue> for Encoding {
  fn from(value: &HeaderValue) -> Self {
    match value {
      val if val == "application/json" => Self::JSON,
      _ => Self::default(),
    }
  }
}

#[cfg(feature = "backend")]
impl actix_web::http::header::TryIntoHeaderValue for Encoding {
  type Error = actix_web::http::header::InvalidHeaderValue;

  fn try_into_value(self) -> Result<HeaderValue, Self::Error> {
    match self {
      Self::JSON => Ok(HeaderValue::from_static("application/json")),
      Self::Rkyv => Ok(HeaderValue::from_static("application/rkyv")),
    }
  }
}

#[cfg(feature = "backend")]
impl<T: ApiObject<'static>> ApiObject<'static> for Vec<T> where
  <T as rkyv::Archive>::Archived: rkyv::Deserialize<T, rkyv::de::deserializers::SharedDeserializeMap>
    + bytecheck::CheckBytes<rkyv::validation::validators::DefaultValidator<'static>>
{
}

#[cfg(feature = "backend")]
impl ApiObject<'static> for u64 {}
#[cfg(feature = "backend")]
impl ApiObject<'static> for u32 {}
#[cfg(feature = "backend")]
impl ApiObject<'static> for u16 {}
#[cfg(feature = "backend")]
impl ApiObject<'static> for u8 {}
#[cfg(feature = "backend")]
impl ApiObject<'static> for i64 {}
#[cfg(feature = "backend")]
impl ApiObject<'static> for i32 {}
#[cfg(feature = "backend")]
impl ApiObject<'static> for i16 {}
#[cfg(feature = "backend")]
impl ApiObject<'static> for i8 {}

thread_local! {
  static RKYV_DESERIALIZER: RefCell<SharedDeserializeMap> = RefCell::new(SharedDeserializeMap::new());
}

// #[cfg(feature = "backend")]
// pub fn decode<T>(encoding: Encoding, bytes: &[u8]) -> Result<T, ::actix_web::Error>
// where
//   T: ApiObject<'static> + Clone,
//   <T as rkyv::Archive>::Archived: rkyv::Deserialize<T, SharedDeserializeMap>
//     + rkyv::CheckBytes<rkyv::validation::validators::DefaultValidator<'static>>,
// {
//   use actix_web::error::ErrorBadRequest;
//   use rkyv::de::deserializers;
//   use rkyv::Deserialize;

//   match encoding {
//     Encoding::JSON => Ok(serde_json::from_slice(&bytes)?),
//     Encoding::Rkyv => {
//       let val = RKYV_DESERIALIZER.with_borrow_mut(|de| {
//         let archived = rkyv::check_archived_root::<T>(bytes).map_err(|e| ErrorBadRequest(e.to_string()));

//         archived.map(|archived| archived.deserialize(de).unwrap().clone())
//       });
//       val
//     }
//   }
// }


/**
Implement the [`actix_web::FromRequest`] trait for any type serializable with `rkyv::from_bytes`. The entire payload has to be processed in one "transaction" (using this extractor will prevent you from using other extractors operating on the payload).

A blanket generic implementation is typically not possible due to orphaning restrictions. Use this macro to manually create a monomorphised implementation.
*/
#[macro_export]
#[cfg(feature = "backend")]
macro_rules! impl_from_request {
  ($param:ty) => {
    #[automatically_derived]
        impl $crate::ApiObject<'static> for $param {}

        #[automatically_derived]
        impl actix_web::FromRequest for $param {
          type Error = ::actix_web::Error;

          type Future = ::futures::future::LocalBoxFuture<'static, std::result::Result<Self, Self::Error>>;

          fn from_request(_req: &::actix_web::HttpRequest, payload: &mut ::actix_web::dev::Payload) -> Self::Future {
            let payload = payload.take();
            let payload_encoding = $crate::Encoding::from(_req.headers().get("Content-Type"));
            ::tracing::debug!("Payload encoding: {:?}", payload_encoding);

            Box::pin(async move {
              let bytes = match $crate::util::collect_from_payload(payload).await {
                Ok(val) => ::std::sync::Arc::new(val),
                Err(e) => {
                  ::log::debug!("Rejecting request due to error while collecting from actix_web Payload: {e}");
                  return Err(::actix_web::error::ErrorInternalServerError(e.to_string()));
                }
              };

              let val: $param = match payload_encoding {
                $crate::Encoding::JSON => match serde_json::from_slice(&bytes) {
                  Ok(val) => val,
                  Err(e) => {
                    ::log::debug!("Rejecting request due to error while deserializing JSON: {e}");
                    return Err(::actix_web::error::ErrorBadRequest(e.to_string()));
                  }
                },
                $crate::Encoding::Rkyv => match rkyv::from_bytes(&bytes) {
                  Ok(val) => val,
                  Err(e) => {
                    ::log::debug!("Rejecting request due to error while deserializing Rkyv: {e}");
                    return Err(::actix_web::error::ErrorBadRequest(e.to_string()));
                  }
                },
              };
              Ok(val)
            })
          }
        }
  };
}
