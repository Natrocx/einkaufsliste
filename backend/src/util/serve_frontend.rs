use std::str::FromStr;

use actix_web::http::header::ContentType;
use mime::Mime;

/**
 * WARNING THIS IS INSECURE
 * only use for development
 */
#[actix_web::get("/dev/{filename:.*}")]
#[cfg(feature = "serve_frontend")]
async fn serve_frontend(
  filename: actix_web::web::Path<String>,
) -> Result<actix_files::NamedFile, crate::response::ResponseError> {
  use super::errors::not_found;
  use crate::util::errors::bad_request;

  let path = filename.parse::<std::path::PathBuf>()?;
  let extension = path.extension();

  let extension = FileType::from(
    extension
      .ok_or_else(|| bad_request(()))?
      .to_str()
      .ok_or_else(|| bad_request(()))?,
  );

  let path: std::path::PathBuf = format!("./web_root/{}", path.to_string_lossy()).parse()?;
  tracing::debug!("Serving static file: {}", path.to_string_lossy());

  Ok(
    actix_files::NamedFile::open_async(&path)
      .await
      .map_err(not_found)?
      .set_content_type(extension.into())
      .use_last_modified(true),
  )
}

enum FileType {
  Html,
  Css,
  Js,
  Wasm,
  Svg,
  Other,
}

impl From<FileType> for Mime {
  fn from(ft: FileType) -> Mime {
    match ft {
      FileType::Html => mime::TEXT_HTML_UTF_8,
      FileType::Css => mime::TEXT_CSS_UTF_8,
      FileType::Js => mime::APPLICATION_JAVASCRIPT_UTF_8,
      FileType::Wasm => Mime::from_str("application/wasm").unwrap(),
      FileType::Svg => Mime::from_str("image/svg+xml").unwrap(),
      FileType::Other => mime::TEXT_PLAIN_UTF_8,
    }
  }
}

impl From<FileType> for ContentType {
  fn from(ft: FileType) -> Self {
    ContentType(ft.into())
  }
}

impl From<&str> for FileType {
  fn from(param: &str) -> Self {
    match param {
      "html" => FileType::Html,
      "css" => FileType::Css,
      "js" => FileType::Js,
      "wasm" => FileType::Wasm,
      "svg" => FileType::Svg,
      _ => FileType::Other,
    }
  }
}
