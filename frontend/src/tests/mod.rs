#[cfg(test)]
mod api_tests {

  #[test]
  fn test_requests() {
    use einkaufsliste::model::item::{Item, Unit};
    use einkaufsliste::model::list::List;
    use einkaufsliste::model::requests::{LoginUserV1, RegisterUserV1, StoreItemAttached};
    use einkaufsliste::model::shop::Shop;
    use rand::distributions::Alphanumeric;
    use rand::Rng;

    use crate::service::api::APIService;

    let rt = tokio::runtime::Runtime::new().unwrap();
    let api_service = APIService::insecure().unwrap();

    rt.block_on(async {
      let rng = rand::thread_rng();
      let user_name = format!(
        "test_user_{}",
        rng
          .sample_iter(Alphanumeric)
          .take(15)
          .map(|num| num as char)
          .collect::<String>()
      );

      println!("-------------- unauthenticated POST /itemList -------------");
      api_service
        .push_new_item_list(List {
          id: 0,
          name: "vier".to_owned(),
          shop: None,
          image_id: None,
          items: vec![],
        })
        .await
        .unwrap_err();
      println!("Failed successfully!");

      println!("-------------- POST /register/v1 -------------------");
      let user_id = api_service
        .register_v1(&RegisterUserV1 {
          name: user_name.clone(),
          password: "EinPasswortMit8Zeichen".to_owned(),
        })
        .await
        .unwrap();
      println!("New user id: {user_id}");

      println!("-------------- POST /login/v1 ----------------------");
      api_service
        .login_v1(&LoginUserV1 {
          name: user_name.clone(),
          password: "EinPasswortMit8Zeichen".to_string(),
        })
        .await
        .unwrap();
      println!("Successful login");

      println!("-------------- POST /login/v1 with incorrect credentials ----------");
      api_service
        .login_v1(&LoginUserV1 {
          name: user_name.clone(),
          password: "EinPasswortMit8ZeichenUndFehler".to_string(),
        })
        .await
        .unwrap_err();
      println!("Successfully failed login");

      println!("-------------- POST /itemList ----------------------");
      let id = api_service
        .push_new_item_list(List {
          id: 0,
          name: "Vierfünf".to_owned(),
          shop: None,
          image_id: None,
          items: vec![],
        })
        .await
        .unwrap();
      println!("id returned: {}", id);

      println!("------------------ GET /itemList/{{}}/flat --------------");
      let list = api_service.get_flat_items_list(id).await.unwrap();
      println!("Number of items in list: {}", list.items.len());

      println!("--------------- POST /item/attached -----------------");
      let item_id = api_service
        .push_item_attached(StoreItemAttached {
          item: Item {
            id: 0,
            name: "Test".to_owned(),
            checked: false,
            article_id: None,
            alternative_article_ids: None,
            amount: Some(1),
            unit: Some(Unit::KiloGram),
          },
          list_id: id,
        })
        .await
        .unwrap();
      println!("Success");

      println!("------------------ GET /item/{item_id} --------------");
      assert!(api_service.get_item(item_id).await.unwrap().id == item_id);
      println!("Success");

      println!("------------------ GET /itemList/{{}}/flat --------------");
      let list = api_service.get_flat_items_list(id).await.unwrap();
      println!(
        "Number of items in list: {}, with total size: {}",
        list.items.len(),
        std::mem::size_of_val(&list)
      );

      println!("------------------ GET /user/lists --------------");
      assert!(api_service.get_users_lists().await.unwrap().list.len() == 1);
      println!("Success");

      println!("------------------ POST /shop ----------------");
      let shop_id = api_service
        .store_shop(Shop {
          id: 0,
          name: "LIDL".to_owned(),
          image_id: None,
        })
        .await
        .unwrap();
      println!("New id: {:?}", shop_id);

      println!("------------------ GET /shop/{{}} --------------");
      let shop = api_service.get_shop(shop_id).await.unwrap();
      println!("{}", shop.name);
    });
  }
}
