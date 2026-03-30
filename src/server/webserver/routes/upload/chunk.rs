/*
 * kystash - A simple image/file sharing server/client
 * Copyright (C) 2026 2kybe3 <kybe@kybe.xyz>
 */

use std::os::unix::fs::FileExt;

use actix_web::{HttpMessage, HttpResponse, Responder, post, web};
use tokio::fs::OpenOptions;
use tracing::debug;

use crate::server::webserver::middleware::auth::AuthClient;

#[post("/upload/chunk")]
pub async fn chunk(req: actix_web::HttpRequest, body: web::Bytes) -> impl Responder {
    let user = match req.extensions().get::<AuthClient>().cloned() {
        Some(v) => v,
        None => return HttpResponse::InternalServerError().finish(),
    };

    let upload_id = match req.headers().get("Upload-ID") {
        Some(v) => v.to_str().unwrap_or_default(),
        None => return HttpResponse::BadRequest().body("Missing Upload-ID"),
    };

    let offset = match req.headers().get("Offset") {
        Some(v) => v
            .to_str()
            .ok()
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(0),
        None => return HttpResponse::BadRequest().body("Missing Offset"),
    };

    if body.is_empty() {
        return HttpResponse::BadRequest().body("Empty chunk");
    }

    if upload_id.len() > 128 {
        return HttpResponse::BadRequest().body("Invalid upload id");
    }

    debug!(
        "{} uploading chunk for {} at offset {}",
        user.name, upload_id, offset
    );

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

    let result = tokio::task::spawn_blocking(move || file.write_at(&data, offset)).await;

    match result {
        Ok(Ok(_)) => HttpResponse::Ok().finish(),
        Ok(Err(e)) => HttpResponse::InternalServerError().body(e.to_string()),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}
