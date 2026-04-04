/*
 * kystash - A simple image/file sharing server/client
 * Copyright (C) 2026 2kybe3 <kybe@kybe.xyz>
 */

use std::path::Path;
use tokio::process::Command;
use tracing::warn;

use crate::utils::create_dir_file_parent;

const DEFAULT_EDITOR: &str = "vim";

pub async fn open(path: impl AsRef<Path>) {
    create_dir_file_parent(&path).await;

    let editor = std::env::var("EDITOR").ok().unwrap_or_else(|| {
        warn!("EDITOR env not set defaulting to {DEFAULT_EDITOR}");
        DEFAULT_EDITOR.to_string()
    });
    Command::new(editor)
        .arg(path.as_ref().as_os_str())
        .status()
        .await
        .ok();
}
