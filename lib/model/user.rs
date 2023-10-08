use std::ops::Deref;

use rkyv::{Archive, Deserialize, Serialize};

use super::Identifiable;
#[cfg(feature = "backend")]
use crate::impl_api_traits;

#[derive(Archive, Serialize, Deserialize, Debug, serde::Serialize, serde::Deserialize, Clone)]
#[archive_attr(derive(bytecheck::CheckBytes))]
pub struct User {
  pub id: <Self as Identifiable>::Id,
  pub name: String,
  pub profile_picture_id: Option<u64>,
}

#[cfg(feature = "backend")]
impl_api_traits!(User);

#[derive(Archive, Serialize, Deserialize, Debug, serde::Serialize, serde::Deserialize, Clone)]
#[archive_attr(derive(bytecheck::CheckBytes))]
pub struct UserWithPassword {
  pub user: User,
  pub password: Password,
}

#[cfg(feature = "backend")]
impl_api_traits!(UserWithPassword);

#[derive(Archive, Serialize, Deserialize, Debug, serde::Serialize, serde::Deserialize, Clone)]
#[archive_attr(derive(bytecheck::CheckBytes))]
pub struct Password {
  pub hash: Vec<u8>,
  pub salt: Vec<u8>,
}

impl Identifiable for User {
  type Id = u64;
}

#[derive(Archive, Serialize, Deserialize, Debug, Clone, serde::Serialize, serde::Deserialize)]
#[archive_attr(derive(bytecheck::CheckBytes))]
pub struct ObjectList {
  pub typ: u64,
  pub list: Vec<u64>,
}

impl ObjectList {
  pub fn new(typ: u64) -> Self {
    Self { typ, list: vec![] }
  }
}

#[cfg(feature = "backend")]
impl_api_traits!(ObjectList);

#[derive(Archive, Serialize, Deserialize, Debug, Default, serde::Serialize, serde::Deserialize, Clone)]
#[archive_attr(derive(bytecheck::CheckBytes))]
pub struct UsersObjectLists {
  pub lists: Vec<ObjectList>,
}

impl Deref for UsersObjectLists {
  type Target = Vec<ObjectList>;

  fn deref(&self) -> &Self::Target {
    &self.lists
  }
}
