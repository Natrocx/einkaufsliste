use dioxus::prelude::*;
use dioxus_router::prelude::*;

use self::auth::AuthService;
use crate::service::api::ApiService;

pub mod auth;
pub mod error;
pub mod home;
mod list;
pub mod scaffold;

pub fn app(cx: Scope) -> Element {
  let api_service = cx.provide_context(ApiService::new("https://localhost:8443".into()).unwrap());

  // The user state is located in the app to facilitate rerendering the entire app when the user relogs. There is nothing one can do without being logged in.
  let user: &UseState<Option<einkaufsliste::model::user::User>> = use_state(cx, || None);
  let _user_context = cx.provide_context(user.clone());
  let _auth_service = cx.provide_context(AuthService::new(api_service, UseState::clone(user)));

  cx.render(rsx! { Router::<Route> {} })
}

#[inline_props]
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
