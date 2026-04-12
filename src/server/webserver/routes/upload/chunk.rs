/*
 * kystash - A simple image/file sharing server/client
 * Copyright (C) 2026 2kybe3 <kybe@kybe.xyz>
 */

use actix_web::{HttpMessage, HttpResponse, http::header::HeaderMap, post, web};
use std::os::unix::fs::FileExt;
use tokio::fs::OpenOptions;
use tracing::trace;

use crate::{
    server::{WebserverState, webserver::middleware::auth::AuthClient},
    shared::upload_identity::{UploadId, UploadIdentity},
};

struct Headers {
    upload_id: UploadId,
    total_chunks: usize,
    current_chunk: usize,
    chunk_size: usize,
}

impl Headers {
    fn log_start(&self) {
        trace!(
            "{} {}/{} @ {}",
            self.upload_id, self.current_chunk, self.total_chunks, self.chunk_size
        );
    }
}

#[post("/upload/chunk")]
pub async fn chunk(
    req: actix_web::HttpRequest,
    body: web::Bytes,
    web_data: web::Data<WebserverState>,
) -> Result<HttpResponse, actix_web::Error> {
    let user = req
        .extensions()
        .get::<AuthClient>()
        .cloned()
        .ok_or(actix_web::error::ErrorInternalServerError("missing auth"))?;

    let headers = extract_headers(req.headers())?;
    headers.log_start();

    let offset = (headers.current_chunk - 1) * headers.chunk_size;

    if body.is_empty() {
        return Err(actix_web::error::ErrorBadRequest("Empty Chunk"));
    }

    let mut folder = web_data.cfg.get_upload_dir().await;
    folder.push(user.settings.folder_id.to_string());
    if let Err(e) = tokio::fs::create_dir_all(&folder).await {
        return Err(actix_web::error::ErrorInternalServerError(e.to_string()));
    }

    let file_path = format!("{}/{}", folder.display(), headers.upload_id);

    let file = match OpenOptions::new()
        .create(true)
        .truncate(false)
        .write(true)
        .open(&file_path)
        .await
    {
        Ok(v) => v,
        Err(e) => return Err(actix_web::error::ErrorInternalServerError(e.to_string())),
    };

    let data = body.to_vec();

    let file = file.into_std().await;
    let result = tokio::task::spawn_blocking(move || file.write_at(&data, offset as u64)).await;

    match result {
        Ok(Ok(_)) => {
            let id = UploadIdentity::new(user.settings.folder_id.to_string(), headers.upload_id);

            web_data.chunk_map.lock().await.set_finished_chunk(
                &id,
                headers.current_chunk - 1,
                headers.total_chunks,
            );

            Ok(HttpResponse::Ok().finish())
        }
        Ok(Err(e)) => Err(actix_web::error::ErrorInternalServerError(e.to_string())),
        Err(e) => Err(actix_web::error::ErrorInternalServerError(e.to_string())),
    }
}

fn extract_headers(headers: &HeaderMap) -> Result<Headers, actix_web::Error> {
    let upload_id = UploadId::try_from(headers)?;

    let total_chunks = headers
        .get("Total-Chunks")
        .and_then(|s| s.to_str().ok())
        .and_then(|s| s.parse::<usize>().ok())
        .ok_or(actix_web::error::ErrorBadRequest("Invalid Total-Chunks"))?;

    let current_chunk = headers
        .get("Current-Chunk")
        .and_then(|s| s.to_str().ok())
        .and_then(|s| s.parse::<usize>().ok())
        .ok_or(actix_web::error::ErrorBadRequest("Invalid Current-Chunk"))?;

    let chunk_size = headers
        .get("Chunk-Size")
        .and_then(|s| s.to_str().ok())
        .and_then(|s| s.parse::<usize>().ok())
        .ok_or(actix_web::error::ErrorBadRequest("Invalid Chunk-Size"))?;

    Ok(Headers {
        upload_id,
        total_chunks,
        current_chunk,
        chunk_size,
    })
}
