/*
 * kystash - A simple image/file sharing server/client
 * Copyright (C) 2026 2kybe3 <kybe@kybe.xyz>
 */

use std::path::Path;

use tokio::process::Command;

pub async fn open(path: impl AsRef<Path>) {
    let env = std::env::var("EDITOR").ok().unwrap_or("vim".to_string());
    Command::new(env)
        .arg(path.as_ref().as_os_str())
        .status()
        .await
        .ok();
}
