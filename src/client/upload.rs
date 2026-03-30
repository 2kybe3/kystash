/*
 * kystash - A simple image/file sharing server/client
 * Copyright (C) 2026 2kybe3 <kybe@kybe.xyz>
 */

use reqwest::Client;
use std::{fs, os::unix::fs::FileExt, path::PathBuf, process::exit, sync::Arc};
use tokio::{
    fs::{File, OpenOptions},
    io::AsyncReadExt,
    sync::Semaphore,
};
use tracing::{error, info};
use xxhash_rust::xxh3;

use crate::config::client::ClientConfig;

pub async fn upload(client_config: Option<PathBuf>, server: Option<String>, file_path: PathBuf) {
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

    match fs::exists(&file_path) {
        Ok(true) => {}
        Ok(false) => {
            error!("{} does not exist", file_path.display());
            exit(1);
        }
        Err(e) => {
            error!("error checking if file exists: {e}");
            crate::error::fatal_error();
        }
    }
    let mut file = match OpenOptions::new()
        .read(true)
        .write(false)
        .open(&file_path)
        .await
    {
        Ok(v) => v,
        Err(e) => {
            error!("failed to open file: {e}");
            crate::error::fatal_error();
        }
    };

    let upload_id = match get_upload_id(&mut file).await {
        Ok(v) => v,
        Err(e) => {
            error!("error processing file: {e}");
            crate::error::fatal_error();
        }
    };
    info!(
        "Starting upload with id: {upload_id} file: {}",
        file_path.display()
    );

    if let Err(e) = upload_file_concurrent(
        &mut file,
        &upload_id,
        server_cfg.server(),
        &server_cfg.auth().await,
        6,
    )
    .await
    {
        error!("error uploading file: {e}");
        crate::error::fatal_error();
    }
}

async fn upload_file_concurrent(
    file: &mut File,
    upload_id: &str,
    upload_url: &str,
    token: &str,
    concurrency: usize,
) -> anyhow::Result<()> {
    let client = Client::new();
    let file_size = file.metadata().await?.len();

    let chunk_size = 256 * 1024;
    let semaphore = Arc::new(Semaphore::new(concurrency));

    let mut offset = 0u64;
    let mut handles = Vec::new();

    while offset < file_size {
        let permit = semaphore.clone().acquire_owned().await?;
        let token = token.to_string();
        let client = client.clone();
        let server_url = upload_url.to_string();
        let upload_id = upload_id.to_string();
        let file = file.try_clone().await?;

        let this_offset = offset;
        let this_chunk_size = std::cmp::min(chunk_size as u64, file_size - this_offset) as usize;

        let handle = tokio::spawn(async move {
            let file = file.into_std().await;

            let buf = match tokio::task::spawn_blocking(move || {
                let mut buf = vec![0u8; this_chunk_size];
                file.read_at(&mut buf, this_offset)?;
                Ok::<std::vec::Vec<u8>, anyhow::Error>(buf)
            })
            .await
            {
                Ok(Ok(v)) => v,
                Ok(Err(e)) => {
                    error!("{e}");
                    crate::error::fatal_error();
                }
                Err(e) => {
                    error!("{e}");
                    crate::error::fatal_error();
                }
            };

            let resp = client
                .post(format!("{server_url}/upload/chunk"))
                .bearer_auth(token)
                .header("Upload-ID", &upload_id)
                .header("Offset", this_offset)
                .body(buf)
                .send()
                .await?;

            if !resp.status().is_success() {
                anyhow::bail!(
                    "Failed to upload chunk at offset {}\nSTATUS: {}\nTEXT: {}",
                    this_offset,
                    resp.status(),
                    resp.text().await.unwrap_or_default(),
                );
            }

            drop(permit);
            Ok(())
        });

        handles.push(handle);
        offset += this_chunk_size as u64;
    }

    for h in handles {
        h.await??;
    }

    Ok(())
}

async fn get_upload_id(file: &mut File) -> anyhow::Result<String> {
    let mut hasher = xxh3::Xxh3Builder::new().build();
    let mut buf = [0u8; 8 * 1024];

    loop {
        let n = file.read(&mut buf).await?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }

    Ok(format!("{:x}", hasher.digest()))
}
