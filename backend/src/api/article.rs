use actix_web::error::{ErrorBadRequest, ErrorInternalServerError};
use actix_web::web::{self};
use actix_web::{get, post, Error, HttpResponse, Result};
use einkaufsliste::model::article::{ArchivedArticle, Article};
use zerocopy::AsBytes;

use crate::util::collect_from_payload;
use crate::DbState;

#[get("/article/test")]
pub(crate) async fn get_example_article(state: web::Data<DbState>) -> Result<HttpResponse, Error> {
  let article = Article {
    id: state.db.generate_id().unwrap(),
    name: "name".to_owned(),
    description: Some("description is present".to_owned()),
    image_id: None,
    shops: None,
  };

  let encoded = rkyv::to_bytes::<_, 384>(&article).unwrap();

  Ok(HttpResponse::Ok().body(encoded.as_bytes().to_owned()))
}

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

  if buffer.len() < std::mem::size_of::<ArchivedArticle>() {
    return Err(ErrorBadRequest("Incomplete data"));
  }
  let archived = match rkyv::check_archived_root::<Article>(buffer) {
    Ok(val) => val,
    Err(err) => return Err(actix_web::error::ErrorBadRequest(err.to_string())),
  };
  let db = &data.article_db;

  db.insert::<&[u8], &[u8]>(archived.id.value().as_bytes(), buffer)
    .map_err(|_| ErrorInternalServerError("Failure storing value"))?;

  Ok(HttpResponse::Created().body(""))
}
