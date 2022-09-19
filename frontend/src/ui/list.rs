use std::ops::Index;
use std::rc::Rc;
use std::sync::Arc;
use std::thread;

use einkaufsliste::model::item::Item;
use einkaufsliste::model::list::{FlatItemsList, List};
use einkaufsliste::model::Identifiable;
use futures::Future;
use web_sys::{Element, HtmlElement, HtmlInputElement};
use yew::prelude::*;

use super::consts::*;
use super::{change_name_callback, fetch_callback, App, AppMessage};
use crate::service::api::APIService;
use crate::ui::util::CircularLoadingIndicator;
use crate::TransmissionError;

#[derive(Default)]
pub struct ListItemView {
  dropdown_active: bool,
}

#[derive(Properties)]
pub struct ListItemProperties {
  pub item: Item,
  pub change_name_callback: Callback<(Item, Arc<APIService>)>,
  pub api_service: Arc<APIService>,
}

impl PartialEq for ListItemProperties {
  fn eq(&self, other: &Self) -> bool {
    self.item.eq(&other.item)
  }
}

/// Represents user Interaction with ListItemView
pub enum ListItemMessage {
  ToggleCheck,
  ToggleUnitDropdown,
  ChangeName(String),
  Delete,
}

impl Component for ListItemView {
  type Message = ListItemMessage;

  type Properties = ListItemProperties;

  fn create(_ctx: &Context<Self>) -> Self {
    Self::default()
  }

  fn view(&self, ctx: &Context<Self>) -> Html {
    let name = match ctx.props().item.name.is_empty() {
      true => None,
      false => Some(ctx.props().item.name.clone()), //TODO: evaluate clone
    };
    let amount = ctx.props().item.amount.map(|val| val.to_string());

    let edit_callback = ctx.link().callback(|e: InputEvent| {
      // TODO: evaulate live editing on multiple devices
      let input: HtmlInputElement = e.target_unchecked_into();
      ListItemMessage::ChangeName(input.value())
    });

    html! {
      <div class="li-container" id={ctx.props().item.id.to_string()}>
        // checkbox for marking as bought
        <div class="li-checkbox">
          <span
            class="material-symbols-outlined"
            onclick={ctx.link().callback(|_| ListItemMessage::ToggleCheck)}>
              { if ctx.props().item.checked {
                CHECKED_CHECK_BOX
              } else {
                UNCHECKED_CHECK_BOX
            }}
          </span>
        </div>


        <div class="li-name-container">
          <input
            class="input li-name"
            type="text"
            placeholder="Add new item"   // TODO: internationalization
            value={name}
            oninput={edit_callback}/>
        </div>


        <div class="li-unit-container">
            <button
              onclick={ctx.link().callback(|_| ListItemMessage::ToggleUnitDropdown)}
              class="button dropdown-trigger"
              aria-haspopup="true"
              aria-controls="dropdown-menu">

              <span>{"Unit"}</span> //TODO: internationalization
              <span class="material-symbols-outlined">
                {ARROW_DROP_DOWN}
              </span>
            </button>

          <div class={if self.dropdown_active {
             "dropdown"
            } else {
              "inactive"
            }}
            id="dropdown-menu"
            role="menu">

            <div class="dropdown-content">
              <a href="#" class="dropdown-item">
                {"None"}
              </a>
              <a class="dropdown-item">
                {"Kilogram"}
              </a>
            </div>
          </div>
        </div>


        <input
          class="input li-amount"
          type="number"
          placeholder="Amount"   // TODO: internationalization
          value={amount}/>
      </div>
    }
  }

  fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
    match msg {
      ListItemMessage::ToggleCheck => todo!(),
      ListItemMessage::ToggleUnitDropdown => {
        self.dropdown_active = !self.dropdown_active;
        true
      }
      ListItemMessage::ChangeName(new) => {
        ctx.props().change_name_callback.emit((
          Item {
            name: new,
            ..ctx.props().item.clone()
          },
          ctx.props().api_service.clone(),
        )); // I guess we have to copy here
        true
      }
      ListItemMessage::Delete => todo!(),
    }
  }

  fn destroy(&mut self, ctx: &Context<Self>) {}
}

#[derive(Properties, Clone)]
pub struct ListProperties {
  pub(crate) api_service: Arc<APIService>,
  pub(crate) id: u64,
  pub(crate) error_callback: Callback<AppMessage>,
}

impl PartialEq for ListProperties {
  fn eq(&self, other: &Self) -> bool {
    self.api_service.base_url == other.api_service.base_url && self.id == other.id
  }
}

pub struct InnerListView {}

#[derive(Properties)]
pub struct InnerListProperties {
  list: FlatItemsList,
  api_service: Arc<APIService>,
  error_callback: Callback<AppMessage>,
}

impl PartialEq for InnerListProperties {
  fn eq(&self, other: &Self) -> bool {
    self.list.eq(&other.list) && self.api_service.base_url == other.api_service.base_url
  }
}

pub enum InnerListMessage {
  NOOP,
  Error(String),
}

impl Component for InnerListView {
  type Message = InnerListMessage;

  type Properties = InnerListProperties;

  fn create(_ctx: &Context<Self>) -> Self {
    Self {}
  }

  fn view(&self, ctx: &Context<Self>) -> Html {
    let api_service = ctx.props().api_service.clone();
    // Do not try to replace this with a closure. You will cry.
    let callback = ctx.link().callback_future(change_name_callback);

    html! {
      <>
      {"Test"}
      {
        ctx.props().list.items.iter().map(|item| {
          html! {
            <ListItemView item={item.clone()} change_name_callback={callback.clone()} api_service={api_service.clone()}/>
          }
        }).collect::<Html>()
      }
     </>
    }
  }

  fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
    match msg {
      InnerListMessage::NOOP => false,
      InnerListMessage::Error(message) => {
        ctx.props().error_callback.emit(AppMessage::Error(message));
        false
      }
    }
  }
}
pub enum ListMessage {
  FetchSuccessful(FlatItemsList),
  FetchUnsuccessful(String),
}

pub struct ListView {
  list: Option<FlatItemsList>,
  fetch_finished: bool,
}

impl Component for ListView {
  type Message = ListMessage;

  type Properties = ListProperties;

  fn create(ctx: &Context<Self>) -> Self {
    ctx
      .link()
      .send_future(fetch_callback((ctx.props().id, ctx.props().api_service.clone())));

    Self {
      list: None,
      fetch_finished: false,
    }
  }

  #[allow(clippy::let_unit_value)]
  fn view(&self, ctx: &Context<Self>) -> Html {
    html! {
      {
        if self.fetch_finished {
          if self.list.is_some() {
            let props = InnerListProperties { list: self.list.clone().unwrap(), api_service: ctx.props().api_service.clone(), error_callback: ctx.props().error_callback.clone() };
            html! {
              <div>
                <InnerListView ..props />
              </div>
            }
          }
          else {
            html! {
              <div>
                <p>{"Error fetching data"}</p>
                <progress></progress>
              </div>
            }
          }
        }
        else {
          html! {
            <div class="list-loading">
              <CircularLoadingIndicator/>
            </div>
          }
        }
      }
    }
  }

  fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
    match msg {
      ListMessage::FetchSuccessful(list) => {
        self.fetch_finished = true;
        self.list = Some(list);
        true
      }
      ListMessage::FetchUnsuccessful(message) => {
        self.fetch_finished = true;
        ctx.props().error_callback.emit(AppMessage::Error(message));
        true
      }
    }
  }

  fn destroy(&mut self, _ctx: &Context<Self>) {}
}
