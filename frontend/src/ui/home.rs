
use dioxus::prelude::*;
use dioxus_router::prelude::{use_navigator};
use einkaufsliste::model::list::List;



use super::error::ErrorService;
use crate::service::api::ApiService;


pub fn homepage(cx: Scope) -> Element {
  let error_handler = cx.consume_context::<ErrorService>().unwrap();
  let _navigator = use_navigator(cx);
  let lists = use_state(cx, std::vec::Vec::new);
  let api = cx.consume_context::<ApiService>()?;

  // enable using the api and error handler in multiple async blocks
  let _api = api.clone();
  let _error_handler = error_handler.clone();

  // fetch the lists from the API when the component is first rendered but do not refetch on local changes to avoid overwriting them
  use_future(cx, (), |()| {
    to_owned![lists];
    async move {
      let fetched_lists = match api.fetch_all_lists().await {
        Ok(lists) => lists,
        Err(e) => {
          error_handler.handle_api_error(e).await;
          return;
        }
      };
      lists.set(fetched_lists);
    }
  });

  let api = _api.clone();
  let on_new = move |_| {
    to_owned![lists];
    let error_handler = _error_handler.clone();
    let api = api.clone();
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
          error_handler.handle_api_error(e).await;
        }
      }
    })
  };

  cx.render(rsx!(
      div {
          class: "flex flex-row flex-wrap gap-1",
          lists.iter().map(|list| {
              //whyever the compiler can't do that itself....
              let api = &_api;
              rsx!(self::list_preview { name: &list.name, image_id: list.image_id.map(|id| api.get_img_url(id)), shop_name: "Testshop" })
          }),
      }
    button {
      class: "flex w-full justify-center rounded-md bg-teal-600 px-3 py-1.5 text-sm font-semibold leading-6 text-white shadow-sm hover:bg-teal-500 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-teal-600",
      onclick: on_new,
      "New List"
    }
  ))
}

#[derive(PartialEq, Clone, Debug, Props)]
pub struct ListPreviewProps<'a> {
  name: &'a str,
  #[props(!optional)]
  image_id: Option<String>,
  shop_name: Option<&'a str>,
}

// A Component that renders ListPreviewPops as a Card, fetching the image from the API or using a placeholder
pub fn list_preview<'a>(cx: Scope<'a, ListPreviewProps<'a>>) -> Element {
  cx.render(rsx!(
      div {
          class: "flex flex-row content-center gap-1 mx-0.5 my-1 p-1 border border-teal-950 hover:border-teal-900 hover:border-2",
          match cx.props.image_id {
          Some(ref src_url) => {
              rsx!(img { src: "{src_url}" })
          },
          None => rsx!( p {
          class: "flex-shrink self-center leading-none pr-0.5",
          "?"
        }),
          },
          div {
              class: "flex flex-col gap-x-1 flex-nowrap",
              p { cx.props.name }
              p { class: "text-xs text-teal-800", cx.props.shop_name}
          }
      }
  ))
}
