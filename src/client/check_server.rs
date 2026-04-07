/*
 * kystash - A simple image/file sharing server/client
 * Copyright (C) 2026 2kybe3 <kybe@kybe.xyz>
 */

use std::path::PathBuf;
use tracing::{debug, error, info};

use crate::{config::client::ClientConfig, shared::version::VersionResponse, utils};

pub async fn check_server(client_config: Option<PathBuf>, server: Option<String>) {
    let path = client_config.unwrap_or(ClientConfig::default_path().await);
    let cfg = ClientConfig::load(path).await;

    let server_name = server.unwrap_or("default".to_string());
    let server_cfg = cfg.get_server_or_exit(&*server_name);

    let server = server_cfg.server();
    info!("{server_name} >> checking server at {server}");

    let client = reqwest::Client::new();

    let res = client
        .get(format!("{server}/version"))
        .bearer_auth(server_cfg.auth().await)
        .send()
        .await
        .unwrap_or_else(|e| {
            error!(error = ?e, "{server_name} >> failed to send request");
            utils::error::fatal_error();
        });

    if !res.status().is_success() {
        error!("{server_name} >> {}", res.status());
        return;
    }

    let body = res.text().await.unwrap_or_else(|e| {
        error!("{server_name} >> {e}");
        utils::error::fatal_error();
    });

    debug!(response = body);

    let version = match serde_json::from_str::<VersionResponse>(&body) {
        Ok(v) => v,
        Err(e) => {
            error!("{server_name} >> invalid response {e}");
            return;
        }
    };

    info!(verified = version.verify(), response = ?version);
}
