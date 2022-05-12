use rkyv::{Archive, Deserialize, Serialize};

#[derive(Archive, Serialize, Deserialize)]
#[archive_attr(derive(bytecheck::CheckBytes))] //rkyv validation
pub struct Item {
  pub id: u64,
  pub article_id: Option<u64>,
  pub alternative_article_ids: Option<Vec<u64>>,
}

#[derive(Archive, Serialize, Deserialize, Debug)]
#[archive_attr(derive(bytecheck::CheckBytes, Debug))]
pub struct Article {
  pub id: u64,
  pub name: String,
  pub description: Option<String>,
  pub image_id: Option<u32>,
  pub shops: Option<Vec<u64>>,
}

#[derive(Archive, Serialize, Deserialize)]
pub struct Shop {
  pub id: u64,
  pub name: String,
  pub image_id: Option<u32>,
}

#[derive(Archive, Serialize, Deserialize)]
pub struct List {
  pub id: u64,
  pub name: String,
  pub shop: u64,
  pub image_id: Option<u32>,
}
