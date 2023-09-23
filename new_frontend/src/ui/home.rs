use dioxus::prelude::*;
use dioxus_router::prelude::Link;

use crate::ui::Route;

pub fn homepage(cx: Scope) -> Element {
  cx.render(rsx! {
      div {
          "Hello Homepage"
          Link { to: Route::Authentication, "Auth" }
      }
  })
}
