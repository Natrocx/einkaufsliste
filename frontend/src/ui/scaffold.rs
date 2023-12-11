use dioxus::html::*;
use dioxus::prelude::*;
use dioxus_router::prelude::{use_navigator, GoBackButton};

use crate::ui::consts;

#[component]
pub fn PageHeader<'a>(cx: Scope, children: Element<'a>) -> Element {
  let navigator = use_navigator(cx);
  let show_back_navigation = navigator.can_go_back();

  render!(
    //page header with title and nav
    div { class: "flex flex-nowrap flex-row",
      if show_back_navigation {
          rsx!(
              GoBackButton {
                  span {
                      class: "m-1 material-symbols-outlined",
                      consts::NAVIGATE_BACK
              }
          }
          )
      }
      children
    }
  )
}
