use actix_web::error::{ErrorBadRequest, ErrorInternalServerError};
use actix_web::web;
use actix_web::{get, post, Error, HttpResponse, Result};
use einkaufsliste::model::article::Article;
use zerocopy::AsBytes;

use crate::util::collect_from_payload;
use crate::DbState;

#[get("/article/{id}")]
async fn get_article_by_id(id: actix_web::web::Path<String>, state: web::Data<DbState>) -> Result<HttpResponse, Error> {
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
async fn store_article(body: actix_web::web::Payload, data: web::Data<DbState>) -> Result<HttpResponse, Error> {
  let params = collect_from_payload(body).await?;
  let buffer = params.as_bytes();

  let mut archived = rkyv::from_bytes::<Article>(buffer).map_err(ErrorBadRequest)?;
  archived.id = data.db.generate_id().map_err(ErrorInternalServerError)?;
  let db = &data.article_db;

  db.insert::<&[u8], &[u8]>(
    archived.id.as_bytes(),
    rkyv::to_bytes::<_, 384>(&archived)
      .map_err(ErrorInternalServerError)?
      .as_slice(),
  )
  .map_err(|_| ErrorInternalServerError("Failure storing value"))?;

  Ok(HttpResponse::Created().body(""))
}
