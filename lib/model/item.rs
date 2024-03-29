use rkyv::{Archive, Deserialize, Serialize};

use super::article::Article;
use super::Identifiable;
use crate::impl_api_traits;

#[derive(Archive, Serialize, Deserialize, Clone, Debug, serde::Serialize, serde::Deserialize)]
#[archive_attr(derive(bytecheck::CheckBytes))]
pub struct Item {
  pub id: <Item as Identifiable>::Id,
  pub checked: bool,
  pub name: String,
  pub amount: Option<u64>,
  pub unit: Option<Unit>,
  pub article_id: Option<<Article as Identifiable>::Id>,
  pub alternative_article_ids: Option<Vec<<Article as Identifiable>::Id>>,
}
impl_api_traits!(Item);

#[derive(PartialEq, Eq, Archive, Serialize, Deserialize, Clone, Debug, serde::Serialize, serde::Deserialize)]
#[archive_attr(derive(bytecheck::CheckBytes))]
pub enum Unit {
  Gram,
  KiloGram,
  Pieces,
  FreeForm(String),
}

impl Identifiable for Item {
  type Id = u64;
}

impl PartialEq for Item {
  fn eq(&self, other: &Self) -> bool {
    self.id == other.id &&
      self.checked == other.checked &&
      self.amount == other.amount &&
      self.unit == other.unit &&
      self.article_id == other.article_id &&
      self.alternative_article_ids == other.alternative_article_ids
  }
}
