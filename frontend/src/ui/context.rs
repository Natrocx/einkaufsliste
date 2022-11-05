use std::rc::Rc;

use crate::service::api::APIService;

#[derive(Clone)]
pub struct APIContext {
  pub service: Rc<APIService>,
}

impl PartialEq for APIContext {
  fn eq(&self, other: &Self) -> bool {
    Rc::ptr_eq(&self.service, &other.service)
  }
}
