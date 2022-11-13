use std::rc::Rc;

use yew::Callback;

use super::AppMessage;
use crate::service::api::APIService;

#[derive(Clone)]
pub struct APIContext {
  pub service: Rc<APIService>,
  pub app_callback: Callback<AppMessage>,
}
//TODO: consider splitting into multiple contexts?

impl PartialEq for APIContext {
  fn eq(&self, other: &Self) -> bool {
    Rc::ptr_eq(&self.service, &other.service)
  }
}
