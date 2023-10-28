#![feature(async_fn_in_trait)]

use tracing::dispatcher::set_global_default;
use tracing_subscriber::filter::{LevelFilter, Targets};

pub mod service;
pub mod ui;

fn main() {
  #[cfg(not(target_arch = "wasm32"))]
  {
    setup_tracing();
    dioxus_desktop::launch(ui::app);
  }

  #[cfg(target_arch = "wasm32")]
  {
    dioxus_web::launch(ui::app);
  }
}

pub fn setup_tracing() {
  use tracing_subscriber::prelude::*;

  let filter_layer = Targets::new()
    .with_target("h2", LevelFilter::OFF)
    .with_target("actix_identity", LevelFilter::ERROR)
    .with_target("sled", LevelFilter::WARN)
    .with_default(LevelFilter::DEBUG);

  let fmt_layer = tracing_subscriber::fmt::layer().pretty();

  let subscriber = tracing_subscriber::registry().with(filter_layer).with(fmt_layer);

  set_global_default(subscriber.into()).unwrap();
}
