use std::collections::HashSet;

use async_std::stream::StreamExt;
use dioxus::prelude::*;
use dioxus_router::prelude::{use_navigator, Navigator, Outlet};

use crate::{service::api::APIError, ui::Route};

#[inline_props]
pub fn error_handler(cx: Scope) -> Element {
  let errors = use_ref(cx, HashSet::new);
  let navigator = use_navigator(cx).clone();

  use_coroutine(cx, |mut rx: UnboundedReceiver<APIError>| {
    to_owned![errors, navigator];
    async move {
      while let Some(error) = rx.next().await {
        // first we check the error type
        // in some cases, like failing a request due to being unauthenticated, we want to navigate to a different page
        let navigation_error = match error {
          APIError::Unauthenticated => navigator.push(Route::Authentication),
          _ => None,
        }
        .map(|e| format!("{e:?}"));

        // then the reported error as well as any possible navigation errors are inserted
        errors.with_mut(|errors| {
          errors.insert(error.to_string());
          if let Some(ref error) = navigation_error {
            errors.insert(error.clone());
          }
        });

        // give the user some time to read
        async_std::task::sleep(std::time::Duration::from_secs(10)).await;

        // remove the error again to avoid polluting the UI
        errors.with_mut(|errors| {
          errors.remove(&error.to_string());
          if let Some(error) = navigation_error {
            errors.remove(&error);
          }
        });
      }
    }
  });

  cx.render(rsx! {
      Outlet::<Route> {}
      errors.read().iter().map(|error| {
                          rsx! {
                          p { error.as_str() }
                          }
                      })
  })
}
