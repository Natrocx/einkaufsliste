#![feature(async_fn_in_trait)]

use frontend::setup_tracing;
use tracing_subscriber::filter::{LevelFilter, Targets};

pub mod service;
pub mod ui;

fn main() {
  setup_tracing();
  #[cfg(not(target_arch = "wasm32"))]
  {
    dioxus_desktop::launch(ui::app);
  }

  #[cfg(target_arch = "wasm32")]
  {
    dioxus_web::launch(ui::app);
  }
}
