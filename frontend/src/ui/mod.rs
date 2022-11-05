use std::rc::Rc;

use einkaufsliste::model::list::{FlatItemsList, List};
use einkaufsliste::model::requests::RegisterUserV1;
use einkaufsliste::model::user::ObjectList;
use gloo_timers::future::TimeoutFuture;
use log::info;
use web_sys::{HtmlDivElement, HtmlElement, HtmlInputElement};
use yew::{html, html_nested, Callback, Component, ContextProvider, Html, NodeRef, Properties};
use yew_router::{BrowserRouter, Switch};

use self::auth::*;
use self::context::APIContext;
use self::modal::*;
use crate::service::api::APIService;
use crate::ui::list::{ListProperties, ListView};
use crate::ui::util::CircularLoadingIndicator;
use crate::util::routing::Page;

mod auth;
mod consts;
pub mod context;
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
    };

    let error_callback = ctx.link().callback(|message: AppMessage| message);
    html! {
      <div>
        <div class="header">
          <p class="page-title">{self.title()}</p>
        </div>

        <ContextProvider<APIContext> context={context} >
          <BrowserRouter>
            <Switch<Page> render={ Switch::render(move |route| { switch(route, error_callback.clone())}) } />
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

fn switch(route: &Page, app_callback: Callback<AppMessage>) -> Html {
  match route {
    Page::Overview => html!(<HomePage app_callback={app_callback}/>),
    Page::List { name: _, id } => {
      let props = ListProperties {
        id: *id,
        error_callback: app_callback,
      };
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

pub struct HomePage {
  lists: Option<Vec<Option<Rc<FlatItemsList>>>>,
  logged_in: bool,
  show_add_list_modal: bool,
}

#[derive(Debug, PartialEq, Properties)]
pub struct HomePageProperties {
  pub app_callback: Callback<AppMessage>,
}

pub enum HomePageMessage {
  ListFetched(FlatItemsList),
  ObjectListFetched(ObjectList),
  LoginSuccessful(u64),
  NewList(String),
  ShowAddListModal,
  CloseAddListModal,
  Error(String),
  None,
}

impl Component for HomePage {
  type Message = HomePageMessage;

  type Properties = HomePageProperties;

  fn create(_ctx: &yew::Context<Self>) -> Self {
    Self {
      lists: None,
      logged_in: false,
      show_add_list_modal: false,
    }
  }

  #[allow(clippy::let_unit_value)]
  fn view(&self, ctx: &yew::Context<Self>) -> Html {
    if !self.logged_in {
      let auth_callback = ctx.link().callback(|msg| match msg {
        Ok(user_id) => HomePageMessage::LoginSuccessful(user_id),
        Err(reason) => HomePageMessage::Error(reason),
      });
      let props = AuthProperties {
        submit_callback: auth_callback,
        cancel_callback: None,
      };

      html! {
        <div>
          <AuthView ..props />
        </div>
      }
    } else if self.lists.is_none() {
      html! {
        <div class="list-loading">
          <CircularLoadingIndicator />
        </div>
      }
    } else {
      let lists = self.lists.as_ref().unwrap();

      let on_add_list_button_pressed = ctx.link().callback(|_| HomePageMessage::ShowAddListModal);

      let modal_refs = vec![NodeRef::default(); 2];

      let modal_props = TextInputModalProps {
        prompt: "Create a new list",
        fields: vec![
          TextInputModalField {
            name: "Name",
            node_ref: modal_refs[0].clone(),
            placeholder: Some("required"),
            required: true,
          },
          TextInputModalField {
            name: "Shop",
            node_ref: modal_refs[1].clone(),
            placeholder: Some("optional"),
            required: false,
          },
        ]
        .into(),
        actions: vec![
          TextInputModalButton {
            prompt: "Cancel",
            callback: ctx.link().callback(|_| HomePageMessage::CloseAddListModal),
          },
          TextInputModalButton {
            prompt: "Submit",
            callback: ctx.link().callback(move |_| {
              let name = modal_refs[0].cast::<HtmlInputElement>().unwrap().value();
              if name.is_empty() {
                return HomePageMessage::Error("You must specify a name for your new list.".into());
              }

              let shop = modal_refs[1].cast::<HtmlInputElement>().unwrap().value();
              log::debug!("{shop}");

              HomePageMessage::NewList(name)
            }),
          },
        ],
      };

      html! {
        <div>
          {
          // render list previews
          if !lists.is_empty() {
            html_nested! {
              <div>
                {
                  lists.iter().map(|list| {
                    html_nested! {<ListPreView list={list} />}
                  })
                  .collect::<Html>()
                }
              </div>
            }
          } // or render placeholder
          else {
            html_nested! {
              <p>{"You do not currently have any lists."}</p>
            }
          }
          }

         {if self.show_add_list_modal {
            html! { <TextInputModal ..modal_props /> }
          }
          else { // make the typechecker happy
              html! {}
            }
          }
          <div class="add-list floating-button">
            <span onclick={on_add_list_button_pressed}class="material-symbols-outlined button"> {"add"} </span>
          </div>
        </div>
      }
    }
  }

  fn update(&mut self, ctx: &yew::Context<Self>, msg: Self::Message) -> bool {
    let api: (APIContext, _) = ctx.link().context(Callback::noop()).unwrap();
    let api = api.0.service;

    match msg {
      HomePageMessage::ObjectListFetched(object_list) => {
        self.lists = Some(Vec::with_capacity(object_list.list.len()));

        for id in object_list.list {
          let api = api.clone();
          ctx.link().send_future(async move {
            match api.get_flat_items_list(id).await {
              Ok(list) => HomePageMessage::ListFetched(list),
              Err(e) => HomePageMessage::Error(e.to_string()),
            }
          });
        }

        true
      }
      HomePageMessage::LoginSuccessful(user_id) => {
        self.logged_in = true;

        // if the user is logged in, fetch their lists asynchronously
        ctx.link().send_future(async move {
          match api.get_users_lists().await {
            Ok(ol) => HomePageMessage::ObjectListFetched(ol),
            Err(e) => HomePageMessage::Error(e.to_string()),
          }
        });
        ctx.props().app_callback.emit(AppMessage::LoginSuccessful(user_id));
        true
      }
      HomePageMessage::Error(reason) => {
        // Pass up the token tree to use centralised error handling
        ctx.props().app_callback.emit(AppMessage::Error(reason));
        false
      }
      //TODO: Evaluate reducing page refreshes/performance impact
      HomePageMessage::ListFetched(list) => {
        self.lists.as_mut().unwrap().push(Some(Rc::new(list)));

        true
      }
      HomePageMessage::ShowAddListModal => {
        // TODO: evaluate performance impact
        self.show_add_list_modal = true;

        true
      }
      HomePageMessage::CloseAddListModal => {
        self.show_add_list_modal = false;

        true
      }
      HomePageMessage::None => false,
      HomePageMessage::NewList(name) => {
        ctx.link().send_message(HomePageMessage::CloseAddListModal);

        ctx.link().send_future(async move {
          let mut list = List {
            id: 0,
            name,
            shop: None,
            image_id: None,
            items: vec![],
          };
          match api.push_new_item_list(&list).await {
            Ok(id) => {
              list.id = id;
              HomePageMessage::ListFetched(FlatItemsList::from_list_and_items(list, vec![]))
            }
            Err(e) => HomePageMessage::Error(e.to_string()),
          }
        });
        true
      }
    }
  }
}

pub struct ListPreView;

#[derive(Clone, PartialEq, Eq, Properties)]
pub struct ListPreviewProperties {
  list: Option<Rc<FlatItemsList>>,
}

impl Component for ListPreView {
  type Message = ();

  type Properties = ListPreviewProperties;

  fn create(_ctx: &yew::Context<Self>) -> Self {
    Self {}
  }

  #[allow(clippy::let_unit_value)]
  fn view(&self, ctx: &yew::Context<Self>) -> yew::Html {
    html! {
      <div class="list-preview-container">
        {
          if let Some(list) = ctx.props().list.clone() {
            if let Some(image_id) = list.image_id {
            let url = format!("https://localhost:8443/image/{}", image_id);
              html! { <img src={url}  alt={format!("List picture for: {}", list.name)} /> }
            }
            // if there is no
            else {
              html! {}
            }
        }
          else {
            html! {
              <CircularLoadingIndicator />
            }
          }
        }
      </div>
    }
  }
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
