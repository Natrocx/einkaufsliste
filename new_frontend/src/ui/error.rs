use std::collections::HashSet;
use std::rc::Rc;


use async_std::sync::Mutex;
use dioxus::prelude::*;
pub struct ErrorHandler {
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

  pub fn handle_result<T, E: std::fmt::Display>(&mut self, result: Result<T, E>) -> Option<T> {
    match result {
      Ok(value) => Some(value),
      Err(error) => {
        self.add_error(error.to_string());
        None
      }
    }
  }
}

#[derive(Clone)]
pub struct ErrorService {
  handler: Rc<Mutex<ErrorHandler>>,
}

impl ErrorService {
  pub fn new(state: UseRef<HashSet<String>>) -> Self {
    Self {
      handler: Rc::new(Mutex::new(ErrorHandler { errors: state })),
    }
  }

  pub async fn handle_error(&self, error: String) {
    self.handler.lock().await.add_error(error.clone());

    async_std::task::sleep(std::time::Duration::from_secs(10)).await;
    self.handler.lock().await.remove_error(&error);
  }
}

#[inline_props]
pub fn error_handler<'a>(cx: Scope<'a>, children: Element<'a>) -> Element<'a> {
  let errors = use_ref(cx, HashSet::new);
  cx.provide_context(ErrorService::new(errors.clone()));

  cx.render(rsx! {
      children,
      div { "Error provider - test" }
      errors.read().iter().map(|error| {
                        rsx! {
                          p { error.as_str() }
                        }
                      })
  })
}
