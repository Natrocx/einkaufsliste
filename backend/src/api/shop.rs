use actix_identity::Identity;
use actix_web::*;
use einkaufsliste::model::shop::Shop;
use einkaufsliste::model::user::User;
use zerocopy::AsBytes;

use crate::response::Response;
use crate::util::identity_ext::IdentityExt;
use crate::DbState;

#[get("/shop/{id}")]
pub async fn get_shop(id: web::Path<u64>, state: web::Data<DbState>, identity: Identity) -> Response {
  let user_id = identity.parse()?;
  state.verify_access::<Shop, User>(*id, user_id)?;

  state.shop_db.get(id.as_bytes()).into()
}

#[post("/shop")]
pub async fn store_shop(mut param: Shop, state: web::Data<DbState>, identity: Identity) -> Response {
  let user_id = identity.parse()?;

  let id = state.db.generate_id()?;
  param.id = id;

  let shop_as_bytes = rkyv::to_bytes::<_, 128>(&param)?;
  state.shop_db.insert(id.as_bytes(), &*shop_as_bytes)?;

  state.create_acl::<Shop, User>(id, user_id)?;

  id.into()
}
