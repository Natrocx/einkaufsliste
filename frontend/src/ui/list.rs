use std::rc::Rc;

use einkaufsliste::model::item::Item;
use einkaufsliste::model::list::FlatItemsList;
use web_sys::HtmlInputElement;
use yew::prelude::*;
use yew_router::prelude::History;
use yew_router::scope_ext::RouterScopeExt;

use super::consts::*;
use super::context::APIContext;
use super::AppMessage;
use crate::service::api::APIService;
use crate::ui::util::CircularLoadingIndicator;

#[derive(Default)]
pub struct ListItemView {
  dropdown_active: bool,
}

#[derive(PartialEq, Properties)]
pub struct ListItemProperties {
  pub item: Item,
  pub change_name_callback: Callback<(Item, Rc<APIService>)>,
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
        let api: APIContext = ctx.link().context(Callback::noop()).unwrap().0;
        ctx.props().change_name_callback.emit((
          Item {
            name: new,
            ..ctx.props().item.clone()
          },
          api.service,
        )); // I guess we have to copy here
        true
      }
      ListItemMessage::Delete => todo!(),
    }
  }
}

#[derive(PartialEq, Properties, Clone)]
pub struct ListProperties {
  pub(crate) id: u64,
  pub(crate) error_callback: Callback<AppMessage>,
}

pub struct InnerListView {}

#[derive(PartialEq, Properties)]
pub struct InnerListProperties {
  list: Rc<FlatItemsList>,
  error_callback: Callback<AppMessage>,
}

pub enum InnerListMessage {
  Noop,
  Error(String),
}

impl Component for InnerListView {
  type Message = InnerListMessage;

  type Properties = InnerListProperties;

  fn create(_ctx: &Context<Self>) -> Self {
    Self {}
  }

  fn view(&self, ctx: &Context<Self>) -> Html {
    let _context: (APIContext, _) = ctx.link().context(Callback::noop()).unwrap();

    let callback = ctx
      .link()
      .callback_future(|(item, api): (Item, Rc<APIService>)| async move {
        match api.update_item(&item).await {
          Ok(_) => InnerListMessage::Noop,
          Err(e) => InnerListMessage::Error(e.to_string()),
        }
      });

    html! {
      <>
      {"Test"}
      {
        ctx.props().list.items.iter().map(|item| {
          html! {
            <ListItemView item={item.clone()} change_name_callback={callback.clone()}/>
          }
        }).collect::<Html>()
      }
     </>
    }
  }

  fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
    match msg {
      InnerListMessage::Noop => false,
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
  list: Option<Rc<FlatItemsList>>,
}

impl Component for ListView {
  type Message = ListMessage;

  type Properties = ListProperties;

  fn create(ctx: &Context<Self>) -> Self {
    let id = ctx.props().id;
    let api: APIContext = ctx
      .link()
      .context(Callback::noop())
      .expect("to be run inside ContextProvider component")
      .0;
    ctx.link().send_future(async move {
      match api.service.get_flat_items_list(id).await {
        Ok(list) => ListMessage::FetchSuccessful(list),
        Err(e) => ListMessage::FetchUnsuccessful(e.to_string()),
      }
    });

    Self { list: None }
  }

  #[allow(clippy::let_unit_value)]
  fn view(&self, ctx: &Context<Self>) -> Html {
    html! {
      {
          if self.list.is_some() {
            let props = InnerListProperties { list: self.list.clone().unwrap(), error_callback: ctx.props().error_callback.clone() };
            html! {
              <div>
                <InnerListView ..props />
              </div>
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
        self.list = Some(Rc::new(list));
        true
      }
      ListMessage::FetchUnsuccessful(message) => {
        // if the list could not be loaded, print error message and go back (there is nothing the user can do here)
        ctx.props().error_callback.emit(AppMessage::Error(message));
        ctx.link().history().unwrap().go(-1);
        true
      }
    }
  }
}
