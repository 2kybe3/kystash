use std::{collections::HashMap, process::exit};

use serde::{Serialize, Deserialize};
use tokio::{fs::{File, OpenOptions}, io::{AsyncReadExt, AsyncWriteExt}};
use tracing::{info, error};
use crate::paths;

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct ServerConfig {
    ip: String,
    port: u16,
    /// Generate client cfg is gonna use this field to determite the url to set
    hostname: String,
    keys: HashMap<String, Key>,
}

impl ServerConfig {
    pub fn get_bind(&self) -> (&str ,u16) {
        (&self.ip, self.port)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Key {
    pub public_key: String,
}

pub async fn get_server_cfg() -> ServerConfig {
    let path = paths::get_config_path(paths::ConfigType::Server).await;
    let path_str = path.clone().as_path().display().to_string();
    info!("server config path is {path_str}");
    if !path.exists() {
        error!("{path_str} doesn't exists! please generate and edit it using kystash server generate-server-config");
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
        Ok(v) =>  v,
        Err(e) => {
            error!("invalid server config: {e}");
            crate::error::fatal_error();
        }
    };

    cfg
}

pub async fn generate_server_cfg() {
    let path = paths::get_config_path(paths::ConfigType::Server).await;
    let path_str = path.clone().as_path().display().to_string();
    info!("server config path is {path_str}");
    if path.exists() {
        error!("{path_str} already exists! please remove if you want to regenerate the config.");
    }

    let cfg = ServerConfig {
        hostname: "https://i.kybe.xyz/".into(),
        ip: "0.0.0.0".into(),
        port: 3000,
        keys: HashMap::new(),
    };

    let cfg = match toml::to_string_pretty(&cfg) {
        Ok(v) => v,
        Err(e) => {
            error!("{e}");
            crate::error::fatal_error();
        },
    };

    let mut file = match OpenOptions::new().write(true).create(true).truncate(true).open(path).await {
        Ok(v) => v,
        Err(e) => {
            error!("{e}");
            crate::error::fatal_error();
        },
    };

    if let Err(e) = file.write_all(cfg.as_bytes()).await {
        error!("{e}");
        crate::error::fatal_error();
    };

    info!("generated {path_str}. please tweak it as needed");
    exit(0);
}
