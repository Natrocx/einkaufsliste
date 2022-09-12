//#![feature(generic_associated_types)]
#![feature(async_closure)]

pub mod service;
pub mod ui;

use std::error::Error;
use std::fmt::Display;
use std::sync::Arc;

use einkaufsliste::model::item::{Item, Unit};
use einkaufsliste::model::list::List;
use einkaufsliste::model::requests::{LoginUserV1, RegisterUserV1, StoreItemAttached};
use einkaufsliste::model::shop::Shop;
use log::Level;
use rand::distributions::{Standard, Alphanumeric};
use rand::prelude::Distribution;
use rand::Rng;
use reqwest::StatusCode;
use rkyv::validation::validators::CheckDeserializeError;
use ui::App;

use crate::service::api::APIService;

fn main() {
  console_log::init_with_level(Level::Debug).unwrap();
  yew::start_app::<App>();
}

#[test]
fn test_requests() {
  let rt = tokio::runtime::Runtime::new().unwrap();
  let api_service = APIService::insecure().unwrap();

  rt.block_on(async {
    let rng = rand::thread_rng();
    let user_name = format!(
      "test_user_{}",
      rng.sample_iter(Alphanumeric).take(15).map(|num| num as char).collect::<String>()
    );

    println!("-------------- unauthenticated POST /itemList -------------");
    api_service
      .push_new_item_list(List {
        id: 0,
        name: "vier".to_owned(),
        shop: None,
        image_id: None,
        items: vec![],
      })
      .await
      .unwrap_err();
    println!("Failed successfully!");

    println!("-------------- POST /register/v1 -------------------");
    let user_id = api_service
      .register_v1(&RegisterUserV1 {
        name: user_name.clone(),
        password: "EinPasswortMit8Zeichen".to_owned(),
      })
      .await
      .unwrap();
    println!("New user id: {user_id}");

    println!("-------------- POST /login/v1 ----------------------");
    api_service
      .login_v1(&LoginUserV1 {
        name: user_name.clone(),
        password: "EinPasswortMit8Zeichen".to_string(),
      })
      .await
      .unwrap();
    println!("Successful login");

    println!("-------------- POST /login/v1 with incorrect credentials ----------");
    api_service
      .login_v1(&LoginUserV1 {
        name: user_name.clone(),
        password: "EinPasswortMit8ZeichenUndFehler".to_string(),
      })
      .await
      .unwrap_err();
    println!("Successfully failed login");

    println!("-------------- POST /itemList ----------------------");
    let id = api_service
      .push_new_item_list(List {
        id: 0,
        name: "Vierfünf".to_owned(),
        shop: None,
        image_id: None,
        items: vec![],
      })
      .await
      .unwrap();
    println!("id returned: {}", id);

    println!("------------------ GET /itemList/{{}}/flat --------------");
    let list = api_service.get_flat_items_list(id).await.unwrap();
    println!("Number of items in list: {}", list.items.len());

    println!("--------------- POST /item/attached -----------------");
    println!(
      "{}",
      match api_service
        .push_item_attached(StoreItemAttached {
          item: Item {
            id: 0,
            name: "Test".to_owned(),
            checked: false,
            article_id: None,
            alternative_article_ids: None,
            amount: Some(1),
            unit: Some(Unit::KiloGram)
          },
          list_id: id,
        })
        .await
      {
        Ok(_) => "Success!",
        Err(_) => "Failure!",
      }
    );

    println!("------------------ GET /itemList/{{}}/flat --------------");
    let list = api_service.get_flat_items_list(id).await.unwrap();
    println!(
      "Number of items in list: {}, with total size: {}",
      list.items.len(),
      std::mem::size_of_val(&list)
    );

    println!("------------------ POST /shop ----------------");
    let shop_id = api_service
      .store_shop(Shop {
        id: 0,
        name: "LIDL".to_owned(),
        image_id: None,
      })
      .await
      .unwrap();
    println!("New id: {:?}", shop_id);

    println!("------------------ GET /shop/{{}} --------------");
    let shop = api_service.get_shop(shop_id).await.unwrap();
    println!("{}", shop.name);
  });
}

#[derive(Debug)]
pub enum TransmissionError {
  SerializationError,
  NetworkError(reqwest::Error),
  InvalidResponseError(String),
  FailedRequest(StatusCode),
  Unknown(String),
}

impl Display for TransmissionError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let error_message = match self {
      TransmissionError::SerializationError => "An Error occured during client-side serialization".to_owned(),
      TransmissionError::NetworkError(e) => format!("A network Error occured during transmission: {}", e),
      TransmissionError::InvalidResponseError(e) => format!("An invalid response was returned from the server: {}", e),
      TransmissionError::FailedRequest(status) => "The request was not successful: {status}".to_owned(),
      TransmissionError::Unknown(e) => format!("An unknown error occured: {e}"),
    };

    write!(f, "{}", error_message)
  }
}

impl From<reqwest::Error> for TransmissionError {
  fn from(e: reqwest::Error) -> Self {
    if e.is_status() {
      TransmissionError::InvalidResponseError(e.to_string())
    } else if e.is_request() {
      TransmissionError::FailedRequest(e.status().unwrap())
    } else if e.is_body() || e.is_decode() {
      TransmissionError::SerializationError
    } else {
      TransmissionError::Unknown(e.to_string())
    }
  }
}

impl<C, D> From<CheckDeserializeError<C, D>> for TransmissionError {
  fn from(_: CheckDeserializeError<C, D>) -> Self {
    TransmissionError::SerializationError
  }
}
