/// Effectively unwraps passed expression, returning from the current function rather than panicking. Useful for non-critical errors, which cannot be reasonably displayed to the user.
/// This is written as a macro to get usable line-numbers in the logs
#[macro_export]
macro_rules! some_or_log {
  ($option:expr) => {
    match $option {
      Some(val) => val,
      None => {
        log::error!("Expected Some value at non-critical section.");
        return;
      }
    }
  };
}

/// Effectively unwraps passed expression, returning from the current function rather than panicking. Useful for non-critical/technical errors, which cannot be reasonably displayed to the user.
/// This is written as a macro to get usable line-numbers in the logs
#[macro_export]
macro_rules! ok_or_log {
  ($result:expr) => {
    match $result {
      Ok(val) => val,
      Err(e) => {
        log::error!("An unexpected error occured: {e:?}");
        return;
      }
    }
  };
}
