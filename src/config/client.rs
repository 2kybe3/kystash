/*
 * kystash - A simple image/file sharing server/client
 * Copyright (C) 2026 2kybe3 <kybe@kybe.xyz>
 */

use std::{collections::HashMap, path::PathBuf};

use crate::config::{
    server::{self, ClientSettings, ServerConfig},
    shared::get_root_config_path,
};
use argon2::{
    Argon2, PasswordHasher,
    password_hash::{SaltString, rand_core},
};
use rand::distr::{Distribution, Uniform};
use serde::{Deserialize, Serialize};
use tracing::{debug, error};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ClientConfig {
    servers: HashMap<String, Server>,
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

impl ClientConfig {
    // Gonna be used when the server creates a client config
    pub fn new(server: String, token: String) -> Self {
        let mut servers = HashMap::new();
        servers.insert("default".into(), Server::new(server, token));

        Self { servers }
    }

    pub async fn get_path() -> PathBuf {
        let mut path = get_root_config_path().await;
        path.push("client.toml");
        path
    }
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
}

pub async fn generate_client_cfg(name: &str) {
    let mut server_cfg = {
        let path = ServerConfig::get_path().await;
        let path_str = path.clone().as_path().display().to_string();
        if !path.exists() {
            error!("{path_str} doesn't exist. please make sure to first generate a server config")
        }

        let server_config = server::get_server_cfg().await;
        debug!("server config (pre): {:?}", server_config);
        server_config
    };

    let (raw_pass, hashed_pass) = {
        let raw_pass = get_random_pass();
        let salt = SaltString::generate(&mut rand_core::OsRng);
        let argon2 = Argon2::default();
        let password_hash = match argon2.hash_password(raw_pass.as_bytes(), &salt) {
            Ok(v) => v.to_string(),
            Err(e) => {
                error!("{e}");
                crate::error::fatal_error();
            }
        };
        (raw_pass, password_hash)
    };

    server_cfg.add_client(name, ClientSettings::new(hashed_pass));
    let server_cfg_str = match toml::to_string_pretty(&server_cfg) {
        Ok(v) => v,
        Err(e) => {
            error!("{e}");
            crate::error::fatal_error();
        }
    };
    debug!("server config (post): {:?}", server_cfg);

    let client_cfg = ClientConfig::new(server_cfg.hostname().to_string(), raw_pass.clone());
    let client_cfg_str = match toml::to_string_pretty(&client_cfg) {
        Ok(v) => v,
        Err(e) => {
            error!("{e}");
            crate::error::fatal_error();
        }
    };
    debug!("client config: {:?}", client_cfg);

    println!("=== CLIENT TOKEN  ===\n{raw_pass}");
    println!("=== CLIENT CONFIG ===\n{client_cfg_str}");
    println!("=== SERVER CONFIG ===\n{server_cfg_str}");
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
