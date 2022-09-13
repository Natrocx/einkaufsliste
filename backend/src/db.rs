use actix_web::error::{ErrorBadRequest, ErrorInternalServerError};
use einkaufsliste::model::article::Article;
use einkaufsliste::model::item::Item;
use einkaufsliste::model::list::List;
use einkaufsliste::model::requests::LoginUserV1;
use einkaufsliste::model::shop::Shop;
use einkaufsliste::model::user::{Password, User, UserWithPassword};
use einkaufsliste::model::{AccessControlList, Identifiable};
use log::debug;
use rand::{thread_rng, Rng};
use rkyv::ser::serializers::AllocSerializer;
use rkyv::Serialize;
use zerocopy::AsBytes;

use crate::api::hash_password_with_salt;
use crate::api::user::PasswordValidationError;
use crate::response::ResponseError;

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
  pub fn check_password(&self, login: &LoginUserV1) -> Result<<User as Identifiable>::Id, PasswordValidationError> {
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
    <<Object as einkaufsliste::model::Identifiable>::Id as rkyv::Archive>::Archived: rkyv::Deserialize<
      <Object as einkaufsliste::model::Identifiable>::Id,
      rkyv::de::deserializers::SharedDeserializeMap,
    >,
    [<<User as einkaufsliste::model::Identifiable>::Id as rkyv::Archive>::Archived]: rkyv::DeserializeUnsized<
      [<User as einkaufsliste::model::Identifiable>::Id],
      rkyv::de::deserializers::SharedDeserializeMap,
    >,
    <<User as einkaufsliste::model::Identifiable>::Id as rkyv::Archive>::Archived: rkyv::Deserialize<
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
    let acl = unsafe { rkyv::from_bytes_unchecked::<AccessControlList<Object, User>>(acl.as_bytes()) }
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

pub trait RkyvStore<T: Serialize<AllocSerializer<SIZE_HINT>>, const SIZE_HINT: usize> {
  fn store(&self, id: u64, value: T) -> Result<Option<sled::IVec>, ResponseError>;
}

impl RkyvStore<Article, 512> for DbState {
  fn store(&self, id: u64, value: Article) -> Result<Option<sled::IVec>, ResponseError> {
    self
      .article_db
      .insert(
        id.as_bytes(),
        rkyv::to_bytes::<_, 512>(&value)
          .map_err(|_| ResponseError::ErrorBadRequest)?
          .as_bytes(),
      )
      .map_err(|_| ResponseError::ErrorInternalServerError)
  }
}

impl RkyvStore<Item, 512> for DbState {
  fn store(&self, id: u64, value: Item) -> Result<Option<sled::IVec>, ResponseError> {
    self
      .item_db
      .insert(
        id.as_bytes(),
        rkyv::to_bytes::<_, 512>(&value)
          .map_err(|_| ResponseError::ErrorBadRequest)?
          .as_bytes(),
      )
      .map_err(|_| ResponseError::ErrorInternalServerError)
  }
}

impl RkyvStore<Shop, 1024> for DbState {
  fn store(&self, id: u64, value: Shop) -> Result<Option<sled::IVec>, ResponseError> {
    self
      .shop_db
      .insert(
        id.as_bytes(),
        rkyv::to_bytes::<_, 1024>(&value)
          .map_err(|_| ResponseError::ErrorBadRequest)?
          .as_bytes(),
      )
      .map_err(|_| ResponseError::ErrorInternalServerError)
  }
}

impl RkyvStore<List, 256> for DbState {
  fn store(&self, id: u64, value: List) -> Result<Option<sled::IVec>, ResponseError> {
    self
      .list_db
      .insert(
        id.as_bytes(),
        rkyv::to_bytes::<_, 256>(&value)
          .map_err(|_| ResponseError::ErrorBadRequest)?
          .as_bytes(),
      )
      .map_err(|_| ResponseError::ErrorInternalServerError)
  }
}

impl<T: Identifiable, S: Identifiable> RkyvStore<AccessControlList<T, S>, 128> for DbState
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
  fn store(&self, id: u64, value: AccessControlList<T, S>) -> Result<Option<sled::IVec>, ResponseError> {
    self
      .acl_db
      .insert(
        id.as_bytes(),
        rkyv::to_bytes(&value)
          .map_err(|_| ResponseError::ErrorBadRequest)?
          .as_bytes(),
      )
      .map_err(|_| ResponseError::ErrorInternalServerError)
  }
}

impl RkyvStore<User, 128> for DbState {
  fn store(&self, id: u64, value: User) -> Result<Option<sled::IVec>, ResponseError> {
    self
      .user_db
      .insert(
        id.as_bytes(),
        rkyv::to_bytes::<_, 128>(&value)
          .map_err(|_| ResponseError::ErrorBadRequest)?
          .as_bytes(),
      )
      .map_err(|_| ResponseError::ErrorInternalServerError)
  }
}

impl RkyvStore<UserWithPassword, 512> for DbState {
  fn store(&self, id: u64, value: UserWithPassword) -> Result<Option<sled::IVec>, ResponseError> {
    self
      .login_db
      .insert(
        id.as_bytes(),
        rkyv::to_bytes::<_, 512>(&value)
          .map_err(|_| ResponseError::ErrorBadRequest)?
          .as_bytes(),
      )
      .map_err(|_| ResponseError::ErrorInternalServerError)
  }
}
