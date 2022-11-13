use std::rc::Rc;

use einkaufsliste::model::list::{FlatItemsList, List};
use einkaufsliste::model::user::ObjectList;
use web_sys::{HtmlInputElement, Node};
use yew::{html, html_nested, Callback, Component, Html, NodeRef, Properties};

use super::auth::AuthProperties;
use super::context::APIContext;
use super::modal::{TextInputModalButton, TextInputModalField, TextInputModalProps};
use super::AppMessage;
use crate::ui::{AuthModal, CircularLoadingIndicator, TextInputModal};

pub struct HomePage {
  lists: Option<Vec<Rc<FlatItemsList>>>,
  unfetched_lists: usize,
  logged_in: bool,
  show_add_list_modal: bool,
  modal_props: TextInputModalProps,
}

#[derive(Debug, PartialEq, Properties)]
pub struct HomePageProperties {}

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

  fn create(ctx: &yew::Context<Self>) -> Self {
    let modal_refs = [NodeRef::default(), NodeRef::default()];

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
            log::debug!("To be implemented: shop: {shop}");

            HomePageMessage::NewList(name)
          }),
        },
      ],
    };

    Self {
      lists: None,
      unfetched_lists: 0,
      logged_in: false,
      show_add_list_modal: false,
      modal_props,
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
          <AuthModal ..props />
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

      html! {
        <div>
          {
            // render list previews
            if !lists.is_empty() && self.unfetched_lists == 0 {
              html_nested! {
                <div>
                  // render all already fetched lists' previews
                  {
                    lists.iter().map(|list| {
                      html_nested! {<ListPreView list={list} />}
                    })
                    .collect::<Html>()
                  }
                  // and fill up with loading indicators for unfetched lists
                  {
                    (0..self.unfetched_lists).map(|_| {
                      html_nested! { <ListPreView list={None} /> }
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

          {
            if self.show_add_list_modal {
              html! { <TextInputModal ..self.modal_props.clone() /> }
            }
            else { // make the typechecker happy
              html! {}
            }
          }
          <button class="add-list floating-button" onclick={on_add_list_button_pressed}>
            <span class="material-symbols-outlined"> {"add"} </span>
          </button>
        </div>
      }
    }
  }

  fn update(&mut self, ctx: &yew::Context<Self>, msg: Self::Message) -> bool {
    let context: APIContext = ctx.link().context(Callback::noop()).unwrap().0;
    let api = context.service;

    match msg {
      HomePageMessage::ObjectListFetched(object_list) => {
        log::debug!("Fetched {} list descriptors.", object_list.list.len());
        self.lists = Some(Vec::with_capacity(object_list.list.len()));
        self.unfetched_lists = object_list.list.len();

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

        // when the user is logged in, fetch their lists asynchronously
        ctx.link().send_future(async move {
          match api.get_users_lists().await {
            Ok(ol) => HomePageMessage::ObjectListFetched(ol),
            Err(e) => HomePageMessage::Error(e.to_string()),
          }
        });
        context.app_callback.emit(AppMessage::LoginSuccessful(user_id));
        true
      }
      // TODO: remove?
      HomePageMessage::Error(reason) => {
        // Pass up the token tree to use centralised error handling
        context.app_callback.emit(AppMessage::Error(reason));
        false
      }
      //TODO: Evaluate reducing page refreshes/performance impact
      HomePageMessage::ListFetched(list) => {
        self.unfetched_lists -= 1;

        log::debug!("Fetched flat list: {}. {} lists remaining.", list.name, self.unfetched_lists);

        self.lists.as_mut().unwrap().push(Rc::new(list));


        true
      }
      HomePageMessage::ShowAddListModal => {
        self.show_add_list_modal = true;

        true
      }
      HomePageMessage::CloseAddListModal => {
        self.show_add_list_modal = false;

        true
      }
      HomePageMessage::None => false,
      HomePageMessage::NewList(name) => {
        self.unfetched_lists += 1;
        log::debug!("Constructing new list created through UI with name: {name}");

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
            // if there is no picture for the list, just display the name
            else {
              html! {
                <p><b>{&list.name}</b></p>
              }
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
