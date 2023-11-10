use dioxus::prelude::*;
use dioxus_router::prelude::use_navigator;
use einkaufsliste::model::list::{FlatItemsList, List};

use crate::service::api::{APIError, ApiService};
use crate::service::list::{use_provide_list_service, ListService};
use crate::ui::scaffold::PageHeader;

#[component]
pub fn ListLoader(cx: Scope, id: u64) -> Element {
  let error_handler: &Coroutine<APIError> = use_coroutine_handle(cx)?;
  let api = use_context::<ApiService>(cx).unwrap();
  let navigator = use_navigator(cx);

  // Fetch list from server and provide as context when successful
  let temp_list: &UseState<Option<FlatItemsList>> = use_state(cx, || None);
  use_provide_list_service(cx, || UseState::make_mut(temp_list).take());
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
      render!("")
    }
    None => {
      render!("Loading")
    }
  }
}

#[component]
pub fn ListPage(cx: Scope) -> Element<'_> {
  let list_service = use_context::<ListService>(cx)?;
  let meta = list_service.meta();
  
  let x = render! {
      PageHeader { page_title: "{meta.read().name}"}
      // todo: add navigation/check all interactivity

  };
  x
}