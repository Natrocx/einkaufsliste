use rkyv::{Archive, Deserialize, Serialize};

use super::Identifiable;

#[derive(Archive, Serialize, Deserialize)]
#[archive_attr(derive(bytecheck::CheckBytes))]
pub struct User {
  pub id: <Self as Identifiable>::Id,
  pub name: String,
  pub profile_picture_id: Option<u64>,
}

#[derive(Archive, Serialize, Deserialize)]
#[archive_attr(derive(bytecheck::CheckBytes))]
pub struct UserWithPassword {
  pub user: User,
  pub password: Password,
}

#[derive(Archive, Serialize, Deserialize)]
#[archive_attr(derive(bytecheck::CheckBytes))]
pub struct Password {
  pub hash: Vec<u8>,
  pub salt: Vec<u8>,
}

impl Identifiable for User {
  type Id = u64;
}
