use rkyv::{Archive, Deserialize, Serialize};

#[derive(Archive, Serialize, Deserialize)]
#[archive_attr(derive(bytecheck::CheckBytes, Debug))]
pub struct Shop {
  pub id: u64,
  pub name: String,
  pub image_id: Option<u32>,
}
