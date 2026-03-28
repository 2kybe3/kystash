/*
 * kystash - A simple image/file sharing server/client
 * Copyright (C) 2026 2kybe3 <kybe@kybe.xyz>
 */

use actix_web::{App, HttpResponse, HttpServer, Responder, get, http::header, web};
use tracing::{error, info};

use crate::{config::server::ServerConfig, server::WebserverState};

#[get("/")]
async fn hello(data: web::Data<WebserverState>) -> impl Responder {
    HttpResponse::PermanentRedirect()
        .insert_header((header::LOCATION, data.cfg.webserver_root_redirect()))
        .finish()
}

#[get("/authorized")]
async fn authorized(
    req: actix_web::HttpRequest,
    data: web::Data<WebserverState>,
) -> impl Responder {
    use base64::{Engine as _, engine::general_purpose};

    let header_str = match req
        .headers()
        .get("authorization")
        .and_then(|v| v.to_str().ok())
    {
        Some(v) => v,
        None => return HttpResponse::Unauthorized().finish(),
    };

    let encoded = match header_str.strip_prefix("Bearer ") {
        Some(v) => v,
        None => return HttpResponse::Unauthorized().finish(),
    };

    let decoded_bytes = match general_purpose::STANDARD.decode(encoded) {
        Ok(v) => v,
        Err(_) => return HttpResponse::Unauthorized().finish(),
    };

    let decoded_str = match String::from_utf8(decoded_bytes) {
        Ok(v) => v,
        Err(_) => return HttpResponse::Unauthorized().finish(),
    };

    let mut parts = decoded_str.splitn(2, ':');
    let id = match parts.next() {
        Some(v) => v,
        None => return HttpResponse::Unauthorized().finish(),
    };

    let token = {
        let v = match parts.next() {
            Some(v) => v,
            None => return HttpResponse::Unauthorized().finish(),
        };
        let v = match general_purpose::STANDARD.decode(v) {
            Ok(v) => v,
            Err(_) => return HttpResponse::Unauthorized().finish(),
        };
        match String::from_utf8(v) {
            Ok(v) => v,
            Err(_) => return HttpResponse::Unauthorized().finish(),
        }
    };

    let client = match data.cfg.get_client_with_token(&token, id) {
        Some(v) => v,
        None => return HttpResponse::Unauthorized().finish(),
    };

    info!("{client:?}");
    HttpResponse::Ok().finish()
}

pub async fn start(cfg: ServerConfig) {
    let value = cfg.clone();
    let server = match HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(WebserverState { cfg: value.clone() }))
            .service(hello)
            .service(authorized)
    })
    .bind(cfg.get_bind())
    {
        Ok(v) => v,
        Err(e) => {
            error!("failed to start web server");
            error!("{e}");
            crate::error::fatal_error();
        }
    };

    match server.run().await {
        Ok(_) => {
            info!("server stopped")
        }
        Err(e) => {
            error!("{e}");
            crate::error::fatal_error();
        }
    };
}
