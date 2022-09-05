use actix_identity::Identity;

use actix_web::*;
use einkaufsliste::model::shop::Shop;
use einkaufsliste::model::user::User;
use zerocopy::AsBytes;

use crate::api::new_generic_acl;
use crate::response::{Response, ResponseError};
use crate::{DbState, SessionState};

#[get("/shop/{id}")]
pub async fn get_shop(
  id: web::Path<String>,
  state: web::Data<DbState>,
  sessions: web::Data<SessionState>,
  identity: Identity,
) -> Response {
  let user_id = sessions.get_id_for_identity(&identity)?;
  let shop_id = id.parse::<u64>().map_err(|_| ResponseError::ErrorBadRequest)?;

  state.verify_access::<Shop, User>(shop_id, user_id)?;

  state.shop_db.get(shop_id.as_bytes()).into()
}

#[post("/shop")]
pub async fn store_shop(
  payload: web::Payload,
  state: web::Data<DbState>,
  sessions: web::Data<SessionState>,
  identity: Identity,
) -> Response {
  let user_id = sessions.get_id_for_identity(&identity)?;
  let shop_db = &state.shop_db;

  let bytes = crate::util::collect_from_payload(payload).await?;

  let id = state.db.generate_id()?;
  let mut shop = rkyv::from_bytes::<Shop>(&bytes).map_err(|_| ResponseError::ErrorBadRequest)?;
  shop.id = id;

  let shop_as_bytes = rkyv::to_bytes::<_, 128>(&shop)?;
  shop_db.insert(id.as_bytes(), shop_as_bytes.as_slice())?;

  new_generic_acl::<Shop, User>(id, user_id, &state.acl_db)?;

  id.into()
}
