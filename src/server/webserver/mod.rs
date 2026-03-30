/*
 * kystash - A simple image/file sharing server/client
 * Copyright (C) 2026 2kybe3 <kybe@kybe.xyz>
 */

use std::sync::Arc;

use actix_web::{App, HttpServer, web};
use tracing::{error, info};

use crate::{
    config::server::ServerConfig,
    server::{
        WebserverState,
        webserver::{
            middleware::auth::Auth,
            routes::{authorized, root, upload},
        },
    },
};

mod middleware;
mod routes;

pub async fn start(cfg: ServerConfig) {
    let cfg = Arc::new(cfg);

    let value = Arc::clone(&cfg);
    let auth = Auth::new(Arc::clone(&value));
    let server = match HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(WebserverState {
                cfg: Arc::clone(&value),
            }))
            .service(root::root)
            .service(upload::chunk::chunk)
            .wrap(auth.clone())
            .service(authorized::authorized)
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
