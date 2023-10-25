use std::{future::Future, pin::Pin, ops::Deref};
use std::marker::PhantomData;
use std::mem::MaybeUninit;
use std::rc::Rc;
use std::sync::{RwLock, RwLockReadGuard};
use std::time::Instant;

use dioxus::prelude::{use_ref, use_shared_state_provider, Coroutine, Scope, SvgAttributes, UseRef, UseState};
use einkaufsliste::model::item::Item;
use einkaufsliste::model::list::{FlatItemsList, List};
use url::form_urlencoded::Target;

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
  pub fn title(&self) -> OwningHandle<'_, Box<ListServiceInner>, String>  {
    let lock = self.inner.read().unwrap();
    OwningHandle::new(lock, |lock| unsafe { &(*lock).data.name as *const String })
  }

  pub fn items(&self) -> OwningHandle<'_, Box<ListServiceInner>, Vec<Item>> {
    let lock = self.inner.read().unwrap();

    OwningHandle::new(lock, |lock| unsafe { &(*lock).data.items as *const Vec<Item>} )
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

  async fn sync(&self) {
    async_std::task::sleep(TIMEOUT).await;
    // if the user has not edited the value since the edit this function was called for, sync to the server now
    let inner = self.inner.read().unwrap();
    if inner.last_edit.elapsed() > TIMEOUT {
      let list_meta = <List as From<&FlatItemsList>>::from(&inner.data);
      // do not hold locks while waiting for the server
      drop(inner);
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
    let mut lock = self.inner.write().unwrap();
    let list_id = lock.data.id;
    match self.api_service.fetch_list(&list_id).await {
      Ok(list) => {
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

pub struct OwningHandle<'a, Container, Inner> {
  inner: RwLockReadGuard<'a, Pin<Container>>,
  extracted: *const Inner,
}

impl<'a, Container, Inner> OwningHandle<'a, Container, Inner> {
  pub fn new<Op: FnOnce(*const Container) -> *const Inner>(
    inner: RwLockReadGuard<'a, Pin<Container>>,
    extractor: Op,
  ) -> Self {
    let container: *const Container = unsafe { std::mem::transmute(inner.deref() as *const Pin<Container>) };
    let extracted = extractor(container);
    Self { inner, extracted }
  }
}

impl<'a, C, I> std::ops::Deref for OwningHandle<'a, C, I> {
  type Target = I;

  fn deref(&self) -> &Self::Target {
    // this is safe as we maintain ownership of the underlying data/lock and our type must be Pin
    unsafe { &*self.extracted }
  }
}
