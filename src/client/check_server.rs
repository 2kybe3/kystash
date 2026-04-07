/*
 * kystash - A simple image/file sharing server/client
 * Copyright (C) 2026 2kybe3 <kybe@kybe.xyz>
 */

use std::{path::PathBuf, process::exit};
use tracing::{debug, error, info};

use crate::{config::client::ClientConfig, shared::version::VersionResponse, utils};

pub async fn check_server(client_config: Option<PathBuf>, server: Option<String>) {
    let server_name = server.unwrap_or("default".to_string());
    let path = client_config.unwrap_or(ClientConfig::default_path().await);
    let cfg = ClientConfig::load(path).await;
    let server_cfg = cfg.get_server(&server_name).unwrap_or_else(|| {
        error!("{server_name} isn't in the cfg");
        exit(1);
    });

    let server = server_cfg.server();
    info!("checking server {server_name} at {server}");

    let client = reqwest::Client::new();

    let res = client
        .get(format!("{server}/version"))
        .bearer_auth(server_cfg.auth().await)
        .send()
        .await
        .unwrap_or_else(|e| {
            error!(error = ?e, "failed to send request");
            utils::error::fatal_error();
        });

    if !res.status().is_success() {
        error!(
            "invalid response from server {server_name} {}",
            res.status()
        );
    }

    let body = match res.text().await {
        Ok(v) => v,
        Err(e) => {
            error!("{e}");
            utils::error::fatal_error();
        }
    };

    debug!(response = body);

    let version = match serde_json::from_str::<VersionResponse>(&body) {
        Ok(v) => v,
        Err(e) => {
            error!("invalid response from server {server_name} {e}");
            return;
        }
    };

    info!(verified = version.verify(), response = ?version);
}
