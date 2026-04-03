/*
 * kystash - A simple image/file sharing server/client
 * Copyright (C) 2026 2kybe3 <kybe@kybe.xyz>
 */

use reqwest::Client;
use std::{fs, io::SeekFrom, os::unix::fs::FileExt, path::PathBuf, process::exit, sync::Arc};
use tokio::{
    fs::{File, OpenOptions},
    io::{AsyncReadExt, AsyncSeekExt},
    sync::Semaphore,
};
use tracing::{debug, error, info, warn};
use xxhash_rust::xxh3;

use crate::{config::client::ClientConfig, error};

const MAX_UPLOAD_ATTEMPT_PER_CHUNK: u32 = 5;

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

    info!("getting upload id this might take a while");
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
    let auth = server_cfg.auth().await;

    // dbg!(
    //     get_upload_status(&upload_id, server_cfg.server(), &auth)
    //         .await
    //         .ok()
    // );

    if let Err(e) =
        upload_file_concurrent(&mut file, &upload_id, server_cfg.server(), &auth, 6).await
    {
        error!("error uploading file: {e}");
        crate::error::fatal_error();
    }
}

// async fn get_upload_status(upload_id: &str, server_url: &str, token: &str) -> anyhow::Result<()> {
//     Ok(())
// }

async fn upload_file_concurrent(
    file: &mut File,
    upload_id: &str,
    server_url: &str,
    token: &str,
    concurrency: usize,
) -> anyhow::Result<()> {
    let client = Arc::new(Client::new());
    let upload_id = Arc::new(upload_id.to_string());
    let upload_url = Arc::new(format!("{server_url}/upload/chunk"));
    let token = Arc::new(token.to_string());

    let file_size = file.metadata().await?.len();
    let chunk_size = 256 * 1024;
    let semaphore = Arc::new(Semaphore::new(concurrency));

    let mut futures = Vec::new();
    let total_chunks = file_size.div_ceil(chunk_size as u64) as usize;
    let mut current_chunk_index = 0;
    let mut offset = 0u64;

    while offset < file_size {
        let permit = semaphore.clone().acquire_owned().await?;
        let file = file.try_clone().await?;

        let client = Arc::clone(&client);
        let upload_url = Arc::clone(&upload_url);
        let upload_id = Arc::clone(&upload_id);
        let token = Arc::clone(&token);

        let this_offset = offset;
        let this_chunk_size = std::cmp::min(chunk_size as u64, file_size - this_offset) as usize;

        let fut = tokio::spawn(async move {
            let _permit = permit;
            let file = file.into_std().await;

            let buf = match tokio::task::spawn_blocking(move || {
                let mut buf = vec![0u8; this_chunk_size];
                file.read_at(&mut buf, this_offset)?;
                Ok::<std::vec::Vec<u8>, anyhow::Error>(buf)
            })
            .await
            {
                Ok(Ok(v)) => v,
                Err(e) => {
                    error!("can't read file {e}");
                    error::fatal_error();
                }
                Ok(Err(e)) => {
                    error!("can't read file {e}");
                    error::fatal_error();
                }
            };

            debug!(
                "{upload_id} >> {}/{total_chunks} @ {chunk_size} @ init",
                current_chunk_index + 1
            );

            for i in 0..MAX_UPLOAD_ATTEMPT_PER_CHUNK {
                let resp = match client
                    .post(&*upload_url)
                    .bearer_auth(&token)
                    .header("Upload-ID", &*upload_id)
                    .header("Total-Chunks", total_chunks)
                    .header("Current-Chunk", current_chunk_index + 1)
                    .header("Chunk-Size", chunk_size)
                    .body(buf.clone())
                    .send()
                    .await
                {
                    Ok(v) => v,
                    Err(e) => {
                        let msg = format!(
                            "{upload_id} >> {}/{total_chunks} @ Failed to upload chunk, attempt {}/{MAX_UPLOAD_ATTEMPT_PER_CHUNK}. {e}",
                            current_chunk_index + 1,
                            i + 1
                        );
                        if i == MAX_UPLOAD_ATTEMPT_PER_CHUNK - 1 {
                            error!("{}", msg);
                            exit(2);
                        } else {
                            warn!("{}", msg);
                        }
                        continue;
                    }
                };

                if !resp.status().is_success() {
                    let msg = format!(
                        "{upload_id} >> {}/{total_chunks} @ Failed to upload chunk, attempt {}/{MAX_UPLOAD_ATTEMPT_PER_CHUNK}\nSTATUS: {}\nTEXT: {}",
                        current_chunk_index + 1,
                        i + 1,
                        resp.status(),
                        resp.text().await.unwrap_or_default(),
                    );
                    if i == MAX_UPLOAD_ATTEMPT_PER_CHUNK - 1 {
                        error!("{}", msg);
                        exit(2);
                    } else {
                        warn!("{}", msg);
                    }
                    continue;
                }
            }

            debug!(
                "{upload_id} >> {}/{total_chunks} @ {chunk_size} @ finish",
                current_chunk_index + 1
            );
        });

        futures.push(fut);
        offset += this_chunk_size as u64;
        current_chunk_index += 1;
    }

    for f in futures {
        f.await?;
    }

    Ok(())
}

async fn get_upload_id(file: &mut File) -> anyhow::Result<String> {
    file.seek(SeekFrom::Start(0)).await?;

    let mut hasher = xxh3::Xxh3Builder::new().build();
    let mut buf = [0u8; 64 * 1024];

    loop {
        let n = file.read(&mut buf).await?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }

    Ok(format!("{:x}", hasher.digest()))
}
