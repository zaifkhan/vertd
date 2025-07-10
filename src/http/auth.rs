use std::{
    collections::HashMap,
    future::{ready, Ready},
    rc::Rc,
};

use actix_web::{
    body::{BoxBody, EitherBody},
    dev::{self, Service, ServiceRequest, ServiceResponse, Transform},
    web, Error, HttpResponse,
};
use futures_util::future::LocalBoxFuture;
use log::warn;

pub struct Authentication;

impl<S, B> Transform<S, ServiceRequest> for Authentication
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B, BoxBody>>;
    type Error = Error;
    type InitError = ();
    type Transform = AuthMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthMiddleware {
            service: Rc::new(service),
        }))
    }
}

pub struct AuthMiddleware<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for AuthMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B, BoxBody>>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    dev::forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let svc = self.service.clone();

        Box::pin(async move {
            let token_from_env = match req.app_data::<web::Data<String>>() {
                Some(token) => token.get_ref().clone(),
                None => {
                    warn!("Auth token not found in app configuration. Denying request.");
                    let (request, _pl) = req.into_parts();
                    let response = HttpResponse::InternalServerError()
                        .finish()
                        .map_into_right_body();
                    return Ok(ServiceResponse::new(request, response));
                }
            };

            let mut token_from_req: Option<String> = None;

            if let Some(auth_header) = req.headers().get("Authorization") {
                if let Ok(auth_str) = auth_header.to_str() {
                    if let Some(token) = auth_str.strip_prefix("Bearer ") {
                        token_from_req = Some(token.to_string());
                    }
                }
            }

            if token_from_req.is_none() {
                if let Ok(query) = serde_qs::from_str::<HashMap<String, String>>(req.query_string()) {
                    if let Some(token) = query.get("authToken") {
                        token_from_req = Some(token.clone());
                    }
                }
            }

            if let Some(token) = token_from_req {
                if token == token_from_env {
                    return svc.call(req).await.map(|res| res.map_into_left_body());
                }
            }

            warn!(
                "Unauthorized access attempt denied for path: {}",
                req.path()
            );
            let (request, _pl) = req.into_parts();
            let response = HttpResponse::Unauthorized().finish().map_into_right_body();
            Ok(ServiceResponse::new(request, response))
        })
    }
}