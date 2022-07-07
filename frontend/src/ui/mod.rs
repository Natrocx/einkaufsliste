use std::rc::Rc;
use std::sync::Arc;

use einkaufsliste::model::item::{Item, Unit};
use einkaufsliste::model::requests::RegisterUserV1;
use einkaufsliste::model::Identifiable;
use gloo_timers::future::TimeoutFuture;
use log::info;
use web_sys::{HtmlDivElement, HtmlElement, Node};
use yew::{html, Component, NodeRef};

use self::list::{InnerListMessage, ListMessage};
use crate::service::api::APIService;
use crate::ui::auth::LoginView;
use crate::ui::list::{ListProperties, ListView};
use crate::TransmissionError;

mod auth;
mod consts;
pub mod list;
mod util;

#[derive(Default)]
pub struct App {
  logged_in: bool,
  error_node_ref: NodeRef,
}

pub enum AppMessage {
  NoOp, // is this necessary?
  Error(String),
  LoginSuccessful, // the login token is saved inside the http client
  LoginFailed(String),
}

impl Component for App {
  type Message = AppMessage;

  type Properties = ();

  fn create(ctx: &yew::Context<Self>) -> Self {
    Self {
      logged_in: false,
      ..Default::default()
    }
  }

  fn view(&self, ctx: &yew::Context<Self>) -> yew::Html {
    let api_service = Arc::new(APIService::new("https://localhost:8443").unwrap());

    let props = ListProperties {
      api_service: api_service.clone(),
      id: 0,
    };

    html! {
      <div>
        {
          if self.logged_in {
            html! {
              <div>
                <ListView ..props />
              </div>
            }
          } else {
            html! {
              <LoginView api_service={api_service}/>
            }
          }
        }

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

async fn reset_error(error_node_ref: NodeRef, timeout: TimeoutFuture) -> AppMessage {
  timeout.await;

  let error_node = error_node_ref.cast::<HtmlElement>().unwrap();
  error_node.class_list().add_1("inactive").unwrap();

  AppMessage::NoOp
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
