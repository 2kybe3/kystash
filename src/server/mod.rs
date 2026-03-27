/*
 * kystash - A simple image/file sharing server
 * Copyright (C) 2026 2kybe3 <kybe@kybe.xyz>
 */

use actix_web::{App, HttpResponse, HttpServer, Responder, get, http::header, web};
use tracing::{debug, error, info};

use crate::{
    Cli,
    server::{commands::ServerCommands, config::ServerConfig},
};

pub mod commands;
pub mod config;

pub async fn handle(_cli: &Cli, command: &ServerCommands) {
    match command {
        ServerCommands::Launch => run().await,
        ServerCommands::GenerateServerConfig => config::generate_server_cfg().await,
        _ => todo!(),
    };
}

struct WebserverState {
    cfg: ServerConfig,
}

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::PermanentRedirect()
        .insert_header((header::LOCATION, "https://kybe.xyz/"))
        .finish()
}

async fn run() {
    let cfg = config::get_server_cfg().await;
    debug!("server cfg loaded: {cfg:?}");

    debug!("starting web server");
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
