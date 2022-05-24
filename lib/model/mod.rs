pub mod article;
pub mod item;
pub mod list;
pub mod requests;
pub mod shop;
pub mod user;

/// Declares the type of the Id of the implementing struct. Note that the Id still needs to be manually implemented.
///
///  The trait serves tight coupling between model objects to prevent divergence (for example in Database objects) when modifying id type later.
pub trait Identifiable {
  type Id: Sized + PartialEq + Eq;
}

/// Access-control-list for all kinds of data objects. Warning: if your ids are generated in an overlapping way, you must seperate the AccessControlLists
pub struct AccessControlList<Object: Identifiable, User: Identifiable> {
  pub list_id: Object::Id,
  pub allowed_user_ids: Vec<User::Id>,
}
