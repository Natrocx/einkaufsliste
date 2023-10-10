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
use actix_session::config::{CookieContentSecurity, PersistentSession};
use actix_session::SessionMiddleware;
use actix_web::cookie::SameSite;
use actix_web::HttpServer;
use api::item::{get_item_list_flat, store_item_attached, store_item_list};
use api::shop::{get_shop, store_shop};
use api::user::{get_users_lists, login_v1, register_v1};
use db::DbState;
use mimalloc::MiMalloc;
use rand::Rng;
use tracing_subscriber::filter::{LevelFilter, Targets};

use crate::util::session_store::SledSessionStore;

// Use a reasonable global allocator to avoid performance problems due to rkyv serialization allocations
#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
  setup_tracing();

  let db = sled::open("./data.sled")?;
  let session_store = SledSessionStore {
    session_db: db.open_tree("sessions")?,
  };
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
    let app = { app.service(crate::util::serve_frontend::serve_frontend) };

    app
      .wrap(cors)
      .wrap(tracing_actix_web::TracingLogger::default())
      .wrap(identity_mw)
      .wrap(
        SessionMiddleware::builder(session_store.clone(), cookie_priv_key.clone())
          .session_lifecycle(PersistentSession::default())
          .cookie_content_security(CookieContentSecurity::Private)
          .cookie_same_site(SameSite::Strict)
          .cookie_path("/".into())
          .cookie_domain(None)
          .cookie_secure(true)
          .cookie_http_only(true)
          .build(),
      )
  })
  .bind_rustls("127.0.0.1:8443", config.tls_config)?
  .run()
  .await
}

pub fn setup_tracing() {
  use tracing_subscriber::prelude::*;

  let filter_layer = Targets::new()
    .with_target("h2", LevelFilter::OFF)
    .with_target("actix_identity", LevelFilter::ERROR)
    .with_target("sled", LevelFilter::WARN)
    .with_default(LevelFilter::DEBUG);

  let fmt_layer = tracing_subscriber::fmt::layer().pretty();

  tracing_subscriber::registry()
    .with(filter_layer)
    .with(fmt_layer)
    .init();
}
