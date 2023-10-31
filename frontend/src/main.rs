use frontend::setup_tracing;

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
