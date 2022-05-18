use actix_web::error::{ErrorBadRequest, ErrorInternalServerError, ErrorNotFound};
use actix_web::web::{
  Payload, {self},
};
use actix_web::{get, post, Error, HttpResponse, Result};
use einkaufsliste::model::article::ArchivedArticle;
use einkaufsliste::model::item::Item;
use einkaufsliste::model::list::{FlatItemsList, List};
use einkaufsliste::model::requests::StoreItemAttached;

use rkyv::AlignedVec;
use sled::transaction::abort;
use zerocopy::AsBytes;

use crate::util::{self, collect_from_payload};
use crate::DbState;

#[get("/item/{id}")]
pub async fn get_item_by_id(
  id: web::Path<String>,
  state: web::Data<DbState>,
) -> actix_web::Result<HttpResponse, actix_web::Error> {
  let idx = id.parse::<u64>().map_err(|_| ErrorBadRequest("Invalid id"))?;

  let db = &state.item_db;
  let data = db
    .get(idx.as_bytes())
    .map_err(|_| ErrorInternalServerError("Error accessing database."))?
    .ok_or_else(|| ErrorNotFound(""))?
    .as_bytes()
    .to_owned();

  Ok(HttpResponse::Ok().body(data))
}

#[post("/item")]
pub async fn store_item_unattached(payload: Payload, state: web::Data<DbState>) -> Result<HttpResponse, Error> {
  let db = &state.item_db;
  let body = util::collect_from_payload(payload).await.map_err(ErrorBadRequest)?;

  //TODO extract method
  let mut aligned_bytes = AlignedVec::with_capacity(body.len() + 64);
  aligned_bytes.extend_from_slice(&body);

  let item = rkyv::from_bytes::<Item>(&aligned_bytes).map_err(ErrorBadRequest)?;
  db.insert(item.id.as_bytes(), aligned_bytes.as_slice())
    .map_err(ErrorInternalServerError)?;

  Ok(HttpResponse::Created().body(""))
}

#[post("/item/attached")]
pub async fn store_item_attached(payload: Payload, state: web::Data<DbState>) -> Result<HttpResponse, Error> {
  let body = util::collect_from_payload(payload).await.map_err(ErrorBadRequest)?;

  //TODO extract method
  let mut aligned_bytes = AlignedVec::with_capacity(body.len() + 64);
  aligned_bytes.extend_from_slice(&body);

  let command = rkyv::from_bytes::<StoreItemAttached>(&aligned_bytes).map_err(ErrorBadRequest)?;

  // insert item
  state
    .item_db
    .insert::<&[u8], &[u8]>(
      command.item.id.as_bytes(),
      rkyv::to_bytes::<_, 128>(&command.item)
        .map_err(ErrorBadRequest)?
        .as_slice(),
    )
    .map_err(ErrorInternalServerError)?;


  state
    .list_db
    .transaction(|tx_id| {
      let current_value = match tx_id.get(command.list_id.as_bytes())? {
        Some(val) => val,
        None => abort(ErrorNotFound("No such list."))?,
      };
      let mut old_list = match unsafe { rkyv::from_bytes_unchecked::<List>(&current_value) } {
        Ok(val) => val,
        Err(_) => abort(ErrorInternalServerError(""))?,
      };
      old_list.items.push(command.item.id);

      let bytes = match rkyv::to_bytes::<_, 256>(&old_list) {
        Ok(bytes) => bytes,
        Err(e) => abort(ErrorInternalServerError(e))?
      };

      match tx_id.insert(command.list_id.as_bytes(), bytes.as_bytes()) {
        Ok(_) => Ok(()),
        Err(e) => abort(ErrorInternalServerError(e))
      }
    })
    .map_err(|e| match e {
      // return inner error if its a sled-user-error:
      sled::transaction::TransactionError::Abort(e) => e,
      // if not, something else went wrong, so generic server error:
      _ => ErrorInternalServerError(e),
    })?;


  // update item List
  Ok(HttpResponse::Created().body(""))
}

#[get("/itemList/{id}/flat")]
pub async fn get_item_list_flat(id: web::Path<String>, state: web::Data<DbState>) -> Result<HttpResponse, Error> {
  let db = &state.list_db;
  let list_bytes = db
    .get(id.parse::<u64>().map_err(ErrorBadRequest)?.as_bytes())
    .map_err(ErrorInternalServerError)?
    .ok_or_else(|| ErrorNotFound("No such item list."))?;
  let list = unsafe {
    // This should be safe, as objects are checked before they are inserted into the db
    rkyv::from_bytes_unchecked::<List>(list_bytes.as_bytes())
  }
  .map_err(ErrorInternalServerError)?;

  //let vec = Vec::with_capacity(list.items.len());
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

  let bytes = rkyv::to_bytes::<_, 256>(&flat_items_list)
    .map_err(ErrorInternalServerError)?
    .to_vec();

  Ok(HttpResponse::Ok().body(bytes))
}

#[post("/itemList")]
pub(crate) async fn store_item_list(payload: Payload, state: web::Data<DbState>) -> Result<HttpResponse, Error> {
  let params = collect_from_payload(payload).await?;
  let db = &state.list_db;

  // sanitize input
  if params.as_bytes().len() < std::mem::size_of::<ArchivedArticle>() {
    return Err(ErrorBadRequest("Incomplete data."));
  }

  // while an id is provided with the archived data, we do not use this id, given, that the client does not know the new id as this is DB-managed information
  let id = state.db.generate_id().map_err(ErrorInternalServerError)?;
  // check archive; TODO: don't deserialize?
  let mut archived = rkyv::from_bytes::<List>(params.as_bytes()).map_err(ErrorBadRequest)?;
  archived.id = id;
  db.insert(
    id.as_bytes(),
    rkyv::to_bytes::<_, 64>(&archived)
      .map_err(ErrorInternalServerError)?
      .as_bytes(),
  )
  .map_err(ErrorInternalServerError)?;

  // we need to return the newly generated id to the client
  Ok(HttpResponse::Created().body(id.as_bytes().to_owned()))
}
