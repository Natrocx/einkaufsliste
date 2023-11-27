use std::cell::Cell;
use std::collections::HashMap;

use async_std::stream::StreamExt;
use dioxus::prelude::*;
use dioxus_router::prelude::use_navigator;
use dioxus_signals::{use_signal, Effect, Signal};
use einkaufsliste::model::item::Item;
use einkaufsliste::model::list::{FlatItemsList, List};

use crate::service::api::{APIError, ApiService};
use crate::ui::consts::*;
use crate::ui::scaffold::PageHeader;
use crate::ui::Route;

enum SyncType {
  Meta(Signal<List>),
  NewItem(u64, Signal<Item>),
  UpdateItem(Signal<Item>),
}
#[component]
pub fn ListLoader(cx: Scope, id: u64) -> Element {
  let error_handler: &Coroutine<APIError> = use_coroutine_handle(cx)?;
  let api = use_context::<ApiService>(cx).unwrap();
  let navigator = use_navigator(cx);
  use_coroutine(cx, move |mut rx| {
    to_owned![api, error_handler];
    async move {
      while let Some(message) = rx.next().await {
        let api_result = match message {
          SyncType::Meta(meta) => api.update_list_with_ref(meta.read()).await,
          SyncType::UpdateItem(item) => api.update_item_with_ref(item.read()).await,
          // When a new item is created, we need to tell the backend which list it belongs to;
          // the backend will further generate a new id for the item which is set here.
          SyncType::NewItem(list_id, item) => api
            .new_item(list_id, item.read().clone())
            .await
            .map(|id| item.write().id = id),
        };

        // Display potential errors to the user
        match api_result {
          Ok(_) => {}
          Err(e) => {
            error_handler.send(e);
          }
        }
      }
    }
  });

  // Fetch list from server and provide as context when successful
  let list = use_future(cx, id, move |id| {
    // copy Rc's
    to_owned![api, error_handler, navigator];
    async move {
      let list_res = api.fetch_list(id).await;
      match list_res {
        Ok(list) => {
          tracing::debug!("List fetched: {:?}", list.name.as_str());
          let (meta, items) = list.into_list_and_items();
          let meta = Signal::new(meta);
          let items: Signal<Vec<Signal<Item>>> = Signal::new(items.into_iter().map(Signal::new).collect());
          Ok((meta, items))
        }
        Err(e) => {
          error_handler.send(e);
          // If the initial fetch fails, there is nothing the user can do on this page so we redirect them to the home page
          if navigator.can_go_back() {
            navigator.go_back();
          } else {
            navigator.replace(Route::Home);
          };
          Err(())
        }
      }
    }
  });

  match list.value() {
    Some(Ok((meta, items))) => render!( ListPage { meta: *meta, items: *items } ),
    Some(Err(_)) => {
      render!("An error occured. You are being redirected.")
    }
    None => {
      render!("Loading")
    }
  }
}

#[component]
pub fn ListPage(cx: Scope, meta: Signal<List>, items: Signal<Vec<Signal<Item>>>) -> Element {
  let owned_meta = *meta;
  let syncer = use_coroutine_handle::<SyncType>(cx)?.clone();
  
  // Register effect to sync meta data to backend
  dioxus_signals::use_effect(cx, move || {
    // register effect and discard the unneeded RefGuard (it cannot be sent to the coroutine)
    let _ = owned_meta.read();
    syncer.send(SyncType::Meta(owned_meta));
  });

  let meta = *meta;
  let items = *items;
  let syncer = use_coroutine_handle::<SyncType>(cx)?.clone();

  // The compiler demands a binding!
  let x = render! {
    PageHeader { 
        input {
            class: "w-full {PRIMARY_BG}",
            onchange: move |evt| meta.write().name = evt.value.clone(),
            value: "{meta.read().name.as_str()}"
        }
    }
    div { class: "space-y-1",
        for item in items {
            ItemView { item: item }
        }
    }
    div { class: "flex",
        button {
            class: "material-symbols-outlined",
            onclick: move |_| {
                let new_item = Signal::new(Item {
                    name: "".to_string(),
                    checked: false,
                    id: 0,
                    amount: None,
                    unit: None,
                    article_id: None,
                    alternative_article_ids: None,
                });
                syncer.send(SyncType::NewItem(meta.read().id, new_item));
                items.write().push(new_item)
            },
        ADD
        }
    }
    input { "search" }
    span { class: "material-symbols-outlined", SEARCH }
};
x
}

#[component]
pub(super) fn ItemView(cx: Scope, item: Signal<Item>) -> Element {
  let syncer = use_coroutine_handle::<SyncType>(cx)?.clone();
  to_owned![item];
  dioxus_signals::use_effect(cx, move || {
    let _ = item.read();
    syncer.send(SyncType::UpdateItem(item));
  });

  // Unnecessary bindings are the price we pay for the compiler to be happy
  // You know the saying - happy compiler, happy life
  let x = render!(
    div { class: "flex",
        span { class: "material-symbols-outlined",
            if item.read().checked {
            CHECKBOX_CHECKED
            } else {
            CHECKBOX_UNCHECKED
            }
        }
        input {
            class: "{INLINE_INPUT} flex-grow",
            onchange: move |evt| item.write().name = evt.value.clone(),
            value: "{item.read().name}"
        }
        span { class: "material-symbols-outlined", DELETE }
    }
  );
    x
}
