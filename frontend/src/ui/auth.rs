use std::sync::Arc;

use log::debug;
use web_sys::HtmlInputElement;
use yew::{html, use_node_ref, Callback, Component, NodeRef, Properties};

use super::{login_callback, register_callback, App};
use crate::service::api::APIService;

pub(super) struct LoginView;

#[derive(Properties)]
pub(super) struct AuthProperties {
  pub api_service: Arc<APIService>,
}

impl PartialEq for AuthProperties {
  fn eq(&self, other: &Self) -> bool {
    self.api_service.base_url == other.api_service.base_url
  }
}

impl Component for LoginView {
  type Message = ();

  type Properties = AuthProperties;

  fn create(_ctx: &yew::Context<Self>) -> Self {
    Self
  }

  fn view(&self, ctx: &yew::Context<Self>) -> yew::Html {
    let name_ref = NodeRef::default();
    let _name_ref = name_ref.clone();
    let pw_ref = NodeRef::default();
    let _pw_ref = pw_ref.clone();
    let api_service = ctx.props().api_service.clone(); // double clone for the win... (the compiler will complain for whatever reason if we dont clone the Arc here)

    let callback = ctx
      .link()
      .get_parent()
      .cloned()
      .unwrap()
      .downcast::<App>() // TODO: is there a better way to do this?
      .callback_future(move |_| {
        let name = _name_ref.cast::<HtmlInputElement>().unwrap().value();
        let pw = _pw_ref.cast::<HtmlInputElement>().unwrap().value();

        login_callback((name, pw, api_service.clone()))
      });

    html! {
      <div class="login">
        <label for="login_user_name">{"Username:"}</label>
        <input type="text" id="login_user_name" placeholder="Username" ref={name_ref} />

        <label for="login_password">{"Password:"}</label>
        <input type="password" id="login_password" placeholder="password" ref={pw_ref} />

        <button onclick={callback}>{"Login"}</button>
      </div>
    }
  }

  fn update(&mut self, ctx: &yew::Context<Self>, msg: Self::Message) -> bool {
    false
  }
}
