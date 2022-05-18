use std::error::Error;
use std::fmt::Display;
use std::process::exit;

use bytes::Buf;
use einkaufsliste::model::article::Article;
use einkaufsliste::model::item::Item;
use einkaufsliste::model::list::{FlatItemsList, List};
use einkaufsliste::model::requests::StoreItemAttached;
use einkaufsliste::model::shop::Shop;
use reqwest::StatusCode;
use rkyv::AlignedVec;

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
  println!("------------- GET /article/test ---------------");
  let article = get_example_article().await?;
  println!("{}", article.description.unwrap());

  println!("-------------- POST /itemList ----------------------");
  let id = push_new_item_list(List {
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
  let list = get_flat_items_list(id).await.unwrap();
  println!("Number of items in list: {}", list.items.len());

  println!("--------------- POST /item/attached -----------------");
  println!(
    "{}",
    match push_item_attached(StoreItemAttached {
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
  let list = get_flat_items_list(id).await.unwrap();
  println!("Number of items in list: {}", list.items.len());

  println!("------------------ POST /shop ----------------");
  let shop_id = store_shop(Shop {
    id,
    name: "LIDL".to_owned(),
    image_id: None,
  })
  .await
  .unwrap();
  println!("New id: {:?}", shop_id);

  println!("------------------ GET /shop/{{}} --------------");
  let shop = get_shop(shop_id).await.unwrap();
  println!("{}", shop.name);

  println!("All tests successful.");
  Ok(())
}

pub(crate) async fn store_shop(shop: Shop) -> Result<u64, TransmissionError> {
  const base_url: &str = "http://127.0.0.1:8080";
  const uri: &str = "/shop";

  let mut url = String::from(base_url);
  url.push_str(uri);
  let bytes = rkyv::to_bytes::<_, 128>(&shop).map_err(|_| TransmissionError::SerializationError)?;
  let client = reqwest::Client::new();
  let response = client
    .post(url)
    .body::<Vec<u8>>(bytes.into())
    .send()
    .await
    .map_err(TransmissionError::NetworkError)?;

  let mut new_id_bytes = response
    .bytes()
    .await
    .map_err(|e| TransmissionError::InvalidResponseError(e.into()))?;

  // i hate this api...
  if new_id_bytes.len() < 8 {
    Err(TransmissionError::InvalidResponseError("Answer was too short.".into()))
  } else {
    Ok(new_id_bytes.get_u64())
  }
}

pub(crate) async fn get_shop(id: u64) -> Result<Shop, TransmissionError> {
  let response = reqwest::get(format!("http://127.0.0.1:8080/shop/{}", id))
    .await
    .map_err(TransmissionError::NetworkError)?;

  let response_bytes = match response.status() {
    StatusCode::OK => response.bytes().await.map_err(|_| TransmissionError::FailedRequest)?,
    _ => return Err(TransmissionError::FailedRequest),
  };

  // the alignment is apparently lost along the way so we need to reallocate + realign (by copying)
  let mut buffer = AlignedVec::with_capacity(response_bytes.len() - (response_bytes.len() % 64) + 64);
  buffer.extend_from_slice(&response_bytes);

  let shop = rkyv::from_bytes::<Shop>(&buffer).map_err(|e| TransmissionError::InvalidResponseError(e.into()))?;

  Ok(shop)
}

pub(crate) async fn push_new_item_list(list: List) -> Result<u64, TransmissionError> {
  const base_url: &str = "http://127.0.0.1:8080";
  const uri: &str = "/itemList";

  let mut url = String::from(base_url);
  url.push_str(uri);
  let bytes = rkyv::to_bytes::<_, 1024>(&list).map_err(|_| TransmissionError::SerializationError)?;
  let client = reqwest::Client::new();
  let response = client
    .post(url)
    .body::<Vec<u8>>(bytes.into())
    .send()
    .await
    .map_err(TransmissionError::NetworkError)?;

  let mut new_id_bytes = response
    .bytes()
    .await
    .map_err(|e| TransmissionError::InvalidResponseError(e.into()))?;

  // i hate this api...
  if new_id_bytes.len() < 8 {
    Err(TransmissionError::InvalidResponseError("Answer was too short.".into()))
  } else {
    Ok(new_id_bytes.get_u64_le()) //FIXME: Endianness
  }
}

pub(crate) async fn get_flat_items_list(id: u64) -> Result<FlatItemsList, TransmissionError> {
  let response = reqwest::get(format!("http://127.0.0.1:8080/itemList/{}/flat", id))
    .await
    .map_err(TransmissionError::NetworkError)?;

  let response_bytes = match response.status() {
    StatusCode::OK => response.bytes().await.map_err(|_| TransmissionError::FailedRequest)?,
    _ => return Err(TransmissionError::FailedRequest),
  };

  // the alignment is apparently lost along the way so we need to reallocate + realign (by copying)
  let mut buffer = AlignedVec::with_capacity(response_bytes.len() - (response_bytes.len() % 64) + 64);
  buffer.extend_from_slice(&response_bytes);

  let item_list =
    rkyv::from_bytes::<FlatItemsList>(&buffer).map_err(|e| TransmissionError::InvalidResponseError(e.into()))?;

  Ok(item_list)
}

#[cfg(debug_assertions)]
#[allow(dead_code)]
pub(crate) async fn get_example_article() -> Result<Article, reqwest::Error> {
  let response = reqwest::get("http://127.0.0.1:8080/article/test")
    .await?
    .bytes()
    .await?;

  // the alignment is apparently lost along the way so we need to reallocate + realign
  let mut s = AlignedVec::with_capacity(response.len() - (response.len() % 64) + 64);
  s.extend_from_slice(&response);

  let value = match rkyv::from_bytes::<Article>(&s) {
    Ok(val) => val,
    Err(e) => {
      println!("Failure building received data: {}", e);
      exit(0);
    }
  };

  Ok(value)
}

async fn push_item_attached(command: StoreItemAttached) -> Result<(), TransmissionError> {
  const base_url: &str = "http://127.0.0.1:8080";
  const uri: &str = "/item/attached";

  let mut url = String::from(base_url);
  url.push_str(uri);
  let bytes = rkyv::to_bytes::<_, 128>(&command).map_err(|_| TransmissionError::SerializationError)?;
  let client = reqwest::Client::new();
  let response = client
    .post(url)
    .body::<Vec<u8>>(bytes.into())
    .send()
    .await
    .map_err(TransmissionError::NetworkError)?;

  match response.status() {
    StatusCode::CREATED => Ok(()),
    _ => Err(TransmissionError::FailedRequest),
  }
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
