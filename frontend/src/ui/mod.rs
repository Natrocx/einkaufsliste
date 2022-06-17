use std::rc::Rc;

use einkaufsliste::model::item::{Item, Unit};
use yew::{Component, html};

use crate::{service::api::APIService, TransmissionError};

use self::list::{ListItemProperties, ListItemView};


pub mod list;
mod consts;

pub struct App;


impl Component for App {
    type Message = ();

    type Properties = ();

    fn create(ctx: &yew::Context<Self>) -> Self {
        Self
    }

    fn view(&self, ctx: &yew::Context<Self>) -> yew::Html {
      let api_service = APIService::new("https://localhost:8443").unwrap();

      let props = ListItemProperties {
        item: &Item {
          id: 0,
          name: "Schinken".to_owned(),
          checked: false,
          amount: Some(2),
          unit: Some(Unit::KiloGram),
          article_id: None,
          alternative_article_ids: None,
        },
        change_name_callback: ctx.link().callback(| name| {todo!()}),
      };

      html!{
        <ListItemView ..props />
      }
      
    }
}