use actix_identity::Identity;
use actix_web::error::{ErrorBadRequest, ErrorInternalServerError};

use actix_web::{get, post, put, web, Result};
use einkaufsliste::model::article::Article;
use einkaufsliste::model::user::User;
use einkaufsliste::model::{AccessControlList, Identifiable};
use zerocopy::AsBytes;

use crate::api::{new_generic_acl, store_in_db};
use crate::response::{Response, ResponseError};
use crate::util::identity_ext::IdentityExt;
use crate::{DbState};

#[get("/article/{id}")]
async fn get_article_by_id(
  article_id: actix_web::web::Path<u64>,
  state: web::Data<DbState>,
  identity: Identity,
) -> Response {
  // check if the user has access:
  check_article_acl(*article_id, &state, identity)?;

  state.article_db.get(article_id.as_bytes()).into()
}

#[put("/article")]
async fn update_article(article: Article, data: web::Data<DbState>, identity: Identity) -> Response {
  // before submitting parsed article to db we check the permissions:
  check_article_acl(article.id, &data, identity)?;

  store_in_db::<Article, 384>(article.id, article, &data.article_db)?;

  Response::empty()
}

#[post("/article")]
async fn store_article(mut article: Article, data: web::Data<DbState>, identity: Identity) -> Response {
  let user_id = identity.parse()?;

  article.id = data.db.generate_id()?;
  let db = &data.article_db;

  db.insert(article.id.as_bytes(), rkyv::to_bytes::<_, 384>(&article)?.as_slice())?;

  // since this is a new object we need to create an acl for this
  new_generic_acl::<Article, User>(article.id, user_id, &data.article_db)?;

  Response::empty()
}

fn check_article_acl(
  article_id: <Article as Identifiable>::Id,
  state: &DbState,
  identity: Identity,
) -> Result<(), ResponseError> {
  let user_id = identity.parse()?;

  let acl = rkyv::from_bytes::<AccessControlList<Article, User>>(
    state
      .acl_db
      .get(article_id.as_bytes())
      .map_err(ErrorInternalServerError)?
      .ok_or_else(|| ErrorBadRequest("No acl for this object."))?
      .as_bytes(),
  )
  .map_err(ErrorInternalServerError)?;

  if acl.owner == user_id || acl.allowed_user_ids.contains(&user_id) {
    Ok(())
  } else {
    Err(ResponseError::ErrorUnauthorized)
  }
}
