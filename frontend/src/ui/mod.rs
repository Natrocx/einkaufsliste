use dioxus::prelude::*;
use dioxus_router::{Route, Router};

pub fn app(cx: Scope) -> Element {
  cx.render(rsx! (
      style {
          include_str!("../../assets/einkaufsliste.css")
      }
      Router {
          h3 { "Einkaufsliste - placeholder"}
          Route { to: "/auth", self::authentication_form {} }
      }
      self::authentication_form {}
  ))
}

fn authentication_form(cx: Scope) -> Element {
  let user_name = use_state(&cx, String::new);
  let user_password = use_state(&cx, String::new);

  cx.render(rsx!(
  div {
      class: "login",
      label {
          r#for: "user_name",
          "Username"
      }
      input {
          id: "user_name",
          r#type: "text",
          placeholder: "Enter your username here" ,
          oninput: move |event| {user_name.set(event.value.clone())},
          value: "{user_name}"
      }
      br {}
      label {
          r#for: "user_password",
          "Password"
      }
      input {
          id: "user_password",
          r#type: "password",
          placeholder: "Enter your password here",
          oninput: move |event| {user_password.set(event.value.clone())},
          value: "{user_password}"
      }
      button {
          "Login"
      }
      button {
          "Register"
      }
  }))
}
