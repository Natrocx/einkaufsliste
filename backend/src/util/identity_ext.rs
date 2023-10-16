use std::future::{ready, Ready};

use actix_identity::Identity;
use actix_web::FromRequest;

use crate::response::ResponseError;

pub struct AuthenticatedUser {
  pub id: u64,
}

impl FromRequest for AuthenticatedUser {
  type Error = ResponseError;

  type Future = Ready<Result<Self, Self::Error>>;

  fn from_request(
    req: &actix_web::HttpRequest,
    payload: &mut actix_web::dev::Payload,
  ) -> Self::Future {
    if let Ok(identity) = Identity::from_request(req, payload).into_inner() {
      if let Ok(user_string) = identity.id() {
        if let Ok(user) = user_string.parse::<u64>() {
          return ready(Ok(AuthenticatedUser { id: user }));
        }
      }
    }
    ready(Err(ResponseError::ErrorUnauthenticated))
  }
}
