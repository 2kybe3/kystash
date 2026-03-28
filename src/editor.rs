/*
 * kystash - A simple image/file sharing server/client
 * Copyright (C) 2026 2kybe3 <kybe@kybe.xyz>
 */

use std::path::PathBuf;

use tokio::process::Command;

pub async fn open(path: PathBuf) {
    let env = std::env::var("EDITOR").ok().unwrap_or("vim".to_string());
    Command::new(env)
        .arg(path.into_os_string())
        .status()
        .await
        .ok();
}
