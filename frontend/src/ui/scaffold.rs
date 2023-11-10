use dioxus::html::*;
use dioxus::prelude::*;
use dioxus_router::prelude::use_navigator;
use dioxus_signals::Signal;

use crate::ui::consts;

#[component]
pub fn PageHeader<'a>(cx: Scope, page_title: &'a str) -> Element {
  let navigator = use_navigator(cx);
  let show_back_navigation = navigator.can_go_back();

  render!(
    //page header with title and nav
    div {
        if show_back_navigation {
            rsx!(span {
            class: "material-symbols-outlined",
            consts::NAVIGATE_BACK
            }
            )
        }
        p { page_title }
    }
)
}
