use actix_identity::Identity;
use actix_web::error::{ErrorBadRequest, ErrorInternalServerError};
use actix_web::web::Payload;
use actix_web::{get, post, put, web, Result};
use einkaufsliste::model::article::Article;
use einkaufsliste::model::user::User;
use einkaufsliste::model::{AccessControlList, Identifiable};
use zerocopy::AsBytes;

use crate::api::{new_generic_acl, preprocess_payload, store_in_db};
use crate::response::{Response, ResponseError};
use crate::{DbState, SessionState};

#[get("/article/{id}")]
async fn get_article_by_id(
  id: actix_web::web::Path<String>,
  state: web::Data<DbState>,
  sessions: web::Data<SessionState>,
  identity: Identity,
) -> Response {
  let article_id = id.as_ref().parse::<u64>().map_err(|_| ResponseError::ErrorBadRequest)?;

  // check if the user has access:
  check_article_acl(article_id, &state, &sessions, identity)?;

  state.article_db.get(article_id.as_bytes()).into()
}

#[put("/article")]
async fn update_article(
  body: Payload,
  data: web::Data<DbState>,
  sessions: web::Data<SessionState>,
  identity: Identity,
) -> Response {
  let bytes = preprocess_payload::<384>(body).await?;

  let article = rkyv::from_bytes::<Article>(&bytes).map_err(|_| ResponseError::ErrorBadRequest)?;

  // before submitting parsed article to db we check the permissions:
  check_article_acl(article.id, &data, &sessions, identity)?;

  store_in_db::<Article, 384>(article.id, article, &data.article_db)?;

  Response::empty()
}

#[post("/article")]
async fn store_article(
  body: actix_web::web::Payload,
  data: web::Data<DbState>,
  sessions: web::Data<SessionState>,
  identity: Identity,
) -> Response {
  // the only security check done here is login, since the id is generated and no data can be overwritten/read
  sessions.confirm_user_login(&identity)?;

  let bytes = preprocess_payload::<256>(body).await?;

  let user_id = sessions.get_id_for_session(identity.id().map_err(|_| ResponseError::ErrorUnauthorized)?)?;

  let mut archived = rkyv::from_bytes::<Article>(&bytes).map_err(|_| ResponseError::ErrorBadRequest)?;
  archived.id = data.db.generate_id()?;
  let db = &data.article_db;

  db.insert::<&[u8], &[u8]>(archived.id.as_bytes(), rkyv::to_bytes::<_, 384>(&archived)?.as_slice())?;

  // since this is a new object we need to create an acl for this
  new_generic_acl::<Article, User>(archived.id, user_id, &data.article_db)?;

  Response::empty()
}

fn check_article_acl(
  article_id: <Article as Identifiable>::Id,
  state: &DbState,
  sessions: &SessionState,
  identity: Identity,
) -> Result<(), ResponseError> {
  let user_id = match identity.id() {
    Ok(session_id) => sessions.get_id_for_session(session_id)?,
    Err(_) => return Err(ResponseError::ErrorUnauthenticated),
  };

  let acl = rkyv::from_bytes::<AccessControlList<Article, User>>(
    state
      .acl_db
      .get(article_id.as_bytes())
      .map_err(ErrorInternalServerError)?
      .ok_or_else(|| ErrorBadRequest("No acl for this object."))?
      .as_bytes(),
  )
  .map_err(ErrorInternalServerError)?;

  if acl.owner == user_id || acl.allowed_user_ids.contains(&user_id) {
    Ok(())
  } else {
    Err(ResponseError::ErrorUnauthorized)
  }
}

/*
fn new_article_acl(
  article_id: <Article as Identifiable>::Id,
  user_id: <User as Identifiable>::Id,
  db: &Tree,
) -> Result<Option<sled::IVec>, Error> {
  let new_acl = AccessControlList::<Article, User> {
    object_id: article_id,
    allowed_user_ids: vec![user_id],
  };
  db.insert(
    article_id.as_bytes(),
    rkyv::to_bytes::<_, 256>(&new_acl)
      .map_err(ErrorInternalServerError)?
      .to_vec(),
  )
  .map_err(ErrorInternalServerError)
}
*/
/*
fn check_generic_acl<'a, Object, User>(
  article_id: &'a <Object as Identifiable>::Id,
  state: &DbState,
  sessions: &SessionState,
  identity: Identity,
) -> Result<(), Error>
where
  <<User as einkaufsliste::model::Identifiable>::Id as rkyv::Archive>::Archived: 'a
    + rkyv::Deserialize<<User as einkaufsliste::model::Identifiable>::Id, rkyv::de::deserializers::SharedDeserializeMap>,
  <<User as einkaufsliste::model::Identifiable>::Id as rkyv::Archive>::Archived:
    bytecheck::CheckBytes<rkyv::validation::validators::DefaultValidator<'a>>,
  <<Object as einkaufsliste::model::Identifiable>::Id as rkyv::Archive>::Archived:
    'a + bytecheck::CheckBytes<rkyv::validation::validators::DefaultValidator<'a>>,
  <<Object as einkaufsliste::model::Identifiable>::Id as rkyv::Archive>::Archived: 'a
    + rkyv::Deserialize<<Object as einkaufsliste::model::Identifiable>::Id, rkyv::de::deserializers::SharedDeserializeMap>,
  Object: Identifiable + 'a,
  User: Identifiable + 'a,
{
  let user_id = match identity.identity() {
    Some(session_id) => {
      let user_id_bytes = sessions
        .get_id_for_session(session_id)
        .map_err(ErrorInternalServerError)?
        .ok_or_else(|| ErrorUnauthorized(""))?;

      rkyv::from_bytes::<<User as Identifiable>::Id>(&user_id_bytes).map_err(ErrorInternalServerError)?
    }
    None => return Err(ErrorUnauthorized("")),
  };
  let acl = rkyv::from_bytes::<AccessControlList<Object, User>>(
    state
      .acl_db
      .get(article_id.as_bytes())
      .map_err(ErrorInternalServerError)?
      .ok_or_else(|| ErrorBadRequest("No acl for this object."))?
      .as_bytes(),
  )
  .map_err(ErrorInternalServerError)?;

  if acl.allowed_user_ids.contains(&user_id) {
    Ok(())
  } else {
    Err(ErrorForbidden(""))
  }
}
*/
