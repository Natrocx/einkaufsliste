use dioxus::prelude::*;
use dioxus_router::prelude::use_navigator;
use dioxus_signals::Signal;
use einkaufsliste::model::item::Item;
use einkaufsliste::model::list::{FlatItemsList, List};

use crate::service::api::{APIError, ApiService};
use crate::service::list::{use_provide_list_service, ListService, use_list_service};
use crate::ui::consts::*;
use crate::ui::scaffold::PageHeader;

#[component]
pub fn ListLoader(cx: Scope, id: u64) -> Element {
  let error_handler: &Coroutine<APIError> = use_coroutine_handle(cx)?;
  let api = use_context::<ApiService>(cx).unwrap();
  let navigator = use_navigator(cx);

  // Fetch list from server and provide as context when successful
  let temp_list: &UseRef<Option<FlatItemsList>> = use_ref(cx, || None);
  use_provide_list_service(cx, temp_list.write().take());
  let list_fetch_success = use_future(cx, id, move |id| {
    // copy Rc's
    to_owned![api, error_handler, navigator, temp_list];
    async move {
      let list = api.fetch_list(id).await;
      match list {
        Ok(list) => {
          temp_list.set(Some(list));
          true
        }
        Err(e) => {
          error_handler.send(e);
          navigator.go_back();
          false
        }
      }
    }
  });

  match list_fetch_success.value() {
    Some(true) => {
      render!(ListPage {})
    }
    Some(false) => {
      render!("Error??")
    }
    None => {
      render!("Loading")
    }
  }
}

#[component]
pub fn ListPage(cx: Scope) -> Element<'_> {
  let list_service = use_list_service(cx);
  let meta = list_service.meta();
  let items = list_service.items();
  
  let x = render! {
      PageHeader { page_title: "{meta.read().name.as_str()}"}
      // todo: add navigation/check all interactivity
      for item in items {
        rsx!(ItemView {item: item })
      }
    };
  x
}

#[component]
pub fn ItemView(cx: Scope, item: Signal<Item>) -> Element {
  render!(
    div {
      span {
        class: "material-symbols-outlined",
        if item.read().checked {
          CHECKBOX_CHECKED
        } else {
          CHECKBOX_UNCHECKED
        }
      }
      span {
        "{item.read().name}"
      }
    }
  )  
}