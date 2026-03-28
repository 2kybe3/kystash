/*
 * kystash - A simple image/file sharing server
 * Copyright (C) 2026 2kybe3 <kybe@kybe.xyz>
 */

use tracing::debug;

use crate::{
    Cli,
    server::{commands::ServerCommands, config::ServerConfig},
};

pub mod commands;
pub mod config;
mod webserver;

pub async fn handle(_cli: &Cli, command: &ServerCommands) {
    match command {
        ServerCommands::Launch => run().await,
        ServerCommands::GenerateServerConfig { stdout } => {
            config::generate_server_cfg(*stdout).await
        }
        _ => todo!(),
    };
}

struct WebserverState {
    pub cfg: ServerConfig,
}

async fn run() {
    let cfg = config::get_server_cfg().await;
    debug!("server cfg loaded: {cfg:?}");

    webserver::start(cfg).await;
}
