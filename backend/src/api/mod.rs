use actix_web::error::{self, ErrorBadRequest, ErrorForbidden, ErrorInternalServerError};
use actix_web::{web, Error};
use einkaufsliste::model::user::Password;
use einkaufsliste::model::{AccessControlList, Identifiable};
use rand::Rng;
use rkyv::de::deserializers::SharedDeserializeMap;
use rkyv::ser::serializers::AllocSerializer;
use rkyv::{AlignedVec, Deserialize, Serialize};
use sled::Tree;
use zerocopy::AsBytes;

use crate::response::ResponseError;
use crate::util::collect_from_payload;
use crate::DbState;

pub(crate) mod article;
pub(crate) mod item;
pub(crate) mod shop;
pub(crate) mod user;

pub(crate) fn store_in_db<T: Serialize<AllocSerializer<SIZE_HINT>>, const SIZE_HINT: usize>(
  id: u64,
  value: T,
  db: &Tree,
) -> Result<Option<sled::IVec>, actix_web::Error> {
  db.insert(
    id.as_bytes(),
    rkyv::to_bytes(&value)
      .map_err(actix_web::error::ErrorBadRequest)?
      .as_bytes(),
  )
  .map_err(actix_web::error::ErrorInternalServerError)
}

pub(crate) fn align_bytes<const SIZE_HINT: usize>(bytes: &bytes::Bytes) -> AlignedVec {
  let mut vec = AlignedVec::with_capacity(SIZE_HINT);
  vec.extend_from_slice(bytes);
  vec
}

// TODO: is this necessary
pub(crate) async fn preprocess_payload<const SIZE_HINT: usize>(
  payload: web::Payload,
) -> Result<AlignedVec, ResponseError> {
  Ok(align_bytes::<SIZE_HINT>(&collect_from_payload(payload).await?))
}

impl DbState {
  pub(crate) async fn hash_password(&self, password: &str) -> Password {
    // there is no need for the salt to be securely generated as even a normal random number prevents rainbow-table attacks
    let mut salt = [0; 256];
    self.rng.lock().await.fill(&mut salt);

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
  ) -> Result<(), actix_web::Error>
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
      .map_err(ErrorInternalServerError)?
      .ok_or_else(|| ErrorBadRequest("No access control list for specified object. It might not exist."))?;

    //TODO: is this safe?
    let acl = unsafe { rkyv::from_bytes_unchecked::<AccessControlList<Object, User>>(acl.as_bytes()) }
      .map_err(ErrorBadRequest)?;

    match acl.owner == user_id || acl.allowed_user_ids.contains(&user_id) {
      true => Ok(()),
      false => Err(ErrorForbidden("")),
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
}

pub(crate) fn hash_password_with_salt(password: &str, salt: &[u8]) -> Vec<u8> {
  let mut hasher = blake3::Hasher::new();
  hasher.update(password.as_bytes());
  hasher.update(salt);

  hasher.finalize().as_bytes().to_vec()
}

pub async fn handle_generic_post<
  'a,
  T: 'a + Serialize<AllocSerializer<SIZE_HINT>> + Deserialize<T, SharedDeserializeMap>,
  const SIZE_HINT: usize,
>(
  payload: web::Payload,
) -> Result<(), actix_web::Error>
where
  <T as rkyv::Archive>::Archived: 'a + bytecheck::CheckBytes<rkyv::validation::validators::DefaultValidator<'a>>,
  <T as rkyv::Archive>::Archived: rkyv::Deserialize<T, rkyv::de::deserializers::SharedDeserializeMap>,
{
  let bytes = collect_from_payload(payload).await.map_err(error::ErrorBadRequest)?;
  let _aligned_bytes = align_bytes::<SIZE_HINT>(&bytes);

  //let checked_val = rkyv::from_bytes::<T>(&aligned_bytes).map_err(error::ErrorBadRequest)?;
  // i cant figure out how, since rkyv requires a reference of 'a lifetime, which we cannot obtain here. Without this, we effectively have store_in_db
  Ok(())
}

/// Generates and stores new AccessControlList
fn new_generic_acl<Object: Identifiable, User: Identifiable>(
  object_id: <Object as Identifiable>::Id,
  user_id: <User as Identifiable>::Id,
  db: &Tree,
) -> Result<Option<sled::IVec>, Error>
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

  db.insert(
    object_id.as_bytes(),
    rkyv::to_bytes::<_, 256>(&new_acl)
      .map_err(ErrorInternalServerError)?
      .to_vec(),
  )
  .map_err(ErrorInternalServerError)
}
