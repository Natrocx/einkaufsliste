use std::{future::Future, rc::Rc};
use std::ops::Deref;
use std::pin::Pin;
use std::sync::{RwLock, RwLockReadGuard};
use std::time::Instant;

use dioxus::prelude::*;
use einkaufsliste::model::item::Item;
use einkaufsliste::model::list::{FlatItemsList, List};

use super::api::{APIError, ApiService};

//TODO: make configurable
pub static TIMEOUT: std::time::Duration = std::time::Duration::from_secs(1);

pub struct ListServiceInner {
  // basically a Rc<RefCell<FlatItemsList>>
  data: FlatItemsList,
  last_edit: Instant,
  //TODO: maintain diff?
}

/**
 This service is used to sync a list with the server. It facilitates batching of writes and ratelimiting of requests to the server.
  Error handling is integrated with [error_handler](crate::ui::error::error_handler)/it's [Coroutine].

*/
pub struct ListService {
  inner: RwLock<Pin<Box<ListServiceInner>>>,
  api_service: ApiService,
  error_handler: Coroutine<APIError>,
}

/**
 This hook creates a service to sync a list with the server.

# Panics
The hook will panic if any of the following services are not available:
  * [ApiService]
  * ErrorHandlers [Coroutine<APIError>]

Panics may also occur, if the underlying UseRef is written to outside of the hook/service.
*/
pub fn use_provide_list_service<T>(cx: Scope<'_, T>, init: impl FnOnce() -> FlatItemsList) {
  use_shared_state_provider(cx, || {
    let api_service = cx.consume_context::<ApiService>().unwrap();
    let error_handler = cx.consume_context::<Coroutine<APIError>>().unwrap();
    ListService::new(init(), api_service, error_handler)
  });
}

impl ListService {
  pub fn new(data: FlatItemsList, api_service: ApiService, error_handler: Coroutine<APIError>) -> Self {
    Self {
      inner: RwLock::new(Box::pin(ListServiceInner {
        data,
        last_edit: Instant::now(),
      })),
      api_service,
      error_handler,
    }
  }

  // the code is so ugly, I'd rather hide it here
  pub fn title(&self) -> OwningHandle<RwLockReadGuard<'_, Pin<Box<ListServiceInner>>>, Box<ListServiceInner>, String> {
    let lock = self.inner.read().unwrap();
    OwningHandle::new(lock, |lock| unsafe { &(*lock).data.name as *const String })
  }

  pub fn items(
    &self,
  ) -> OwningHandle<RwLockReadGuard<'_, Pin<Box<ListServiceInner>>>, Box<ListServiceInner>, Vec<Rc<Item>>> {
    let lock = self.inner.read().unwrap();

    OwningHandle::new(lock, |lock| unsafe { &(*lock).data.items as *const Vec<Rc<Item>>} )
  }

  /// Immediately updates the list with the provided updater and returns a future to sync with the server.
  pub fn update(&self, updater: impl FnOnce(&mut FlatItemsList)) -> impl Future + '_ {
    // batch writes and drop writeguard
    {
      let mut inner = self.inner.write().unwrap();
      updater(&mut inner.data);
      inner.last_edit = Instant::now();
    }

    // update the server after a timeout/ratelimit
    self.sync()
  }

  // this lint is incorrect as clippy apparently cannot deal with the drop() call
  #[allow(clippy::await_holding_lock)]
  async fn sync(&self) {
    async_std::task::sleep(TIMEOUT).await;
    // if the user has not edited the value since the edit this function was called for, sync to the server now

    let lock = self.inner.read().unwrap();
    if lock.last_edit.elapsed() > TIMEOUT {
      let list_meta = <List as From<&FlatItemsList>>::from(&lock.data);
      // do not hold locks while waiting for the server
      drop(lock);
      let result = self.api_service.update_list(&list_meta).await;

      match result {
        Ok(()) => (),
        Err(e) => {
          self.error_handler.send(e);
        }
      }
    }
  }

  /**
   * s
  This function will attempt to fetch the [FlatItemsList] corresponding to the provided id and update the UIs use_ref.
  Returns `true` if the fetch was successful, otherwise it will return `false` and display the error to the user. Use this information to consider restarting the request.
  */
  pub async fn get_items(&self) -> bool {
    let list_id = {
      let lock = self.inner.read().unwrap();
      lock.data.id
    };
    match self.api_service.fetch_list(&list_id).await {
      Ok(list) => {
        let mut lock = self.inner.write().unwrap();
        lock.data = list;

        true
      }
      Err(e) => {
        self.error_handler.send(e);
        false
      }
    }
  }
}

/// This struct enables one to derefence to any value that is protected by a ReadGuard/Smart Pointer which requires Ownership of the Smart Pointer. It further enables one to access only some fields of the inner type and therefore hide the inner structure of the data from consumers.
///
/// # Safety
/// As long as the provided pointers are not manipulated and only dereferenced, using this struct is safe.
pub struct OwningHandle<ReadGuard: Deref<Target = Pin<Container>>, Container, Inner> {
  // while inner is never read after extraction, it is still necesarry to hold ownership
  _inner: ReadGuard,
  extracted: *const Inner,
}

impl<ReadGuard: Deref<Target = Pin<Container>>, Container, Inner> OwningHandle<ReadGuard, Container, Inner> {
  pub fn new<Op: FnOnce(*const Container) -> *const Inner>(inner: ReadGuard, extractor: Op) -> Self {
    // we "cast" the Pin away since it is a `#[layout(transparent)]` struct and thus has the same memory layout as the inner type, which we require for our extractor
    let container: *const Container = unsafe { std::mem::transmute(inner.deref() as *const Pin<Container>) };
    let extracted = extractor(container);
    Self { _inner: inner, extracted }
  }
}

impl<R: Deref<Target = Pin<C>>, C, I> std::ops::Deref for OwningHandle<R, C, I> {
  type Target = I;

  fn deref(&self) -> &Self::Target {
    // this is safe as we maintain ownership of the underlying data/lock and our type must be Pin
    unsafe { &*self.extracted }
  }
}
