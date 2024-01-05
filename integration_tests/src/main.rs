use std::path::PathBuf;
use std::process::Command;
use std::rc::Rc;
use std::time::Instant;

use einkaufsliste::model::list::List;
use einkaufsliste::model::requests::LoginUserV1;
use einkaufsliste::Encoding;
use frontend::service::api::{ApiClient, ClientConfig};
use futures::future::join_all;
use futures::stream::FuturesUnordered;
use futures::StreamExt;
use tokio::task::{spawn_local, LocalSet};

#[tokio::main]
async fn main() {
  //TODO: save logs
  // let backend = Command::new("cargo")
  //   .args(["run", "--bin", "backend"])
  //   .spawn()
  //   .expect("failed to start backend");
  //frontend::setup_tracing();

  let client = Rc::new(
    ApiClient::new_with_config(
      "https://localhost:8443".to_string(),
      ClientConfig {
        encoding: einkaufsliste::Encoding::Rkyv,
        cookie_store_base_path: PathBuf::from("./"),
      },
    )
    .unwrap(),
  );
  println!("Unauthenticated lists: {:?}", client.fetch_all_lists().await);

  println!(
    "Login with rkyv: {:?}",
    client
      .login(LoginUserV1 {
        name: "vier".to_string(),
        password: "viervier".to_string(),
      })
      .await
      .expect("Login with rkyv to be successful")
  );

  // run 100 fetch_all_lists requests concurrently
  let mut tasks = (0..100).map(|_| {
    let client = client.clone();
    async move {
      client
        .fetch_all_lists()
        .await
        .expect("fetch_all_lists to be successful")
    }
  });

  join_all(tasks).await;
  println!("Successfully ran 100 fetch_all_lists requests concurrently");

  println!(
    "Users lists with rkyv: {:?}",
    client
      .fetch_all_lists()
      .await
      .expect("Fetch lists with rkyv to be successful")
  );

  client.set_encoding(einkaufsliste::Encoding::JSON);

  println!(
    "Login with json: {:?}",
    client
      .login(LoginUserV1 {
        name: "vier".into(),
        password: "viervier".into(),
      })
      .await
      .expect("Login with json to be successful"),
  );

  client.set_encoding(einkaufsliste::Encoding::JSON);
  println!("Item Mass creation test with json...");
  let now = Instant::now();
  many_new_items(client.clone()).await;
  println!("Item Mass creation test with json took {:?}", now.elapsed());

  client.set_encoding(Encoding::Rkyv);
  println!("Item Mass creation test with rkyv...");
  let now = Instant::now();
  many_new_items(client.clone()).await;
  println!("Item Mass creation test with rkyv took {:?}", now.elapsed());
}

pub async fn many_new_items(client: Rc<ApiClient>) {
  let list_id = client
    .create_list(&List {
      id: 0,
      name: "Mass creation test".to_string(),
      shop: None,
      image_id: None,
      items: vec![],
    })
    .await
    .unwrap();

  for i in 0..100 {
    let mut tasks = (0..100).map(|j| {
      let client = client.clone();
      async move {
        client
          .new_item(
            list_id,
            einkaufsliste::model::item::Item {
              id: 0,
              name: format!("Item {}", i * 100 + j),
              checked: true,
              amount: None,
              unit: None,
              article_id: None,
              alternative_article_ids: None,
            },
          )
          .await
          .expect("create_item to be successful")
      }
    });

    join_all(tasks).await;
}
}
