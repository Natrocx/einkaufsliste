use std::rc::Rc;
use std::sync::Arc;

use einkaufsliste::model::item::{Item, Unit};
use einkaufsliste::model::list::List;
use einkaufsliste::model::requests::RegisterUserV1;
use einkaufsliste::model::Identifiable;
use gloo_timers::future::TimeoutFuture;
use log::info;
use web_sys::{HtmlDivElement, HtmlElement, Node};
use yew::{html, Callback, Component, Html, NodeRef, Properties};
use yew_router::prelude::*;

use self::list::{InnerListMessage, ListMessage};
use crate::service::api::{self, APIService};
use crate::ui::auth::LoginView;
use crate::ui::list::{ListProperties, ListView};
use crate::ui::util::CircularLoadingIndicator;
use crate::TransmissionError;

mod auth;
mod consts;
mod list;
mod util;

#[derive(Clone, Routable, PartialEq)]
pub enum Page {
  #[at("/")]
  Overview,
  #[at("/list/:id/:name")]
  List {
    id: <List as Identifiable>::Id,
    name: String,
  },
  #[at("/settings")]
  Settings,
  #[not_found]
  #[at("/404")]
  NotFound,
}

pub struct App {
  logged_in: bool,
  error_node_ref: NodeRef,
  current_page: Page,
  api_service: Arc<APIService>,
}

pub enum AppMessage {
  NoOp, // is this necessary?
  Error(String),
  LoginSuccessful, // the login token is saved inside the http client - no need to pass it for now
  LoginFailed(String),
}

impl Component for App {
  type Message = AppMessage;

  type Properties = ();

  fn create(ctx: &yew::Context<Self>) -> Self {
    Self {
      logged_in: false,
      error_node_ref: Default::default(),
      api_service: Arc::new(APIService::new("https://localhost:8443").unwrap()),
      current_page: Page::Overview,
    }
  }

  fn view(&self, ctx: &yew::Context<Self>) -> yew::Html {
    let props = ListProperties {
      api_service: self.api_service.clone(),
      id: 0,
    };
    let api_service = self.api_service.clone();

    html! {
      <div>
        <div class="header">
          <p class="page-title">{self.title()}</p>
        </div>

        <BrowserRouter>
          <Switch<Page> render={ Switch::render(move |route| { switch(route, api_service.clone())}) } />
        </BrowserRouter>

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
          .send_future(reset_error(self.error_node_ref.clone(), TimeoutFuture::new(20_000)));

        // no rerendering necessary as the error is displayed imperatively
        false
      }
      AppMessage::LoginSuccessful => {
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
      Page::Overview => "Einkaufsliste".to_string(),
      Page::List { id: _, name } => name.clone(),
      Page::Settings => "Settings".to_string(),
      Page::NotFound => "Resource not found".to_string(),
    }
  }
}

fn switch(route: &Page, api_service: Arc<APIService>) -> Html {
  match route {
    Page::Overview => html!(<HomePage api_service={api_service}/>),
    Page::List { name: _, id } => html! {<ListView id={*id} api_service={api_service} />}, /* TODO: replace clone? */
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

pub struct HomePage {
  lists: Vec<<List as Identifiable>::Id>,
  loaded_lists: bool,
}

#[derive(Properties)]
pub struct HomePageProperties {
  pub api_service: Arc<APIService>,
}
impl PartialEq for HomePageProperties {
  fn eq(&self, other: &Self) -> bool {
    self.api_service.base_url == other.api_service.base_url
  }
}

impl Component for HomePage {
  type Message = ();

  type Properties = HomePageProperties;

  fn create(ctx: &yew::Context<Self>) -> Self {
    Self {
      lists: vec![],
      loaded_lists: false,
    }
  }

  fn view(&self, ctx: &yew::Context<Self>) -> Html {
    if !self.loaded_lists {
      let api_service = ctx.props().api_service.as_ref();
      html! {
        <div class="list-loading">
          <CircularLoadingIndicator />
        </div>
      }
    } else {
      html!()
    }
  }
}

pub struct ListPreView;

#[derive(Clone, PartialEq, Eq, Properties)]
pub struct ListPreviewProperties {
  name: String,
  image: u32, // TODO: place actual image
}

impl Component for ListPreView {
  type Message = ();

  type Properties = ListPreviewProperties;

  fn create(ctx: &yew::Context<Self>) -> Self {
    Self {}
  }

  fn view(&self, ctx: &yew::Context<Self>) -> yew::Html {
    let url = format!("https://localhost:8443/image/{}", ctx.props().image);

    html! {
      <div class="list-preview-container">
        <img src={url}  alt={format!("List picture for: {}", ctx.props().name)} />
      </div>
    }
  }
}

// ============================ api helpers ===================================

/// Wrapper function for use with yew
async fn fetch_callback((id, api_service): (<Item as Identifiable>::Id, Arc<APIService>)) -> ListMessage {
  match api_service.get_flat_items_list(id).await {
    Ok(val) => ListMessage::FetchSuccessful(val),
    Err(e) => ListMessage::Error(e.to_string()),
  }
}

/// Wrapper function for use with yew
async fn change_name_callback((item, api_service): (Item, Arc<APIService>)) -> InnerListMessage {
  match api_service.update_item(&item).await {
    Ok(_) => InnerListMessage::NOOP,
    Err(e) => InnerListMessage::Error(e.to_string()),
  }
}

/// Wrapper function for use with yew
async fn login_callback((name, pw, api_service): (String, String, Arc<APIService>)) -> AppMessage {
  match api_service
    .login_v1(&einkaufsliste::model::requests::LoginUserV1 { name, password: pw })
    .await
  {
    Ok(_) => AppMessage::LoginSuccessful,
    Err(error) => AppMessage::LoginFailed(error.to_string()),
  }
}

/// Wrapper function for use with yew. Will panic if the request fails
async fn register_callback((name, pw, api_service): (String, String, Arc<APIService>)) -> () {
  api_service
    .register_v1(&RegisterUserV1 { name, password: pw })
    .await
    .unwrap();
}
