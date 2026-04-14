/*
 * kystash - A simple image/file sharing server/client
 * Copyright (C) 2026 2kybe3 <kybe@kybe.xyz>
 */

use actix_web::{App, HttpServer, web};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{error, info};

use crate::{
    config::server::ServerConfig,
    server::{
        WebserverState,
        chunk_map::ChunkMap,
        webserver::{
            middleware::auth::Auth,
            routes::{metadata, root, upload, version},
        },
    },
    utils,
};

mod middleware;
mod routes;

pub async fn start(cfg: ServerConfig) {
    let cfg = Arc::new(cfg);
    let cfg_clone = Arc::clone(&cfg);
    let auth = Auth::new(Arc::clone(&cfg_clone));
    let chunk_map = Arc::new(Mutex::new(ChunkMap::default()));

    let server = match HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(WebserverState {
                cfg: Arc::clone(&cfg_clone),
                chunk_map: Arc::clone(&chunk_map),
            }))
            .service(root::root)
            .service(upload::chunk::chunk)
            .service(upload::status::status)
            .service(metadata::get_metadata)
            .wrap(auth.clone())
            .service(version::version)
    })
    .bind(cfg.get_bind())
    {
        Ok(v) => v,
        Err(e) => {
            error!("failed to start web server");
            error!("{e}");
            utils::error::fatal_error();
        }
    };

    match server.run().await {
        Ok(_) => {
            info!("server stopped")
        }
        Err(e) => {
            error!("{e}");
            utils::error::fatal_error();
        }
    };
}
