use argon2::Argon2;
use einkaufsliste::model::article::Article;
use einkaufsliste::model::item::Item;
use einkaufsliste::model::list::List;
use einkaufsliste::model::requests::LoginUserV1;
use einkaufsliste::model::shop::Shop;
use einkaufsliste::model::user::{ObjectList, Password, User, UserWithPassword, UsersObjectLists};
use einkaufsliste::model::{AccessControlList, HasTypeDenominator, Identifiable};
use einkaufsliste::ApiObject;
use rand::{thread_rng, Rng};
use rkyv::de::deserializers::{SharedDeserializeMap, SharedDeserializeMapError};
use rkyv::ser::serializers::{AllocScratchError, CompositeSerializerError};
use rkyv::Archive;
use sled::transaction::{abort, TransactionError, TransactionalTree, UnabortableTransactionError};
use tracing::debug;
use zerocopy::AsBytes;

use crate::api::user;
use crate::util::errors::{abort_error, error};

#[derive(Clone)]
pub struct DbState {
  pub db: sled::Db,
  pub article_db: sled::Tree,
  pub item_db: sled::Tree,
  pub shop_db: sled::Tree,
  pub list_db: sled::Tree,
  pub acl_db: sled::Tree,
  pub user_db: sled::Tree,
  pub login_db: sled::Tree,
  pub object_list_db: sled::Tree,
}

impl DbState {
  pub fn check_password(&self, login: &LoginUserV1) -> Result<UserWithPassword, DbError> {
    let user = self.get_user(&login.name)?;
    let request_pw_hash = Self::hash_password_with_salt(&login.password, &user.password.salt);

    if Self::hash_password_with_salt(&login.password, &user.password.salt)? == user.password.hash {
      Ok(user)
    } else {
      debug!(
        "Password validation error: {:?}, {:?}",
        request_pw_hash, user.password.hash
      );
      Err(DbError::Mismatch)
    }
  }

  pub(crate) fn hash_password_with_salt(password: &str, salt: &[u8]) -> Result<Vec<u8>, DbError> {
    // Hash length does not matter much for password verification - 256 bit is definitely fine
    let mut bytes = vec![0; 32];

    Argon2::default()
      .hash_password_into(password.as_bytes(), salt, &mut bytes)
      .map_err(|e| DbError::IO(e.to_string().into()))?;

    Ok(bytes)
  }

  pub(crate) fn hash_password(password: &str) -> Result<Password, DbError> {
    // there is no need for the salt to be securely generated as even a normal random number prevents rainbow-table attacks
    let mut salt = [0; 32];
    thread_rng().fill(&mut salt);

    let hash = Self::hash_password_with_salt(password, &salt)?;

    Ok(Password {
      hash,
      salt: salt.to_vec(),
    })
  }

  pub(crate) fn verify_access<Object: Identifiable, User: Identifiable>(
    &self,
    object_id: <Object as Identifiable>::Id,
    user_id: <User as Identifiable>::Id,
  ) -> Result<(), DbError>
  // don't ask me bro... thanks to rust analyzer we dont have to ask bro
  where
    <<Object as einkaufsliste::model::Identifiable>::Id as rkyv::Archive>::Archived:
      rkyv::Deserialize<
        <Object as einkaufsliste::model::Identifiable>::Id,
        rkyv::de::deserializers::SharedDeserializeMap,
      >,
    [<<User as einkaufsliste::model::Identifiable>::Id as rkyv::Archive>::Archived]:
      rkyv::DeserializeUnsized<
        [<User as einkaufsliste::model::Identifiable>::Id],
        rkyv::de::deserializers::SharedDeserializeMap,
      >,
    <<User as einkaufsliste::model::Identifiable>::Id as rkyv::Archive>::Archived:
      rkyv::Deserialize<
        <User as einkaufsliste::model::Identifiable>::Id,
        rkyv::de::deserializers::SharedDeserializeMap,
      >,
  {
    let acl = self
      .acl_db
      .get(object_id.as_bytes())?
      .ok_or(DbError::NotFound)?;

    let acl =
      unsafe { rkyv::from_bytes_unchecked::<AccessControlList<Object, User>>(acl.as_bytes()) }?;

    match acl.owner == user_id || acl.allowed_user_ids.contains(&user_id) {
      true => Ok(()),
      false => Err(DbError::Mismatch),
    }
  }

  pub fn copy_acl<List: Identifiable, Item: Identifiable>(
    &self,
    list_id: <List as Identifiable>::Id,
    item_id: <Item as Identifiable>::Id,
  ) -> Result<(), DbError> {
    let list_acl = self
      .acl_db
      .get(list_id.as_bytes())?
      .ok_or(DbError::NotFound)?;

    self.acl_db.insert(item_id.as_bytes(), list_acl)?;

    Ok(())
  }

  /// Generates and stores new AccessControlList
  pub fn create_acl<Object: Identifiable, User: Identifiable>(
    &self,
    object_id: <Object as Identifiable>::Id,
    user_id: <User as Identifiable>::Id,
  ) -> Result<Option<sled::IVec>, DbError>
  where
    <Object as einkaufsliste::model::Identifiable>::Id: rkyv::Serialize<
      rkyv::ser::serializers::CompositeSerializer<
        rkyv::ser::serializers::AlignedSerializer<rkyv::AlignedVec>,
        rkyv::ser::serializers::FallbackScratch<
          rkyv::ser::serializers::HeapScratch<256_usize>,
          rkyv::ser::serializers::AllocScratch,
        >,
        rkyv::ser::serializers::SharedSerializeMap,
      >,
    >,
    <User as einkaufsliste::model::Identifiable>::Id: rkyv::Serialize<
      rkyv::ser::serializers::CompositeSerializer<
        rkyv::ser::serializers::AlignedSerializer<rkyv::AlignedVec>,
        rkyv::ser::serializers::FallbackScratch<
          rkyv::ser::serializers::HeapScratch<256_usize>,
          rkyv::ser::serializers::AllocScratch,
        >,
        rkyv::ser::serializers::SharedSerializeMap,
      >,
    >,
  {
    let new_acl = AccessControlList::<Object, User> {
      object_id: object_id.clone(),
      allowed_user_ids: vec![],
      owner: user_id,
    };

    self
      .acl_db
      .insert(object_id.as_bytes(), &*rkyv::to_bytes::<_, 256>(&new_acl)?)
      .map_err(Into::into)
  }

  pub fn new_user(&self, user: &UserWithPassword) -> Result<(), DbError> {
    match self
      .login_db
      .insert(&user.user.name, &*rkyv::to_bytes::<_, 256>(user)?)
    {
      Ok(_) => Ok(()),
      Err(e) => Err(e.into()),
    }
  }

  pub fn get_user(&self, user_name: &str) -> Result<UserWithPassword, DbError> {
    let bytes = self.login_db.get(user_name)?.ok_or(DbError::Mismatch)?;

    let user = unsafe { rkyv::from_bytes_unchecked::<UserWithPassword>(&bytes) }?;

    Ok(user)
  }

  /**
  Get an object from the database using an automatically selected tree.
  This will not check the Object using ByteCheck but rather trust, that the trees have not been manually filled with garbage.
  */
  pub fn get_unchecked<T: ApiObject<'static>>(&self, id: u64) -> Result<T, DbError>
  where
    <T as rkyv::Archive>::Archived:
      rkyv::Deserialize<T, rkyv::de::deserializers::SharedDeserializeMap>,
    <T as rkyv::Archive>::Archived:
      rkyv::CheckBytes<rkyv::validation::validators::DefaultValidator<'static>>,
    Self: ObjectTree<T>,
  {
    let tree = self.get_tree();

    let bytes = unsafe { <sled::Tree as RawRkyvStore<T, 4096>>::get_unchecked(tree, id) }?;

    Ok(bytes)
  }

  pub fn store_unlisted<T: ApiObject<'static>>(&self, value: &T, id: u64) -> Result<(), DbError>
  where
    <T as rkyv::Archive>::Archived:
      rkyv::Deserialize<T, rkyv::de::deserializers::SharedDeserializeMap>,
    <T as rkyv::Archive>::Archived:
      rkyv::CheckBytes<rkyv::validation::validators::DefaultValidator<'static>>,
    Self: ObjectTree<T>,
  {
    let tree = <Self as ObjectTree<T>>::get_tree(&self);

    unsafe { <sled::Tree as RawRkyvStore<T, 4096>>::store_unlisted(tree, id, value) }?;

    Ok(())
  }

  pub fn store_listed<T: ApiObject<'static> + HasTypeDenominator>(
    &self,
    value: &T,
    user_id: u64,
    id: u64,
  ) -> Result<(), DbError>
  where
    <T as rkyv::Archive>::Archived:
      rkyv::Deserialize<T, rkyv::de::deserializers::SharedDeserializeMap>,
    <T as rkyv::Archive>::Archived:
      rkyv::CheckBytes<rkyv::validation::validators::DefaultValidator<'static>>,
    Self: ObjectTree<T>,
  {
    let tree = <Self as ObjectTree<T>>::get_tree(self);

    unsafe { <Self as ObjectStore<_, _, 4096>>::store_listed(self, user_id, id, tree, value) }?;

    Ok(())
  }
}

pub trait ObjectTree<T> {
  fn get_tree(&self) -> &sled::Tree;
}

impl ObjectTree<User> for DbState {
  fn get_tree(&self) -> &sled::Tree {
    &self.user_db
  }
}

impl<T: Identifiable> ObjectTree<AccessControlList<T, User>> for DbState {
  fn get_tree(&self) -> &sled::Tree {
    &self.acl_db
  }
}

impl ObjectTree<Item> for DbState {
  fn get_tree(&self) -> &sled::Tree {
    &self.item_db
  }
}

impl ObjectTree<List> for DbState {
  fn get_tree(&self) -> &sled::Tree {
    &self.list_db
  }
}

impl ObjectTree<Article> for DbState {
  fn get_tree(&self) -> &sled::Tree {
    &self.article_db
  }
}

impl ObjectTree<Shop> for DbState {
  fn get_tree(&self) -> &sled::Tree {
    &self.shop_db
  }
}

impl ObjectTree<UserWithPassword> for DbState {
  fn get_tree(&self) -> &sled::Tree {
    &self.login_db
  }
}

impl ObjectTree<ObjectList> for DbState {
  fn get_tree(&self) -> &sled::Tree {
    &self.object_list_db
  }
}

// The following traits are unsafe, because they do not validate the tree's content. You must manually ensure that you choose the correct tree for your type.
// If these functions are only used through DbStates methods autochoosing the treex, they should be safe.
pub trait ObjectStore<
  T: rkyv::Serialize<
      rkyv::ser::serializers::CompositeSerializer<
        rkyv::ser::serializers::AlignedSerializer<rkyv::AlignedVec>,
        rkyv::ser::serializers::FallbackScratch<
          rkyv::ser::serializers::HeapScratch<SIZE_HINT>,
          rkyv::ser::serializers::AllocScratch,
        >,
        rkyv::ser::serializers::SharedSerializeMap,
      >,
    > + HasTypeDenominator,
  ObjectTree: RawRkyvStore<T, SIZE_HINT>,
  const SIZE_HINT: usize,
> where
  <T as Archive>::Archived: rkyv::Deserialize<T, SharedDeserializeMap>,
{
  unsafe fn store_listed(
    &self,
    user_id: u64,
    object_id: u64,
    db: &ObjectTree,
    object: &T,
  ) -> Result<(), DbError>;

  fn object_list(&self, user_id: u64) -> Result<ObjectList, DbError>;
}

impl<T, const SIZE_HINT: usize, ObjectTree> ObjectStore<T, ObjectTree, SIZE_HINT> for DbState
where
  T: HasTypeDenominator
    + rkyv::Serialize<
      rkyv::ser::serializers::CompositeSerializer<
        rkyv::ser::serializers::AlignedSerializer<rkyv::AlignedVec>,
        rkyv::ser::serializers::FallbackScratch<
          rkyv::ser::serializers::HeapScratch<SIZE_HINT>,
          rkyv::ser::serializers::AllocScratch,
        >,
        rkyv::ser::serializers::SharedSerializeMap,
      >,
    >,
  <T as Archive>::Archived: rkyv::Deserialize<T, SharedDeserializeMap>,

  ObjectTree: RawRkyvStore<T, SIZE_HINT>,
{
  unsafe fn store_listed(
    &self,
    user_id: u64,
    object_id: u64,
    db: &ObjectTree,
    object: &T,
  ) -> Result<(), DbError> {
    match self.object_list_db.transaction(|tx_db| {
      // maintain object list for user
      let mut current_ol = unsafe {
        match tx_db.get(user_id.to_ne_bytes())? {
          Some(bytes) => {
            rkyv::from_bytes_unchecked::<UsersObjectLists>(&bytes).map_err(abort_error)?
          }
          None => {
            let new_uol = UsersObjectLists { lists: vec![] };
            RawRkyvStore::<UsersObjectLists, 128>::store_unlisted(&tx_db, user_id, &new_uol)
              .map_err(abort_error)?;
            new_uol
          }
        }
      };

      // if there already is a list of object ids of this user the id is simply added, otherwise a new list is created and the id is added
      if let Some(ol) = current_ol
        .lists
        .iter_mut()
        .find(|list| list.typ == T::DENOMINATOR)
      {
        ol.list.push(object_id);
      } else {
        let mut new_ol = ObjectList::new(T::DENOMINATOR);
        new_ol.list.push(object_id);
        current_ol.lists.push(new_ol);
      }

      Ok(tx_db.insert(
        &user_id.to_ne_bytes(),
        &*rkyv::to_bytes::<_, 512>(&current_ol).map_err(abort_error)?,
      ))
    }) {
      Ok(_) => {}
      Err(TransactionError::Storage(e)) => return Err(DbError::IO(e.into())),
      Err(TransactionError::Abort(e)) => return Err(e),
    }

    db.store_unlisted(object_id, object)
  }

  fn object_list(&self, user_id: u64) -> Result<ObjectList, DbError> {
    let object_list = match self.object_list_db.get(user_id.to_ne_bytes())? {
      Some(bytes) => bytes,
      None => return Ok(ObjectList::new(T::DENOMINATOR)), /* no object list stored for user: user has not created any objects */
    };
    let object_list = unsafe { rkyv::from_bytes_unchecked::<UsersObjectLists>(&object_list) }?;
    Ok(
      object_list
        .lists
        .into_iter()
        .find(|list| list.typ == T::DENOMINATOR)
        .unwrap_or_else(|| ObjectList::new(T::DENOMINATOR)),
    )
  }
}

/// This trait allows you to store objects serializable with [rkyv].
pub trait RawRkyvStore<
  T: rkyv::Serialize<
    rkyv::ser::serializers::CompositeSerializer<
      rkyv::ser::serializers::AlignedSerializer<rkyv::AlignedVec>,
      rkyv::ser::serializers::FallbackScratch<
        rkyv::ser::serializers::HeapScratch<SIZE_HINT>,
        rkyv::ser::serializers::AllocScratch,
      >,
      rkyv::ser::serializers::SharedSerializeMap,
    >,
  >,
  const SIZE_HINT: usize,
> where
  <T as Archive>::Archived: rkyv::Deserialize<T, SharedDeserializeMap>,
{
  unsafe fn store_unlisted(&self, id: u64, value: &T) -> Result<(), DbError>;

  /// # Safety
  /// You must manually ensure, that objects in the tree represent an archive of the generic type.
  /// Calling this with the wrong generic type should be an easy catch through unit testing.
  unsafe fn get_unchecked(&self, id: u64) -> Result<T, DbError>;
}

impl<T, const SIZE_HINT: usize> RawRkyvStore<T, SIZE_HINT> for sled::Tree
where
  T: rkyv::Serialize<
    rkyv::ser::serializers::CompositeSerializer<
      rkyv::ser::serializers::AlignedSerializer<rkyv::AlignedVec>,
      rkyv::ser::serializers::FallbackScratch<
        rkyv::ser::serializers::HeapScratch<SIZE_HINT>,
        rkyv::ser::serializers::AllocScratch,
      >,
      rkyv::ser::serializers::SharedSerializeMap,
    >,
  >,
  <T as Archive>::Archived: rkyv::Deserialize<T, SharedDeserializeMap>,
{
  unsafe fn store_unlisted(&self, id: u64, value: &T) -> Result<(), DbError> {
    match self.insert(id.to_ne_bytes(), &*rkyv::to_bytes(value)?) {
      Ok(_) => Ok(()),
      Err(e) => Err(e.into()),
    }
  }

  unsafe fn get_unchecked(&self, id: u64) -> Result<T, DbError> {
    let bytes = self.get(id.to_ne_bytes())?.ok_or(DbError::NotFound)?;

    rkyv::from_bytes_unchecked(&bytes).map_err(Into::into)
  }
}

impl<T, const SIZE_HINT: usize> RawRkyvStore<T, SIZE_HINT> for &TransactionalTree
where
  T: rkyv::Serialize<
    rkyv::ser::serializers::CompositeSerializer<
      rkyv::ser::serializers::AlignedSerializer<rkyv::AlignedVec>,
      rkyv::ser::serializers::FallbackScratch<
        rkyv::ser::serializers::HeapScratch<SIZE_HINT>,
        rkyv::ser::serializers::AllocScratch,
      >,
      rkyv::ser::serializers::SharedSerializeMap,
    >,
  >,
  <T as Archive>::Archived: rkyv::Deserialize<T, SharedDeserializeMap>,
{
  unsafe fn store_unlisted(&self, id: u64, value: &T) -> Result<(), DbError> {
    match self.insert(&id.to_ne_bytes(), &*rkyv::to_bytes(value)?) {
      Ok(_) => Ok(()),
      Err(e) => Err(e.into()),
    }
  }

  unsafe fn get_unchecked(&self, id: u64) -> Result<T, DbError> {
    let bytes = self.get(id.to_ne_bytes())?.ok_or(DbError::NotFound)?;

    rkyv::from_bytes_unchecked(&bytes).map_err(Into::into)
  }
}

#[derive(Debug)]
pub enum DbError {
  /// Maps to [`ResponseError::ErrorInternalServerError`]
  IO(Box<dyn std::error::Error>),
  /// Maps to [`ResponseError::ErrorInternalServerError`]
  Encoding(Box<dyn std::error::Error>),
  /// Maps to [`ResponseError::ErrorNotFound`]
  NotFound,
  /// This error specifies, that the user request mismatched the data in the db. Examples: invalid login, not in AccessControlList
  /// Maps to [`ResponseError::ErrorBadRequest`]
  Mismatch,
}

impl From<sled::Error> for DbError {
  fn from(e: sled::Error) -> Self {
    DbError::IO(e.into())
  }
}

impl<S: std::error::Error, C: std::error::Error, H: std::error::Error>
  From<CompositeSerializerError<S, C, H>> for DbError
{
  fn from(value: CompositeSerializerError<S, C, H>) -> Self {
    DbError::Encoding(value.to_string().into())
  }
}

impl From<SharedDeserializeMapError> for DbError {
  fn from(e: SharedDeserializeMapError) -> Self {
    DbError::Encoding(Box::new(e))
  }
}

impl From<AllocScratchError> for DbError {
  fn from(e: AllocScratchError) -> Self {
    DbError::Encoding(Box::new(e))
  }
}

impl From<UnabortableTransactionError> for DbError {
  fn from(value: UnabortableTransactionError) -> Self {
    DbError::IO(value.into())
  }
}

impl std::fmt::Display for DbError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      DbError::IO(e) => write!(f, "IO error while accessing database: {}", e),
      DbError::Encoding(e) => write!(
        f,
        "Unexpected encoding error while accessing database {}",
        e
      ),
      DbError::NotFound => write!(f, "Object not found in database"),
      DbError::Mismatch => write!(f, "Mismatch between user request and database state"),
    }
  }
}

impl std::error::Error for DbError {
  fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
    match self {
      DbError::IO(e) => Some(e.as_ref()),
      DbError::Encoding(e) => Some(e.as_ref()),
      DbError::NotFound | DbError::Mismatch => None,
    }
  }
}
