use std::collections::HashMap;
use std::sync::Arc;

use einkaufsliste::model::list::{FlatItemsList, List};
use einkaufsliste::model::shop::{self, Shop};
use iced::advanced::Widget;
use iced::widget::{button, text, Column, Container, Row};
use iced::{theme, Command, Element, Length, Theme};
use iced_aw::native::{wrap_horizontal, wrap_vertical};
use iced_aw::{floating_element, Icon};

use super::styles::{CircleButtonStyle, ListPreviewContainerStyle, DEFAULT_TEXT_SIZE};
use super::MainMessage;
use crate::service::api::ApiService;

pub(crate) struct HomeView {
  api_service: ApiService,
  lists: Arc<Vec<List>>,
  shops: Arc<HashMap<u64, Shop>>,
  selection_mode: bool,
  selected_lists: Vec<u64>,
}

#[derive(Debug, Clone)]
pub(crate) enum HomeMessage {
  NewList,
  ToggleSelectionMode,
  SelectList(u64),
  DeselectList(u64),
  DeleteSelection,
}

impl HomeView {
  pub fn new(lists: Arc<Vec<List>>, shops: Arc<HashMap<u64, Shop>>, api_service: ApiService) -> Self {
    Self {
      api_service,
      shops,
      lists,
      selection_mode: false,
      selected_lists: Vec::new(),
    }
  }

  pub fn update(&mut self, message: HomeMessage) -> Command<MainMessage> {
    match message {
      HomeMessage::NewList => {
        let api_service = self.api_service.clone();

        Command::perform(
          async move { api_service.create_list(List::default()).await },
          |res| match res {
            Ok(list) => MainMessage::NewLists(vec![FlatItemsList::from_list_and_items(list, vec![])]),
            Err(err) => MainMessage::Toast(err.into()),
          },
        )
      }
      HomeMessage::ToggleSelectionMode => {
        self.selection_mode = !self.selection_mode;
        Command::none()
      }
      HomeMessage::SelectList(id) => {
        self.selected_lists.push(id);
        Command::none()
      }
      HomeMessage::DeselectList(id) => {
        self.selected_lists.retain(|&x| x != id);
        Command::none()
      }
      HomeMessage::DeleteSelection => {
        todo!()
      }
    }
  }

  pub fn view(&self) -> Element<HomeMessage> {
    let double_text_size = DEFAULT_TEXT_SIZE * 2.0;
    let previews: Vec<_> = self
      .lists
      .iter()
      .map(|list| {
        let shop_name = self.shops.get(&list.id).map_or("No shop", |shop| shop.name.as_str());
        let preview = Container::new(Row::with_children(vec![
          // TODO: Add icon
          text("?").size(double_text_size).into(),
          Column::with_children(vec![text(list.name.as_str()).into(), text(shop_name).into()]).into(),
        ]))
        .width(Length::Shrink)
        .height(Length::Shrink)
        .style(theme::Container::Custom(Box::new(ListPreviewContainerStyle::new(
          theme::Container::Transparent,
        ))))
        .padding(10.0);

        preview.into()
      })
      .collect();

    let mut previews = wrap_horizontal(previews).spacing(10.0).padding(5.0);
    // why no setters?
    previews.height = Length::Fill;
    previews.width = Length::Fill;

    floating_element(
      previews,
      //Why does this not work???
      //button(text(Icon::Plus))
      button("+")
        .style(theme::Button::Custom(Box::new(CircleButtonStyle::new(
          theme::Button::Primary,
        ))))
        .on_press(HomeMessage::NewList),
    )
    .into()
  }
}
