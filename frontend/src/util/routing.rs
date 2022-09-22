use einkaufsliste::model::list::List;
use einkaufsliste::model::Identifiable;
use einkaufsliste_codegen::Routable;

#[derive(Clone, Routable, PartialEq)]
#[cfg(feature = "dev_router")]
#[route_prefix("/dev")]
pub enum Page {
  #[at("/index.html")]
  Overview,
  #[at("/list/:id/:name")]
  List {
    id: <List as Identifiable>::Id,
    name: String,
  },
  #[at("/settings")]
  Settings,
  #[not_found]
  #[at("/404")]
  NotFound,
}

#[derive(Clone, Routable, PartialEq)]
#[cfg(not(feature = "dev_router"))]
pub enum Page {
  #[at("/")]
  Overview,
  #[at("/list/:id/:name")]
  List {
    id: <List as Identifiable>::Id,
    name: String,
  },
  #[at("/settings")]
  Settings,
  #[not_found]
  #[at("/404")]
  NotFound,
}
