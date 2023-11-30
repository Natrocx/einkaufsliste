use rkyv::{Archive, Deserialize, Serialize};

use super::item::Item;
use super::list::List;
use super::Identifiable;
use crate::impl_api_traits;

/// Command-pattern based structs to be used as request parameters
#[derive(Debug, Archive, Serialize, Deserialize, serde::Serialize, serde::Deserialize, Clone)]
#[archive_attr(derive(bytecheck::CheckBytes))]
pub struct StoreItemAttached {
  pub item: Item,
  pub list_id: u64,
}
impl_api_traits!(StoreItemAttached);

#[derive(Debug, Archive, Serialize, Deserialize, serde::Serialize, serde::Deserialize, Clone)]
#[archive_attr(derive(bytecheck::CheckBytes))]
pub struct RegisterUserV1 {
  pub name: String,
  pub password: String,
}
impl_api_traits!(RegisterUserV1);

#[derive(Debug, Archive, Serialize, Deserialize, serde::Serialize, serde::Deserialize, Clone)]
#[archive_attr(derive(bytecheck::CheckBytes))]
pub struct LoginUserV1 {
  pub name: String,
  pub password: String,
}
impl_api_traits!(LoginUserV1);


#[derive(Debug, Archive, Serialize, Deserialize, serde::Serialize, serde::Deserialize, Clone)]
#[archive_attr(derive(bytecheck::CheckBytes))]
pub struct DeleteItem {
  pub list_id: <List as Identifiable>::Id,
  pub item_id: <Item as Identifiable>::Id,
}
impl_api_traits!(DeleteItem);