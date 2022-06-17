use actix_identity::Identity;
use actix_web::error::{ErrorBadRequest, ErrorInternalServerError, ErrorNotFound};
use actix_web::*;
use einkaufsliste::model::shop::Shop;
use einkaufsliste::model::user::User;
use zerocopy::AsBytes;

use crate::api::new_generic_acl;
use crate::{DbState, SessionState};

#[get("/shop/{id}")]
pub async fn get_shop(
  id: web::Path<String>,
  state: web::Data<DbState>,
  sessions: web::Data<SessionState>,
  identity: Identity,
) -> Result<HttpResponse, Error> {
  let user_id = sessions.get_id_for_identity(identity)?;
  let shop_id = id.parse::<u64>().map_err(ErrorBadRequest)?;

  state.verify_access::<Shop, User>(shop_id, user_id)?;

  let shop = state
    .shop_db
    .get(shop_id.as_bytes())
    .map_err(ErrorInternalServerError)?
    .ok_or_else(|| ErrorNotFound("No such item list."))?
    .as_bytes()
    .to_owned();

  Ok(HttpResponse::Ok().body(shop))
}

#[post("/shop")]
pub async fn store_shop(
  payload: web::Payload,
  state: web::Data<DbState>,
  sessions: web::Data<SessionState>,
  identity: Identity,
) -> Result<HttpResponse, Error> {
  let user_id = sessions.get_id_for_identity(identity)?;
  let shop_db = &state.shop_db;

  let kek = crate::util::collect_from_payload(payload)
    .await
    .map_err(ErrorBadRequest)?;

  let id = state.db.generate_id().map_err(ErrorInternalServerError)?;
  let mut shop = rkyv::from_bytes::<Shop>(&kek).map_err(ErrorBadRequest)?;
  shop.id = id;

  let shop_as_bytes = rkyv::to_bytes::<_, 128>(&shop).map_err(ErrorInternalServerError)?;
  shop_db
    .insert(id.as_bytes(), shop_as_bytes.as_slice())
    .map_err(ErrorInternalServerError)?;

  new_generic_acl::<Shop, User>(id, user_id, &state.acl_db)?;

  Ok(HttpResponse::Created().body(id.to_be_bytes().to_vec()))
}
