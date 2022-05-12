
use actix_web::error::{ErrorBadRequest, ErrorInternalServerError, ErrorNotFound};
use actix_web::web::{self};
use actix_web::{get, post, HttpResponse};
use bytes::Bytes;
use einkaufsliste::model::*;
use futures::StreamExt;
use zerocopy::AsBytes;

use crate::DbState;


#[get("/item/{id}")]
pub async fn get_item_by_id(
  id: web::Path<String>,
  state: web::Data<DbState>,
) -> actix_web::Result<HttpResponse, actix_web::Error> {
  let idx = id.parse::<u64>().map_err(|_| ErrorBadRequest("Invalid id"))?;

  let db = &state.item_db;
  let data = db
    .get(idx.as_bytes())
    .map_err(|_| ErrorInternalServerError("Error accessing database."))?
    .ok_or_else(|| ErrorNotFound(""))?
    .as_bytes()
    .to_owned();

  Ok(HttpResponse::Ok().body(data))
}
