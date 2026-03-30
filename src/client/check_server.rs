/*
 * kystash - A simple image/file sharing server/client
 * Copyright (C) 2026 2kybe3 <kybe@kybe.xyz>
 */

use reqwest::Client;
use std::{path::PathBuf, process::exit};
use tracing::{error, info};

use crate::{config::client::ClientConfig, error};

pub async fn check_server(client_config: Option<PathBuf>, server: Option<String>) {
    let server_name = server.unwrap_or("default".to_string());
    let path = client_config.unwrap_or(ClientConfig::default_path().await);
    let cfg = ClientConfig::load(path).await;
    let server_cfg = match cfg.get_server(&server_name) {
        Some(v) => v,
        None => {
            error!("{server_name} isn't in the cfg");
            exit(2);
        }
    };

    let server = server_cfg.server();
    info!("checking server {server_name} at {server}");

    let client = Client::new();

    let res = match client
        .get(format!("{server}/authorized"))
        .bearer_auth(server_cfg.auth().await)
        .send()
        .await
    {
        Ok(v) => v,
        Err(e) => {
            error!(error = ?e, "failed to send request");
            error::fatal_error();
        }
    };

    if res.status().is_success() {
        let body = match res.text().await {
            Ok(v) => v,
            Err(e) => {
                error!("{e}");
                error::fatal_error();
            }
        };
        println!("Response: {body}");
    } else {
        eprintln!("Request failed: {}", res.status());
    }
}
