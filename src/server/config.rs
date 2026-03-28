/*
 * kystash - A simple image/file sharing server/client
 * Copyright (C) 2026 2kybe3 <kybe@kybe.xyz>
 */

use std::{collections::HashMap, process::exit};

use crate::paths;
use serde::{Deserialize, Serialize};
use tokio::{
    fs::{File, OpenOptions},
    io::{AsyncReadExt, AsyncWriteExt},
};
use tracing::{error, info};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ServerConfig {
    ip: String,
    port: u16,
    webserver_root_redirect: String,
    /// Generate client cfg is gonna use this field to determite the url to set
    hostname: String,
    keys: HashMap<String, Key>,
}

impl ServerConfig {
    pub fn example() -> Self {
        Self {
            ip: "0.0.0.0".into(),
            port: 3000,
            webserver_root_redirect: "https://kybe.xyz/kystash".into(),
            hostname: "https://i.kybe.xyz/".into(),
            keys: HashMap::new(),
        }
    }

    pub fn get_bind(&self) -> (&str, u16) {
        (&self.ip, self.port)
    }

    pub fn webserver_root_redirect(&self) -> &str {
        &self.webserver_root_redirect
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Key {
    pub public_key: String,
}

pub async fn get_server_cfg() -> ServerConfig {
    let path = paths::get_config_path(paths::ConfigType::Server).await;
    let path_str = path.clone().as_path().display().to_string();
    info!("server config path is {path_str}");
    if !path.exists() {
        error!(
            "{path_str} doesn't exists! please generate and edit it using kystash server generate-server-config"
        );
    }

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

    let cfg: ServerConfig = match toml::from_str(&str) {
        Ok(v) => v,
        Err(e) => {
            error!("invalid server config: {e}");
            crate::error::fatal_error();
        }
    };

    cfg
}

pub async fn generate_server_cfg(stdout: bool) {
    let path = paths::get_config_path(paths::ConfigType::Server).await;
    let path_str = path.clone().as_path().display().to_string();
    info!("server config path is {path_str}");
    if path.exists() && !stdout {
        error!(
            "{path_str} already exists! please remove if you want to regenerate the config or run with the --stdout argument"
        );
    }

    let cfg = match toml::to_string_pretty(&ServerConfig::example()) {
        Ok(v) => v,
        Err(e) => {
            error!("{e}");
            crate::error::fatal_error();
        }
    };

    if stdout {
        println!("{}", cfg);
    } else {
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

        if let Err(e) = file.write_all(cfg.as_bytes()).await {
            error!("{e}");
            crate::error::fatal_error();
        };

        info!("generated {path_str}. please tweak it as needed");
    }

    exit(0);
}
