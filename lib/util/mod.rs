/*
 * Adapted from actix documentation
*/
#[cfg(feature = "backend")]
const MAX_SIZE: usize = 4 * 1024 * 1024 * 1024; // max payload is 4mb - this should be sufficient for lists up to 20k items and all reasonably compressed images
#[cfg(feature = "backend")]
pub async fn collect_from_payload(
  mut payload: ::actix_web::dev::Payload,
) -> ::std::result::Result<::std::vec::Vec<u8>, ::actix_web::Error> {
  use actix_web::error::ErrorBadRequest;
  use futures::StreamExt;

  let mut bytes = ::std::vec::Vec::new();

  while let Some(chunk) = payload.next().await {
    let chunk = chunk?;
    if (bytes.len() + chunk.len()) > MAX_SIZE {
      return Err(ErrorBadRequest("Request too long."));
    }
    bytes.extend_from_slice(&chunk);
  }

  Ok(bytes)
}
