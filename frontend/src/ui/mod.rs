use std::rc::Rc;


use einkaufsliste::model::requests::RegisterUserV1;

use gloo_timers::future::TimeoutFuture;
use log::info;
use web_sys::{HtmlDivElement, HtmlElement};
use yew::{html, Component, ContextProvider, Html, NodeRef};
use yew_router::{BrowserRouter, Switch};

use self::auth::*;
use self::context::APIContext;
use self::modal::*;
use crate::service::api::APIService;
use crate::ui::home::HomePage;
use crate::ui::list::{ListProperties, ListView};
use crate::ui::util::CircularLoadingIndicator;
use crate::util::routing::Page;

mod auth;
mod consts;
pub mod context;
pub mod home;
mod list;
pub mod modal;
mod util;

pub struct App {
  logged_in: bool,
  error_node_ref: NodeRef,
  current_page: Page,
  api_service: Rc<APIService>,
}

pub enum AppMessage {
  NoOp, // is this necessary?
  Error(String),
  LoginSuccessful(u64), // the login token is saved inside the http client - no need to pass it for now
  LoginFailed(String),
}

impl Component for App {
  type Message = AppMessage;

  type Properties = ();

  fn create(_ctx: &yew::Context<Self>) -> Self {
    Self {
      logged_in: false,
      error_node_ref: Default::default(),
      api_service: Rc::new(APIService::new("https://localhost:8443").unwrap()),
      current_page: Page::Overview,
    }
  }

  fn view(&self, ctx: &yew::Context<Self>) -> yew::Html {
    let context = APIContext {
      service: self.api_service.clone(),
      app_callback: ctx.link().callback(|msg| msg),
    };

    let _error_callback = ctx.link().callback(|message: AppMessage| message);
    html! {
      <div>
        <div class="header">
          <p class="page-title">{self.title()}</p>
        </div>

        <ContextProvider<APIContext> context={context} >
          <BrowserRouter>
            <Switch<Page> render={ Switch::render(switch) } />
          </BrowserRouter>
        </ContextProvider<APIContext>>

        <div class="error-container">
          <p class="error-message inactive" ref={&self.error_node_ref}/>
        </div>
      </div>
    }
  }

  fn update(&mut self, ctx: &yew::Context<Self>, msg: Self::Message) -> bool {
    match msg {
      AppMessage::Error(e) => {
        log::error!("Error: {}", &e);

        let error_node = self.error_node_ref.cast::<HtmlDivElement>().unwrap();
        error_node.set_text_content(Some(&e));
        let class_list = error_node.class_list();
        class_list.remove_1("inactive").unwrap();

        ctx
          .link()
          .send_future(reset_error(self.error_node_ref.clone(), TimeoutFuture::new(5_000)));

        // no rerendering necessary as the error is displayed imperatively
        false
      }
      AppMessage::LoginSuccessful(_id) => {
        self.logged_in = true;
        info!(" successfully logged in");
        true
      }
      AppMessage::LoginFailed(error_message) => {
        //TODO: show message to log in again
        self.update(ctx, AppMessage::Error(error_message));
        info!("failed to log in");
        true
      }
      AppMessage::NoOp => false,
    }
  }
}

impl App {
  fn title(&self) -> String {
    match &self.current_page {
      Page::Overview => "Einkaufsliste - Home".to_string(),
      Page::List { id: _, name } => name.clone(),
      Page::Settings => "Settings".to_string(),
      Page::NotFound => "Resource not found".to_string(),
    }
  }
}

fn switch(route: &Page) -> Html {
  match route {
    Page::Overview => html!(<HomePage />),
    Page::List { name: _, id } => {
      let props = ListProperties { id: *id };
      html! {<ListView ..props />}
    }
    Page::Settings => todo!(),
    Page::NotFound => html! {
      <h1>{"There is no such resource."}</h1>
    },
  }
}

async fn reset_error(error_node_ref: NodeRef, timeout: TimeoutFuture) -> AppMessage {
  timeout.await;

  let error_node = match error_node_ref.cast::<HtmlElement>() {
    Some(node) => node,
    None => return AppMessage::Error("Unexpected Error: Error node not found".to_string()), /* maybe add a variant for internal errors like this? */
  };
  error_node.class_list().add_1("inactive").unwrap();

  AppMessage::NoOp
}
// ============================ api helpers ===================================
// used to convert generic APIService returns to yew component messages
// this is only preferable over closures, since we are dealing with async here
//
// For reasons unbeknownst to me, rust does not support capturing Arcs in async
// closures and they therefore have to be passed to async fns

/// Wrapper function for use with yew
async fn login_callback((name, pw, api_service): (String, String, Rc<APIService>)) -> AuthMessage {
  match api_service
    .login_v1(&einkaufsliste::model::requests::LoginUserV1 { name, password: pw })
    .await
  {
    Ok(id) => AuthMessage::Ok(id),
    Err(error) => AuthMessage::Err(error.to_string()),
  }
}

/// Wrapper function for use with yew
async fn register_callback((name, pw, api_service): (String, String, Rc<APIService>)) -> AuthMessage {
  api_service
    .register_v1(&RegisterUserV1 { name, password: pw })
    .await
    .map_err(|e| e.to_string())
}
