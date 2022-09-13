// nightly for some MUCH cleaner code
#![feature(generic_arg_infer)]
#![feature(associated_type_bounds)]
#![feature(try_trait_v2)]

mod api;
mod consts;
pub mod db;
pub mod response;
mod util;

use std::fmt::Display;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use std::time::Duration;

use actix_identity::IdentityMiddleware;
use actix_session::storage::CookieSessionStore;
use actix_session::SessionMiddleware;

use actix_web::web::{self};
use actix_web::HttpServer;

use api::item::{get_item_list_flat, store_item_attached, store_item_list};
use api::shop::{get_shop, store_shop};
use api::user::{login_v1, register_v1};
use db::DbState;




use mimalloc::MiMalloc;
use rand::{Rng};



use rustls::{Certificate, PrivateKey, ServerConfig};
use rustls_pemfile::pkcs8_private_keys;



// Use a reasonable global allocator to avoid performance problems due to rkyv serialization allocations
#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
  env_logger::init();

  let config = load_rustls_config(Path::new("./cert.pem"), Path::new("./key.pem")).unwrap();

  let db = sled::open("./data.sled")?;
  let application_state = DbState {
    article_db: db.open_tree("article")?,
    item_db: db.open_tree("item")?,
    shop_db: db.open_tree("shop")?,
    list_db: db.open_tree("list")?,
    acl_db: db.open_tree("list_acl")?,
    user_db: db.open_tree("user")?,
    login_db: db.open_tree("login")?,
    object_list_db: db.open_tree("ol")?,
    db,
    //rng: Arc::new(Mutex::new(rand_xoshiro::Xoshiro256PlusPlus::from_entropy())),
  };

  let mut key = [0u8; 64];
  rand::thread_rng().fill(&mut key);

  let cookie_priv_key = actix_web::cookie::Key::from(&key);

  HttpServer::new(move || {
    let cors_config = actix_cors::Cors::permissive();
    let identity_mw = IdentityMiddleware::builder()
      .visit_deadline(Some(Duration::from_secs(60 * 60 * 24 * 30)))
      .logout_behaviour(actix_identity::config::LogoutBehaviour::PurgeSession)
      .build();

    actix_web::App::new()
      .app_data(web::Data::new(application_state.clone()))
      .service(crate::api::article::store_article)
      .service(crate::api::article::get_article_by_id)
      .service(crate::api::item::get_item_by_id)
      .service(get_item_list_flat)
      .service(store_item_list)
      .service(store_item_attached)
      .service(get_shop)
      .service(store_shop)
      .service(register_v1)
      .service(login_v1)
      .wrap(cors_config)
      .wrap(actix_web::middleware::Logger::default())
      .wrap(identity_mw)
      .wrap(SessionMiddleware::new(
        CookieSessionStore::default(),
        cookie_priv_key.clone(),
      ))
  })
  .bind_rustls("127.0.0.1:8443", config)?
  .run()
  .await
}

fn load_rustls_config(cert_path: &Path, key_path: &Path) -> Result<rustls::ServerConfig, TlsInitError> {
  // init server config builder with safe defaults
  let config = ServerConfig::builder().with_safe_defaults().with_no_client_auth();

  // load TLS key/cert files
  let cert_file = &mut BufReader::new(File::open(cert_path).map_err(TlsInitError::ReadingParameterPaths)?);
  let key_file = &mut BufReader::new(File::open(key_path).map_err(TlsInitError::ReadingParameterPaths)?);

  // convert files to key/cert objects
  let cert_chain = rustls_pemfile::certs(cert_file)
    .map_err(TlsInitError::BuildingChain)?
    .into_iter()
    .map(Certificate)
    .collect();
  let mut keys: Vec<PrivateKey> = pkcs8_private_keys(key_file)
    .map_err(TlsInitError::BuildingChain)?
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
