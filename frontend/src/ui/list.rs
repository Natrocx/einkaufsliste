use dioxus::prelude::*;
use dioxus_router::prelude::use_navigator;
use dioxus_signals::Signal;
use einkaufsliste::model::item::Item;
use einkaufsliste::model::list::{FlatItemsList, List};

use crate::service::api::{APIError, ApiService};
use crate::service::list::{use_item_effects, use_list_service, use_provide_list_service, ListService};
use crate::ui::consts::*;
use crate::ui::scaffold::PageHeader;
use crate::ui::Route;

#[component]
pub fn ListLoader(cx: Scope, id: u64) -> Element {
  let error_handler: &Coroutine<APIError> = use_coroutine_handle(cx)?;
  let api = use_context::<ApiService>(cx).unwrap();
  let navigator = use_navigator(cx);

  // Fetch list from server and provide as context when successful
  let temp_list: &UseRef<Option<FlatItemsList>> = use_ref(cx, || None);
  let list_fetch_success = use_future(cx, id, move |id| {
    // copy Rc's
    to_owned![api, error_handler, navigator, temp_list];
    async move {
      let list = api.fetch_list(id).await;
      match list {
        Ok(list) => {
          tracing::debug!("List fetched: {:?}", list.name.as_str());
          *temp_list.write_silent() = Some(list);
          true
        }
        Err(e) => {
          error_handler.send(e);
          if navigator.can_go_back() {
            navigator.go_back();
          } else {
            navigator.replace(Route::Home);
          };
          false
        }
      }
    }
  });
  use_provide_list_service(cx, temp_list.write_silent().take());

  match list_fetch_success.value() {
    Some(true) => {
      render!( ListPage {} )
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
pub(super) fn ListPage(cx: Scope) -> Element<'_> {
  let list_service = use_list_service(cx);
  let meta = list_service.meta();
  let items = list_service.items();

  let x = render! {
    PageHeader { 
        input {
            onchange: move |evt| meta.write().name = evt.value.clone(),
            value: "{meta.read().name.as_str()}"
        }
    }
    for item in items {
        ItemView { item: item }
    }
    div { class: "flex",
        button {
            class: "material-symbols-outlined",
            onclick: move |_| {
                items
                    .write()
                    .push(
                        Signal::new(Item {
                            name: "".to_string(),
                            checked: false,
                            id: 0,
                            amount: None,
                            unit: None,
                            article_id: None,
                            alternative_article_ids: None,
                        }),
                    )
            },
            ADD
        }
        input { "search" }
        span { class: "material-symbols-outlined", SEARCH }
    }
};
  x
}

#[component]
pub(super) fn ItemView(cx: Scope, item: Signal<Item>) -> Element {
  let list_service = use_list_service(cx);
  // sync updates
  use_item_effects(cx, list_service.clone(), *item);
  render!(
    div { class: "flex",
        span { class: "material-symbols-outlined",
            if item.read().checked {
            CHECKBOX_CHECKED
            } else {
            CHECKBOX_UNCHECKED
            }
        }
        input {
            class: "flex-grow",
            onchange: move |evt| item.write().name = evt.value.clone(),
            value: "{item.read().name}"
        }
        span { class: "material-symbols-outlined", DELETE }
    }
)
}
