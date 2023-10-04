use dioxus::prelude::*;
use dioxus_router::prelude::use_navigator;
use einkaufsliste::model::requests::{LoginUserV1, RegisterUserV1};
use einkaufsliste::model::user::User;

use super::error::ErrorService;
use crate::service::api::{APIError, ApiService};

pub fn authentication_form(cx: Scope) -> Element {
  let _api: AuthService = cx.consume_context()?;
  let api = _api.clone(); // Clone for the closure :(
  let _error_handler: ErrorService = cx.consume_context()?;

  let error_handler = _error_handler.clone();
  let navigator = use_navigator(cx);

  let username = use_state(cx, String::new);
  let password = use_state(cx, String::new);

  let onlogin = move |_| {
    let navigator = navigator.clone();
    let api = api.clone();
    let error_handler = error_handler.clone();

    let name = username.get().clone();
    let password = password.get().clone();

    cx.spawn(async move {
      let resp = api.login(LoginUserV1 { name, password }).await;

      match resp {
        Ok(_) => {
          navigator.go_back();
        }

        //Handle any errors from the fetch here
        Err(_err) => {
          error_handler.handle_error(_err.to_string()).await;
        }
      }
    });
  };

  let onregister = move |_| {
    let navigator = navigator.clone();
    let api = _api.clone();
    let error_handler = _error_handler.clone();

    let name = username.get().clone();
    let password = password.get().clone();

    cx.spawn(async move {
      let resp = api.register(RegisterUserV1 { name, password }).await;

      match resp {
        // Parse data from here, such as storing a response token
        Ok(_data) => {
          navigator.go_back();
        }

        //Handle any errors from the fetch here
        Err(_err) => {
          error_handler.handle_error(_err.to_string()).await;
        }
      }
    });
  };

  cx.render(rsx! {
    h1 { "Login" }
    label { "Username" }
    input {
        r#type: "text",
        id: "username",
        name: "username",
        onchange: |evt| username.set(evt.value.clone())
    }
    br {}
    label { "Password" }
    input {
        r#type: "password",
        id: "password",
        name: "password",
        onchange: |evt| password.set(evt.value.clone())
    }
    br {}
    button { onclick: onlogin, "Login" }
    button { onclick: onregister, "Register" }
})
}

#[derive(Clone)]
pub struct AuthService {
  api: ApiService,
  user: UseState<Option<User>>,
}

impl AuthService {
  pub fn new(api: ApiService, user: UseState<Option<User>>) -> Self {
    Self { api, user }
  }

  async fn login(&self, credentials: LoginUserV1) -> Result<(), APIError> {
    match self.api.login(credentials).await {
      Ok(user) => {
        self.user.set(Some(user));
        Ok(())
      }
      Err(e) => {
        self.user.set(None);
        Err(e)
      }
    }
  }

  async fn register(&self, credentials: RegisterUserV1) -> Result<(), APIError> {
    match self.api.register(credentials).await {
      Ok(user) => {
        self.user.set(Some(user));
        Ok(())
      }
      Err(e) => {
        self.user.set(None);
        Err(e)
      }
    }
  }
}
