use actix_identity::Identity;
use actix_web::*;
use einkaufsliste::model::shop::Shop;
use einkaufsliste::model::user::User;

use crate::response::Response;
use crate::util::identity_ext::AuthenticatedUser;
use crate::DbState;

#[get("/shop/{id}")]
pub async fn get_shop(
  id: web::Path<u64>,
  state: web::Data<DbState>,
  user: AuthenticatedUser,
) -> Response<Shop> {
  state.verify_access::<Shop, User>(*id, user.id)?;

  let shop: Shop = state.get_unchecked(*id)?;

  Response::from(shop)
}

#[post("/shop")]
pub async fn store_shop(
  mut param: Shop,
  state: web::Data<DbState>,
  user: AuthenticatedUser,
) -> Response<u64> {
  let id = state.db.generate_id()?;
  param.id = id;

  state.store_unlisted(&param, id)?;

  state.create_acl::<Shop, User>(id, user.id)?;

  id.into()
}
