mod api;

use actix_web::web::{self};
use actix_web::{middleware, HttpServer};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
  env_logger::init();

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
      .wrap(middleware::Logger::default())
  })
  .bind(("127.0.0.1", 8080))?
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
