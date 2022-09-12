use actix_identity::Identity;
use actix_web::*;
use einkaufsliste::model::shop::Shop;
use einkaufsliste::model::user::User;
use zerocopy::AsBytes;

use crate::api::new_generic_acl;
use crate::response::{Response, ResponseError};
use crate::{DbState};

#[get("/shop/{id}")]
pub async fn get_shop(id: web::Path<u64>, state: web::Data<DbState>, identity: Identity) -> Response {
  let user_id = identity
    .id()
    .map_err(|_| ResponseError::ErrorUnauthorized)?
    .parse()
    .map_err(|_| ResponseError::ErrorBadRequest)?;

  state.verify_access::<Shop, User>(*id, user_id)?;

  state.shop_db.get(id.as_bytes()).into()
}

#[post("/shop")]
pub async fn store_shop(mut param: Shop, state: web::Data<DbState>, identity: Identity) -> Response {
  let user_id = identity
    .id()
    .map_err(|_| ResponseError::ErrorUnauthorized)?
    .parse()
    .map_err(|_| ResponseError::ErrorBadRequest)?;

  let id = state.db.generate_id()?;
  param.id = id;

  let shop_as_bytes = rkyv::to_bytes::<_, 128>(&param)?;
  state.shop_db.insert(id.as_bytes(), &*shop_as_bytes)?;

  new_generic_acl::<Shop, User>(id, user_id, &state.acl_db)?;

  id.into()
}
