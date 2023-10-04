use std::collections::HashMap;

use actix_session::storage::{LoadError, SaveError, SessionKey, SessionStore, UpdateError};
use actix_web::cookie::time::Duration;
use anyhow::{anyhow, Error};
use einkaufsliste::model::session::{Session, SessionState};
use rand::distributions::Alphanumeric;
use rand::rngs::OsRng;
use rand::Rng as _;

#[derive(Clone)]
pub struct SledSessionStore {
  pub(crate) session_db: sled::Tree,
}

#[async_trait::async_trait(?Send)]
impl SessionStore for SledSessionStore {
  async fn load(
    &self,
    session_key: &SessionKey,
  ) -> Result<Option<HashMap<String, String>>, LoadError> {
    // first see if a session is saved
    match self.session_db.get(session_key.as_ref()) {
      // if so, parse it
      Ok(Some(session_data)) => unsafe {
        let session = rkyv::from_bytes_unchecked::<Session>(&session_data)
          .map_err(|e| LoadError::Deserialization(e.into()))?;

        if session.is_valid() {
          Ok(Some(session.state))
        } else {
          // clear the invalid session here to avoid unnecessary buildup
          // TODO: implement periodic scanning for invalid sessions
          // also: ignore result, as this is not required to succeed in order to service the users request
          let _ = self.session_db.remove(session_key.as_ref());
          Ok(None)
        }
      },
      Ok(None) => Ok(None),
      Err(e) => Err(LoadError::Other(e.into())),
    }
  }

  async fn save(
    &self,
    session_state: SessionState,
    ttl: &Duration,
  ) -> Result<SessionKey, SaveError> {
    let session_key = generate_session_key();
    let state = Session {
      time_to_logout: Session::get_current_time() + ttl.whole_seconds(),
      state: session_state,
    };

    let bytes =
      rkyv::to_bytes::<_, 4096>(&state).map_err(|err| SaveError::Serialization(err.into()))?;

    self
      .session_db
      .insert(session_key.as_ref(), &*bytes)
      .map_err(|err| SaveError::Other(err.into()))?;

    Ok(session_key)
  }

  async fn update(
    &self,
    session_key: SessionKey,
    session_state: SessionState,
    ttl: &Duration,
  ) -> Result<SessionKey, UpdateError> {
    let current_state = self
      .session_db
      .get(session_key.as_ref())
      .map_err(|e| UpdateError::Other(e.into()))?;
    let mut state = match current_state {
      None => return Err(UpdateError::Other(anyhow!("No such session"))),
      Some(bytes) => unsafe {
        rkyv::from_bytes_unchecked::<Session>(&bytes)
          .map_err(|e| UpdateError::Serialization(e.into()))?
      },
    };
    state.state = session_state;
    state.refresh(ttl.whole_seconds());

    self
      .session_db
      .insert(
        session_key.as_ref(),
        &*rkyv::to_bytes::<_, 4096>(&state).map_err(|e| UpdateError::Serialization(e.into()))?,
      )
      .map_err(|e| UpdateError::Other(e.into()))?;

    Ok(session_key)
  }

  async fn update_ttl(&self, session_key: &SessionKey, ttl: &Duration) -> Result<(), Error> {
    let current_state = self
      .session_db
      .get(session_key.as_ref())
      .map_err(|e| UpdateError::Other(e.into()))?;
    let mut state = match current_state {
      None => return Err(anyhow!("No such session")),
      Some(bytes) => unsafe {
        rkyv::from_bytes_unchecked::<Session>(&bytes)
          .map_err(|e| UpdateError::Serialization(e.into()))?
      },
    };
    state.refresh(ttl.whole_seconds());

    self
      .session_db
      .insert(
        session_key.as_ref(),
        &*rkyv::to_bytes::<_, 4096>(&state).map_err(|e| UpdateError::Serialization(e.into()))?,
      )
      .map_err(|e| UpdateError::Other(e.into()))?;

    Ok(())
  }

  async fn delete(&self, session_key: &SessionKey) -> Result<(), Error> {
    match self.session_db.remove(session_key.as_ref()) {
      Ok(_) => Ok(()),
      Err(e) => Err(e.into()),
    }
  }
}

/// sample 256 bit of data from alphanumeric distribution
fn generate_session_key() -> SessionKey {
  let value = std::iter::repeat(())
    .map(|()| OsRng.sample(Alphanumeric))
    .take(64)
    .collect::<Vec<_>>();

  // These unwraps will never panic because pre-conditions are always verified
  // (i.e. length and character set)
  String::from_utf8(value).unwrap().try_into().unwrap()
}
