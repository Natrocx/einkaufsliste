use rkyv::{Archive, Deserialize, Serialize};

use super::item::Item;

/// Command-pattern based structs to be used as request parameters
#[derive(Archive, Serialize, Deserialize)]
#[archive_attr(derive(bytecheck::CheckBytes))]
pub struct StoreItemAttached {
  pub item: Item,
  pub list_id: u64,
}

#[derive(Archive, Serialize, Deserialize)]
#[archive_attr(derive(bytecheck::CheckBytes))]
pub struct RegisterUserV1 {
  pub name: String,
  pub password: String,
}

#[derive(Archive, Serialize, Deserialize)]
#[archive_attr(derive(bytecheck::CheckBytes))]
pub struct LoginUserV1 {
  pub name: String,
  pub password: String,
}
