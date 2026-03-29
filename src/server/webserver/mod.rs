/*
 * kystash - A simple image/file sharing server/client
 * Copyright (C) 2026 2kybe3 <kybe@kybe.xyz>
 */

use std::sync::Arc;

use actix_web::{App, HttpMessage, HttpResponse, HttpServer, Responder, get, http::header, web};
use tracing::{error, info};

use crate::{
    config::server::ServerConfig,
    server::{
        WebserverState,
        webserver::middleware::auth::{Auth, AuthClient},
    },
};

mod middleware;

#[get("/")]
async fn hello(data: web::Data<WebserverState>) -> impl Responder {
    HttpResponse::PermanentRedirect()
        .insert_header((header::LOCATION, data.cfg.webserver_root_redirect()))
        .finish()
}

#[get("/authorized")]
async fn authorized(req: actix_web::HttpRequest) -> impl Responder {
    let user = match req.extensions().get::<AuthClient>().cloned() {
        Some(v) => v,
        None => return HttpResponse::InternalServerError().finish(),
    };

    info!("{} authorized with settings {:?}", user.name, user.settings);
    HttpResponse::Ok().finish()
}

pub async fn start(cfg: ServerConfig) {
    let cfg = Arc::new(cfg);

    let value = Arc::clone(&cfg);
    let auth = Auth::new(Arc::clone(&value));
    let server = match HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(WebserverState {
                cfg: Arc::clone(&value),
            }))
            .service(hello)
            .wrap(auth.clone())
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
