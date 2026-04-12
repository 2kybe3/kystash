/*
 * kystash - A simple image/file sharing server/client
 * Copyright (C) 2026 2kybe3 <kybe@kybe.xyz>
 */

use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    process::exit,
};

use crate::{
    config::{
        server::{self, ClientSettings},
        shared::get_root_config_path,
    },
    utils,
};
use base64::{Engine, engine::general_purpose};
use chrono::Utc;
use rand::distr::{Distribution, Uniform};
use serde::{Deserialize, Serialize};
use tokio::{
    fs::{self, File, OpenOptions},
    io::{AsyncReadExt, AsyncWriteExt},
    process::Command,
};
use tracing::{debug, error, info, warn};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ClientConfig {
    /// Server Name -> Server Config
    servers: HashMap<String, Server>,
}

impl ClientConfig {
    pub fn new(server: String, token: String) -> Self {
        let mut servers = HashMap::new();
        servers.insert("default".into(), Server::new(server, token));

        Self { servers }
    }

    pub async fn default_path() -> PathBuf {
        if let Ok(ow) = std::env::var("KYSTASH_CLIENT_PATH") {
            debug!("KYSTASH_CLIENT_PATH {ow}");
            return PathBuf::from(ow);
        }

        let mut path = get_root_config_path().await;
        path.push("client.toml");
        path
    }

    pub fn to_toml(&self) -> String {
        toml::to_string_pretty(self).unwrap_or_else(|e| {
            error!("{e}");
            utils::error::fatal_error();
        })
    }

    pub async fn save(&self, path: impl AsRef<Path>) {
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(path)
            .await
            .unwrap_or_else(|e| {
                error!("failed to open file to write too\n{e}");
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
            error!("failed to open config file.\n{e}");
            utils::error::fatal_error();
        });

        let mut str = String::new();
        file.read_to_string(&mut str).await.unwrap_or_else(|e| {
            error!("failed to read config file.\n{e}");
            utils::error::fatal_error();
        });

        toml::from_str(&str).unwrap_or_else(|e| {
            error!("invalid client config.\n{e}");
            utils::error::fatal_error();
        })
    }

    pub fn has_server(&self, name: &str) -> bool {
        self.get_server(name).is_some()
    }

    pub fn get_server(&self, name: &str) -> Option<&Server> {
        self.servers.get(name)
    }

    pub fn get_server_or_exit<'a>(&self, name: impl Into<Option<&'a str>>) -> &Server {
        let name = name.into().unwrap_or("default");
        self.get_server(name).unwrap_or_else(|| {
            error!("{name} isn't in the cfg");
            exit(1);
        })
    }
}

#[derive(Serialize, Deserialize, derive_more::Debug, Clone)]
pub struct Server {
    // The URL used to reach the server
    server: String,
    // Token used to authenticate with the server
    #[debug("CENSORED")]
    token: Option<String>,
    // A command to run to get the token to authenticate with the server
    token_cmd: Option<String>,
    // A file to read to get the token to authenticate with the server
    token_file: Option<String>,
}

impl Server {
    pub fn new(server: String, token: String) -> Self {
        Self {
            server,
            token: Some(token),
            token_cmd: None,
            token_file: None,
        }
    }

    pub fn server(&self) -> &str {
        &self.server
    }

    pub async fn auth(&self) -> String {
        general_purpose::STANDARD.encode(&self.token().await)
    }

    pub async fn token(&self) -> String {
        let total = self.token.is_some() as u8
            + self.token_cmd.is_some() as u8
            + self.token_file.is_some() as u8;
        if total < 1 {
            error!("no token set for {}", self.server);
            exit(1);
        }
        if total > 1 {
            warn!("multiple token sources are set for {}", self.server);
            info!("defaulting in order token_file > token_cmd > token");
        }

        if let Some(token_file) = &self.token_file {
            let path = PathBuf::from(token_file);
            let mut file = OpenOptions::new().read(true).open(&path).await.unwrap_or_else(|e|  {
                    error!(error = ?e, expected = ?path, server = ?self.server, "token_file not found");
                    exit(1);
            });
            let mut buffer = String::new();
            file.read_to_string(&mut buffer).await.unwrap_or_else(|e| {
                error!("{e}");
                utils::error::fatal_error();
            });
            buffer
        } else if let Some(token_cmd) = &self.token_cmd {
            let mut parts = token_cmd.split_whitespace();

            let program = parts.next().unwrap_or_else(|| utils::error::fatal_error());

            let res = Command::new(program)
                .args(parts)
                .output()
                .await
                .unwrap_or_else(|e| {
                    error!("{e}");
                    utils::error::fatal_error();
                });

            String::from_utf8_lossy(&res.stdout).to_string()
        } else if let Some(token) = &self.token {
            token.to_owned()
        } else {
            utils::error::fatal_error();
        }
    }
}

pub async fn generate_client_cfg(name: &str, overwrite: bool, server_path: impl AsRef<Path>) {
    let server_path = server_path.as_ref();
    let server_path_display = server_path.display();

    if !server_path.exists() {
        error!(
            "{} doesn't exist. please make sure to first generate a server config",
            server_path_display
        );
        exit(1);
    }

    let tmp_dir = std::env::temp_dir();
    let timestamp = Utc::now().format("%Y%m%d%H%M%S");
    let backup_path = tmp_dir.join(format!(
        "{}.{}.bak",
        server_path
            .file_name()
            .unwrap_or_else(|| std::ffi::OsStr::new("server.toml"))
            .to_string_lossy(),
        timestamp
    ));

    match fs::copy(&server_path, &backup_path).await {
        Ok(_) => info!("Server config backed up to {}", backup_path.display()),
        Err(e) => {
            error!("Failed to backup server config: {}", e);
            utils::error::fatal_error();
        }
    }

    let mut server_cfg = server::get_server_cfg(&server_path).await;
    debug!("server config (pre): {server_cfg:?}");

    if !overwrite && server_cfg.has_client(name) {
        error!("Server config already has a client {name}. Use --overwrite to ignore");
        return;
    }

    let raw_pass = get_random_pass();
    let hashed_pass = utils::sha::sha256(&raw_pass);

    server_cfg.add_client(name, ClientSettings::new(&hashed_pass));
    server_cfg.save(&server_path).await;
    debug!("server config (post): {server_cfg:?}");
    info!(
        "Added {name} to {}. writing client config to stdout",
        server_path_display
    );

    info!("Run kystash edit to edit the client config");
    info!("Run kystash server edit to edit the server config");

    let client_cfg = ClientConfig::new(server_cfg.hostname().to_string(), raw_pass);
    let client_cfg_str = client_cfg.to_toml();

    print!("{client_cfg_str}");
}

fn get_random_pass() -> String {
    let mut rng = rand::rng();
    let dist = Uniform::new_inclusive(b'(', b'~').unwrap_or_else(|e| {
        error!("{e}");
        utils::error::fatal_error();
    });

    (0..256).map(|_| dist.sample(&mut rng) as char).collect()
}
