mod api;
mod util;

use std::error::Error;
use std::fmt::Display;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use actix_web::web::{self};
use actix_web::{middleware, HttpServer};
use api::item::{get_item_list_flat, store_item_attached, store_item_list};
use api::shop::{get_shop, store_shop};
use rustls::{Certificate, PrivateKey, ServerConfig};
use rustls_pemfile::pkcs8_private_keys;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
  env_logger::init();

  let config = load_rustls_config(Path::new("./cert.pem"), Path::new("./key.pem")).unwrap();

  let db = sled::open("./sled")?;
  let state = DbState {
    article_db: db.open_tree("article")?,
    item_db: db.open_tree("item")?,
    shop_db: db.open_tree("shop")?,
    list_db: db.open_tree("list")?,
    db,
  };
  HttpServer::new(move || {
    actix_web::App::new()
      .app_data(web::Data::new(state.clone()))
      .service(crate::api::article::get_example_article)
      .service(crate::api::article::store_article)
      .service(crate::api::article::get_article_by_id)
      .service(crate::api::item::get_item_by_id)
      .service(get_item_list_flat)
      .service(store_item_list)
      .service(store_item_attached)
      .service(get_shop)
      .service(store_shop)
      .wrap(middleware::Logger::default())
  })
  .bind_rustls("127.0.0.1:8443", config)?
  .run()
  .await
}

#[derive(Clone)]
pub struct DbState {
  db: sled::Db,
  article_db: sled::Tree,
  item_db: sled::Tree,
  shop_db: sled::Tree,
  list_db: sled::Tree,
}

fn load_rustls_config(cert_path: &Path, key_path: &Path) -> Result<rustls::ServerConfig, TlsInitError> {
  // init server config builder with safe defaults
  let config = ServerConfig::builder().with_safe_defaults().with_no_client_auth();

  // load TLS key/cert files
  let cert_file = &mut BufReader::new(File::open(cert_path).map_err(|e| TlsInitError::ReadingParameterPaths(e))?);
  let key_file = &mut BufReader::new(File::open(key_path).map_err(|e| TlsInitError::ReadingParameterPaths(e))?);

  // convert files to key/cert objects
  let cert_chain = rustls_pemfile::certs(cert_file)
    .map_err(|e| TlsInitError::BuildingChain(e))?
    .into_iter()
    .map(Certificate)
    .collect();
  let mut keys: Vec<PrivateKey> = pkcs8_private_keys(key_file)
    .map_err(|e| TlsInitError::BuildingChain(e))?
    .into_iter()
    .map(PrivateKey)
    .collect();

  // exit if no keys could be parsed
  if keys.is_empty() {
    eprintln!("Could not locate PKCS 8 private keys.");
    Err(TlsInitError::MissingKeys)
  } else {
    Ok(config.with_single_cert(cert_chain, keys.remove(0)).unwrap())
  }
}

#[derive(Debug)]
enum TlsInitError {
  ReadingParameterPaths(std::io::Error),
  BuildingChain(std::io::Error),
  MissingKeys,
}

impl Display for TlsInitError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let error_message = match self {
      Self::ReadingParameterPaths(e) => format!("Could not access files at the provided path: {}", e),
      TlsInitError::BuildingChain(e) => format!("An Error occurred while building Keychain: {}", e),
      TlsInitError::MissingKeys => "Missing PEM files".to_owned(),
    };

    write!(f, "{}", error_message)
  }
}
