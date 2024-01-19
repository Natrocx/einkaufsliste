use std::collections::HashMap;
use std::sync::Arc;

use einkaufsliste::model::article::Article;
use einkaufsliste::model::item::Item;
use einkaufsliste::model::list::{FlatItemsList, List};
use einkaufsliste::model::shop::Shop;
use einkaufsliste::model::user::User;
use einkaufsliste::model::Identifiable;
use iced::widget::{text, Column};
use iced::{font, Application, Command};

use self::error::{GuiMessage, Toast};
use crate::service::api::{ApiError, ApiService};

pub mod error;
pub mod home;
pub mod list;
pub mod login;
pub mod styles;

pub struct Einkaufsliste {
  user: Option<User>,

  api_service: ApiService,

  login_view: login::LoginView,
  home_view: home::HomeView,
  current_page: Page,

  toasts: Vec<error::Toast>,
  // TODO: abstract these into a repository struct
  /// This vector contains the metadata of each list
  lists: Arc<Vec<List>>,
  /// This map contains the items of each list, indexed by the list id
  items: HashMap<<List as Identifiable>::Id, Vec<Item>>,
  /// All articles currently available locally, indexed by their id for fast lookup
  articles: HashMap<<Article as Identifiable>::Id, Article>,
  /// All shops currently available locally, indexed by their id for fast lookup
  shops: Arc<HashMap<<Shop as Identifiable>::Id, Shop>>,
}

impl Einkaufsliste {
  fn borrow_lists_mut(&mut self) -> &mut Vec<List> {
    unsafe {
      // This is safe because we are at the top-level of the application and all users of this arc have the exact same lifetime as this struct
      // Furthermore we have a mutable reference to this struct and therefore to all users of this arc and can therefore guarantee that no other user of this arc is currently holding a reference to it
      Arc::get_mut_unchecked(&mut self.lists)
    }
  }

  fn borrow_shops_mut(&mut self) -> &mut HashMap<<Shop as Identifiable>::Id, Shop> {
    unsafe {
      // This is safe because we are at the top-level of the application and all users of this arc have the exact same lifetime as this struct
      // Furthermore we have a mutable reference to this struct and therefore to all users of this arc and can therefore guarantee that no other user of this arc is currently holding a reference to it
      Arc::get_mut_unchecked(&mut self.shops)
    }
  }
}

#[derive(Debug, Clone)]
enum Page {
  Home,
  Login,
  List(u64),
  Article(u64),
  Shop(u64),
  Settings,
}

#[derive(Debug, Clone)]
pub enum MainMessage {
  None,

  UserChanged(User),
  /// Store new lists in the local cache - this does not perform API calls
  NewLists(Vec<FlatItemsList>),
  FetchArticles(Vec<<Article as Identifiable>::Id>),
  FetchShops(Vec<<Shop as Identifiable>::Id>),
  /// Actively query the API for the latest data depending on the current page
  Refresh,
  ListMetaChanged(List),
  PageChanged(Page),
  Toast(GuiMessage),
  CloseToast(usize),

  // pass-through messages
  Login(login::LoginMessage),
  Home(home::HomeMessage),
}

impl Application for Einkaufsliste {
  type Executor = iced::executor::Default;

  type Message = MainMessage;

  type Theme = iced::Theme;

  // there are no command line or other arguments that can be passed at this time of development
  type Flags = ();

  fn new(_flags: Self::Flags) -> (Self, iced::Command<Self::Message>) {
    let api_service = ApiService::new("https://localhost:8443".to_owned()).unwrap();

    let lists = Arc::new(Vec::new());
    let shops = Arc::new(HashMap::new());
    (
      Einkaufsliste {
        home_view: home::HomeView::new(lists.clone(), shops.clone(), api_service.clone()),
        login_view: login::LoginView::new(api_service.clone()),
        api_service,
        user: None,
        toasts: Vec::new(),
        lists,
        items: HashMap::new(),
        articles: HashMap::new(),
        shops,
        current_page: Page::Home,
      },
      Command::batch([
        font::load(iced_aw::graphics::icons::ICON_FONT_BYTES).map(|_| MainMessage::None),
        Command::perform(async move {}, |()| MainMessage::Refresh),
      ]),
    )
  }

  fn title(&self) -> String {
    let title = match self.current_page {
      Page::Home => "Home",
      Page::Login => "Login",
      Page::List(id) => self
        .lists
        .iter()
        .find(|l| l.id == id)
        .map(|list| list.name.as_str())
        .unwrap_or("Unknown list"),
      Page::Article(id) => self
        .articles
        .get(&id)
        .map(|article| article.name.as_str())
        .unwrap_or("Unknown article"),
      Page::Shop(id) => self
        .shops
        .get(&id)
        .map(|shop| shop.name.as_str())
        .unwrap_or("Unknown shop"),
      Page::Settings => "Settings",
    };

    format!("Einkaufsliste - {}", title)
  }

  fn update(&mut self, message: Self::Message) -> iced::Command<Self::Message> {
    match message {
      MainMessage::UserChanged(user) => {
        self.user = Some(user);

        // After logging in it makes sense to go to the home page
        self.update(MainMessage::PageChanged(Page::Home))
      }
      MainMessage::NewLists(lists) => {
        for list in lists {
          let (list, items) = list.into_list_and_items();

          self.items.insert(list.id, items);
          self.borrow_lists_mut().push(list);
        }

        Command::none()
      }
      MainMessage::FetchArticles(_) => todo!(),
      MainMessage::FetchShops(_) => todo!(),
      MainMessage::Refresh => match self.current_page {
        Page::Home => {
          let api_service = self.api_service.clone();

          let fetch_future = async move { api_service.fetch_all_lists().await };

          Command::perform(fetch_future, |result| match result {
            Ok(lists) => {
              let lists = lists
                .into_iter()
                .map(|list| FlatItemsList::from_list_and_items(list, vec![]))
                .collect();
              MainMessage::NewLists(lists)
            }
            Err(e) => MainMessage::Toast(e.into()),
          })
        }
        _ => Command::none(),
      },
      MainMessage::ListMetaChanged(_) => todo!(),
      MainMessage::PageChanged(page) => {
        self.current_page = page;

        Command::none()
      }
      MainMessage::Toast(e) => {
        match e {
          GuiMessage::ApiError(e) => {
          match &*e {
            ApiError::Unauthenticated => {
              self.user = None;

              // Display a message as to why the page changed (suddenly)
              self.toasts.push(error::Toast {
                title: "You are not authenticated".to_owned(),
                body: e.to_string(),
                status: error::Status::Primary
              });

              self.update(MainMessage::PageChanged(Page::Login))
            }, 
            ApiError::Network(_inner) => {
              self.toasts.push(error::Toast {
                title: "Network error".to_owned(),
                body: e.to_string(),
                status: error::Status::Primary
              });

              Command::none()
            }
            _ => {
              self.toasts.push(error::Toast {
                title: "Unknown error".to_owned(),
                body: e.to_string(),
                status: error::Status::Danger
              });

              Command::none()
            },
          }
        },
        GuiMessage::Other(toast) => {
          self.toasts.push(toast);

          Command::none()
        }
      }
      }
      MainMessage::CloseToast(index) => {
        self.toasts.remove(index);

        Command::none()
      }
      MainMessage::Login(message) => {
        //noop

        //passthrough
        self.login_view.update(message)
      }
      MainMessage::Home(message) => {
        //noop

        //passthrough
        self.home_view.update(message)
      }
      MainMessage::None => Command::none(),
    }
  }

  fn view(&self) -> iced::Element<'_, Self::Message, iced::Renderer<Self::Theme>> {
    let full_page = Column::new().push(text(self.title()).size(30));

    let content = match self.current_page {
      Page::Home => self.home_view.view().map(MainMessage::Home),
      Page::Login => self.login_view.view().map(MainMessage::Login),
      Page::List(id) => todo!(),
      Page::Article(id) => todo!(),
      Page::Shop(id) => todo!(),
      Page::Settings => todo!(),
    };

    let full_page = full_page.push(content);

    let toast_handler = error::Manager::new(full_page, &self.toasts, MainMessage::CloseToast);

    toast_handler.into()
  }

  fn theme(&self) -> Self::Theme {
    iced::Theme::Dark
  }
}
