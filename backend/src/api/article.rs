use std::ops::Try;

use actix_web::{get, post, put, web};
use einkaufsliste::model::article::Article;
use einkaufsliste::model::user::User;

use crate::db::RawRkyvStore;
use crate::response::Response;
use crate::util::identity_ext::AuthenticatedUser;
use crate::DbState;

#[get("/article/{id}")]
pub(crate) async fn get_article_by_id(
  article_id: actix_web::web::Path<u64>,
  state: web::Data<DbState>,
  identity: AuthenticatedUser,
) -> Response<Article> {
  // check if the user has access:
  state.verify_access::<Article, User>(*article_id, identity.id)?;

  let article = unsafe {
    <sled::Tree as RawRkyvStore<Article, 4096>>::get_unchecked(&state.article_db, *article_id)?
  };
  Response::from_output(article)
}

#[put("/article")]
async fn update_article(
  article: Article,
  data: web::Data<DbState>,
  identity: AuthenticatedUser,
) -> Response<()> {
  data.verify_access::<Article, User>(article.id, identity.id)?;

  data.store_unlisted(&article, article.id)?;
  Response::empty()
}

#[post("/article")]
pub(crate) async fn store_article(
  mut article: Article,
  data: web::Data<DbState>,
  identity: AuthenticatedUser,
) -> Response<u64> {
  let user_id = identity.id;

  // variable not inlineable because...??????? fuck you
  let new_id = data.db.generate_id()?;
  article.id = new_id;

  data.store_unlisted(&article, new_id)?;

  // since this is a new object we need to create an acl for this
  data.create_acl::<Article, User>(new_id, user_id)?;

  Response::from(new_id)
}
