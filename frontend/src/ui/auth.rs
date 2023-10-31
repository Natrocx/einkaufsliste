use dioxus::prelude::*;
use dioxus_router::prelude::use_navigator;
use einkaufsliste::model::requests::{LoginUserV1, RegisterUserV1};
use einkaufsliste::model::user::User;

use crate::service::api::{APIError, ApiService};

pub fn authentication_form(cx: Scope) -> Element {
  let error_handler: &Coroutine<APIError> = use_coroutine_handle(cx)?;
  let navigator = use_navigator(cx);
  let use_user = use_shared_state::<Option<User>>(cx)?;

  let username = use_state(cx, String::new);
  let password = use_state(cx, String::new);

  let onlogin = move |_| {
    to_owned![error_handler, navigator, use_user];
    let api: ApiService = cx.consume_context().unwrap();

    let name = username.get().clone();
    let password = password.get().clone();

    cx.spawn(async move {
      let resp = api.login(LoginUserV1 { name, password }).await;

      match resp {
        Ok(user) => {
          use_user.with_mut(|use_user| *use_user = Some(user));
          navigator.go_back();
        }

        //Handle any errors from the fetch here
        Err(err) => {
          error_handler.send(err);
        }
      }
    });
  };

  let onregister = move |_| {
    to_owned![error_handler, navigator, use_user];
    let api: ApiService = cx.consume_context().unwrap();

    let name = username.get().clone();
    let password = password.get().clone();

    cx.spawn(async move {
      let resp = api.register(RegisterUserV1 { name, password }).await;

      match resp {
        // Parse data from here, such as storing a response token
        Ok(user) => {
          use_user.with_mut(|use_user| *use_user = Some(user));
          navigator.go_back();
        }

        //Handle any errors from the fetch here
        Err(err) => {
          error_handler.send(err);
        }
      }
    });
  };

  let api_service = cx.consume_context::<ApiService>()?;
  let error_handler = cx.consume_context::<Coroutine<APIError>>()?;
  let fetch_lists = move |_| {
    to_owned![api_service, error_handler];
    cx.spawn(async move {
      let resp = api_service.fetch_all_lists().await;

      match resp {
        // Parse data from here, such as storing a response token
        Ok(_data) => {
          println!("Fetched lists");
        }

        //Handle any errors from the fetch here
        Err(err) => {
          error_handler.send(err);
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
    button {onclick: fetch_lists, "Fetch Lists"}
})
}
