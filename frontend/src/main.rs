pub mod service;
mod tests;
pub mod ui;
pub mod util;

use std::fmt::Display;

use log::Level;
use reqwest::StatusCode;
use rkyv::validation::validators::CheckDeserializeError;

fn main() {
  setup_logger().unwrap();
  dioxus_desktop::launch(ui::app);
}

fn setup_logger() -> Result<(), fern::InitError> {
  fern::Dispatch::new()
    .format(|out, message, record| {
      out.finish(format_args!(
        "{}[{}][{}] {}",
        chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
        record.target(),
        record.level(),
        message
      ))
    })
    .level(log::LevelFilter::Debug)
    .chain(std::io::stdout())
    .chain(fern::log_file("output.log")?)
    .apply()?;
  Ok(())
}

#[derive(Debug)]
pub enum TransmissionError {
  SerializationError,
  NetworkError(reqwest::Error),
  InvalidResponseError(String),
  FailedRequest(StatusCode),
  Unknown(String),
}

impl Display for TransmissionError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let error_message = match self {
      TransmissionError::SerializationError => "An Error occured during client-side serialization".to_owned(),
      TransmissionError::NetworkError(e) => format!("A network Error occured during transmission: {e}"),
      TransmissionError::InvalidResponseError(e) => format!("An invalid response was returned from the server: {e}"),
      TransmissionError::FailedRequest(status) => format!("The request was not successful: {status}"),
      TransmissionError::Unknown(e) => format!("An unknown error occured: {e}"),
    };

    write!(f, "{}", error_message)
  }
}

impl From<reqwest::Error> for TransmissionError {
  fn from(e: reqwest::Error) -> Self {
    // weird API, or is this stupid?
    if e.is_status() {
      TransmissionError::InvalidResponseError(e.to_string())
    } else if e.is_request() {
      TransmissionError::FailedRequest(e.status().unwrap())
    } else if e.is_body() || e.is_decode() {
      TransmissionError::SerializationError
    } else {
      TransmissionError::Unknown(e.to_string())
    }
  }
}

impl<C, D> From<CheckDeserializeError<C, D>> for TransmissionError {
  fn from(_: CheckDeserializeError<C, D>) -> Self {
    TransmissionError::SerializationError
  }
}
