use actix_identity::Identity;

use crate::response::ResponseError;

pub trait IdentityExt {
  fn parse(&self) -> std::result::Result<u64, ResponseError>;
}

impl IdentityExt for Identity {
  fn parse(&self) -> std::result::Result<u64, ResponseError> {
    self
      .id()
      // if this fails it is a bug in the identity middleware and not a client error
      .map_err(|_| ResponseError::ErrorInternalServerError)?
      .parse()
      // if this fails, the semantics of the identity middleware have probably changed and it is not a client error
      .map_err(|_| ResponseError::ErrorInternalServerError)
  }
}
