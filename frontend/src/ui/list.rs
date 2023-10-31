use dioxus::prelude::*;
use einkaufsliste::model::item::Item;
use einkaufsliste::model::list::FlatItemsList;

use crate::service::list::{use_provide_list_service, ListService};


#[component(no_case_check)]
pub fn list_view(cx: Scope<'_>, id: u64) -> Element<'_> {
  use_provide_list_service(cx, || FlatItemsList {
    id: 0,
    name: "title".into(),
    shop: None,
    image_id: None,
    items: vec![
        Item {
            id: 0,
            name: "item1".into(),
            amount: None,
            unit: None,
            checked: false,
            article_id: None,
            alternative_article_ids: None
        }
    ],
  });
  let list_service = use_shared_state::<ListService>(cx)?;
  let lock = list_service.read();

  let x = render! {
    h1 { lock.title().as_str() }
    // todo: add navigation/check all interactivity

    div { class: "flex gap-1" }
};
  x
}

#[component(no_case_check)]
fn item_view<'a>(cx: Scope, item: &'a Item) -> Element<'a> {
  render! {
    div {
        span { item.name.as_str() }
    }
}
}
