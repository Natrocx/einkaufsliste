use actix_identity::Identity;
use actix_web::{get, post, put, web};
use einkaufsliste::model::article::Article;
use einkaufsliste::model::user::User;
use zerocopy::AsBytes;

use crate::db::RawRkyvStore;
use crate::response::Response;
use crate::util::identity_ext::IdentityExt;
use crate::DbState;

#[get("/article/{id}")]
pub(crate) async fn get_article_by_id(
  article_id: actix_web::web::Path<u64>,
  state: web::Data<DbState>,
  identity: Identity,
) -> Response {
  // check if the user has access:
  state.verify_access::<Article, User>(*article_id, identity.parse()?)?;

  state.article_db.get(article_id.as_bytes()).into()
}

#[put("/article")]
async fn update_article(
  article: Article,
  data: web::Data<DbState>,
  identity: Identity,
) -> Response {
  // before submitting parsed article to db we check the permissions:
  data.verify_access::<Article, User>(article.id, identity.parse()?)?;

  // reverse turbofish UwU
  <sled::Tree as RawRkyvStore<einkaufsliste::model::article::Article, 512>>::store_unlisted(
    &data.article_db,
    article.id,
    &article,
  )?;

  Response::empty()
}

#[post("/article")]
pub(crate) async fn store_article(
  mut article: Article,
  data: web::Data<DbState>,
  identity: Identity,
) -> Response {
  let user_id = identity.parse()?;

  // variable not inlineable because...???????
  let new_id = data.db.generate_id()?;
  article.id = new_id;

  <sled::Tree as RawRkyvStore<einkaufsliste::model::article::Article, 512>>::store_unlisted(
    &data.article_db,
    article.id,
    &article,
  )?;

  // since this is a new object we need to create an acl for this
  data.create_acl::<Article, User>(new_id, user_id)?;

  Response::empty()
}
