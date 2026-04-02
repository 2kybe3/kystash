/*
 * kystash - A simple image/file sharing server/client
 * Copyright (C) 2026 2kybe3 <kybe@kybe.xyz>
 */

use std::{collections::HashMap, path::PathBuf, sync::Arc};

use tokio::sync::Mutex;
use tracing::debug;

use crate::{
    config::{self, server::ServerConfig},
    server::commands::ServerCommands,
};

pub mod commands;
mod webserver;

pub async fn handle(command: &ServerCommands, server_config: Option<PathBuf>) {
    match command {
        ServerCommands::Launch => run(server_config).await,
        ServerCommands::Edit => config::server::edit(server_config).await,
        ServerCommands::GenerateServerConfig { stdout } => {
            config::server::generate_server_cfg(*stdout, server_config).await
        }
        ServerCommands::GenerateClientConfig { name, overwrite } => {
            config::client::generate_client_cfg(name, *overwrite, server_config).await
        }
    };
}

type ChunkMap = HashMap<(String, String), bitvec::vec::BitVec>;

struct WebserverState {
    pub cfg: Arc<config::server::ServerConfig>,
    pub chunk_map: Arc<Mutex<ChunkMap>>,
}

async fn run(server_config: Option<PathBuf>) {
    let path = server_config.unwrap_or(ServerConfig::default_path().await);
    let cfg = config::server::get_server_cfg(path).await;
    debug!("server cfg loaded: {cfg:?}");

    webserver::start(cfg).await;
}
