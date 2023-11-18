use std::process::Command;
use std::rc::Rc;

use einkaufsliste::model::requests::LoginUserV1;
use frontend::service::api::ApiClient;
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
  frontend::setup_tracing();

  let client = Rc::new(ApiClient::new("https://localhost:8443".to_string()).unwrap());
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
}
