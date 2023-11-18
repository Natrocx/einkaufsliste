use std::cell::Cell;
use std::collections::HashMap;
use std::future::Future;
use std::mem::MaybeUninit;
use std::rc::Rc;
use std::time::Instant;

use async_std::stream::StreamExt;
use dioxus::prelude::*;
use dioxus_signals::{use_effect_with_dependencies, use_signal, Effect, ReadOnlySignal, Signal};
use einkaufsliste::model::item::Item;
use einkaufsliste::model::list::{FlatItemsList, List};

use super::api::{APIError, ApiService};

//TODO: make configurable
pub static TIMEOUT: std::time::Duration = std::time::Duration::from_secs(1);

/**
 This service is used to sync a list with the server. It facilitates batching of writes and ratelimiting of requests to the server.
  Error handling is integrated with [error_handler](crate::ui::error::error_handler)/it's [`Coroutine<ApiError>`](Coroutine).

*/
#[derive(Clone)]
pub struct ListService {
  inner: Rc<ListServiceInner>,
}

// Contains data for both list meta and list items since handling items alone makes little sense
pub struct ListServiceInner {
  meta: Signal<List>,
  meta_last_edit: Cell<Instant>,
  items: Signal<Vec<Signal<Item>>>,
  items_last_edit: Cell<Instant>,
  changed_items: RefCell<HashMap<u64, Signal<Item>>>,
  api_service: ApiService,
  error_handler: Coroutine<APIError>,
}

pub enum SyncType {
  Meta,
  Items,
}

/**
 This hook creates a service to sync a list with the server.

# Panics
The hook will panic if any of the following services are not available:
  * [ApiService]
  * ErrorHandlers [Coroutine<APIError>]

*/
pub fn use_provide_list_service<Component>(cx: Scope<'_, Component>, initial: Option<FlatItemsList>) {
  tracing::trace!("Provide list service called with: {:?}", &initial);

  // this code does unsafe things with dioxus' use_ hooks. When a hook is called in this conditional, a hook of the same type must be called in the else branch.
  #[allow(unused_unsafe)]
  // this might actually just explode if rewritten carelessly. Dioxus does not forward the unsafe keyword to the hooks though.
  unsafe {
    if let Some(list) = initial {
      cx.provide_context({
        let (meta, items) = list.into_list_and_items();

        // load required services from context
        let api_service = cx.consume_context::<ApiService>().unwrap();
        let error_handler = cx.consume_context::<Coroutine<APIError>>().unwrap();

        // register seperate signals for items (for ItemViews) and Meta (for ListPage)
        let meta = Signal::new(meta);
        let items = Signal::new(items.into_iter().map(Signal::new).collect());

        let list_service = ListService {
          inner: ListServiceInner {
            meta,
            meta_last_edit: Instant::now().into(),
            items,
            items_last_edit: Instant::now().into(),
            changed_items: Default::default(),
            api_service,
            error_handler,
          }
          .into(),
        };

        // we register an effect that will sync the meta info with the server if it has not been updated for a while.
        // since we require a handle to list_service here, we need to use a MaybeUninit to make a self-reference
        let syncer = use_coroutine(cx, |mut rx: UnboundedReceiver<SyncType>| {
          to_owned![list_service];
          async move {
            while let Some(sync_type) = rx.next().await {
              match sync_type {
                SyncType::Meta => list_service.sync_meta().await,
                SyncType::Items => list_service.sync_items().await,
              }
            }
          }
        })
        .clone();

        let effect_service = list_service.clone();
        dioxus_signals::use_effect(cx, move || {
          let _ = effect_service.meta().read();
          tracing::trace!("Sending meta sync request to list_service");
          syncer.send(SyncType::Meta);
        });

        list_service
      });
    } else {
      use_coroutine(cx, |mut rx: UnboundedReceiver<SyncType>| async {});
      dioxus_signals::use_effect(cx, || {});
    }
  };
}

pub fn use_list_service<Component>(cx: Scope<'_, Component>) -> &ListService {
  // Get required services from context
  let service = use_context::<ListService>(cx).unwrap();

  service
}

/// Create all the effects required to sync the list with the server. 
/// The effects will not be returned as there is no need to directly interact with them.
pub fn use_item_effects<Component>(cx: Scope<'_, Component>, list_service: ListService, item: Signal<Item>) {
  dioxus_signals::use_effect(cx, move || {
    to_owned![list_service];
    list_service.item_changed(item);
  });
}

impl ListService {
  pub fn meta(&self) -> Signal<List> {
    self.inner.meta
  }

  /// Returns a read-only Signal to the items of the list.
  ///
  /// If you wish to modify the items, you must register your own effects to synchronise the changes with the server.
  pub fn items(&self) -> Signal<Vec<Signal<Item>>> {
    self.inner.items
  }

  pub async fn sync_meta(&self) {
    if Self::debounce(&self.inner.meta_last_edit).await {
      let list_meta = self.inner.meta;
      let fut = self.inner.api_service.update_list_with_ref(list_meta.read());
      let result = fut.await;

      match result {
        Ok(()) => (),
        Err(e) => {
          self.inner.error_handler.send(e);
        }
      }
    }
    // otherwise drop the future produced by this function and have the future, that was created by the edit, sync the meta info
  }

  pub async fn sync_items(&self) {
    todo!()
  }

  pub fn item_changed(&self, item: Signal<Item>) {
    let item_id = item.read().id;
    let mut changed_items = self.inner.changed_items.borrow_mut();
    changed_items.insert(item_id, item);
  }

  /// Checks if edits have occured during the timeout.
  ///
  /// Returns true if the event "passed" the debouncing test and the reaction (syncing in this case) should be executed.
  async fn debounce(last_edit: &Cell<Instant>) -> bool {
    // memorize the time of the edit that triggered this function
    last_edit.set(Instant::now());

    async_std::task::sleep(TIMEOUT).await;

    // see if any edits occured during the timeout
    let last_edit = last_edit.get();

    // if no edits occured, sync the meta info with the server
    last_edit.elapsed() > TIMEOUT
  }
}
