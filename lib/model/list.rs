use rkyv::{Archive, Deserialize, Serialize};

use super::item::Item;
use super::shop::Shop;
use super::{HasTypeDenominator, Identifiable};
#[cfg(feature = "backend")]
use crate::impl_from_request;

#[derive(Archive, Serialize, Deserialize, PartialEq, Clone)]
#[archive_attr(derive(bytecheck::CheckBytes, Debug))]
pub struct List {
  pub id: <List as Identifiable>::Id,
  pub name: String,
  pub shop: Option<<Shop as Identifiable>::Id>,
  pub image_id: Option<u32>,
  pub items: Vec<<Item as Identifiable>::Id>,
}

#[cfg(feature = "backend")]
impl_from_request!(List);

#[derive(Archive, Serialize, Deserialize, PartialEq, Clone)]
#[archive_attr(derive(bytecheck::CheckBytes))]
pub struct FlatItemsList {
  pub id: <List as Identifiable>::Id, // this is intentionally Lists id, as they have to have the same Type
  pub name: String,
  pub shop: Option<<Shop as Identifiable>::Id>,
  pub image_id: Option<u32>,
  pub items: Vec<Item>,
}

#[cfg(feature = "backend")]
impl_from_request!(FlatItemsList);

impl FlatItemsList {
  pub fn from_list_and_items(list: List, vec: Vec<Item>) -> Self {
    FlatItemsList {
      id: list.id,
      name: list.name,
      shop: list.shop,
      image_id: list.image_id,
      items: vec,
    }
  }
}

impl Identifiable for List {
  type Id = u64;
}

impl HasTypeDenominator for List {
  const DENOMINATOR: u64 = 0;
}
