#![feature(async_fn_in_trait)]

use tracing_subscriber::filter::LevelFilter;

mod service;
mod ui;
pub mod util;

fn main() {
  #[cfg(not(target_arch = "wasm32"))]
  {
    let max_level = if cfg!(debug_assertions) {
      LevelFilter::DEBUG
    } else {
      LevelFilter::INFO
    };
    tracing_subscriber::fmt().pretty().with_max_level(max_level).init();
    dioxus_desktop::launch(ui::app);
  }

  #[cfg(target_arch = "wasm32")]
  {
    dioxus_web::launch(ui::app);
  }
}
