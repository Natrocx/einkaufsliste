use std::ops::Index;
use std::time::Instant;

use dioxus::prelude::*;
use dioxus_router::prelude::use_navigator;
use einkaufsliste::model::list::List;

use crate::service::api::{APIError, ApiService};
use crate::ui::consts::{ADD, CHECKBOX_CHECKED, CHECKBOX_UNCHECKED};
use crate::ui::scaffold::PageHeader;
use crate::ui::Route;

pub fn homepage(cx: Scope) -> Element {
  let error_handler: &Coroutine<APIError> = use_coroutine_handle(cx)?;
  let _navigator = use_navigator(cx);
  let lists = use_state(cx, std::vec::Vec::new);
  let api = use_context::<ApiService>(cx)?;
  let selection_mode = use_state(cx, || false);
  let selected_lists = use_ref(cx, || std::vec::Vec::<u64>::new());

  // fetch the lists from the API when the component is first rendered but do not refetch on local changes to avoid overwriting them
  use_future(cx, (), |()| {
    to_owned![api, lists, error_handler];
    async move {
      let fetched_lists = match api.fetch_all_lists().await {
        Ok(lists) => lists,
        Err(e) => {
          error_handler.send(e);
          return;
        }
      };
      lists.set(fetched_lists);
    }
  });

  let on_new = move |_| {
    to_owned![api, error_handler, lists];
    let mut list = List {
      name: "New List".to_string(),
      image_id: None,
      id: 0,
      shop: None,
      items: vec![],
    };

    cx.spawn(async move {
      match api.create_list(&list).await {
        Ok(id) => {
          list.id = id;
          lists.with_mut(|lists| lists.push(list));
        }
        Err(e) => {
          error_handler.send(e);
        }
      }
    })
  };

  render!(
    PageHeader { "Home" }
    div { class: "flex flex-row flex-wrap gap-1",
      if !lists.is_empty() {
      rsx!(
      lists.iter().map(|list| {
          rsx!(
              self::ListPreview { list: &list, selection_mode: selection_mode.clone(), selected_lists: selected_lists }
              )
          })
          )
      }
      else {
      rsx!(p { "You have no lists yet." })
      }
    }

    button {
      class: "flex justify-center rounded-full bg-teal-600 px-2.5 py-2.5 text-sm font-semibold leading-6 text-white shadow-sm hover:bg-teal-500 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-teal-600",
      onclick: on_new,
      span { class: "material-symbols-outlined", ADD }
    }
    button {
      class: "flex justify-center rounded-full bg-teal-600 px-2.5 py-2.5 text-sm font-semibold leading-6 text-white shadow-sm hover:bg-teal-500 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-teal-600",
      onclick: move |_| {
        selection_mode.set(!*selection_mode.get());
      },
      "Toggle selection"
    }
  )
}

#[component]
pub fn ListPreview<'a>(
  cx: Scope<'a>,
  list: &'a List,
  selection_mode: UseState<bool>,
  selected_lists: &'a UseRef<Vec<u64>>,
) -> Element {
  let pointer_down = use_state(cx, || None::<Instant>);
  let toggle = move |selected: bool| {
    if selected {
      selected_lists.with_mut(|selected_lists| {
        selected_lists.push(list.id);
      })
    } else {
      selected_lists.with_mut(|selected_lists| {
        selected_lists.retain(|id| *id != list.id);
      })
    };
  };

  let class_list = if *selection_mode.get() { "opacity-30" } else { "" };

  let first_render = cx.generation() == 0;
  let on_pointer_down = use_future(cx, (), |()| {
    to_owned![selection_mode];
    async move {
      if !first_render {
      async_std::task::sleep(std::time::Duration::from_millis(1000)).await;
      selection_mode.set(!*selection_mode.get());
      }
    }
  });

  render!(
    div {
      style: "contain: paint;",
    div { class: "flex flex-row content-center gap-1 mx-0.5 my-1 p-1 border border-teal-950 hover:border-teal-900 hover:border-2 {class_list}",
      onpointerdown: move |event| {
        pointer_down.set(Some(Instant::now()));
        on_pointer_down.restart();
      },
      onpointercancel: move |_| {
        pointer_down.set(None);
        on_pointer_down.cancel(cx);
      },
      onpointerup: move |event| {
        match pointer_down.get() {
          Some(instant) if instant.elapsed().as_millis() > 1000 => {
            event.stop_propagation();
          },
          _ => {
            let navigator = use_navigator(cx);
            navigator.push(Route::List {id: list.id });
          }
        };
      },
      match list.image_id {
      Some(ref src_url) => {
          rsx!(img { src: "{src_url}" })
      },
      None => rsx!( p {
      class: "flex-shrink self-center leading-none pr-0.5",
      "?"
      }),
      },
      div { class: "flex flex-col gap-x-1 flex-nowrap",
        p { list.name.as_str() }
        p { class: "text-xs text-teal-800", "TODO" }
      }
    }
    if *selection_mode.get() {
      rsx!(SelectionOverlay {
      selected: selected_lists.read().contains(&list.id),
      toggle: toggle,
      })
    }
  }
  )
}

#[component]
pub fn SelectionOverlay<SelectionToggle: Fn(bool)>(cx: Scope, selected: bool, toggle: SelectionToggle) -> Element<'a> {
  render!(div {
    class: "w-full h-full z-10 inset-0 bg-black bg-opacity-75 mix-blend-darken",
    onclick: move |evt| {
      evt.stop_propagation();
      tracing::debug!("Clicked toggle overlay");
      toggle(!selected);
    },
    span {
      class: "material-symbols-outlined z-20 absolute top-0 right-0 m-2",
      if *selected {
        CHECKBOX_CHECKED
      }
      else {
        CHECKBOX_UNCHECKED
      }
    }
  })
  }
