use sled::transaction::ConflictableTransactionError;

use crate::db::DbError;
use crate::response::ResponseError;

pub fn error<T: std::error::Error + 'static>(e: T) -> ResponseError {
  tracing::error!("An unexcpected Error occurred! Failed to serve request. Reason: {e}");

  ResponseError::ErrorInternalServerError(e.into())
}

pub fn abort_error<T: Into<DbError> + 'static>(e: T) -> ConflictableTransactionError<DbError> {
  ConflictableTransactionError::Abort(e.into())
}

pub fn bad_request<T>(e: T) -> ResponseError
where
  T: core::fmt::Debug,
{
  tracing::debug!("A user submitted a bad request. Rejecting: {:?}", e);

  ResponseError::ErrorBadRequest
}

pub fn not_found<T>(_e: T) -> ResponseError {
  ResponseError::ErrorNotFound
}
