use actix_web::dev::Response;
use argon2::Argon2;
use einkaufsliste::model::requests::LoginUserV1;
use einkaufsliste::model::user::{ObjectList, Password, User, UserWithPassword, UsersObjectLists};
use einkaufsliste::model::{AccessControlList, HasTypeDenominator, Identifiable};
use log::debug;
use rand::{thread_rng, Rng};
use rkyv::de::deserializers::SharedDeserializeMap;
use rkyv::Archive;
use sled::transaction::{TransactionError, TransactionalTree};
use zerocopy::AsBytes;

use crate::api::user::PasswordValidationError;
use crate::response::ResponseError;
use crate::util::errors::{abort_error, bad_request, error};

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
  pub fn check_password(
    &self,
    login: &LoginUserV1,
  ) -> Result<<User as Identifiable>::Id, PasswordValidationError> {
    let stored_user = self
      .login_db
      .get(&login.name)
      .map_err(PasswordValidationError::DbAccessError)?
      .ok_or(PasswordValidationError::NoSuchUserError)?;

    let user = unsafe {
      rkyv::from_bytes_unchecked::<UserWithPassword>(&stored_user)
        .map_err(|_| PasswordValidationError::RkyvValidationError)?
    };

    let request_pw_hash = Self::hash_password_with_salt(&login.password, &user.password.salt);

    if Self::hash_password_with_salt(&login.password, &user.password.salt)? == user.password.hash {
      Ok(user.user.id)
    } else {
      debug!(
        "Password validation error: {:?}, {:?}",
        request_pw_hash, user.password.hash
      );
      Err(PasswordValidationError::InvalidPassword)
    }
  }

  pub(crate) fn hash_password_with_salt(
    password: &str,
    salt: &[u8],
  ) -> Result<Vec<u8>, PasswordValidationError> {
    // Hash length does not matter much for password verification - 256 bit is definitely fine
    let mut bytes = vec![0; 32];

    Argon2::default()
      .hash_password_into(password.as_bytes(), salt, &mut bytes)
      .map_err(|_| PasswordValidationError::InvalidPassword)?;

    Ok(bytes)
  }

  pub(crate) fn hash_password(password: &str) -> Result<Password, ResponseError> {
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
  ) -> Result<(), ResponseError>
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
      .ok_or(ResponseError::ErrorNotFound)?;

    let acl =
      unsafe { rkyv::from_bytes_unchecked::<AccessControlList<Object, User>>(acl.as_bytes()) }
        .map_err(error)?; // if this fails it's a bug

    match acl.owner == user_id || acl.allowed_user_ids.contains(&user_id) {
      true => Ok(()),
      false => Err(ResponseError::ErrorUnauthorized),
    }
  }

  pub fn copy_acl<List: Identifiable, Item: Identifiable>(
    &self,
    list_id: <List as Identifiable>::Id,
    item_id: <Item as Identifiable>::Id,
  ) -> Result<(), actix_web::Error> {
    let list_acl = self
      .acl_db
      .get(list_id.as_bytes())
      .map_err(error)?
      .ok_or_else(|| bad_request(()))?;

    self
      .acl_db
      .insert(item_id.as_bytes(), list_acl)
      .map_err(error)?;

    Ok(())
  }

  /// Generates and stores new AccessControlList
  pub fn create_acl<Object: Identifiable, User: Identifiable>(
    &self,
    object_id: <Object as Identifiable>::Id,
    user_id: <User as Identifiable>::Id,
  ) -> Result<Option<sled::IVec>, ResponseError>
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
      .insert(
        object_id.as_bytes(),
        rkyv::to_bytes::<_, 256>(&new_acl).map_err(error)?.to_vec(),
      )
      .map_err(error)
  }
}

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
  fn store_listed(
    &self,
    user_id: u64,
    object_id: u64,
    db: &ObjectTree,
    object: &T,
  ) -> Result<Option<sled::IVec>, ResponseError>;

  fn object_list(&self, user_id: u64) -> Result<ObjectList, ResponseError>;
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
  fn store_listed(
    &self,
    user_id: u64,
    object_id: u64,
    db: &ObjectTree,
    object: &T,
  ) -> Result<Option<sled::IVec>, ResponseError> {
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
      Err(TransactionError::Storage(e)) => return Err(error(e)),
      Err(TransactionError::Abort(e)) => return Err(e),
    }

    db.store_unlisted(object_id, object)
  }

  fn object_list(&self, user_id: u64) -> Result<ObjectList, ResponseError> {
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
  fn store_unlisted(&self, id: u64, value: &T) -> Result<Option<sled::IVec>, ResponseError>;

  /// # Safety
  /// You must manually ensure, that objects in the tree represent an archive of the generic type.
  /// Calling this with the wrong generic type should be an easy catch through unit testing.
  unsafe fn get_unchecked(&self, id: u64) -> Result<T, ResponseError>;
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
  fn store_unlisted(&self, id: u64, value: &T) -> Result<Option<sled::IVec>, ResponseError> {
    self
      .insert(id.to_ne_bytes(), &*rkyv::to_bytes(value).map_err(error)?)
      .map_err(error)
  }

  unsafe fn get_unchecked(&self, id: u64) -> Result<T, ResponseError> {
    let bytes = self.get(id.to_ne_bytes())?.ok_or_else(|| bad_request(()))?;

    rkyv::from_bytes_unchecked(&bytes).map_err(error)
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
  fn store_unlisted(&self, id: u64, value: &T) -> Result<Option<sled::IVec>, ResponseError> {
    self
      .insert(&id.to_ne_bytes(), &*rkyv::to_bytes(value).map_err(error)?)
      .map_err(error)
  }

  unsafe fn get_unchecked(&self, id: u64) -> Result<T, ResponseError> {
    let bytes = self
      .get(id.to_ne_bytes())
      .map_err(error)?
      .ok_or_else(|| bad_request(()))?;

    rkyv::from_bytes_unchecked(&bytes).map_err(error)
  }
}
