use web_sys::{HtmlInputElement, KeyboardEvent, MouseEvent};
use yew::{html, Callback, Component, Html, NodeRef, Properties};

use super::context::APIContext;
use super::modal::{TextInputModalButton, TextInputModalField, TextInputModalProps};
use crate::ok_or_log;
use crate::ui::TextInputModal;
#[derive(Default)]
pub struct AuthModal {}

impl Component for AuthModal {
  type Message = ();

  type Properties = AuthProperties;

  fn create(_ctx: &yew::Context<Self>) -> Self {
    Self::default()
  }

  fn view(&self, ctx: &yew::Context<Self>) -> Html {
    let node_refs = [NodeRef::default(), NodeRef::default()];
    let fields = vec![
      TextInputModalField {
        name: "Username",
        node_ref: node_refs[0].clone(),
        placeholder: Some("Enter your username here"),
        required: true,
      },
      TextInputModalField {
        name: "Password",
        node_ref: node_refs[1].clone(),
        placeholder: Some("Enter your password here"),
        required: true,
      },
    ]
    .into();

    let mut actions = vec![
      TextInputModalButton {
        prompt: "Login",
        callback: ctx.link().callback(|_| {}),
      },
      TextInputModalButton {
        prompt: "Register",
        callback: ctx.link().callback(|_| {}),
      },
    ];

    if let Some(callback) = ctx.props().cancel_callback.clone() {
      actions.push(TextInputModalButton {
        prompt: "Cancel",
        callback: ctx.link().callback(move |_| {
          callback.emit(());
        }),
      });
    };

    let modal_props = TextInputModalProps {
      prompt: "Login or register",
      fields,
      actions,
    };

    html! {
      <TextInputModal ..modal_props />
    }
  }
}

#[derive(Default)]
pub(super) struct AuthView {
  name_input: NodeRef,
  password_input: NodeRef,
}

#[derive(PartialEq, Properties)]
pub struct AuthProperties {
  pub submit_callback: Callback<Result<u64, String>>,
  pub cancel_callback: Option<Callback<()>>,
}

pub type AuthMessage = Result<u64, String>;

impl Component for AuthView {
  type Message = AuthMessage;

  type Properties = AuthProperties;

  fn create(_ctx: &yew::Context<Self>) -> Self {
    Self::default()
  }

  fn view(&self, ctx: &yew::Context<Self>) -> yew::Html {
    // we need to clone every Rc outside the closures since they would move them in otherwise
    let name_ref = self.name_input.clone();
    let pw_ref = self.password_input.clone();
    let api: APIContext = ctx.link().context(Callback::noop()).unwrap().0;
    let api_service = api.service.clone();

    let login_callback = ctx.link().callback_future(move |_| {
      // sadly, the input element value extraction has to be done inline in the closures. It would be nice to move them into a function in the future to avoid rewriting the same code
      let name = name_ref.cast::<HtmlInputElement>().unwrap().value();
      let pw = pw_ref.cast::<HtmlInputElement>().unwrap().value();

      crate::ui::login_callback((name, pw, api_service.clone()))
    });

    let name_ref = self.name_input.clone();
    let pw_ref = self.password_input.clone();
    let api_service = api.service;
    let register_callback = ctx.link().callback_future(move |_| {
      let name = name_ref.cast::<HtmlInputElement>().unwrap().value();
      let pw = pw_ref.cast::<HtmlInputElement>().unwrap().value();

      crate::ui::register_callback((name, pw, api_service.clone()))
    });

    let _login_callback = login_callback.clone();
    let keypress_handler = ctx.link().batch_callback(move |event: KeyboardEvent| {
      let key = event.key();
      if key.eq("Enter") {
        _login_callback.emit(MouseEvent::new("").unwrap());
      }
      None
    });

    html! {
      <div class="login">
        <label for="login_user_name" >{"Username:"}</label>
        <input type="text" id="login_user_name" onkeypress={keypress_handler.clone()} placeholder="Username" ref={self.name_input.clone()} />

        <label for="login_password">{"Password:"}</label>
        <input type="password" id="login_password"  onkeypress={keypress_handler} placeholder="Password" ref={self.password_input.clone()} />

        <div class="button-list">
        {
          if let Some(cancel_callback) = ctx.props().cancel_callback.clone() {
            html! {
              <button onclick={move |_| cancel_callback.emit(())}>{"Cancel"}</button>
            }
          }
          else {html! {}}
        }
          <button onclick={login_callback}>{"Login"}</button>
          <button onclick={register_callback}>{"Register"}</button>
        </div>
      </div>
    }
  }

  fn update(&mut self, _ctx: &yew::Context<Self>, msg: Self::Message) -> bool {
    match msg {
      Ok(user_id) => {
        _ctx.props().submit_callback.emit(Ok(user_id));
        false
      }
      Err(reason) => {
        _ctx.props().submit_callback.emit(Err(reason));
        false
      }
    }
  }

  fn rendered(&mut self, _ctx: &yew::Context<Self>, first_render: bool) {
    if first_render {
      ok_or_log!(self.name_input.cast::<HtmlInputElement>().unwrap().focus());
    }
  }
}
