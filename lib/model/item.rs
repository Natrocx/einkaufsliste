use rkyv::{Archive, Deserialize, Serialize};

use super::article::Article;
use super::Identifiable;

#[derive(Archive, Serialize, Deserialize)]
#[archive_attr(derive(bytecheck::CheckBytes))] //rkyv validation
pub struct Item {
  pub id: <Item as Identifiable>::Id,
  pub checked: bool,
  pub amount: Option<u64>,
  pub unit: Option<Unit>,
  pub article_id: Option<<Article as Identifiable>::Id>,
  pub alternative_article_ids: Option<Vec<<Article as Identifiable>::Id>>,
}

#[derive(Archive, Serialize, Deserialize)]
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
