use std::rc::Rc;

use actix_web::{get, post, put, web};
use einkaufsliste::model::item::Item;
use einkaufsliste::model::list::{FlatItemsList, List};
use einkaufsliste::model::requests::StoreItemAttached;
use einkaufsliste::model::user::User;
use sled::transaction::{abort, TransactionalTree};
use zerocopy::AsBytes;

use crate::response::{Response, ResponseError};
use crate::util::errors::{error, not_found};
use crate::util::identity_ext::AuthenticatedUser;
use crate::{db, DbState};

#[get("/item/{id}")]
pub async fn get_item_by_id(
  id: web::Path<u64>,
  state: web::Data<DbState>,
  user: AuthenticatedUser,
) -> Response<Item> {
  state.verify_access::<Item, User>(*id, user.id)?;

  let item: Item = state.get_unchecked(*id)?;

  Response::from(item)
}

#[post("/item/attached")]
pub async fn store_item_attached(
  mut param: StoreItemAttached,
  state: web::Data<DbState>,
  user: AuthenticatedUser,
) -> Response<u64> {
  let item_id = state.db.generate_id()?;

  state.verify_access::<List, User>(param.list_id, user.id)?;
  param.item.id = item_id;

  // insert item
  state.store_unlisted(&param.item, item_id)?;

  // direct usage of trees is unsafe as it can lead to storing the wrong type of object in a tree
  unsafe {
    state
      .list_db
      .transaction(|tx_db| {
        let mut current_list =
          match <&TransactionalTree as db::RawRkyvStore<List, 512>>::get_unchecked(
            &tx_db,
            param.list_id,
          ) {
            Ok(val) => val,
            Err(e) => return abort(not_found(e)),
          };
        current_list.items.push(param.item.id);
        match <&TransactionalTree as db::RawRkyvStore<List, 512>>::store_unlisted(
          &tx_db,
          param.list_id,
          &current_list,
        ) {
          Ok(_) => Ok(()),
          Err(e) => abort(error(e)),
        }
      })
      .map_err(|e| match e {
        // return inner error if its a sled-user-error:
        sled::transaction::TransactionError::Abort(e) => e,
        // if not, something else went wrong, so generic server error:
        _ => error(e),
      })?;
  }
  // ensure that we can get items independent of their corresponding list
  state.copy_acl::<List, Item>(param.list_id, param.item.id)?;

  Response::from(item_id)
}

#[get("/itemList/{id}/flat")]
pub async fn get_item_list_flat(
  list_id: web::Path<u64>,
  state: web::Data<DbState>,
  user: AuthenticatedUser,
) -> Response<FlatItemsList> {
  state.verify_access::<List, User>(*list_id, user.id)?;

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
    .map(|idx| item_db.get(idx.to_ne_bytes()))
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

  Response::from(flat_items_list)
}

#[post("/itemList")]
pub(crate) async fn store_item_list(
  mut param: List,
  state: web::Data<DbState>,
  user: AuthenticatedUser,
) -> Response<u64> {
  // while an id is provided with the archived data, we do not use this id, given that the client does not know the new id as this is DB-managed information
  let id = state.db.generate_id()?;
  param.id = id;

  state.store_listed(&param, user.id, id)?;
  state.create_acl::<List, User>(id, user.id)?;

  // we need to return the newly generated id to the client
  id.into()
}

// update only the metadata of a list, not the items
#[put("/itemList")]
pub async fn update_item_list(
  param: List,
  state: web::Data<DbState>,
  user: AuthenticatedUser,
) -> Response<()> {
  state.verify_access::<List, User>(param.id, user.id)?;

  state.store_listed(&param, user.id, param.id)?;

  Response::from(())
}
