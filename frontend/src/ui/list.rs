use async_std::stream::StreamExt;
use dioxus::prelude::*;
use dioxus_router::prelude::use_navigator;
use dioxus_signals::{use_signal, Effect, Signal};
use einkaufsliste::model::item::Item;
use einkaufsliste::model::list::{FlatItemsList, List};
use einkaufsliste::model::requests::DeleteItem;
use tracing::debug;

use crate::service::api::{APIError, ApiService};
use crate::ui::consts::*;
use crate::ui::scaffold::PageHeader;
use crate::ui::Route;

#[derive(Debug, Clone)]
enum SyncType {
  Meta(Signal<List>),
  NewItem(Signal<Item>),
  UpdateItem(Signal<Item>),
  DeleteItem(u64),
}

#[component]
pub fn ListLoader(cx: Scope, id: u64) -> Element {
  let error_handler: &Coroutine<APIError> = use_coroutine_handle(cx)?;
  let api = use_context::<ApiService>(cx).unwrap();
  let navigator = use_navigator(cx);
  use_coroutine(cx, move |mut rx| {
    to_owned![api, error_handler];
    let list_id = *id;
    async move {
      while let Some(message) = rx.next().await {
        tracing::debug!("Syncing with backend: {:?}", message);
        let api_result = match message {
          SyncType::Meta(meta) => api.update_list_with_ref(meta.read()).await,
          SyncType::UpdateItem(item) => api.update_item_with_ref(item.read()).await,
          // When a new item is created, we need to tell the backend which list it belongs to;
          // the backend will further generate a new id for the item which is set here.
          SyncType::NewItem(item) => {
            let item_data = item.read().clone();

            api.new_item(list_id, item_data).await.map(|id| item.write().id = id)
          },
          SyncType::DeleteItem(item_id) => api.delete_item(DeleteItem {list_id, item_id}).await,
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
    Some(Ok((meta, items))) => render!(ListPage {
      meta: *meta,
      items: *items
    }),
    Some(Err(_)) => {
      render!("An error occured. You are being redirected.")
    }
    None => {
      render!("Loading")
    }
  }
}

/// This component serves as an "inner" component for the ListPage, maintaining effects and other
/// state that needs to be initialized conditionally.
#[component]
pub fn ListPage(cx: Scope, meta: Signal<List>, items: Signal<Vec<Signal<Item>>>) -> Element {
  let owned_meta = *meta;
  let syncer = use_coroutine_handle::<SyncType>(cx)?.clone();
  let first_render = use_state(cx, || true).clone();

  // Register effect to sync meta data to backend
  dioxus_signals::use_effect(cx, move || {
    // register effect and discard the unneeded RefGuard (it cannot be sent to the coroutine)
    let _ = owned_meta.read();

    if !*first_render.current() {
      syncer.send(SyncType::Meta(owned_meta));
    } else {
      first_render.set(false);
    }
  });

  let meta = *meta;
  let items = *items;

  // The compiler demands a binding!
  let x = render! {
    div {
      class: "flex flex-col h-full",
      PageHeader {
          input {
              class: "w-full {PRIMARY_BG}",
              onchange: move |evt| meta.write().name = evt.value.clone(),
              value: "{meta.read().name.as_str()}"
          }
      }
      div { class: "space-y-1 flex-grow",
          for item in items {
              //TODO: read untracked
              ItemView { key: "{item.read().id}", item: item, all_items: items}
          }
      }
      AddItemView { items: items }
    }
  };
  x
}

#[component]
pub fn AddItemView(cx: Scope, items: Signal<Vec<Signal<Item>>>) -> Element {
  let syncer = use_coroutine_handle(cx)?;
  let new_item_name = use_signal(cx, String::new);

  let x = render!(
      form {
          class: "flex m-1",
          onsubmit: move |evt| {
              let item_name = evt.values["new-item-name"][0].clone();
              let new_item = Signal::new(Item {
                  name: item_name,
                  checked: false,
                  id: 0,
                  amount: None,
                  unit: None,
                  article_id: None,
                  alternative_article_ids: None,
              });
              syncer.send(SyncType::NewItem(new_item));
              items.write().push(new_item)
          },
          button { class: "material-symbols-outlined", r#type: "submit", ADD }

          input {
              id: "new-item-name",
              name: "new-item-name",
              class: "flex-grow {INLINE_INPUT}",
              onchange: move |evt| *new_item_name.write() = evt.value.clone(),
              value: "{new_item_name.read()}"
          }
          span { class: "material-symbols-outlined", SEARCH }
      }
  );
  x
}

#[component]
pub(super) fn ItemView(cx: Scope, item: Signal<Item>, all_items: Signal<Vec<Signal<Item>>>) -> Element {
  let _syncer = use_coroutine_handle::<SyncType>(cx)?;
  let first_render = use_state(cx, || true).clone();
  to_owned![item];

  let syncer = _syncer.clone();
  dioxus_signals::use_effect(cx, move || {
    let _ = item.read();
    if !*first_render.current() {
      syncer.send(SyncType::UpdateItem(item));
    } else {
      first_render.set(false);
    }
  });

  // This cannot go inline, as it will cause the underlying RefCell to panic
  let checked = item.read().checked;
  
  let syncer = _syncer.clone();

  // Unnecessary bindings are the price we pay for the compiler to be happy
  // You know the saying - happy compiler, happy life
  let x = render!(
      div { class: "flex",
          button {
              class: "material-symbols-outlined",
              onclick: move |_| {
                  item.write().checked = !checked;
              },
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
          button {
              class: "material-symbols-outlined",
              onclick: move |_| {
                  let mut all_items = all_items.write();
                  //TODO: read untracked - shouldn't matter much since the component will be destroyed but it will be faster once implemented
                  let idx = all_items.iter().position(|i| i.read().id == item.read().id).unwrap();
                  all_items.remove(idx);
                  syncer.send(SyncType::DeleteItem(item.read().id));
              },
              DELETE
          }
      }
  );
  x
}
