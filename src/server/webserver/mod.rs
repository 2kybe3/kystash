/*
 * kystash - A simple image/file sharing server/client
 * Copyright (C) 2026 2kybe3 <kybe@kybe.xyz>
 */

use actix_web::{App, HttpResponse, HttpServer, Responder, get, http::header, web};
use tracing::{error, info};

use crate::server::{WebserverState, config::ServerConfig};

#[get("/")]
async fn hello(data: web::Data<WebserverState>) -> impl Responder {
    HttpResponse::PermanentRedirect()
        .insert_header((header::LOCATION, data.cfg.webserver_root_redirect()))
        .finish()
}

pub async fn start(cfg: ServerConfig) {
    let value = cfg.clone();
    let server = match HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(WebserverState { cfg: value.clone() }))
            .service(hello)
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
