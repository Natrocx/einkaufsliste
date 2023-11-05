use std::rc::Rc;

use rkyv::{Archive, Deserialize, Serialize};

use super::item::Item;
use super::shop::Shop;
use super::{HasTypeDenominator, Identifiable};
use crate::impl_api_traits;

#[derive(Archive, Serialize, Deserialize, PartialEq, Eq, Clone, Debug, serde::Serialize, serde::Deserialize)]
#[archive_attr(derive(bytecheck::CheckBytes, Debug))]
pub struct List {
  pub id: <List as Identifiable>::Id,
  pub name: String,
  pub shop: Option<<Shop as Identifiable>::Id>,
  pub image_id: Option<u64>,
  pub items: Vec<<Item as Identifiable>::Id>,
}

impl From<&FlatItemsList> for List {
  fn from(value: &FlatItemsList) -> Self {
    Self {
      id: value.id,
      name: value.name.clone(),
      shop: value.shop,
      image_id: value.image_id,
      items: value.items.iter().map(|item| item.id).collect(),
    }
  }
}

impl_api_traits!(List);

#[derive(Archive, Serialize, Deserialize, PartialEq, Clone, serde::Serialize, serde::Deserialize)]
#[archive_attr(derive(bytecheck::CheckBytes))]
pub struct FlatItemsList {
  pub id: <List as Identifiable>::Id, // this is intentionally Lists id, as they have to have the same Type
  pub name: String,
  pub shop: Option<<Shop as Identifiable>::Id>,
  pub image_id: Option<u64>,
  pub items: Vec<Rc<Item>>,
}

impl Eq for FlatItemsList {}

impl_api_traits!(FlatItemsList);

impl FlatItemsList {
  pub fn from_list_and_items(list: List, vec: Vec<Rc<Item>>) -> Self {
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

unsafe impl HasTypeDenominator for List {
  const DENOMINATOR: u64 = 0;
}
