use std::{
    collections::HashMap,
    future::{ready, Ready},
    sync::Arc,
};

use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    web::Query,
    Error, HttpMessage,
};
use futures::executor::block_on;
use futures_util::future::LocalBoxFuture;

use crate::db::DbManager;

// There are two steps in middleware processing.
// 1. Middleware initialization, middleware factory gets called with
//    next service in chain as parameter.
// 2. Middleware's call method gets called with normal request.
pub(crate) struct Auth;

// Middleware factory is `Transform` trait
// `S` - type of the next service
// `B` - type of response's body
impl<S, B> Transform<S, ServiceRequest> for Auth
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = AuthMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthMiddleware { service }))
    }
}

pub struct AuthMiddleware<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for AuthMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let mut api_token = String::new();
        if let Some(token) = req.headers().get("authorization") {
            api_token = token
                .to_str()
                .unwrap()
                .split(' ')
                .last()
                .unwrap()
                .to_string();
        }

        if api_token.is_empty() {
            if let Some(token) = Query::<HashMap<String, String>>::from_query(req.query_string())
                .unwrap()
                .get("_token")
            {
                api_token = token.clone();
            }
        }

        if !api_token.is_empty() {
            if let Some(db_manager) = req.app_data::<Arc<DbManager>>() {
                if let Some(client) =
                    block_on(db_manager.client_repo().find_by_api_token(&api_token))
                {
                    req.request().extensions_mut().insert(client);
                }
            }
        }

        // TODO: Learn how to return the response from here
        // return Box::pin(async move {
        //     Ok(req.into_response(HttpResponse::Unauthorized().finish().map_into_boxed_body()))
        // });

        let fut = self.service.call(req);
        Box::pin(async move {
            let res = fut.await?;

            Ok(res)
        })
    }
}
