/*
 * kystash - A simple image/file sharing server/client
 * Copyright (C) 2026 2kybe3 <kybe@kybe.xyz>
 */

use std::{collections::HashMap, path::PathBuf, process::exit};

use crate::{
    config::{
        server::{self, ClientSettings, ServerConfig},
        shared::get_root_config_path,
    },
    error, sha,
};
use base64::{Engine, engine::general_purpose};
use rand::distr::{Distribution, Uniform};
use serde::{Deserialize, Serialize};
use tokio::{
    fs::{File, OpenOptions},
    io::{AsyncReadExt, AsyncWriteExt},
    process::Command,
};
use tracing::{debug, error, info, warn};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ClientConfig {
    servers: HashMap<String, Server>,
}

impl ClientConfig {
    pub fn has_server(&self, name: &str) -> bool {
        self.get_server(name).is_some()
    }

    pub fn get_server(&self, name: &str) -> Option<&Server> {
        self.servers.get(name)
    }

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
        match toml::to_string_pretty(self) {
            Ok(v) => v,
            Err(e) => {
                error!("{e}");
                crate::error::fatal_error();
            }
        }
    }

    pub async fn save(&self, path: PathBuf) {
        let mut file = match OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(path)
            .await
        {
            Ok(v) => v,
            Err(e) => {
                error!("{e}");
                crate::error::fatal_error();
            }
        };

        if let Err(e) = file.write_all(self.to_toml().as_bytes()).await {
            error!("{e}");
            crate::error::fatal_error();
        };
    }

    pub async fn load(path: PathBuf) -> Self {
        let mut file = match File::open(path).await {
            Ok(v) => v,
            Err(e) => {
                error!("{e}");
                crate::error::fatal_error();
            }
        };

        let mut str = String::new();
        if let Err(e) = file.read_to_string(&mut str).await {
            error!("{e}");
            crate::error::fatal_error();
        };

        match toml::from_str(&str) {
            Ok(v) => v,
            Err(e) => {
                error!("invalid client config: {e}");
                crate::error::fatal_error();
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Server {
    // The URL used to reach the server
    server: String,
    // Token used to authenticate with the server
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
            let mut file = match OpenOptions::new().read(true).open(path.clone()).await {
                Ok(v) => v,
                Err(e) => {
                    error!(error = ?e, expected = ?path, server = ?self.server, "token_file not found");
                    exit(1);
                }
            };
            let mut buffer = String::new();
            if let Err(e) = file.read_to_string(&mut buffer).await {
                error!("{e}");
                crate::error::fatal_error();
            }
            buffer
        } else if let Some(token_cmd) = &self.token_cmd {
            let mut parts = token_cmd.split_whitespace();

            let program = match parts.next() {
                Some(v) => v,
                None => {
                    error::fatal_error();
                }
            };

            let res = match Command::new(program).args(parts).output().await {
                Ok(v) => v,
                Err(e) => {
                    error!("{e}");
                    error::fatal_error();
                }
            };

            String::from_utf8_lossy(&res.stdout).to_string()
        } else if let Some(token) = &self.token {
            token.clone()
        } else {
            unreachable!()
        }
    }
}

pub async fn generate_client_cfg(name: &str, overwrite: bool, server_config_path: Option<PathBuf>) {
    let server_path = server_config_path.unwrap_or(ServerConfig::default_path().await);
    let mut server_cfg = {
        if !server_path.exists() {
            error!(
                "{} doesn't exist. please make sure to first generate a server config",
                server_path.clone().as_path().display().to_string()
            )
        }

        let server_config = server::get_server_cfg(server_path.clone()).await;
        debug!("server config (pre): {server_config:?}");
        server_config
    };

    let raw_pass = get_random_pass();
    let hashed_pass = sha::sha256(&raw_pass);

    if !overwrite && server_cfg.has_client(name) {
        error!("Server config already has a client {name}. Use --overwrite to ignore");
        return;
    }
    server_cfg.add_client(name, ClientSettings::new(&hashed_pass));
    debug!("server config (post): {server_cfg:?}");

    let client_cfg = ClientConfig::new(server_cfg.hostname().to_string(), raw_pass.clone());
    let client_cfg_str = match toml::to_string_pretty(&client_cfg) {
        Ok(v) => v,
        Err(e) => {
            error!("{e}");
            crate::error::fatal_error();
        }
    };

    server_cfg.save(server_path).await;

    info!("run kystash edit to edit the client config");

    print!("{client_cfg_str}");
}

fn get_random_pass() -> String {
    let mut rng = rand::rng();
    let dist = match Uniform::new_inclusive(b'(', b'~') {
        Ok(v) => v,
        Err(e) => {
            error!("{e}");
            crate::error::fatal_error();
        }
    };

    (0..256).map(|_| dist.sample(&mut rng) as char).collect()
}
