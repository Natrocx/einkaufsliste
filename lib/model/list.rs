use einkaufsliste_codegen::impl_from_request;
use rkyv::{Archive, Deserialize, Serialize};

use super::item::Item;
use super::shop::Shop;
use super::Identifiable;

#[derive(Archive, Serialize, Deserialize, PartialEq, Clone)]
#[archive_attr(derive(bytecheck::CheckBytes, Debug))]
pub struct List {
  pub id: <List as Identifiable>::Id,
  pub name: String,
  pub shop: Option<<Shop as Identifiable>::Id>,
  pub image_id: Option<u32>,
  pub items: Vec<<Item as Identifiable>::Id>,
}

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
