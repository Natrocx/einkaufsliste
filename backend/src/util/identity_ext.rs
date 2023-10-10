use std::num::ParseIntError;

use actix_identity::Identity;
use mime::FromStrError;

use crate::response::ResponseError;

//TODO: make an extractor out of this
pub trait IdentityExt {
  fn parse(&self) -> std::result::Result<u64, ResponseError>;
}

impl IdentityExt for Identity {
  fn parse(&self) -> std::result::Result<u64, ResponseError> {
    self
      .id()
      // if this fails it is a bug in the identity middleware and not a client error
      .map_err(|e|  ResponseError::ErrorInternalServerError(e.into()))?
      .parse()
      // if this fails, the semantics of the identity middleware have probably changed and it is not a client error
      .map_err(|e: ParseIntError| ResponseError::ErrorInternalServerError(e.into()))
  }
}
