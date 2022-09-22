use std::sync::Arc;

use einkaufsliste::model::item::Item;
use einkaufsliste::model::list::List;
use einkaufsliste::model::requests::RegisterUserV1;
use einkaufsliste::model::user::{ObjectList, UsersObjectLists};
use einkaufsliste::model::Identifiable;
use einkaufsliste_codegen::Routable;
use gloo_timers::future::TimeoutFuture;
use log::info;
use web_sys::{HtmlDivElement, HtmlElement};
use yew::{html, Callback, Component, Html, NodeRef, Properties};
use yew_router::{BrowserRouter, Switch};

use self::auth::*;
use self::list::{InnerListMessage, ListMessage};
use crate::{service::api::{self, APIService}, util::routing::Page};
use crate::ui::list::{ListProperties, ListView};
use crate::ui::util::CircularLoadingIndicator;

mod auth;
mod consts;
mod list;
mod util;

pub struct App {
  logged_in: bool,
  error_node_ref: NodeRef,
  current_page: Page,
  api_service: Arc<APIService>,
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
      api_service: Arc::new(APIService::new("https://localhost:8443").unwrap()),
      current_page: Page::Overview,
    }
  }

  fn view(&self, ctx: &yew::Context<Self>) -> yew::Html {
    let api_service = self.api_service.clone();

    let error_callback = ctx.link().callback(|message: AppMessage| message);
    html! {
      <div>
        <div class="header">
          <p class="page-title">{self.title()}</p>
        </div>

        <BrowserRouter>
          <Switch<Page> render={ Switch::render(move |route| { switch(route, api_service.clone(), error_callback.clone())}) } />
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
      AppMessage::LoginSuccessful(id) => {
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

fn switch(route: &Page, api_service: Arc<APIService>, app_callback: Callback<AppMessage>) -> Html {
  match route {
    Page::Overview => html!(<HomePage api_service={api_service} app_callback={app_callback}/>),
    Page::List { name: _, id } => {
      let props = ListProperties {
        api_service,
        id: *id,
        error_callback: app_callback,
      };
      html! {<ListView ..props />}
    } /* TODO: replace clone? */
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
  logged_in: bool,
}

#[derive(Properties)]
pub struct HomePageProperties {
  pub api_service: Arc<APIService>,
  pub app_callback: Callback<AppMessage>,
}
impl PartialEq for HomePageProperties {
  fn eq(&self, other: &Self) -> bool {
    self.api_service.base_url == other.api_service.base_url && self.app_callback == other.app_callback
  }
}

pub enum HomePageMessage {
  ListFetchSuccess(ObjectList),
  LoginSuccessful(u64),
  Error(String),
}

impl Component for HomePage {
  type Message = HomePageMessage;

  type Properties = HomePageProperties;

  fn create(_ctx: &yew::Context<Self>) -> Self {
    Self {
      lists: vec![],
      loaded_lists: false,
      logged_in: false,
    }
  }

  #[allow(clippy::let_unit_value)]
  fn view(&self, ctx: &yew::Context<Self>) -> Html {
    if !self.logged_in {
      let callback = ctx.link().callback(|msg| match msg {
        Ok(user_id) => HomePageMessage::LoginSuccessful(user_id),
        Err(reason) => HomePageMessage::Error(reason),
      });
      let props = AuthProperties {
        api_service: ctx.props().api_service.clone(),
        callback,
      };
      html! {
        <LoginView ..props />
      }
    } else if !self.loaded_lists {
      let api_service = ctx.props().api_service.clone();
      ctx.link().send_future(get_users_lists_callback(api_service));

      html! {
        <div class="list-loading">
          <CircularLoadingIndicator />
        </div>
      }
    } else {
      html! {
        <p>{"Vier"}</p>
      }
    }
  }

  fn update(&mut self, ctx: &yew::Context<Self>, msg: Self::Message) -> bool {
    match msg {
      HomePageMessage::ListFetchSuccess(_lists) => true,
      HomePageMessage::LoginSuccessful(user_id) => {
        self.logged_in = true;
        ctx.props().app_callback.emit(AppMessage::LoginSuccessful(user_id));
        true
      }
      HomePageMessage::Error(reason) => {
        // Pass up the token tree to use centralised error handling
        ctx.props().app_callback.emit(AppMessage::Error(reason));
        false
      }
    }
  }

  fn changed(&mut self, ctx: &yew::Context<Self>) -> bool {
    true
  }

  fn rendered(&mut self, ctx: &yew::Context<Self>, first_render: bool) {}

  fn destroy(&mut self, ctx: &yew::Context<Self>) {}
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

  fn create(_ctx: &yew::Context<Self>) -> Self {
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
// used to convert generic APIService returns to yew component messages
// this is only preferable over closures, since we are dealing with async here

/// Wrapper function for use with yew
async fn fetch_callback((id, api_service): (<Item as Identifiable>::Id, Arc<APIService>)) -> ListMessage {
  match api_service.get_flat_items_list(id).await {
    Ok(val) => ListMessage::FetchSuccessful(val),
    Err(e) => ListMessage::FetchUnsuccessful(e.to_string()),
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
async fn login_callback((name, pw, api_service): (String, String, Arc<APIService>)) -> AuthMessage {
  match api_service
    .login_v1(&einkaufsliste::model::requests::LoginUserV1 { name, password: pw })
    .await
  {
    Ok(id) => AuthMessage::Ok(id),
    Err(error) => AuthMessage::Err(error.to_string()),
  }
}

/// Wrapper function for use with yew
async fn register_callback((name, pw, api_service): (String, String, Arc<APIService>)) -> AuthMessage {
  api_service
    .register_v1(&RegisterUserV1 { name, password: pw })
    .await
    .map_err(|e| e.to_string())
}

async fn get_users_lists_callback(api_service: Arc<APIService>) -> HomePageMessage {
  match api_service.get_users_lists().await {
    Ok(val) => HomePageMessage::ListFetchSuccess(val),
    Err(reason) => HomePageMessage::Error(reason.to_string()),
  }
}
