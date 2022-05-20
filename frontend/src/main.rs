pub mod service;

use std::error::Error;
use std::fmt::Display;
use std::path::Path;

use einkaufsliste::model::{item::Item, requests::StoreItemAttached, shop::Shop, list::List};

use crate::service::api::APIService;

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
  let api_service = APIService::new("https://localhost:8443", Path::new("./cert.pem")).unwrap();

  println!("-------------- POST /itemList ----------------------");
  let id = api_service
    .push_new_item_list(List {
      id: 0,
      name: "Vierfünf".to_owned(),
      shop: 0,
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
          article_id: None,
          alternative_article_ids: None,
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
  println!("Number of items in list: {}", list.items.len());

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

  println!("All tests successful.");
  Ok(())
}

#[derive(Debug)]
enum TransmissionError {
  SerializationError,
  NetworkError(reqwest::Error),
  InvalidResponseError(Box<dyn Error>),
  FailedRequest,
}

impl Display for TransmissionError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let error_message = match self {
      TransmissionError::SerializationError => "An Error occured during client-side serialization".to_owned(),
      TransmissionError::NetworkError(e) => format!("A network Error occured during transmission: {}", e),
      TransmissionError::InvalidResponseError(e) => format!("An invalid response was returned from the server: {}", e),
      TransmissionError::FailedRequest => "The request was not successfull (non 200-return)".to_owned(),
    };

    write!(f, "{}", error_message)
  }
}
