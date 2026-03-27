use std::path::PathBuf;

use tokio::fs;

pub enum ConfigType {
    Server,
    Client,
}

/// Also creates it
async fn get_root_config_path() -> PathBuf {
    let mut path = dirs_next::config_dir()
        .unwrap_or_else(|| std::env::current_dir().unwrap());
    path.push("kystash");
    fs::create_dir_all(&path).await.ok();
    path
}

pub async fn get_config_path(config_type: ConfigType) -> PathBuf {
    let mut path = get_root_config_path().await;
    path.push(match config_type {
        ConfigType::Server => "server.toml",
        ConfigType::Client => "client.toml",
    });
    path
}
