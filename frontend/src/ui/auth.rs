use dioxus::prelude::*;
use dioxus_router::prelude::use_navigator;
use einkaufsliste::model::requests::{LoginUserV1, RegisterUserV1};
use einkaufsliste::model::user::User;

use crate::service::api::{APIError, ApiService};
use crate::ui::consts::*;

#[derive(Debug, Clone, Copy)]
enum AuthType {
  Login,
  Register,
  None,
}

pub fn authentication_form(cx: Scope) -> Element {
  let error_handler: &Coroutine<APIError> = use_coroutine_handle(cx)?;
  let navigator = use_navigator(cx);
  let use_user = use_shared_state::<Option<User>>(cx)?;

  let auth_type = use_state(cx, || AuthType::None);

  let do_auth = move |username: String, password: String| {
    to_owned![error_handler, navigator, use_user, auth_type];
    let api: ApiService = cx.consume_context().unwrap();

    async move {
      let resp = match auth_type.get() {
        AuthType::Login => {
          api
            .login(LoginUserV1 {
              name: username,
              password,
            })
            .await
        }
        AuthType::Register => {
          api
            .register(RegisterUserV1 {
              name: username,
              password,
            })
            .await
        }
        AuthType::None => panic!("Unexpected unrecoverable synchronization error"),
      };

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
    }
  };

  cx.render(rsx! {
    form {
        class: "flex flex-col items-center object-center space-y-4 p-2 ",
        onsubmit: move |evt| {
            tracing::debug!("Encountered event: {:?}", evt);
            evt.stop_propagation();
            let username = evt.values["username"].first().unwrap().clone();
            let password = evt.values["password"].first().unwrap().clone();
            do_auth(username, password)
        },
        div { class: "flex flex-col text-left w-full",
            label { class: "pl-2", r#for: "username", "Username" }
            input {
                class: "w-full p-1 {INPUT_PRIMARY}",
                r#type: "text",
                id: "username",
                name: "username",
                autofocus: true
            }
        }
        div { class: "flex flex-col text-left w-full",
            label { r#for: "password", class: "pl-2", "Password" }
            input {
                class: "w-full p-1 {INPUT_PRIMARY}",
                r#type: "password",
                id: "password",
                name: "password"
            }
        }
        // if we allow bubbling of the events here, the requested action will be performed through the forms onsubmit

        div { class: "flex flex-row space-x-4",
            button {
                class: "{BORDER_SMALL} {SECONDARY_BG} dark:hover:bg-zinc-700 hover:bg-gray-200",
                r#type: "submit",
                onclick: |_| {
                    auth_type.set(AuthType::Login);
                },
                "Login"
            }
            button {
                class: "{BORDER_SMALL} {SECONDARY_BG} dark:hover:bg-zinc-700 hover:bg-gray-200",
                r#type: "submit",
                onclick: |_| {
                    auth_type.set(AuthType::Register);
                },
                "Register"
            }
        }
    }
})
}
