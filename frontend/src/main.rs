#![feature(get_mut_unchecked)]

use frontend::setup_tracing;
use frontend::ui::Einkaufsliste;
use iced::{Application, Settings};
use ui::styles::DEFAULT_TEXT_SIZE;

pub mod service;
pub mod ui;

fn main() {
  setup_tracing();

  let settings = Settings {
    antialiasing: true,
    default_text_size: DEFAULT_TEXT_SIZE,
    ..Settings::default()
  };
  Einkaufsliste::run(settings).unwrap();
}
