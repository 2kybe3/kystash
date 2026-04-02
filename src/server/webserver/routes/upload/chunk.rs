/*
 * kystash - A simple image/file sharing server/client
 * Copyright (C) 2026 2kybe3 <kybe@kybe.xyz>
 */

use actix_web::{HttpMessage, HttpResponse, Responder, post, web};
use bitvec::vec::BitVec;
use std::os::unix::fs::FileExt;
use tokio::fs::OpenOptions;
use tracing::debug;

use crate::server::{WebserverState, webserver::middleware::auth::AuthClient};

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

    let upload_id = match req.headers().get("Upload-ID").and_then(|s| s.to_str().ok()) {
        Some(v) => v,
        None => return HttpResponse::BadRequest().body("Invalid Upload-ID"),
    };

    let total_chunks = match req
        .headers()
        .get("Total-Chunks")
        .and_then(|s| s.to_str().ok())
        .and_then(|s| s.parse::<usize>().ok())
    {
        Some(v) => v,
        None => return HttpResponse::BadRequest().body("Invalid Total-Chunks"),
    };

    let current_chunk = match req
        .headers()
        .get("Current-Chunk")
        .and_then(|s| s.to_str().ok())
        .and_then(|s| s.parse::<usize>().ok())
    {
        Some(v) => v,
        None => return HttpResponse::BadRequest().body("Invalid Current-Chunk"),
    };

    let chunk_size = match req
        .headers()
        .get("Chunk-Size")
        .and_then(|s| s.to_str().ok())
        .and_then(|s| s.parse::<usize>().ok())
    {
        Some(v) => v,
        None => return HttpResponse::BadRequest().body("Missing Chunk-Size"),
    };

    let offset = (current_chunk - 1) * chunk_size;

    if body.is_empty() {
        return HttpResponse::BadRequest().body("Empty chunk");
    }

    if upload_id.len() > 128 {
        return HttpResponse::BadRequest().body("Invalid upload id");
    }

    debug!("{upload_id} {current_chunk}/{total_chunks} @ {chunk_size}");
    let folder = format!("./uploads/{}", user.settings.folder_id);
    if let Err(e) = tokio::fs::create_dir_all(&folder).await {
        return HttpResponse::InternalServerError().body(e.to_string());
    }

    let file_path = format!("{}/{}", folder, upload_id);

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
            let id = (user.settings.folder_id.to_string(), upload_id.to_owned());
            let chunk_map = &web_data.chunk_map;
            let idx = current_chunk - 1;

            let mut map = chunk_map.lock().await;
            let bv = map
                .entry(id.clone())
                .or_insert_with(|| BitVec::repeat(false, total_chunks));

            bv.set(idx, true);

            HttpResponse::Ok().finish()
        }
        Ok(Err(e)) => HttpResponse::InternalServerError().body(e.to_string()),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}
