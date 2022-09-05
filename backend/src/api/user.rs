use std::fmt::Display;

use actix_identity::Identity;
use actix_web::error::{ErrorBadRequest, ErrorInternalServerError, ErrorNotFound};
use actix_web::{self, get, post, web};
use einkaufsliste::model::requests::{LoginUserV1, RegisterUserV1};
use einkaufsliste::model::user::{ObjectList, User, UserWithPassword};
use einkaufsliste::model::Identifiable;
use log::debug;
use zerocopy::AsBytes;

use super::hash_password_with_salt;
use crate::api::preprocess_payload;
use crate::consts::object_list::ITEM_LIST_TYP;
use crate::response::*;
use crate::{DbState, SessionState};

#[post("/register/v1")]
pub(crate) async fn register_v1(
  payload: web::Payload,
  data: web::Data<DbState>,
  sessions: web::Data<SessionState>,
  identity: Identity,
) -> Response {
  let bytes = preprocess_payload::<128>(payload).await?;

  let parameter = rkyv::from_bytes::<RegisterUserV1>(&bytes)?;

  // validate registration request
  if parameter.password.len() < 8 {
    return ResponseError::ErrorBadRequest.into();
  }
  if data.login_db.get(&parameter.name)?.is_some() {
    return ResponseError::ErrorBadRequest.into();
  }

  let hashed_pw = data.hash_password(&parameter.password).await;
  let id = data.db.generate_id().map_err(ErrorInternalServerError)?;

  let value = rkyv::to_bytes::<_, 256>(&UserWithPassword {
    user: User {
      id,
      name: parameter.name.clone(),
      profile_picture_id: None,
    },
    password: hashed_pw,
  })?;

  data.user_db.insert(id.as_bytes(), value.as_bytes())?;
  data.login_db.insert(&parameter.name, value.as_bytes())?;

  sessions.insert_id_for_session(id, &identity.id().map_err(|_| ResponseError::ErrorUnauthenticated)?)?;

  id.into()
}

/// calling this with correct login data will invalidate any old session and set a cookie to enable access to protected resources
#[post("/login/v1")]
pub(crate) async fn login_v1(
  payload: web::Payload,
  state: web::Data<DbState>,
  sessions: web::Data<SessionState>,
  identity: Identity,
) -> Response {
  let bytes = preprocess_payload::<128>(payload).await?;

  let login_request = rkyv::from_bytes::<LoginUserV1>(&bytes)?;

  let id = check_password(&login_request, &state.login_db)?;

  // remember user id for session
  sessions.insert_id_for_session(id, &identity.id().map_err(|_| ResponseError::ErrorUnauthenticated)?)?;

  Response::empty()
}

fn check_password<'a>(
  login: &'a LoginUserV1,
  user_db: &'a sled::Tree,
) -> Result<<User as Identifiable>::Id, PasswordValidationError> {
  let stored_user = user_db
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
      "Password validation error: {}, {}: {:?}, {:?}",
      login.name, login.password, request_pw_hash, user.password.hash
    );
    Err(PasswordValidationError::InvalidPassword)
  }
}

#[allow(clippy::enum_variant_names)] // this is an error enum
#[derive(Debug)]
pub enum PasswordValidationError {
  DbAccessError(sled::Error),
  NoSuchUserError,
  RkyvValidationError,
  InvalidPassword,
}

impl Display for PasswordValidationError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let error_message = match self {
      Self::DbAccessError(e) => format!("Error accessing user data: {}", e),
      PasswordValidationError::NoSuchUserError => "There exists no user for this id".to_owned(),
      PasswordValidationError::RkyvValidationError => {
        "An unexpected error occured when deserialising from db.".to_owned()
      }
      PasswordValidationError::InvalidPassword => "Invalid password".to_owned(),
    };

    write!(f, "{}", error_message)
  }
}

#[get("/user/{id}")]
pub(crate) async fn get_user_profile(
  state: web::Data<DbState>,
  id: web::Path<String>,
  sessions: web::Data<SessionState>,
  identity: Identity,
) -> Response {
  sessions.confirm_user_login(&identity)?;

  let requested_users_id = match id.parse() {
    Ok(id) => id,
    Err(..) => sessions.get_id_for_identity(&identity)?,
  };

  // Objects may be served directly from db
  state.user_db.get(requested_users_id.as_bytes()).into()
}

#[get("/user/lists")]
pub(crate) async fn get_users_lists(
  state: web::Data<DbState>,
  sessions: web::Data<SessionState>,
  identity: Identity,
) -> Response {
  let user_id = sessions.get_id_for_identity(&identity)?;
  let bytes = state
    .object_list_db
    .get(user_id.as_bytes())
    .map_err(ErrorInternalServerError)?
    .ok_or_else(|| ErrorNotFound(""))?;

  let value = rkyv::from_bytes::<Vec<ObjectList>>(&bytes).map_err(ErrorBadRequest)?;
  let lists = value
    .iter()
    .find(|val| val.typ == ITEM_LIST_TYP)
    .ok_or_else(|| ErrorNotFound(""))?;

  Response::from(lists)
}
