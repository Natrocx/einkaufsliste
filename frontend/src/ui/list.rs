use std::{rc::Rc, ops::Index};

use einkaufsliste::model::{item::Item, Identifiable};
use web_sys::{Element, HtmlElement, HtmlInputElement};
use yew::prelude::*;

use super::consts::*;
use crate::service::api::APIService;


#[derive(Default)]
pub struct ListItemView {
  dropdown_active: bool,
}

#[derive(Properties)]
pub struct ListItemProperties {
  pub item: Rc<Item>,
  pub change_name_callback: Callback<String>,
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
        ctx.props().change_name_callback.emit(new);
        true
      }
      ListItemMessage::Delete => todo!(),
    }
  }

  fn changed(&mut self, ctx: &Context<Self>) -> bool {
    true
  }

  fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {}

  fn destroy(&mut self, ctx: &Context<Self>) {}
}


#[derive(Properties)]
pub struct ListProperties {
  api_service: APIService
}

impl PartialEq for ListProperties {
    fn eq(&self, other: &Self) -> bool {
        self.api_service.base_url == other.api_service.base_url
    }
}

pub struct ListView {
  api_service: APIService,
  items: Vec<Item>
}


enum ListMessage {
  NOOP,
}

impl Component for ListView {
    type Message = ListMessage;

    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        Self {
          items: vec![],
          api_service: APIService::new("https://localhost:8443").unwrap()
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
      let callback = ctx.link().callback_future(|(id,  name)| Self::item_name_changed(&self, id, name));
             
      html! {
          <>
          {"Test"}
          {
            self.items.iter().map(|item| {
              html! {
                <ListItemView item={item} change_name_callback={callback}/>
              }
            }).collect::<Html>()
          }
         </>
        }
    }
}

impl ListView {
  pub async fn item_name_changed(&mut self, id: <Item as Identifiable>::Id, name: String) -> ListMessage {
    let mut item = self.items.iter_mut().find(|item| item.id == id).unwrap();
    item.name = name;

    let status = self.api_service.update_item(item).await;
    ListMessage::NOOP
  }
}