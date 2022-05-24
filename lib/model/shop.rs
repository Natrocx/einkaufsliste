use rkyv::{Archive, Deserialize, Serialize};

use super::Identifiable;

#[derive(Archive, Serialize, Deserialize)]
#[archive_attr(derive(bytecheck::CheckBytes, Debug))]
pub struct Shop {
  pub id: <Shop as Identifiable>::Id,
  pub name: String,
  pub image_id: Option<u32>,
}

impl Identifiable for Shop {
  type Id = u64;
}
