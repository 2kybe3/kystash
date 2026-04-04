/*
 * kystash - A simple image/file sharing server/client
 * Copyright (C) 2026 2kybe3 <kybe@kybe.xyz>
 */

use std::{
    collections::HashMap,
    env,
    path::{Path, PathBuf},
    sync::Arc,
};

use tokio::{fs, sync::Mutex};
use tracing::{debug, error};

use crate::{
    config::{self, server::ServerConfig},
    editor,
    server::commands::ServerCommands,
    utils,
};

pub mod commands;
mod webserver;

pub async fn handle(command: &ServerCommands, server_config_path: Option<PathBuf>) {
    let path = server_config_path
        .unwrap_or(ServerConfig::default_path().await)
        .canonicalize()
        .unwrap_or_else(|e| {
            error!("{e}");
            crate::error::fatal_error();
        });
    let dir = utils::get_dir_file_parent(&path);
    fs::create_dir_all(&dir).await.unwrap_or_else(|e| {
        error!("{e}");
        crate::error::fatal_error();
    });
    if let Err(e) = env::set_current_dir(&dir) {
        error!("failed to set current cwd");
        error!("{e}");
        crate::error::fatal_error();
    };

    match command {
        ServerCommands::Launch => run(path).await,
        ServerCommands::Edit => editor::open(path).await,
        ServerCommands::GenerateServerConfig { stdout } => {
            config::server::generate_server_cfg(*stdout, path).await
        }
        ServerCommands::GenerateClientConfig { name, overwrite } => {
            config::client::generate_client_cfg(name, *overwrite, path).await
        }
    };
}

type ChunkMap = HashMap<(String, String), bitvec::vec::BitVec>;

struct WebserverState {
    pub cfg: Arc<config::server::ServerConfig>,
    pub chunk_map: Arc<Mutex<ChunkMap>>,
}

async fn run(path: impl AsRef<Path>) {
    let cfg = config::server::get_server_cfg(path).await;
    debug!("server cfg loaded: {cfg:?}");

    webserver::start(cfg).await;
}
