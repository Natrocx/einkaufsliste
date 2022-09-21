// nightly for some MUCH cleaner code
#![feature(generic_arg_infer)]
#![feature(associated_type_bounds)]
#![feature(try_trait_v2)]

mod api;
pub mod db;
pub mod response;
mod util;

use std::path::PathBuf;
use std::str::FromStr;
use std::time::Duration;

use actix_identity::IdentityMiddleware;
use actix_session::storage::CookieSessionStore;
use actix_session::SessionMiddleware;
use actix_web::http::header::ContentType;
use actix_web::{get, HttpServer, Responder};
use api::item::{get_item_list_flat, store_item_attached, store_item_list};
use api::shop::{get_shop, store_shop};
use api::user::{get_users_lists, login_v1, register_v1};
use db::DbState;
use mimalloc::MiMalloc;
use mime::Mime;
use rand::Rng;

use crate::response::ResponseError;

// Use a reasonable global allocator to avoid performance problems due to rkyv serialization allocations
#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
  env_logger::init();

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
  };

  let mut key = [0u8; 64];
  rand::thread_rng().fill(&mut key);

  let cookie_priv_key = actix_web::cookie::Key::from(&key);

  let config = util::config::load_config().unwrap();
  let __config = config.clone();
  HttpServer::new(move || {
    let cors = __config.extract_cors();
    let identity_mw = IdentityMiddleware::builder()
      .visit_deadline(Some(Duration::from_secs(config.cookie_timeout)))
      .logout_behaviour(actix_identity::config::LogoutBehaviour::PurgeSession)
      .build();

    let app = actix_web::App::new()
      .app_data(actix_web::web::Data::new(application_state.clone()))
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
      .service(get_users_lists);

    #[cfg(feature = "serve_frontend")]
    let app = { app.service(serve_frontend) };

    app
      .wrap(cors)
      .wrap(actix_web::middleware::Logger::default())
      .wrap(identity_mw)
      .wrap(SessionMiddleware::new(
        CookieSessionStore::default(),
        cookie_priv_key.clone(),
      ))
  })
  .bind_rustls("127.0.0.1:8443", config.tls_config)?
  .run()
  .await
}

#[get("/dev/{filename}")]
#[cfg(feature = "serve_frontend")]
async fn serve_frontend(
  filename: actix_web::web::Path<String>,
) -> Result<actix_files::NamedFile, ResponseError> {
  use crate::util::errors::bad_request;

  let path = filename.parse::<PathBuf>().unwrap();
  let extension = path.extension();

  let extension = FileType::from(
    extension
      .ok_or_else(|| bad_request(()))?
      .to_str()
      .ok_or_else(|| bad_request(()))?,
  );

  let path: std::path::PathBuf =
    format!("./web_root/{}", path.file_name().unwrap().to_string_lossy())
      .parse()
      .unwrap();
  log::debug!("Serving static file: {}", path.to_string_lossy());

  Ok(
    actix_files::NamedFile::open(path)?
      .set_content_type(extension.into())
      .use_last_modified(true),
  )
}

enum FileType {
  Html,
  Css,
  Js,
  Wasm,
  Other,
}

impl From<FileType> for Mime {
  fn from(ft: FileType) -> Mime {
    match ft {
      FileType::Html => mime::TEXT_HTML_UTF_8,
      FileType::Css => mime::TEXT_CSS_UTF_8,
      FileType::Js => mime::APPLICATION_JAVASCRIPT_UTF_8,
      FileType::Wasm => Mime::from_str("application/wasm").unwrap(),
      FileType::Other => mime::TEXT_PLAIN_UTF_8,
    }
  }
}

impl From<FileType> for ContentType {
  fn from(ft: FileType) -> Self {
    ContentType(ft.into())
  }
}

impl From<&str> for FileType {
  fn from(param: &str) -> Self {
    match param {
      "html" => FileType::Html,
      "css" => FileType::Css,
      "js" => FileType::Js,
      "wasm" => FileType::Wasm,
      _ => FileType::Other,
    }
  }
}
