/**
Implement the [`actix_web::FromRequest`] trait for any type serializable with `rkyv::from_bytes`. The entire payload has to be processed in one "transaction" (using this extractor will prevent you from using other extractors operating on the payload).

A blanket implementation is typically not possible due to orphaning restrictions. Use this macro to manually create a monomorphised implementation.
*/
#[macro_export]
#[allow(clippy::crate_in_macro_def)] // intended behaviour due to trait impl restrictions
macro_rules! impl_from_request {
  ($param:ty) => {
    #[cfg(feature = "backend")]
    impl actix_web::FromRequest for $param {
      type Error = ::actix_web::Error;

      type Future = ::futures::future::LocalBoxFuture<'static, std::result::Result<Self, Self::Error>>;

      fn from_request(req: &::actix_web::HttpRequest, payload: &mut ::actix_web::dev::Payload) -> Self::Future {
        let payload = payload.take();

        Box::pin(async move {
          let bytes = match crate::util::collect_from_payload(payload).await {
            Ok(val) => val,
            Err(e) => {
              ::log::debug!("Rejecting request due to error while collecting from actix_web Payload: {e}");
              return Err(::actix_web::error::ErrorInternalServerError(e.to_string()));
            }
          };

          let val = match ::rkyv::from_bytes::<$param>(&*bytes) {
            Ok(val) => val,
            Err(e) => {
              ::log::debug!("Rejecting request due to error parsing archived bytes: {e}");
              return Err(::actix_web::error::ErrorBadRequest(e.to_string()));
            }
          };

          Ok(val)
        })
      }
    }
  };
}
