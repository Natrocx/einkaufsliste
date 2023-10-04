use core::slice::SlicePattern;

use bytes::Bytes;
use reqwest::cookie::CookieStore;
use reqwest::header::HeaderValue;

/**
 * A slightly hazardous implementation of a CookieStore that will panic the application if supplied with invalid data.
 */
pub struct SledCookieStore {
  db: sled::Db,
  cookie_tree: sled::Tree,
}

impl CookieStore for SledCookieStore {
  fn set_cookies(&self, cookie_headers: &mut dyn Iterator<Item = &reqwest::header::HeaderValue>, url: &url::Url) {
    let key = url.as_str();
    let cookie_bytes_iter: Vec<Vec<u8>> = cookie_headers
      .map(|cookie_header| cookie_header.as_bytes().to_owned())
      .collect();
    let value = rkyv::to_bytes::<_, 4096>(&cookie_bytes_iter).unwrap();

    self.cookie_tree.insert(key, value.as_slice()).unwrap();
  }

  fn cookies(&self, url: &url::Url) -> Option<reqwest::header::HeaderValue> {
    let key = url.as_str();

    let bytes = self.cookie_tree.get(key).unwrap();

    bytes.map(|bytes| {
      let raw_cookies = rkyv::from_bytes_unchecked::<Vec<Vec<u8>>>(bytes.as_ref()).unwrap();
      let parsed_cookies = raw_cookies
        .into_iter()
        .map(|cookie_bytes| HeaderValue::from_bytes(&cookie_bytes.as_slice()).unwrap())
        .collect::<Vec<reqwest::header::HeaderValue>>();
    })
  }
}
