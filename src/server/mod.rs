/*
 * kystash - A simple image/file sharing server
 * Copyright (C) 2026 2kybe3 <kybe@kybe.xyz>
 */

use actix_web::{App, HttpResponse, HttpServer, Responder, get, http::header};
use tracing::{debug, info, error};

use crate::{Cli, server::commands::ServerCommands};

pub mod commands;
pub mod config;

pub async fn handle(_cli: &Cli, command: &ServerCommands) {
    match command {
        ServerCommands::Launch => run().await,
        ServerCommands::GenerateServerConfig => config::generate_server_cfg().await,
        _ => todo!(),
    };
}

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::PermanentRedirect()
        .insert_header((header::LOCATION, "https://kybe.xyx/"))
        .finish()
}

async fn run() {
    let cfg = config::get_server_cfg().await;
    debug!("server cfg loaded: {cfg:?}");

    debug!("starting web server");
    let server = match HttpServer::new(|| {
        App::new()
            .service(hello)
    })
    .bind(cfg.get_bind()) {
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

