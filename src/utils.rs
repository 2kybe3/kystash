/*
 * kystash - A simple image/file sharing server/client
 * Copyright (C) 2026 2kybe3 <kybe@kybe.xyz>
 */

use std::path::{Path, PathBuf};
use tokio::fs;
use tracing::error;

pub async fn create_dir_file_parent(path: impl AsRef<Path>) {
    fs::create_dir_all(get_dir_file_parent(path))
        .await
        .unwrap_or_else(|e| {
            error!("{e}");
            crate::error::fatal_error();
        });
}

pub fn get_dir_file_parent(path: impl AsRef<Path>) -> PathBuf {
    let mut dir = path.as_ref().to_path_buf();
    dir.pop();
    dir
}
