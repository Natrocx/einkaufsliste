use actix_web::error::{ErrorBadRequest, ErrorInternalServerError};
use einkaufsliste::model::article::Article;
use einkaufsliste::model::item::Item;
use einkaufsliste::model::list::List;
use einkaufsliste::model::requests::LoginUserV1;
use einkaufsliste::model::shop::Shop;
use einkaufsliste::model::user::{ObjectList, Password, User, UserWithPassword, UsersObjectLists};
use einkaufsliste::model::{AccessControlList, HasTypeDenominator, Identifiable};
use log::debug;
use rand::{thread_rng, Rng};


use sled::transaction::{TransactionError, TransactionalTree};
use zerocopy::AsBytes;

use crate::api::hash_password_with_salt;
use crate::api::user::PasswordValidationError;
use crate::response::ResponseError;
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

    let request_pw_hash = hash_password_with_salt(&login.password, &user.password.salt);

    if hash_password_with_salt(&login.password, &user.password.salt) == user.password.hash {
      Ok(user.user.id)
    } else {
      debug!(
        "Password validation error: {:?}, {:?}",
        request_pw_hash, user.password.hash
      );
      Err(PasswordValidationError::InvalidPassword)
    }
  }

  pub(crate) async fn hash_password(&self, password: &str) -> Password {
    // there is no need for the salt to be securely generated as even a normal random number prevents rainbow-table attacks
    let mut salt = [0; 256];
    thread_rng().fill(&mut salt);

    let mut hasher = blake3::Hasher::new();
    hasher.update(password.as_bytes());
    hasher.update(&salt);

    Password {
      hash: hasher.finalize().as_bytes().to_vec(),
      salt: salt.to_vec(),
    }
  }

  pub(crate) fn verify_access<Object: Identifiable, User: Identifiable>(
    &self,
    object_id: <Object as Identifiable>::Id,
    user_id: <User as Identifiable>::Id,
  ) -> Result<(), ResponseError>
  // don't ask me bro...
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
      .get(object_id.as_bytes())
      .map_err(|_| ResponseError::ErrorInternalServerError)?
      .ok_or(ResponseError::ErrorNotFound)?;

    //TODO: is this safe?
    let acl =
      unsafe { rkyv::from_bytes_unchecked::<AccessControlList<Object, User>>(acl.as_bytes()) }
        .map_err(|_| ResponseError::ErrorInternalServerError)?; // if this fails it's a bug

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
      .map_err(ErrorInternalServerError)?
      .ok_or_else(|| ErrorBadRequest(""))?;

    self
      .acl_db
      .insert(item_id.as_bytes(), list_acl)
      .map_err(ErrorInternalServerError)?;

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
        rkyv::to_bytes::<_, 256>(&new_acl)
          .map_err(|_| ResponseError::ErrorInternalServerError)?
          .to_vec(),
      )
      .map_err(|_| ResponseError::ErrorInternalServerError)
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
  const SIZE_HINT: usize,
>: RawRkyvStore<T, SIZE_HINT>
{
  fn store_listed(
    &self,
    user_id: u64,
    object_id: u64,
    object: &T,
  ) -> Result<Option<sled::IVec>, ResponseError>;

  fn object_list(&self, user_id: u64) -> Result<ObjectList, ResponseError>;
}

impl<T, const SIZE_HINT: usize> ObjectStore<T, SIZE_HINT> for DbState
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
  DbState: RawRkyvStore<T, SIZE_HINT>,
{
  fn store_listed(
    &self,
    user_id: u64,
    object_id: u64,
    object: &T,
  ) -> Result<Option<sled::IVec>, ResponseError> {
    match self.object_list_db.transaction(|tx_db| {
      // TODO: Maybe extract to function?
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

    self.store_unlisted(object_id, object)
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
///
/// # Implementation
/// This trait should be implemented manually on sled dbs for proper Tree selection. A macro may potentially also be used.
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
>
{
  fn store_unlisted(&self, id: u64, value: &T) -> Result<Option<sled::IVec>, ResponseError>;
}

impl<T, const SIZE_HINT: usize> RawRkyvStore<T, SIZE_HINT> for &sled::Tree
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
{
  fn store_unlisted(&self, id: u64, value: &T) -> Result<Option<sled::IVec>, ResponseError> {
    self
      .insert(id.to_ne_bytes(), &*rkyv::to_bytes(value).map_err(error)?)
      .map_err(error)
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
{
  fn store_unlisted(&self, id: u64, value: &T) -> Result<Option<sled::IVec>, ResponseError> {
    self
      .insert(&id.to_ne_bytes(), &*rkyv::to_bytes(value).map_err(error)?)
      .map_err(error)
  }
}

// manual implementations to allow for proper [sled::Tree] selection

impl RawRkyvStore<Article, 512> for DbState {
  fn store_unlisted(&self, id: u64, value: &Article) -> Result<Option<sled::IVec>, ResponseError> {
    self
      .article_db
      .insert(
        id.to_ne_bytes(),
        &*rkyv::to_bytes::<_, 512>(value).map_err(|_| ResponseError::ErrorBadRequest)?,
      )
      .map_err(|_| ResponseError::ErrorInternalServerError)
  }
}

impl RawRkyvStore<Item, 512> for DbState {
  fn store_unlisted(&self, id: u64, value: &Item) -> Result<Option<sled::IVec>, ResponseError> {
    self
      .item_db
      .insert(
        id.to_ne_bytes(),
        &*rkyv::to_bytes::<_, 512>(value).map_err(|_| ResponseError::ErrorBadRequest)?,
      )
      .map_err(|_| ResponseError::ErrorInternalServerError)
  }
}

impl RawRkyvStore<Shop, 1024> for DbState {
  fn store_unlisted(&self, id: u64, value: &Shop) -> Result<Option<sled::IVec>, ResponseError> {
    self
      .shop_db
      .insert(
        id.to_ne_bytes(),
        &*rkyv::to_bytes::<_, 1024>(value).map_err(|_| ResponseError::ErrorBadRequest)?,
      )
      .map_err(|_| ResponseError::ErrorInternalServerError)
  }
}

impl RawRkyvStore<List, 512> for DbState {
  fn store_unlisted(&self, id: u64, value: &List) -> Result<Option<sled::IVec>, ResponseError> {
    self
      .list_db
      .insert(
        id.to_ne_bytes(),
        &*rkyv::to_bytes::<_, 256>(value).map_err(|_| ResponseError::ErrorBadRequest)?,
      )
      .map_err(|_| ResponseError::ErrorInternalServerError)
  }
}

impl<T: Identifiable, S: Identifiable> RawRkyvStore<AccessControlList<T, S>, 128> for DbState
where
  <T as einkaufsliste::model::Identifiable>::Id: rkyv::Serialize<
    rkyv::ser::serializers::CompositeSerializer<
      rkyv::ser::serializers::AlignedSerializer<rkyv::AlignedVec>,
      rkyv::ser::serializers::FallbackScratch<
        rkyv::ser::serializers::HeapScratch<128>,
        rkyv::ser::serializers::AllocScratch,
      >,
      rkyv::ser::serializers::SharedSerializeMap,
    >,
  >,
  <S as einkaufsliste::model::Identifiable>::Id: rkyv::Serialize<
    rkyv::ser::serializers::CompositeSerializer<
      rkyv::ser::serializers::AlignedSerializer<rkyv::AlignedVec>,
      rkyv::ser::serializers::FallbackScratch<
        rkyv::ser::serializers::HeapScratch<128>,
        rkyv::ser::serializers::AllocScratch,
      >,
      rkyv::ser::serializers::SharedSerializeMap,
    >,
  >,
{
  fn store_unlisted(
    &self,
    id: u64,
    value: &AccessControlList<T, S>,
  ) -> Result<Option<sled::IVec>, ResponseError> {
    self
      .acl_db
      .insert(
        id.to_ne_bytes(),
        &*rkyv::to_bytes(value).map_err(|_| ResponseError::ErrorBadRequest)?,
      )
      .map_err(|_| ResponseError::ErrorInternalServerError)
  }
}

impl RawRkyvStore<User, 128> for DbState {
  fn store_unlisted(&self, id: u64, value: &User) -> Result<Option<sled::IVec>, ResponseError> {
    self
      .user_db
      .insert(
        id.to_ne_bytes(),
        &*rkyv::to_bytes::<_, 128>(value).map_err(|_| ResponseError::ErrorBadRequest)?,
      )
      .map_err(|_| ResponseError::ErrorInternalServerError)
  }
}

impl RawRkyvStore<UserWithPassword, 512> for DbState {
  fn store_unlisted(
    &self,
    id: u64,
    value: &UserWithPassword,
  ) -> Result<Option<sled::IVec>, ResponseError> {
    self
      .login_db
      .insert(
        id.to_ne_bytes(),
        &*rkyv::to_bytes::<_, 512>(value).map_err(|_| ResponseError::ErrorBadRequest)?,
      )
      .map_err(|_| ResponseError::ErrorInternalServerError)
  }
}
