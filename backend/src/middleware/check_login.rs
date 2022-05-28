use actix_web::body::EitherBody;
use actix_web::dev::{self, Service, ServiceRequest, ServiceResponse, Transform};
use futures::future::{ready, LocalBoxFuture, Ready};

// TODO: can this efficiently solve our problems? skipping for now...
pub struct CheckLogin {
  pub session_db: sled::Tree,
}

impl<GenericService, Body> Transform<GenericService, ServiceRequest> for CheckLogin
where
  GenericService: Service<ServiceRequest, Response = ServiceResponse<Body>, Error = actix_web::Error>,
  GenericService::Future: 'static,
  Body: 'static,
{
  type Response = ServiceResponse<EitherBody<Body>>;
  type Error = actix_web::Error;
  type InitError = ();
  type Transform = CheckLoginMiddleware<GenericService>;
  type Future = Ready<Result<Self::Transform, Self::InitError>>;

  fn new_transform(&self, service: GenericService) -> Self::Future {
    ready(Ok(CheckLoginMiddleware {
      service,
      session_db: self.session_db.clone(),
    }))
  }
}

pub struct CheckLoginMiddleware<Service> {
  service: Service,
  session_db: sled::Tree,
}

impl<GenericService, Body> Service<ServiceRequest> for CheckLoginMiddleware<GenericService>
where
  GenericService: Service<ServiceRequest, Response = ServiceResponse<Body>, Error = actix_web::Error>,
  GenericService::Future: 'static,
  Body: 'static,
{
  type Response = ServiceResponse<EitherBody<Body>>;

  type Error = actix_web::Error;

  type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

  fn call(&self, req: ServiceRequest) -> Self::Future {
    todo!()
  }

  dev::forward_ready!(service);
}
