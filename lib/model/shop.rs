use rkyv::{Archive, Deserialize, Serialize};

use super::Identifiable;
use crate::impl_api_traits;

#[derive(Debug, Clone, Archive, Serialize, Deserialize, serde::Serialize, serde::Deserialize)]
#[archive_attr(derive(bytecheck::CheckBytes, Debug))]
pub struct Shop {
  pub id: <Shop as Identifiable>::Id,
  pub name: String,
  pub image_id: Option<u32>,
}

impl Identifiable for Shop {
  type Id = u64;
}

impl_api_traits!(Shop);
