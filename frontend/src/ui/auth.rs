use std::sync::Arc;

use einkaufsliste::model::requests::{LoginUserV1, RegisterUserV1};
use web_sys::HtmlInputElement;
use yew::{html, Callback, Component, Html, NodeRef, Properties};

use crate::service::api::{self, APIService};

pub(super) struct AuthView;

#[derive(Properties)]
pub(super) struct AuthProperties {
  pub api_service: Arc<APIService>,
  pub callback: Callback<Result<u64, String>>,
}

impl PartialEq for AuthProperties {
  fn eq(&self, other: &Self) -> bool {
    self.api_service.base_url == other.api_service.base_url
  }
}

pub type AuthMessage = Result<u64, String>;

impl Component for AuthView {
  type Message = AuthMessage;

  type Properties = AuthProperties;

  fn create(_ctx: &yew::Context<Self>) -> Self {
    Self
  }

  fn view(&self, ctx: &yew::Context<Self>) -> yew::Html {
    let name_ref = NodeRef::default();
    let pw_ref = NodeRef::default();
    let _name_ref = name_ref.clone();
    let _pw_ref = pw_ref.clone();
    let api_service = ctx.props().api_service.clone();

    let login_callback = ctx.link().callback_future(move |_| {
      let name = _name_ref.cast::<HtmlInputElement>().unwrap().value();
      let pw = _pw_ref.cast::<HtmlInputElement>().unwrap().value();

      crate::ui::login_callback((name, pw, api_service.clone()))
    });

    let _name_ref = name_ref.clone();
    let _pw_ref = pw_ref.clone();
    let api_service = ctx.props().api_service.clone(); // double clone for the win... (the compiler will complain for whatever reason if we dont clone the Arc here)

    let register_callback = ctx.link().callback_future(move |_| {
      let name = _name_ref.cast::<HtmlInputElement>().unwrap().value();
      let pw = _pw_ref.cast::<HtmlInputElement>().unwrap().value();

      crate::ui::register_callback((name, pw, api_service.clone()))
    });

    html! {
      <div class="login">
        <label for="login_user_name">{"Username:"}</label>
        <input type="text" id="login_user_name" placeholder="Username" ref={name_ref} />

        <label for="login_password">{"Password:"}</label>
        <input type="password" id="login_password" placeholder="Password" ref={pw_ref} />

        <div>
          <button onclick={login_callback}>{"Login"}</button>
          <button onclick={register_callback}>{"Register"}</button>
        </div>
      </div>
    }
  }

  fn update(&mut self, _ctx: &yew::Context<Self>, msg: Self::Message) -> bool {
    match msg {
      Ok(user_id) => {
        _ctx.props().callback.emit(Ok(user_id));
        false
      }
      Err(reason) => {
        _ctx.props().callback.emit(Err(reason));
        false
      }
    }
  }
}
