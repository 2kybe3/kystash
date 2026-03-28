/*
 * kystash - A simple image/file sharing server/client
 * Copyright (C) 2026 2kybe3 <kybe@kybe.xyz>
 */

use tracing::debug;

use crate::{Cli, config, server::commands::ServerCommands};

pub mod commands;
mod webserver;

pub async fn handle(_cli: &Cli, command: &ServerCommands) {
    match command {
        ServerCommands::Launch => run().await,
        ServerCommands::GenerateServerConfig { stdout } => {
            config::server::generate_server_cfg(*stdout).await
        }
        ServerCommands::GenerateClientConfig { name } => {
            config::client::generate_client_cfg(name).await
        }
    };
}

struct WebserverState {
    pub cfg: config::server::ServerConfig,
}

async fn run() {
    let cfg = config::server::get_server_cfg().await;
    debug!("server cfg loaded: {cfg:?}");

    webserver::start(cfg).await;
}
