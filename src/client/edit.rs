/*
 * kystash - A simple image/file sharing server/client
 * Copyright (C) 2026 2kybe3 <kybe@kybe.xyz>
 */

use std::path::PathBuf;

use crate::{config::client::ClientConfig, editor};

pub async fn edit(client_config: Option<PathBuf>) {
    editor::open(client_config.unwrap_or(ClientConfig::default_path().await)).await;
}
