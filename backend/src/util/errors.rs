use sled::transaction::{ConflictableTransactionError};

use crate::response::ResponseError;

pub fn error<T: std::fmt::Display>(e: T) -> ResponseError {
  log::error!("An unexcpected Error occurred! Failed to serve request. Reason: {e}");

  ResponseError::ErrorInternalServerError
}

pub fn abort_error<T: std::fmt::Display>(e: T) -> ConflictableTransactionError<ResponseError> {
  ConflictableTransactionError::Abort(error(e))
}
