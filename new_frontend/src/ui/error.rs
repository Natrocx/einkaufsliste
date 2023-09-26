use std::collections::HashSet;

use std::rc::Rc;

use async_std::sync::Mutex;
use dioxus::prelude::*;
use dioxus_router::prelude::{use_navigator, Navigator, Outlet};

use super::Route;
use crate::service::api::APIError;
struct ErrorHandler {
  errors: UseRef<HashSet<String>>,
}
impl ErrorHandler {
  pub(crate) fn add_error(&mut self, error: String) {
    tracing::error!("{error}");
    // we can happily write() here since an error definitely did occur and we do want to rerender

    let mut errors = self.errors.write();
    errors.insert(error);
  }

  pub(crate) fn remove_error(&mut self, error: &String) {
    self.errors.write().remove(error);
  }
}

#[derive(Clone)]
pub struct ErrorService {
  handler: Rc<Mutex<ErrorHandler>>,
  navigator: Navigator,
}

impl ErrorService {
  pub fn new(state: UseRef<HashSet<String>>, navigator: Navigator) -> Self {
    Self {
      handler: Rc::new(Mutex::new(ErrorHandler { errors: state })),
      navigator,
    }
  }

  pub async fn handle_error(&self, error: String) {
    self.handler.lock().await.add_error(error.clone());

    async_std::task::sleep(std::time::Duration::from_secs(10)).await;
    self.handler.lock().await.remove_error(&error);
  }

  pub async fn handle_api_error(&self, error: crate::service::api::APIError) {
    // handle geneneral case
    self.handle_error(error.to_string()).await;

    // handle specific cases requiring navigation/user interaction
    let navigation_error = match error {
      APIError::Unauthenticated => self.navigator.push(Route::Authentication),
      _ => None,
    };
    if let Some(error) = navigation_error {
      self.handle_error(format!("{error:?}")).await;
    }
  }
}

#[inline_props]
pub fn error_handler(cx: Scope) -> Element {
  let errors = use_ref(cx, HashSet::new);
  let navigator = use_navigator(cx).clone();
  cx.provide_context(ErrorService::new(errors.clone(), navigator));

  cx.render(rsx! {
      Outlet::<Route> {}
      div { "Error provider - test" }
      errors.read().iter().map(|error| {
                          rsx! {
                          p { error.as_str() }
                          }
                      })
  })
}
