/*
 * Adapted from actix documentation
 */
#[cfg(feature = "backend")]
const MAX_SIZE: usize = 32_768; // max payload size in bytes
#[cfg(feature = "backend")]
pub async fn collect_from_payload(
  mut payload: ::actix_web::dev::Payload,
) -> ::std::result::Result<::bytes::Bytes, ::actix_web::Error> {
  use actix_web::error::ErrorBadRequest;
  use futures::StreamExt;

  let mut bytes = ::actix_web::web::BytesMut::new();

  while let Some(chunk) = payload.next().await {
    let chunk = chunk?;
    if (bytes.len() + chunk.len()) > MAX_SIZE {
      return Err(ErrorBadRequest("Request too long."));
    }
    bytes.extend_from_slice(&chunk);
  }

  Ok(bytes.into())
}
