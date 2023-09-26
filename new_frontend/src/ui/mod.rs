use dioxus::prelude::*;
use dioxus_router::prelude::*;

use crate::service::api::ApiService;

pub mod auth;
pub mod error;
pub mod home;

pub fn app(cx: Scope) -> Element {
  let _api_service = cx.provide_context(ApiService::new("https://localhost:8443".into()).unwrap());

  cx.render(rsx! { Router::<Route> {} })
}

#[inline_props]
fn not_found(cx: Scope, _route: Vec<String>) -> Element {
  let navigator = use_navigator(cx).clone();
  // a use_future that waits for 1 second to pass
  let _timeout = use_future(cx, (), |_| async move {
    async_std::task::sleep(std::time::Duration::from_secs(3)).await;
    navigator.push(Route::Home);
  });
  let route = _route.join("/");

  cx.render(rsx! {
    div { "The requested page at {route} could not be found. You are being redirected." }
})
}

#[derive(Routable, Clone)]
#[rustfmt::skip]
pub enum Route {
  #[layout(error::error_handler)]
  #[route("/", home::homepage)]
  Home,
  #[route("/auth", auth::authentication_form)]
  Authentication,
  #[route("/:.._route", not_found)]
  PageNotFound { _route: Vec<String>},
}
