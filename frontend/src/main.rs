#![feature(slice_pattern)]

use core::slice::SlicePattern;
use std::fmt::Error;
use std::process::exit;

use bytes::Bytes;
use einkaufsliste::model::Article;
use rkyv::{AlignedVec, Infallible};

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
  let response = reqwest::get("http://127.0.0.1:8080/article/test")
    .await?
    .bytes()
    .await?;
  
  // the alignment is apparently lost along the way so we need to reallocate + realign
  let mut s = AlignedVec::with_capacity(response.len() - (response.len() % 64) + 64);
  s.extend_from_slice(&response);

  let value = match rkyv::from_bytes::<Article>(&s) {
    Ok(val) => val,
    Err(e) => {
      println!("Failure building received data: {}", e);
      exit(0);
    }
  };
  println!("Received data: {:?}", value);

  Ok(())
}

