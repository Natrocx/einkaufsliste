use std::rc::Rc;

use web_sys::{HtmlInputElement, KeyboardEvent};
use yew::{html, html_nested, Callback, Component, Html, NodeRef, Properties};

use crate::ok_or_log;

/// Reusable, configurable modal
///
/// ## Internal event handlers:
/// * On pressing `Enter` the last action will be emitted. Make sure the standard action (i.e. the submit of your modal) is the last action passed to the component props; it will also be rendered right-most for end-user consistency.
#[derive(Default)]
pub struct TextInputModal;
//TODO: evaluate generified modal: fields may be an enum over field type like: text, number, radio
//TODO: evaluate field verifiers

#[derive(PartialEq, Properties, Clone, Debug)]
pub struct TextInputModalProps {
  /// A title/prompt for the modal, so the user knows what they are doing
  pub prompt: &'static str,
  pub fields: Rc<Vec<TextInputModalField>>,
  pub actions: Vec<TextInputModalButton>,
}

#[derive(PartialEq, Debug)]
pub struct TextInputModalField {
  pub name: &'static str,
  pub node_ref: NodeRef,
  pub placeholder: Option<&'static str>,
  pub required: bool,
}

#[derive(PartialEq, Debug, Clone)]
pub struct TextInputModalButton {
  pub prompt: &'static str,
  pub callback: Callback<()>,
}

impl Component for TextInputModal {
  type Message = ();

  type Properties = TextInputModalProps;

  fn create(_ctx: &yew::Context<Self>) -> Self {
    Self
  }

  fn view(&self, ctx: &yew::Context<Self>) -> yew::Html {
    let key_press_handler = if let Some(submit) = ctx.props().actions.last() {
      let callback = submit.callback.clone();
      let fields = ctx.props().fields.clone();
      Some(ctx.link().callback(move |event: KeyboardEvent| {
        let key = event.key();
        if key.eq("Enter") && check_required_fields(&fields) {
          callback.emit(());
        }
      }))
    } else {
      None
    };

    html! {
      <div class="modal-background">
        <div class="modal-container">
          <div class="modal-grid">
          {
            ctx.props().fields.iter().map(|modal_field| {
              let placeholder = if let Some(placeholder) = modal_field.placeholder {
                placeholder
              }
              else {
                ""
              };

              html! {
                <>
                  <p class="modal-item">{modal_field.name}</p>
                  <input class="input modal-item" onkeypress={key_press_handler.clone()} ref={modal_field.node_ref.clone()} type="text" placeholder={placeholder} />
                </>
              }
            }).collect::<Html>()
          }
          </div>
          <div class="button-list">
            {
              // cloning should be fine since we can generally expect this to be very few elements
              ctx.props().actions.clone().into_iter().map(move |action| {
                html_nested! {
                  <button onclick={move |_| action.callback.emit(())}>{action.prompt}</button>
                }
              }).collect::<Html>()
            }
          </div>
        </div>
      </div>
    }
  }

  // after the modal is rendered, it should:
  // * focus the first element so the user can start typing instantly
  // * not cause any unnecessary errors
  fn rendered(&mut self, ctx: &yew::Context<Self>, first_render: bool) {
    if first_render {
      if let Some(field) = ctx.props().fields.first() {
        let element: HtmlInputElement = field.node_ref.cast().expect("Input element to be present");
        ok_or_log!(element.focus())
      }
    }
  }
}

pub fn check_required_fields(fields: &[TextInputModalField]) -> bool {
  fields
    .iter()
    .filter(|field| field.required)
    .all(|field| !field.node_ref.cast::<HtmlInputElement>().unwrap().value().is_empty())
}
