

use rkyv::de::deserializers::SharedDeserializeMap;
use rkyv::{Archive, Deserialize, Serialize};
use zerocopy::AsBytes;

pub mod article;
pub mod item;
pub mod list;
pub mod requests;
pub mod shop;
pub mod user;

// TODO: cam parameters to proc macros solve this?
// since traits cannot be partially implemented, we cannot use codegen without crying so here we go:
pub trait MarkerTrait<T: Identifiable> {
  fn try_from_u8_slice() -> Result<<T as Identifiable>::Id, &'static str>;
}

/// Declares the type of the Id of the implementing struct. Note that the Id still needs to be manually implemented.
///
///  The trait serves tight coupling between model objects to prevent divergence (for example in Database objects) when modifying id type later.
pub trait Identifiable {
  type Id: Sized + PartialEq + Eq + rkyv::Serialize<SharedDeserializeMap> + AsBytes + Clone;
}

/// Access-control-list for all kinds of data objects. Warning: if your ids are generated in an overlapping way, you must seperate the AccessControlLists in seperate DBs/keyspaces
#[derive(Archive, Serialize, Deserialize)]
#[archive_attr(derive(bytecheck::CheckBytes))]
pub struct AccessControlList<Object: Identifiable, User: Identifiable> {
  pub object_id: Object::Id,
  pub owner: User::Id,
  pub allowed_user_ids: Vec<User::Id>,
}
