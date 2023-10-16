use std::fmt::Display;

use actix_identity::Identity;
use actix_web::dev::Extensions;
use actix_web::{self, get, post, web, HttpMessage, HttpRequest};
use einkaufsliste::model::list::List;
use einkaufsliste::model::requests::{LoginUserV1, RegisterUserV1};
use einkaufsliste::model::user::{User, UserWithPassword};
use einkaufsliste::model::Identifiable;

use crate::response::*;
use crate::util::errors::{bad_request, error};
use crate::util::identity_ext::AuthenticatedUser;
use crate::{db, DbState};

#[post("/register/v1")]
pub(crate) async fn register_v1(
  parameter: RegisterUserV1,
  data: web::Data<DbState>,
  request: HttpRequest,
) -> Response<User> {
  // validate registration request- kekw
  if parameter.password.len() < 8 {
    return bad_request("Password too short").into();
  }
  if data.login_db.get(&parameter.name)?.is_some() {
    return bad_request("User already exists").into();
  }

  let hashed_pw = DbState::hash_password(&parameter.password)?;
  let id = data.db.generate_id().map_err(error)?;

  let value = UserWithPassword {
    user: User {
      id,
      name: parameter.name.clone(),
      profile_picture_id: None,
    },
    password: hashed_pw,
  };

  data.new_user(&value)?;
  data.store_unlisted(&value.user, id)?;

  // there isn't really a point in not logging the user in here
  login_user(&request.extensions(), id)?;

  Response::from(value.user)
}

/// calling this with correct login data will set a cookie to enable access to protected resources
#[post("/login/v1")]
pub(crate) async fn login_v1(
  login_request: LoginUserV1,
  state: web::Data<DbState>,
  request: HttpRequest,
) -> Response<User> {
  let user = state.check_password(&login_request)?;

  // remember user id for session
  login_user(&request.extensions(), user.user.id)?;

  Response::from(user.user)
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
  id: web::Path<Option<u64>>,
  user: AuthenticatedUser,
) -> Response<User> {
  let requested_users_id = match *id {
    Some(id) => id,
    None => user.id,
  };

  let user: User = state.get_unchecked(requested_users_id)?;

  Response::from(user)
}

#[get("/user/lists")]
pub(crate) async fn get_users_lists(
  state: web::Data<DbState>,
  user: AuthenticatedUser,
) -> Response<Vec<List>> {
  // read ObjectList from DB
  let list_ids =
    <db::DbState as db::ObjectStore<List, sled::Tree, 512>>::object_list(&state, user.id)?;

  let lists = list_ids
    .list
    .into_iter()
    .map(|id| state.get_unchecked(id))
    .collect::<Result<Vec<List>, _>>()?;

  Response::from(lists)
}

pub fn login_user(
  exts: &Extensions,
  id: <User as Identifiable>::Id,
) -> std::result::Result<(), ResponseError> {
  Identity::login(exts, id.to_string())
    .map_err(|e| ResponseError::ErrorInternalServerError(e.into()))?;

  Ok(())
}
