use dioxus::prelude::*;
use dioxus_router::prelude::*;
use einkaufsliste::model::user::User;

use crate::service::api::use_provide_api_service;


pub mod auth;
pub mod error;
pub mod home;
mod list;
pub mod scaffold;

pub fn app(cx: Scope) -> Element {
  // we provide our api service through the context api
  use_provide_api_service(&cx, "https://localhost:8443".to_string());

  // The user state is located in the app to facilitate rerendering the entire app when the user relogs. There is nothing one can do without being logged in.
  use_shared_state_provider::<Option<User>>(cx, || None);

  cx.render(rsx! { Router::<Route> {} })
}

#[component(no_case_check)]
fn not_found(cx: Scope, _route: Vec<String>) -> Element {
  let navigator = use_navigator(cx).clone();
  let _timeout = use_future(cx, (), |_| async move {
    //async_std::task::sleep(std::time::Duration::from_secs(3)).await;
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
  #[route("/list/:id", list::list_view)]
  List { id: u64 },
}
