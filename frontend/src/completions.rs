use std::collections::BTreeMap;

use dioxus_signals::Signal;
use einkaufsliste::model::item::Item;

pub fn complete(items: &Signal<Vec<Signal<Item>>>, beginning: &str) -> Vec<Signal<Item>> {
  if beginning.is_empty() {
    return vec![];
  }
  items
    .read()
    .iter()
    .filter(|item| item.read().checked)
    .filter_map(|item| {
      if item.read().name.starts_with(beginning) {
        Some(*item)
      } else {
        None
      }
    })
    .collect()
}
