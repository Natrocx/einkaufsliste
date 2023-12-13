use async_std::stream::StreamExt;
use dioxus::html::div;
use dioxus::prelude::*;
use dioxus_router::prelude::use_navigator;
use dioxus_signals::{use_signal, Signal};
use einkaufsliste::model::item::Item;
use einkaufsliste::model::list::List;
use einkaufsliste::model::requests::DeleteItem;
use rs_complete::CompletionTree;

use crate::completions::complete;
use crate::service::api::{APIError, ApiService};
use crate::ui::consts::*;
use crate::ui::item::ItemView;
use crate::ui::scaffold::PageHeader;
use crate::ui::Route;

#[derive(Debug, Clone)]
pub(super) enum SyncType {
  Meta(Signal<List>),
  NewItem(String),
  UpdateItem(Signal<Item>),
  DeleteItem(u64),
}

// This component is the entry point for the list page. It will fetch the list from the server and needs to be its own component as to enable using hooks conditionally.
#[component]
pub fn ListLoader(cx: Scope, id: u64) -> Element {
  let error_handler: &Coroutine<APIError> = use_coroutine_handle(cx)?;
  let api = use_context::<ApiService>(cx).unwrap();
  let navigator = use_navigator(cx);

  // Fetch list from server and provide as context when successful
  let list = use_future(cx, id, move |id| {
    to_owned![api, error_handler, navigator];
    async move {
      let list_res = api.fetch_list(id).await;
      match list_res {
        Ok(list) => {
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
  let syncer = make_syncer(cx, *items, meta.read().id);
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

  // Drag and drop related functionality that needs to be shared between ItemView components
  let dragged_item = use_state(cx, || None::<u64>);
  let ondrop = move |target: u64| {
    let dragged_id = dragged_item.get().unwrap();
    let mut idx_dragged = None;
    let mut idx_drop_target = None;
    let mut items = items.write();

    items.iter().enumerate().for_each(|(idx, item)| {
      let id = item.read().id;
      if id == dragged_id {
        idx_dragged = Some(idx);
      } else if id == target {
        idx_drop_target = Some(idx);
      }
    });

    if let Some(idx_dragged) = idx_dragged {
      if let Some(idx_drop_target) = idx_drop_target {
        // optimize? might copy all items around
        let dragged_item = items.remove(idx_dragged);
        items.insert(idx_drop_target, dragged_item);
      }
    }
  };

  let ondragstart = move |id: u64| {
    dragged_item.set(Some(id));
  };

  let meta = *meta;
  let items = *items;

  // The compiler demands a binding!
  let x = render! {
    div { class: "flex flex-col h-full",
      PageHeader {
        input {
          class: "w-full {PRIMARY_BG}",
          onchange: move |evt| meta.write().name = evt.value.clone(),
          value: "{meta.read().name.as_str()}"
        }
      }
      div { class: "flex-grow space-y-1",
        for item in items.into_iter().filter(|item| !item.read().checked) {
          //TODO: read untracked
          ItemView { key: "{item.read().id}", item: item, dragstart: ondragstart, drag_drop: ondrop }
        }
        hr { class: "h-px bg-zinc-500 border-0 mx-4 my-2" }
        for item in items.into_iter().filter(|item| item.read().checked) {

          ItemView { key: "{item.read().id}", item: item, dragstart: ondragstart, drag_drop: ondrop }
        }
      }
      AddItemView {items: items}
    }
  };
  x
}

#[component]
pub fn AddItemView(cx: Scope, items: Signal<Vec<Signal<Item>>>) -> Element {
  let syncer = use_coroutine_handle(cx)?;

  // This signal is used to reset the input field after submitting
  let new_item_name = use_signal(cx, String::new);

  to_owned![items];
  let completions = dioxus_signals::use_selector(cx, move || {
    let completions = complete(&items, &new_item_name.read());

    tracing::trace!("Completions for {{{new_item_name}}}: {:?}", completions);

    completions
  });

  let oninput = move |evt: Event<FormData>| {
    let input = &evt.value;
    *new_item_name.write() = input.clone();
  };

  // the input field has a high tabindex such that it is selected after the elements and we need to
  // make the generated completions selectable in the same order as they are rendered
  let mut rolling_tabindex = 20002;
  let x = render!(
    div {
      class: "mx-6 flex flex-col items-start {SECONDARY_BG}",
      for item in completions.value() {
        button {
          tabindex: rolling_tabindex,
          class: "flex-grow",
          onclick: move |_| {
            item.write().checked = false;
            syncer.send(SyncType::UpdateItem(item));
          },
          "{item.read().name}"
        }
        {rolling_tabindex += 1;}
      }
    }
    form {
      class: "flex m-1",
      onsubmit: move |evt| {
          let item_name = evt.values["new-item-name"][0].clone();
          new_item_name.write().clear();
          syncer.send(SyncType::NewItem(item_name));
      },
      button { tabindex: 20000, class: "material-symbols-outlined", r#type: "submit", ADD }

      input {
        tabindex: 20001,
        id: "new-item-name",
        name: "new-item-name",
        class: "flex-grow {INLINE_INPUT}",
        oninput: oninput,
        value: "{new_item_name.read()}"
      }
      // TODO: remove?
      span { class: "material-symbols-outlined", SEARCH }
    }
  );
  x
}

fn make_syncer<T>(cx: Scope<T>, items: Signal<Vec<Signal<Item>>>, list_id: u64) -> Coroutine<SyncType> {
  use_coroutine(cx, move |mut rx| {
    let api: ApiService = cx.consume_context().unwrap();
    let error_handler: Coroutine<APIError> = cx.consume_context().unwrap();

    #[allow(clippy::await_holding_refcell_ref)]
    async move {
      while let Some(message) = rx.next().await {
        tracing::debug!("Syncing with backend: {:?}", message);
        let api_result = match message {
          SyncType::Meta(meta) => api.update_list_with_ref(meta.read()).await,
          SyncType::UpdateItem(item) => api.update_item_with_ref(item.read()).await,
          // When a new item is created, we need to tell the backend which list it belongs to;
          // the backend will further generate a new id for the item which is set here.
          SyncType::NewItem(name) => {
            let new_item = Signal::new(Item {
              name,
              checked: false,
              id: 0,
              amount: None,
              unit: None,
              article_id: None,
              alternative_article_ids: None,
            });
            items.write().push(new_item);

            let item = new_item.read().clone();
            api.new_item(list_id, item).await.map(|id| new_item.write().id = id)
          }
          SyncType::DeleteItem(item_id) => {
            let idx = items
              .write()
              .iter()
              .enumerate()
              .find_map(|(idx, item)| if item.read().id == item_id { Some(idx) } else { None })
              .expect("Item to delete not found in list of all items");

            items.write().remove(idx);

            api.delete_item(DeleteItem { list_id, item_id }).await
          }
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
  })
  .clone()
}
