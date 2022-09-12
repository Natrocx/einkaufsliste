use bytes::Bytes;
use futures::StreamExt;

use crate::response::ResponseError;

/*
 * Adapted from actix documentation
 */
const MAX_SIZE: usize = 32_768; // max payload size in bytes
pub async fn collect_from_payload(mut payload: actix_web::web::Payload) -> Result<Bytes, ResponseError> {
  let mut bytes = actix_web::web::BytesMut::new();

  while let Some(chunk) = payload.next().await {
    let chunk = chunk?;
    if (bytes.len() + chunk.len()) > MAX_SIZE {
      return Err(ResponseError::ErrorBadRequest);
    }
    bytes.extend_from_slice(&chunk);
  }

  Ok(bytes.into())
}
