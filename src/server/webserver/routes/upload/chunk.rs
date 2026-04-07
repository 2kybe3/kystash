/*
 * kystash - A simple image/file sharing server/client
 * Copyright (C) 2026 2kybe3 <kybe@kybe.xyz>
 */

use actix_web::{HttpMessage, HttpResponse, Responder, http::header::HeaderMap, post, web};
use bitvec::vec::BitVec;
use std::os::unix::fs::FileExt;
use tokio::fs::OpenOptions;
use tracing::trace;

use crate::{
    server::{WebserverState, webserver::middleware::auth::AuthClient},
    shared::UploadIdentity,
};

struct Headers<'a> {
    upload_id: &'a str,
    total_chunks: usize,
    current_chunk: usize,
    chunk_size: usize,
}

impl<'a> Headers<'a> {
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
) -> impl Responder {
    let user = match req.extensions().get::<AuthClient>().cloned() {
        Some(v) => v,
        None => return HttpResponse::InternalServerError().finish(),
    };

    let headers = match extract_headers(req.headers()) {
        Ok(v) => v,
        Err(http) => return http,
    };

    headers.log_start();

    let offset = (headers.current_chunk - 1) * headers.chunk_size;

    if body.is_empty() {
        return HttpResponse::BadRequest().body("Empty chunk");
    }

    if headers.upload_id.len() != 16 {
        return HttpResponse::BadRequest().body("Invalid upload id");
    }

    let mut folder = web_data.cfg.get_upload_dir().await;
    folder.push(user.settings.folder_id.to_string());
    if let Err(e) = tokio::fs::create_dir_all(&folder).await {
        return HttpResponse::InternalServerError().body(e.to_string());
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
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    };

    let data = body.to_vec();

    let file = file.into_std().await;
    let result = tokio::task::spawn_blocking(move || file.write_at(&data, offset as u64)).await;

    match result {
        Ok(Ok(_)) => {
            let id = UploadIdentity::new(
                user.settings.folder_id.to_string(),
                headers.upload_id.to_owned(),
            );
            let chunk_map = &web_data.chunk_map;
            let idx = headers.current_chunk - 1;

            let mut map = chunk_map.lock().await;
            let bv = map
                .entry(id)
                .or_insert_with(|| BitVec::repeat(false, headers.total_chunks));

            // TODO: maybe allow total_chunks changes
            bv.set(idx, true);

            HttpResponse::Ok().finish()
        }
        Ok(Err(e)) => HttpResponse::InternalServerError().body(e.to_string()),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

fn extract_headers(headers: &HeaderMap) -> Result<Headers<'_>, HttpResponse> {
    let upload_id = match headers.get("Upload-ID").and_then(|s| s.to_str().ok()) {
        Some(v) => v,
        None => return Err(HttpResponse::BadRequest().body("Invalid Upload-ID")),
    };

    let total_chunks = match headers
        .get("Total-Chunks")
        .and_then(|s| s.to_str().ok())
        .and_then(|s| s.parse::<usize>().ok())
    {
        Some(v) => v,
        None => return Err(HttpResponse::BadRequest().body("Invalid Total-Chunks")),
    };

    let current_chunk = match headers
        .get("Current-Chunk")
        .and_then(|s| s.to_str().ok())
        .and_then(|s| s.parse::<usize>().ok())
    {
        Some(v) => v,
        None => return Err(HttpResponse::BadRequest().body("Invalid Current-Chunk")),
    };

    let chunk_size = match headers
        .get("Chunk-Size")
        .and_then(|s| s.to_str().ok())
        .and_then(|s| s.parse::<usize>().ok())
    {
        Some(v) => v,
        None => return Err(HttpResponse::BadRequest().body("Missing Chunk-Size")),
    };

    Ok(Headers {
        upload_id,
        total_chunks,
        current_chunk,
        chunk_size,
    })
}
