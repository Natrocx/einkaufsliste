

use actix_identity::Identity;
use actix_web::error::{ErrorBadRequest, ErrorInternalServerError, ErrorNotFound};
use actix_web::web::{
  Payload, {self},
};
use actix_web::{get, post};
use einkaufsliste::model::article::Article;
use einkaufsliste::model::item::Item;
use einkaufsliste::model::list::{FlatItemsList, List};
use einkaufsliste::model::requests::StoreItemAttached;
use einkaufsliste::model::user::User;
use sled::transaction::abort;
use zerocopy::AsBytes;

use crate::api::{new_generic_acl, preprocess_payload};
use crate::response::{Response, ResponseError};

use crate::{DbState, SessionState};

#[get("/item/{id}")]
pub async fn get_item_by_id(
  id: web::Path<String>,
  state: web::Data<DbState>,
  sessions: web::Data<SessionState>,
  identity: Identity,
) -> Response {
  let user_id = sessions.get_id_for_session(identity.id().map_err(|_| ResponseError::ErrorUnauthenticated)?)?;
  let idx = id.parse::<u64>().map_err(|_| ResponseError::ErrorBadRequest)?;

  state.verify_access::<Item, User>(idx, user_id)?;

  let db = &state.item_db;
  db.get(idx.as_bytes()).into()
}

//TODO: remove?
#[post("/item")]
pub async fn store_item_unattached(
  payload: Payload,
  state: web::Data<DbState>,
  sessions: web::Data<SessionState>,
  identity: Identity,
) -> Response {
  let user_id = sessions.get_id_for_session(identity.id().map_err(|_| ResponseError::ErrorUnauthorized)?)?;
  let aligned_bytes = preprocess_payload::<256>(payload).await?;

  let item = rkyv::from_bytes::<Item>(&aligned_bytes).map_err(ErrorBadRequest)?;
  state.item_db.insert(item.id.as_bytes(), aligned_bytes.as_slice())?;

  new_generic_acl::<Article, User>(item.id, user_id, &state.acl_db)?;

  Response::empty()
}

#[post("/item/attached")]
pub async fn store_item_attached(param: StoreItemAttached, state: web::Data<DbState>, identity: Identity) -> Response {
  let user_id = identity
    .id()
    .map_err(|_| ResponseError::ErrorUnauthorized)?
    .parse()
    .map_err(|_| ResponseError::ErrorBadRequest)?;

  state.verify_access::<List, User>(param.list_id, user_id)?;

  // insert item
  state.item_db.insert(
    param.item.id.as_bytes(),
    rkyv::to_bytes::<_, 128>(&param.item)
      .map_err(|_| ResponseError::ErrorBadRequest)?
      .as_slice(),
  )?;

  // TODO: migrate Errors
  state
    .list_db
    .transaction(|tx_id| {
      let current_value = match tx_id.get(param.list_id.as_bytes())? {
        Some(val) => val,
        None => abort(ErrorNotFound("No such list."))?,
      };
      let mut old_list = match unsafe { rkyv::from_bytes_unchecked::<List>(&current_value) } {
        Ok(val) => val,
        Err(e) => abort(ErrorInternalServerError(e))?,
      };
      old_list.items.push(param.item.id);

      let bytes = match rkyv::to_bytes::<_, 256>(&old_list) {
        Ok(bytes) => bytes,
        Err(e) => abort(ErrorInternalServerError(e))?,
      };

      match tx_id.insert(param.list_id.as_bytes(), bytes.as_bytes()) {
        Ok(_) => Ok(()),
        Err(e) => abort(ErrorInternalServerError(e)),
      }
    })
    .map_err(|e| match e {
      // return inner error if its a sled-user-error:
      sled::transaction::TransactionError::Abort(e) => e,
      // if not, something else went wrong, so generic server error:
      _ => ErrorInternalServerError(e),
    })?;

  // ensure that we can get items independent of their corresponding list
  state.copy_acl::<List, Item>(param.list_id, param.item.id)?;

  Response::empty()
}

#[get("/itemList/{id}/flat")]
pub async fn get_item_list_flat(id: web::Path<String>, state: web::Data<DbState>, identity: Identity) -> Response {
  let user_id = identity
    .id()
    .map_err(|_| ResponseError::ErrorInternalServerError)?
    .parse()
    .map_err(|_| ResponseError::ErrorInternalServerError)?;
  let list_id = id.parse::<u64>().map_err(ErrorBadRequest)?;

  state.verify_access::<List, User>(list_id, user_id)?;

  let list_bytes = state
    .list_db
    .get(list_id.as_bytes())?
    .ok_or(ResponseError::ErrorNotFound)?;

  let list = unsafe {
    // This should be safe, as objects are checked before they are inserted into the db
    rkyv::from_bytes_unchecked::<List>(list_bytes.as_bytes())?
  };

  let item_db = &state.item_db;
  let mut building_success = true;
  let vec = list
    .items
    .iter()
    .map(|idx| item_db.get(idx.as_bytes()))
    .filter_map(|result| match result {
      Ok(val) => val,
      Err(_e) => {
        building_success = false;
        None
      }
    })
    .map(|ivec| unsafe { rkyv::from_bytes_unchecked::<Item>(ivec.as_bytes()).unwrap() })
    .collect::<Vec<_>>();

  let flat_items_list = FlatItemsList::from_list_and_items(list, vec);

  rkyv::to_bytes::<_, 256>(&flat_items_list)?.into()
}

#[post("/itemList")]
pub(crate) async fn store_item_list(mut param: List, state: web::Data<DbState>, identity: Identity) -> Response {
  let user_id = identity
    .id()
    .map_err(|_| ResponseError::ErrorUnauthenticated)?
    .parse()
    .map_err(|_| ResponseError::ErrorInternalServerError)?;

  let db = &state.list_db;

  // while an id is provided with the archived data, we do not use this id, given that the client does not know the new id as this is DB-managed information
  let id = state.db.generate_id()?;

  param.id = id;
  db.insert(id.as_bytes(), rkyv::to_bytes::<_, 64>(&param)?.as_bytes())?;

  new_generic_acl::<List, User>(id, user_id, &state.acl_db)?;

  // we need to return the newly generated id to the client
  id.into()
}
