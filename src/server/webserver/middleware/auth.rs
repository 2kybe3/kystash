/*
 * kystash - A simple image/file sharing server/client
 * Copyright (C) 2026 2kybe3 <kybe@kybe.xyz>
 */

use actix_web::{
    Error, HttpMessage, HttpResponse,
    body::BoxBody,
    dev::{Service, ServiceRequest, ServiceResponse, Transform, forward_ready},
};
use base64::{Engine, engine::general_purpose};
use futures_util::future::LocalBoxFuture;
use std::{
    future::{Ready, ready},
    sync::Arc,
};

use crate::config::{self, server::ClientSettings};

#[derive(Clone)]
pub struct Auth {
    cfg: Arc<config::server::ServerConfig>,
}

impl Auth {
    pub fn new(cfg: Arc<config::server::ServerConfig>) -> Self {
        Self { cfg }
    }
}

impl<S> Transform<S, ServiceRequest> for Auth
where
    S: Service<ServiceRequest, Response = ServiceResponse<BoxBody>, Error = Error>,
    S::Future: 'static,
{
    type Response = ServiceResponse;
    type Error = Error;
    type InitError = ();
    type Transform = AuthMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthMiddleware {
            service,
            cfg: Arc::clone(&self.cfg),
        }))
    }
}

pub struct AuthMiddleware<S> {
    service: S,
    cfg: Arc<config::server::ServerConfig>,
}

impl<S> Service<ServiceRequest> for AuthMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<BoxBody>, Error = Error>,
    S::Future: 'static,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let header = req
            .headers()
            .get("authorization")
            .map(|v| v.to_str().ok().unwrap_or_default().to_owned())
            .unwrap_or_default();
        let res = auth(header, &self.cfg);

        match res {
            Some(client) => {
                req.extensions_mut().insert(AuthClient {
                    name: client.0,
                    settings: client.1,
                });

                Box::pin(self.service.call(req))
            }
            None => Box::pin(async move {
                let response = HttpResponse::Unauthorized();
                Ok(req.into_response(response).map_into_boxed_body())
            }),
        }
    }
}

#[derive(Clone, Debug)]
pub struct AuthClient {
    pub name: String,
    pub settings: ClientSettings,
}

fn auth(
    header: String,
    cfg: &Arc<config::server::ServerConfig>,
) -> Option<(String, ClientSettings)> {
    let encoded = header.strip_prefix("Bearer ")?;

    let decoded_bytes = general_purpose::STANDARD.decode(encoded).ok()?;

    let decoded_str = String::from_utf8(decoded_bytes).ok()?;

    let (client_id, settings) = cfg.get_client_with_token(&decoded_str)?;

    Some((client_id.to_owned(), settings.to_owned()))
}
