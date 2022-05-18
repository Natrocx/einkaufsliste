use rkyv::{Archive, Deserialize, Serialize};

#[derive(Archive, Serialize, Deserialize)]
#[archive_attr(derive(bytecheck::CheckBytes))] //rkyv validation
pub struct Item {
  pub id: u64,
  pub article_id: Option<u64>,
  pub alternative_article_ids: Option<Vec<u64>>,
}
