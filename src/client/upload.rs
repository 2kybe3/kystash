/*
 * kystash - A simple image/file sharing server/client
 * Copyright (C) 2026 2kybe3 <kybe@kybe.xyz>
 */

use bitvec::vec::BitVec;
use reqwest::Client;
use std::{fs, os::unix::fs::FileExt, path::PathBuf, process::exit, sync::Arc};
use tokio::{
    fs::{File, OpenOptions},
    sync::Semaphore,
};
use tracing::{error, info, trace, warn};

use crate::{client::utils::api, config::client::ClientConfig, shared::metadata::Metadata, utils};

const MAX_UPLOAD_ATTEMPT_PER_CHUNK: u32 = 5;

pub async fn upload(client_config: Option<PathBuf>, server: Option<String>, file_path: PathBuf) {
    let server_name = server.unwrap_or("default".to_string());
    let path = client_config.unwrap_or(ClientConfig::default_path().await);
    let cfg = ClientConfig::load(path).await;
    let server_cfg = cfg.get_server(&server_name).unwrap_or_else(|| {
        error!("{server_name} isn't in the cfg");
        exit(1);
    });

    match fs::exists(&file_path) {
        Ok(true) => {}
        Ok(false) => {
            error!("{} does not exist", file_path.display());
            exit(1);
        }
        Err(e) => {
            error!("error checking if file exists: {e}");
            utils::error::fatal_error();
        }
    }

    let mut file = OpenOptions::new()
        .read(true)
        .write(false)
        .open(&file_path)
        .await
        .unwrap_or_else(|e| {
            error!("failed to open file: {e}");
            utils::error::fatal_error();
        });

    info!("getting upload id this might take a while");
    let upload_id = utils::id::get_upload_id(&mut file)
        .await
        .unwrap_or_else(|e| {
            error!("error processing file: {e}");
            utils::error::fatal_error();
        });

    info!(
        "Starting upload with id: {upload_id} file: {}",
        file_path.display()
    );
    let auth = server_cfg.auth().await;

    let client = Client::new();

    let file_metadata = file.metadata().await.unwrap_or_else(|e| {
        error!("failed to get file_metadata: {e}");
        utils::error::fatal_error();
    });
    let file_size = file_metadata.len();
    let chunk_size = 256 * 1024;
    let total_chunks = file_size.div_ceil(chunk_size);

    let bv = api::get_upload_status(
        &client,
        total_chunks,
        server_cfg.server(),
        &upload_id,
        &auth,
    )
    .await;

    upload_file_concurrent(
        client,
        FileInfo { file, file_size },
        ChunkInfo {
            chunk_size,
            total_chunks,
            done: bv,
        },
        &upload_id,
        server_cfg.server(),
        &auth,
    )
    .await
    .unwrap_or_else(|e| {
        error!("error uploading file: {e}");
        utils::error::fatal_error();
    });

    let metadata = Metadata::from_path(file_path, None).await;
    info!("{metadata:?}");

    // TODO: set initial metadata for the upload (server should generate a fitting .meta file also
    // used to know if the upload is finished for gc)
}

const CONCURRENCY: usize = 6;

struct ChunkInfo {
    chunk_size: u64,
    total_chunks: u64,
    done: Option<BitVec>,
}

struct FileInfo {
    file: File,
    file_size: u64,
}

async fn upload_file_concurrent(
    client: reqwest::Client,
    file_info: FileInfo,
    chunk_info: ChunkInfo,
    upload_id: &str,
    server_url: &str,
    token: &str,
) -> anyhow::Result<()> {
    let file = file_info.file;
    let file_size = file_info.file_size;

    let chunk_size = chunk_info.chunk_size;
    let total_chunks = chunk_info.total_chunks;
    let done = chunk_info.done;

    let client = Arc::new(client);
    let upload_id = Arc::new(upload_id.to_string());
    let upload_url = Arc::new(format!("{server_url}/upload/chunk"));
    let token = Arc::new(token.to_string());
    let done = done.map(Arc::new);

    let semaphore = Arc::new(Semaphore::new(CONCURRENCY));

    let mut futures = Vec::new();
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
        let this_chunk_size = std::cmp::min(chunk_size, file_size - this_offset) as usize;

        if let Some(ref bv) = done
            && bv.get(current_chunk_index).map(|v| *v).unwrap_or(false)
        {
            offset += this_chunk_size as u64;
            current_chunk_index += 1;
            continue;
        }

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
                    utils::error::fatal_error();
                }
                Ok(Err(e)) => {
                    error!("can't read file {e}");
                    utils::error::fatal_error();
                }
            };

            trace!(
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
                            exit(1);
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
                        exit(1);
                    } else {
                        warn!("{}", msg);
                    }
                    continue;
                }

                break;
            }

            trace!(
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
