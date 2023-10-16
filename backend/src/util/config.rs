use std::collections::HashMap;
use std::fmt::Display;
use std::io::BufReader;

use actix_cors::Cors;
use actix_web::http::header;
use config::Config;
use rustls::{Certificate, PrivateKey, ServerConfig};
use rustls_pemfile::pkcs8_private_keys;

#[derive(Clone)]
pub(crate) struct BackendConfig {
  pub tls_config: ServerConfig,
  pub cookie_timeout: u64,
  pub cors: Option<String>,
}

impl BackendConfig {
  pub fn extract_cors(&self) -> Cors {
    self
      .cors
      .clone()
      //TODO: validate security/correctness
      .map(|url| {
        actix_cors::Cors::default()
          .allowed_origin(&url)
          .supports_credentials()
          .allowed_methods(vec!["GET", "POST", "PUT"])
          .allowed_headers(vec![header::AUTHORIZATION, header::ACCEPT])
          .allowed_header(header::CONTENT_TYPE)
          .max_age(3600)
      })
      .unwrap_or_else(|| {
        tracing::warn!(
          "Could not find a frontend url in the backend configuration. Attempting to run in \
           restrictive Cors mode. This is likely to fail for non-development setups."
        );
        actix_cors::Cors::default()
      })
  }
}

pub(crate) fn load_config() -> Result<BackendConfig, LoadConfigError> {
  let home_dir = std::env::var("HOME").unwrap_or_else(|_| "~/".into());

  let user_settings = Config::builder();

  let home_dir_config_file_name = format!("{home_dir}/.config/einkaufsliste/backend.toml");
  let home_dir_config_file = std::path::Path::new(&home_dir_config_file_name);
  let local_config_file = std::path::Path::new("./backend.toml");
  let user_settings = if home_dir_config_file.exists() {
    user_settings.add_source(config::File::from(home_dir_config_file))
  }
  // if none of the files exist, the certificates cannot be loaded and the server cannot operate
  else if !local_config_file.exists() {
    return Err(LoadConfigError::MissingAllConfigFiles);
  } else {
    user_settings.add_source(config::File::from(local_config_file))
  };

  let user_settings = match user_settings.set_default("cookie_timeout", 60 * 60 * 24 * 30) {
    Ok(val) => val,
    Err(_) => return Err(LoadConfigError::ConfigCrateError),
  };

  let user_settings = user_settings
    .build()
    .unwrap()
    .try_deserialize::<HashMap<String, String>>()
    .expect("Could not load configuration files. Refusing to operate.");

  let cors = user_settings.get("frontend_url").cloned();

  let cert_path = user_settings
    .get("cert_path")
    .expect("Did not specify path to TLS certificate.");
  let key_path = user_settings
    .get("key_path")
    .expect("Did not specify path to TLS certificates private keys.");

  let server_config = load_rustls_config(
    std::path::Path::new(&cert_path),
    std::path::Path::new(&key_path),
  )?;

  Ok(BackendConfig {
    cors,
    tls_config: server_config,
    cookie_timeout: user_settings
      .get("cookie_timeout")
      .unwrap()
      .parse()
      .unwrap(),
  })
}

fn load_rustls_config(
  cert_path: &std::path::Path,
  key_path: &std::path::Path,
) -> Result<rustls::ServerConfig, LoadConfigError> {
  // init server config builder with safe defaults
  let config = ServerConfig::builder()
    .with_safe_defaults()
    .with_no_client_auth();

  // load TLS key/cert files
  let cert_file = &mut BufReader::new(
    std::fs::File::open(cert_path).map_err(LoadConfigError::ReadingParameterPaths)?,
  );
  let key_file = &mut BufReader::new(
    std::fs::File::open(key_path).map_err(LoadConfigError::ReadingParameterPaths)?,
  );

  // convert files to key/cert objects
  let cert_chain = rustls_pemfile::certs(cert_file)
    .map_err(LoadConfigError::BuildingChain)?
    .into_iter()
    .map(Certificate)
    .collect();
  let mut keys: Vec<PrivateKey> = pkcs8_private_keys(key_file)
    .map_err(LoadConfigError::BuildingChain)?
    .into_iter()
    .map(PrivateKey)
    .collect();

  // exit if no keys could be parsed
  if keys.is_empty() {
    eprintln!("Could not locate PKCS 8 private keys.");
    Err(LoadConfigError::MissingKeys)
  } else {
    Ok(config.with_single_cert(cert_chain, keys.remove(0)).unwrap())
  }
}

#[derive(Debug)]
pub enum LoadConfigError {
  ReadingParameterPaths(std::io::Error),
  BuildingChain(std::io::Error),
  MissingAllConfigFiles,
  MissingKeys,
  ConfigCrateError,
}

impl Display for LoadConfigError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let error_message = match self {
      LoadConfigError::ReadingParameterPaths(e) => {
        format!("Could not access files at the provided path: {e}")
      }
      LoadConfigError::BuildingChain(e) => {
        format!("An Error occurred while building Keychain: {e}")
      }
      LoadConfigError::MissingKeys => "Missing PEM files".to_owned(),
      LoadConfigError::MissingAllConfigFiles => "No config files were found in the standard \
                                                 paths. Cannot operate without information about \
                                                 certificate paths."
        .into(),
      LoadConfigError::ConfigCrateError => "An unexpected error occured while parsing your \
                                            configuration. Maybe you specified the wrong data \
                                            types?"
        .into(),
    };

    write!(f, "{error_message}")
  }
}
