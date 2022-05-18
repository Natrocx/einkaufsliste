use rkyv::{Archive, Deserialize, Serialize};

use super::item::Item;

#[derive(Archive, Serialize, Deserialize)]
#[archive_attr(derive(bytecheck::CheckBytes, Debug))]
pub struct List {
  pub id: u64,
  pub name: String,
  pub shop: u64,
  pub image_id: Option<u32>,
  pub items: Vec<u64>,
}

#[derive(Archive, Serialize, Deserialize)]
#[archive_attr(derive(bytecheck::CheckBytes))]
pub struct FlatItemsList {
  pub id: u64,
  pub name: String,
  pub shop: u64,
  pub image_id: Option<u32>,
  pub items: Vec<Item>,
}

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
