
use actix_web::error::{ErrorBadRequest, ErrorInternalServerError, ErrorNotFound};
use actix_web::web::{self};
use actix_web::{get, post, HttpResponse};
use bytes::Bytes;
use einkaufsliste::model::*;
use futures::StreamExt;
use log::debug;
use zerocopy::AsBytes;

use crate::DbState;

#[get("/article/test")]
pub(crate) async fn get_example_article(state: web::Data<DbState>) -> actix_web::Result<HttpResponse, actix_web::Error> {
  let article = Article {
    id: state.db.generate_id().unwrap(),
    name: "Warum so hässlich????".to_owned(),
    description: Some("Ich hasse Kinder die wichser".to_owned()),
    image_id: None,
    shops: None,
  };
  debug!("Sending object with unencoded size: {} bytes", std::mem::size_of_val(&article));

  let encoded = rkyv::to_bytes::<_, 384>(&article).unwrap();
  let test = rkyv::from_bytes::<Article>(encoded.as_bytes()).unwrap();
  
  debug!("Sending object with in memory size of {} and wire size {}: {:?}", std::mem::size_of_val(&article), std::mem::size_of_val(encoded.as_bytes()), &test);

  Ok(HttpResponse::Ok().body(encoded.as_bytes().to_owned()))
}

#[get("/article/{id}")]
async fn get_article_by_id(
  id: actix_web::web::Path<String>,
  state: web::Data<DbState>,
) -> actix_web::Result<HttpResponse, actix_web::Error> {
  let value = state
    .article_db
    .get(
      id.as_ref()
        .parse::<u64>()
        .map_err(|_| actix_web::error::ErrorBadRequest("Invalid id"))?
        .as_bytes(),
    )
    .map_err(|_| actix_web::error::ErrorNotFound("No such article"))?;

  // we assume that the data is fine, since we validated it before storing
  let data = match value {
    Some(vec) => vec,
    None => return Err(actix_web::error::ErrorNotFound("No such article")),
  }
  .as_bytes()
  .to_owned();
  Ok(HttpResponse::Ok().body(data))
}

#[post("/article")]
async fn store_article(
  body: actix_web::web::Payload,
  data: web::Data<DbState>,
) -> actix_web::Result<HttpResponse, actix_web::Error> {
  let params = collect_from_payload(body).await?;
  let buffer = params.as_bytes();

  if buffer.len() < std::mem::size_of::<ArchivedArticle>() {
    return Err(actix_web::error::ErrorBadRequest("Incomplete data"));
  }
  let archived = match rkyv::check_archived_root::<Article>(buffer) {
    Ok(val) => val,
    Err(err) => return Err(actix_web::error::ErrorBadRequest(err.to_string())),
  };
  let db = &data.article_db;

  db.insert::<&[u8], &[u8]>(archived.id.value().as_bytes(), buffer)
    .map_err(|_| ErrorInternalServerError("Failure storing value"))?;
  log::debug!("{}", archived.name);

  Ok(HttpResponse::Ok().body(""))
}

/*
 * Adapted from actix documentation
 */
const MAX_SIZE: usize = 32_768; // max payload size in bytes
async fn collect_from_payload(mut payload: web::Payload) -> actix_web::Result<Bytes, actix_web::Error> {
  let mut bytes = web::BytesMut::new();

  while let Some(chunk) = payload.next().await {
    let chunk = chunk?;
    if (bytes.len() + chunk.len()) > MAX_SIZE {
      return Err(actix_web::error::ErrorBadRequest("overflow"));
    }
    bytes.extend_from_slice(&chunk);
  }

  Ok(bytes.into())
}