/*
 * kystash - A simple image/file sharing server/client
 * Copyright (C) 2026 2kybe3 <kybe@kybe.xyz>
 */

use std::path::PathBuf;

use tokio::fs;

pub async fn get_root_config_path() -> PathBuf {
    let mut path = dirs_next::config_dir().unwrap_or_else(|| std::env::current_dir().unwrap());
    path.push("kystash");
    fs::create_dir_all(&path).await.ok();
    path
}
