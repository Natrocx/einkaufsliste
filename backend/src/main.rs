// nightly for some MUCH cleaner code
#![feature(generic_arg_infer)]
#![feature(associated_type_bounds)]
#![feature(try_trait_v2)]

mod api;
pub mod db;
pub mod response;
mod util;

use std::time::Duration;

use actix_identity::IdentityMiddleware;
use actix_session::storage::CookieSessionStore;
use actix_session::SessionMiddleware;
use actix_web::web::{self};
use actix_web::HttpServer;
use api::item::{get_item_list_flat, store_item_attached, store_item_list};
use api::shop::{get_shop, store_shop};
use api::user::{get_users_lists, login_v1, register_v1};
use db::DbState;
use mimalloc::MiMalloc;
use rand::Rng;

// Use a reasonable global allocator to avoid performance problems due to rkyv serialization allocations
#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
  env_logger::init();

  let config = util::config::load_config().unwrap();

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

  HttpServer::new(move || {
    let cors_config = actix_cors::Cors::permissive();
    let identity_mw = IdentityMiddleware::builder()
      .visit_deadline(Some(Duration::from_secs(config.cookie_timeout)))
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
      .service(get_users_lists)
      .wrap(cors_config)
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
