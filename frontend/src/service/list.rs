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
use einkaufsliste::model::requests::UpsertItems;

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
  items: Signal<Vec<Signal<Item>>>,
  changed_items: RefCell<HashMap<u64, Signal<Item>>>,
  api_service: ApiService,
  error_handler: Coroutine<APIError>,
}

#[derive(Debug, Clone, Copy)]
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
            items,
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
            tracing::trace!("Received sync request: {sync_type:?}");
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
let syncer = use_coroutine_handle(cx).unwrap().clone();
  dioxus_signals::use_effect(cx, move || {
    to_owned![list_service];
    list_service.item_changed(item);
    syncer.send(SyncType::Items);
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

  #[tracing::instrument(skip(self))]
  pub async fn sync_meta(&self) {
      let list_meta = self.inner.meta;
      // extract binary representation of list meta and drop RefGuard before awaiting
      let fut = self.inner.api_service.update_list_with_ref(list_meta.read());
      let result = fut.await;

      match result {
        Ok(()) => (),
        Err(e) => {
          self.inner.error_handler.send(e);
        }
      }
  }

  #[tracing::instrument(skip(self))]
  pub async fn sync_items(&self) {
    // TODO make untracked when it is implemented
    let list_id = self.inner.meta.read().id;
    let changed_items = self.inner.changed_items.borrow_mut().drain().map(|(_, item)| item()).collect::<Vec<_>>();

    let fut = self.inner.api_service.upsert_items_with_ref( list_id, changed_items); 
    let result = fut.await;

    match result {
        Ok(_) => (),
        Err(e) => {
          self.inner.error_handler.send(e);
        }
    }
  }

  pub fn item_changed(&self, item: Signal<Item>) {
    let item_id = item.read().id;
    let mut changed_items = self.inner.changed_items.borrow_mut();
    changed_items.insert(item_id, item);
  }

}
