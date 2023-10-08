use rkyv::{Archive, Deserialize, Serialize};

use super::shop::Shop;
use super::Identifiable;
#[cfg(feature = "backend")]
use crate::impl_api_traits;

#[derive(Archive, Serialize, Deserialize, Debug, serde::Serialize, serde::Deserialize, Clone)]
#[archive_attr(derive(bytecheck::CheckBytes, Debug))]
pub struct Article {
  pub id: <Article as Identifiable>::Id,
  pub name: String,
  pub description: Option<String>,
  pub image_id: Option<u32>,
  pub shops: Option<Vec<<Shop as Identifiable>::Id>>,
}

impl Identifiable for Article {
  type Id = u64;
}

#[cfg(feature = "backend")]
impl_api_traits!(Article);
