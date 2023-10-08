use std::process::Command;

use einkaufsliste::model::requests::LoginUserV1;
use frontend::service::api::ApiClient;

#[tokio::main]
async fn main() {
  //TODO: save logs
  // let backend = Command::new("cargo")
  //   .args(["run", "--bin", "backend"])
  //   .spawn()
  //   .expect("failed to start backend");

  let client = ApiClient::new("https://localhost:8443".to_string()).unwrap();
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
