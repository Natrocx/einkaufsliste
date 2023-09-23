mod service;
mod ui;

fn main() {
  #[cfg(not(target_arch = "wasm32"))]
  dioxus_desktop::launch(ui::app);

  #[cfg(target_arch = "wasm32")]
  dioxus_web::launch(ui::app);
}
