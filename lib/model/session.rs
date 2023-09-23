use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

use rkyv::{Archive, Deserialize, Serialize};

use super::Identifiable;

pub type SessionState = HashMap<String, String>;

#[derive(Archive, Serialize, Deserialize, serde::Serialize, serde::Deserialize)]
#[archive_attr(derive(bytecheck::CheckBytes, Debug))]
pub struct Session {
  pub time_to_logout: i64,
  pub state: SessionState,
}

impl Session {
  pub fn is_valid(&self) -> bool {
    let now = Self::get_current_time();

    self.time_to_logout > now
  }
  pub fn get_current_time() -> i64 {
    let now = SystemTime::now();
    now.duration_since(UNIX_EPOCH).unwrap().as_secs() as i64
  }

  pub fn refresh(&mut self, ttl: i64) {
    self.time_to_logout = Self::get_current_time() + ttl;
  }
}

impl Identifiable for Session {
  type Id = u64;
}
