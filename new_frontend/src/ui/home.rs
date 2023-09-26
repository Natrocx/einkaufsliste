
use dioxus::prelude::*;
use dioxus_router::prelude::{use_navigator, Link};




use super::error::ErrorService;
use crate::service::api::ApiService;
use crate::ui::Route;

pub fn homepage(cx: Scope) -> Element {
  let error_handler = cx.consume_context::<ErrorService>().unwrap();
  let _navigator = use_navigator(cx);
  let lists = use_state(cx, std::vec::Vec::new);
  let api = cx.consume_context::<ApiService>()?;

  let _api = api.clone();
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

  cx.render(rsx!(
    div {
        lists.iter().map(|list| {
            //whyever the compiler can't do that itself....
            let api = &_api;
            rsx!(self::list_preview { name: &list.name, image_id: list.image_id.map(|id| api.get_img_url(id)) })
        }),
        Link { to: Route::Authentication, "Auth" }
    }
))
}

#[derive(PartialEq, Clone, Debug, Props)]
pub struct ListPreviewProps<'a> {
  name: &'a str,
  #[props(!optional)]
  image_id: Option<String>,
}

// A Component that renders ListPreviewProps as a Card, fetching the image from the API or using a placeholder
pub fn list_preview<'a>(cx: Scope<'a, ListPreviewProps<'a>>) -> Element {
  cx.render(rsx!(
    div {
        match cx.props.image_id {
        Some(ref src_url) => {
            rsx!(img { src: "{src_url}" })
        },
        None => rsx!( p { "?"}),
        
        },
        div {
            p { cx.props.name }
        }
    }
))
}
