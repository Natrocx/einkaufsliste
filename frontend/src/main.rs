use frontend::setup_tracing;

pub mod completions;
pub mod index_desktop_html;
pub mod service;
pub mod ui;

fn main() {
  setup_tracing();
  #[cfg(not(target_arch = "wasm32"))]
  {
    use dioxus_desktop::Config;

    dioxus_desktop::launch_cfg(
      ui::app,
      Config::default().with_custom_index(index_desktop_html::INDEX_HTML.to_string()),
    );
  }

  #[cfg(target_arch = "wasm32")]
  {
    dioxus_web::launch(ui::app);
  }
}
