use rkyv::{Archive, Deserialize, Serialize};

#[derive(Archive, Serialize, Deserialize, Debug)]
#[archive_attr(derive(bytecheck::CheckBytes, Debug))]
pub struct Article {
  pub id: u64,
  pub name: String,
  pub description: Option<String>,
  pub image_id: Option<u32>,
  pub shops: Option<Vec<u64>>,
}
