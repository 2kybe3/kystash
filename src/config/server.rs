/*
 * kystash - A simple image/file sharing server/client
 * Copyright (C) 2026 2kybe3 <kybe@kybe.xyz>
 */

use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    process::exit,
};

use crate::{config::shared::get_root_config_path, utils};
use serde::{Deserialize, Serialize};
use tokio::{
    fs::{self, File, OpenOptions},
    io::{AsyncReadExt, AsyncWriteExt},
};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

const CONFIG_WARN: &str = r"# THIS CONFIG FILE IS GENERATED.
# PLEASE DO NOT ADD COMMENTS.
# THEY WILL BE DELETED.
#
# DOCUMENTATION MIGHT BE ALREADY AVAILABLE AT https://git.kybe.xyz/2kybe3/kystash


";

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ServerConfig {
    ip: String,
    port: u16,
    upload_dir: String,
    webserver_root_redirect: String,
    /// Generate client cfg is gonna use this field to determite the url to set
    hostname: String,
    #[serde(default)]
    clients: HashMap<String, ClientSettings>,
}

impl ServerConfig {
    pub fn example() -> Self {
        Self {
            ip: "0.0.0.0".into(),
            port: 3000,
            upload_dir: "./upload".into(),
            webserver_root_redirect: "https://kybe.xyz/kystash".into(),
            hostname: "https://i.kybe.xyz".into(),
            clients: HashMap::new(),
        }
    }

    pub async fn get_upload_dir(&self) -> PathBuf {
        let dir: PathBuf = self.upload_dir.parse().unwrap_or_else(|e| {
            error!("{e}");
            utils::error::fatal_error();
        });
        fs::create_dir_all(&dir).await.unwrap_or_else(|e| {
            error!("{e}");
            utils::error::fatal_error();
        });
        dir.canonicalize().unwrap_or_else(|e| {
            error!("{e}");
            utils::error::fatal_error();
        })
    }

    pub async fn default_path() -> PathBuf {
        if let Ok(ow) = std::env::var("KYSTASH_SERVER_PATH") {
            debug!("KYSTASH_SERVER_PATH {ow}");
            return PathBuf::from(ow);
        }
        let mut path = get_root_config_path().await;
        path.push("server.toml");
        path
    }

    pub fn to_toml(&self) -> String {
        format!(
            "{}{}",
            CONFIG_WARN,
            match toml::to_string_pretty(self) {
                Ok(v) => v,
                Err(e) => {
                    error!("{e}");
                    utils::error::fatal_error();
                }
            }
        )
    }

    pub async fn save(&self, path: impl AsRef<Path>) {
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(path)
            .await
            .unwrap_or_else(|e| {
                error!("{e}");
                utils::error::fatal_error();
            });

        file.write_all(self.to_toml().as_bytes())
            .await
            .unwrap_or_else(|e| {
                error!("{e}");
                utils::error::fatal_error();
            })
    }

    pub async fn load(path: impl AsRef<Path>) -> Self {
        let mut file = File::open(path).await.unwrap_or_else(|e| {
            error!("{e}");
            utils::error::fatal_error();
        });

        let mut str = String::new();
        file.read_to_string(&mut str).await.unwrap_or_else(|e| {
            error!("{e}");
            utils::error::fatal_error();
        });

        toml::from_str(&str).unwrap_or_else(|e| {
            error!("invalid server config: {e}");
            utils::error::fatal_error();
        })
    }

    pub fn get_client_with_token(&self, token: &str) -> Option<(&String, &ClientSettings)> {
        let hashed = utils::sha::sha256(token);
        let res: Vec<_> = self
            .clients
            .iter()
            .filter(|s| s.1.shared_secret.eq(&hashed))
            .collect();

        if res.len() > 1 {
            warn!("multiple clients share the same hashed key");
        }
        match res.len() {
            0 => None,
            1 => Some(*res.first().expect("we already checked if len is 1")),
            _ => {
                warn!("multiple clients share the same hashed key");
                None
            }
        }
    }

    pub fn has_client(&self, name: &str) -> bool {
        self.get_client(name).is_some()
    }

    pub fn get_client(&self, name: &str) -> Option<&ClientSettings> {
        self.clients.get(name)
    }

    pub fn add_client(&mut self, name: &str, key: ClientSettings) {
        self.clients.insert(name.to_string(), key);
    }

    pub fn get_bind(&self) -> (&str, u16) {
        (&self.ip, self.port)
    }

    pub fn webserver_root_redirect(&self) -> &str {
        &self.webserver_root_redirect
    }

    pub fn hostname(&self) -> &str {
        &self.hostname
    }
}

#[derive(Serialize, Deserialize, derive_more::Debug, Clone)]
pub struct ClientSettings {
    // This is used to figure out where to store uploads (two clients could share a folder)
    pub folder_id: Uuid,
    // The hashed token of the client used for auth
    #[debug("CENSORED")]
    pub shared_secret: String,
}

impl ClientSettings {
    pub fn new(key: &str) -> Self {
        Self {
            folder_id: Uuid::now_v7(),
            shared_secret: key.into(),
        }
    }
}

pub async fn get_server_cfg(path: impl AsRef<Path>) -> ServerConfig {
    let path = path.as_ref();
    info!("server config path is {}", path.display());
    if !path.exists() {
        error!(
            "{} doesn't exists! please generate and edit it using kystash server generate-server-config",
            path.display()
        );
        exit(1);
    }

    ServerConfig::load(path).await
}

pub async fn generate_server_cfg(stdout: bool, path: impl AsRef<Path>) {
    let path = path.as_ref();
    info!("server config path is {}", path.display());
    if path.exists() && !stdout {
        error!(
            "{} already exists! please remove if you want to regenerate the config or run with the --stdout argument",
            path.display(),
        );
        exit(1);
    }

    if stdout {
        println!("{}", ServerConfig::example().to_toml());
    } else {
        ServerConfig::example().save(&path).await;
        info!("generated {}. please tweak it as needed", path.display());
        utils::editor::open(path).await;
    }

    exit(0);
}
