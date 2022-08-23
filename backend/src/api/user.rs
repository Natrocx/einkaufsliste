use std::fmt::Display;

use actix_identity::Identity;
use actix_web::error::{ErrorBadRequest, ErrorInternalServerError, ErrorNotFound};
use actix_web::*;
use einkaufsliste::model::requests::{LoginUserV1, RegisterUserV1};
use einkaufsliste::model::user::{ObjectList, User, UserWithPassword};
use einkaufsliste::model::Identifiable;
use log::debug;
use zerocopy::AsBytes;

use super::hash_password_with_salt;
use crate::api::preprocess_payload;
use crate::consts::object_list::ITEM_LIST_TYP;
use crate::response::{sled_to_response, ResponseBody};
use crate::{DbState, SessionState};

#[post("/register/v1")]
pub(crate) async fn register_v1(
  payload: web::Payload,
  data: web::Data<DbState>,
  sessions: web::Data<SessionState>,
  identity: Identity,
) -> Result<HttpResponse, Error> {
  let bytes = preprocess_payload::<128>(payload).await?;

  let parameter = rkyv::from_bytes::<RegisterUserV1>(&bytes).map_err(ErrorBadRequest)?;

  // validate registration request
  if parameter.password.len() < 8 {
    return Err(ErrorBadRequest("Password is too short"));
  }
  if data
    .login_db
    .get(&parameter.name)
    .map_err(ErrorInternalServerError)?
    .is_some()
  {
    return Err(ErrorBadRequest("A user with this name exists already"));
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
  })
  .map_err(ErrorInternalServerError)?;

  data
    .user_db
    .insert(id.as_bytes(), value.as_bytes())
    .map_err(ErrorInternalServerError)?;
  data
    .login_db
    .insert(&parameter.name, value.as_bytes())
    .map_err(ErrorInternalServerError)?;

  let cookie_secret = sessions
    .insert_new_session_for_id(id)
    .map_err(ErrorInternalServerError)?;

  identity.remember(cookie_secret);

  Ok(HttpResponse::Created().body(id.to_be_bytes().to_vec()))
}

/// calling this with correct login data will invalidate any old session and set a cookie to enable access to protected resources
#[post("/login/v1")]
pub(crate) async fn login_v1(
  payload: web::Payload,
  state: web::Data<DbState>,
  sessions: web::Data<SessionState>,
  identity: Identity,
) -> Result<HttpResponse, Error> {
  let bytes = preprocess_payload::<128>(payload).await?;

  let login_request = rkyv::from_bytes::<LoginUserV1>(&bytes).map_err(ErrorBadRequest)?;

  let id = match check_password(&login_request, &state.login_db) {
    Ok(valid) => valid,
    Err(e) => match e {
      PasswordValidationError::DbAccessError(e) => return Err(ErrorInternalServerError(e)),
      PasswordValidationError::NoSuchUserError => return Err(ErrorBadRequest("No such user")),
      PasswordValidationError::RkyvValidationError => {
        log::error!("Unexpected error interpreting database bytes");
        return Err(ErrorInternalServerError(""));
      }
      PasswordValidationError::InvalidPassword => return Err(ErrorBadRequest("Invalid Password")),
    },
  };

  let cookie_secret = sessions
    .insert_new_session_for_id(id)
    .map_err(ErrorInternalServerError)?;
  identity.remember(cookie_secret);

  Ok(HttpResponse::Ok().body(""))
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

  if request_pw_hash == &*user.password.hash {
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
enum PasswordValidationError {
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
) -> Result<HttpResponse, crate::response::ResponseError> {
  sessions.confirm_user_login(&identity)?;

  let requested_users_id = match id.parse() {
    Ok(id) => id,
    Err(..) => sessions.get_id_for_identity(&identity)?,
  };
  let user_bytes = state.user_db.get(requested_users_id.as_bytes());

  sled_to_response(user_bytes)
}

#[get("/user/lists")]
pub(crate) async fn get_users_lists(
  state: web::Data<DbState>,
  sessions: web::Data<SessionState>,
  identity: Identity,
) -> Result<HttpResponse, crate::response::ResponseError> {
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

  Ok(HttpResponse::Ok().body(ResponseBody::from(lists)))
}
