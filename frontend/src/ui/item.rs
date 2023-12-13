use dioxus::prelude::*;
use dioxus_signals::Signal;
use einkaufsliste::model::item::Item;

use crate::ui::consts::*;
use crate::ui::list::SyncType;

#[component]
pub fn ItemView<DragStartHandler: Fn(u64), DragDropHandler: Fn(u64)>(
  cx: Scope,
  item: Signal<Item>,
  dragstart: DragStartHandler,
  drag_drop: DragDropHandler,
) -> Element {
  to_owned![item];
  // This is a workaround for a bug in WebKitGtk that prevents drag and drop from working when no data is set
  // TODO: remove when fixed: https://bugs.webkit.org/show_bug.cgi?id=265857
  #[cfg(target_os = "linux")]
  {
    let eval_workaround = use_eval(cx);
    use_on_create(cx, || {
      to_owned![eval_workaround];
      async move {
        eval_workaround(&format!(
          r#"
            document.getElementById("item-view-{}").addEventListener("dragstart", (evt) => {{
              evt.dataTransfer.setData("text/plain", " ");
            }});
            "#,
          item.read().id.to_string()
        ))
        .unwrap();
      }
    });
  }

  let _syncer = use_coroutine_handle::<SyncType>(cx)?;
  let first_render = use_state(cx, || true).clone();

  let is_getting_dragged = use_state(cx, || false);
  let is_dragged_over = use_state(cx, || false);

  let syncer = _syncer.clone();
  dioxus_signals::use_effect(cx, move || {
    let _ = item.read();
    if !*first_render.current() {
      syncer.send(SyncType::UpdateItem(item));
    } else {
      first_render.set(false);
    }
  });

  // We sadly have to use ondragover here since the ondragenter/ondragleave events fire when moving
  // into a nested input element causing the visual space to flicker
  let ondragenter = move |evt: DragEvent| {
    evt.stop_propagation();
    tracing::trace!("Drag entered over item: {:?}", item.read());
    if !is_getting_dragged.get() {
      tracing::trace!("Adding visual space for dragged over item");
      is_dragged_over.set(true);
    }
  };

  let ondrop = move |evt: DragEvent| {
    evt.stop_propagation();
    tracing::trace!("Dropped dragged item into: {:?}", item.read());

    is_dragged_over.set(false);
    drag_drop(item.read().id);
  };

  let ondragleave = move |evt: DragEvent| {
    evt.stop_propagation();
    tracing::trace!("Drag left over item : {:?}", item.read());
    is_dragged_over.set(false);
  };

  // This cannot go inline, as it will cause the underlying RefCell to panic
  let checked = item.read().checked;
  let syncer = _syncer.clone();

  let maybe_padding = if *is_dragged_over.get() {
    String::from("pt-8")
  } else {
    String::new()
  };

  // Unnecessary bindings are the price we pay for the compiler to be happy
  // You know the saying - happy compiler, happy life
  let x = render!(
    div {
      id: "item-view-{item.read().id}",
      class: "flex",
      draggable: true,
      // The default ondragover handler is to prevent the drop event from firing
      // It needs to be disabled on the div and all its children
      prevent_default: "ondragover",
      // the ondragenter handlers cannot bubble so they are also registered on the child components
      ondragenter: ondragenter,
      ondragleave: ondragleave,
      ondragover: move |evt| {
          evt.stop_propagation();
      },
      ondragstart: move |evt| {
          evt.stop_propagation();
          tracing::trace!("Drag started: {evt:?}");
          is_getting_dragged.set(true);
          dragstart(item.read().id);
      },
      ondragend: move |evt| {
          evt.stop_propagation();

          tracing::trace!("Drag ended for item: {:?}", item.read());
          is_getting_dragged.set(false);
      },
      ondrop: ondrop,
      button {
        ondragenter: ondragenter,
        prevent_default: "ondragover",
        class: "material-symbols-outlined {maybe_padding}",
        onclick: move |_| {
            item.write().checked = !checked;
        },
        if item.read().checked {
        CHECKBOX_CHECKED
        } else {
        CHECKBOX_UNCHECKED
        }
      }
      input {
        ondragenter: ondragenter,
        prevent_default: "ondragover",
        class: "{INLINE_INPUT} {maybe_padding} flex-grow",
        onchange: move |evt| item.write().name = evt.value.clone(),
        value: "{item.read().name}"
      }
      button {
        ondragenter: ondragenter,
        prevent_default: "ondragover",
        class: "material-symbols-outlined {maybe_padding}",
        onclick: move |_| {
            syncer.send(SyncType::DeleteItem(item.read().id));
        },
        DELETE
      }
    }
  );
  x
}
