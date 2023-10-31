// expose services to enable testing them independently
pub mod service;
pub mod ui;

#[cfg(not(target_arch = "wasm32"))]
pub fn setup_tracing() {
  use tracing::dispatcher::set_global_default;
  use tracing_subscriber::filter::*;
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

#[cfg(target_arch = "wasm32")]
pub fn setup_tracing() {
  use tracing_subscriber::prelude::*;
  use tracing_subscriber_wasm::MakeConsoleWriter;

  let filter_layer = Targets::new()
    .with_target("h2", LevelFilter::OFF)
    .with_target("actix_identity", LevelFilter::ERROR)
    .with_target("sled", LevelFilter::WARN)
    .with_default(LevelFilter::DEBUG);

  let fmt_layer = tracing_subscriber::fmt::layer()
    .with_writer(
      // To avoide trace events in the browser from showing their
      // JS backtrace, which is very annoying, in my opinion
      MakeConsoleWriter::default().map_trace_level_to(tracing::Level::DEBUG),
    )
    // For some reason, if we don't do this in the browser, we get
    // a runtime error.
    .without_time();

  let subscriber = tracing_subscriber::registry().with(filter_layer).with(fmt_layer);

  set_global_default(subscriber.into()).unwrap();
}
