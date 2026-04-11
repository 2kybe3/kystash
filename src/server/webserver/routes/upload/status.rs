/*
 * kystash - A simple image/file sharing server/client
 * Copyright (C) 2026 2kybe3 <kybe@kybe.xyz>
 */

use actix_web::{HttpMessage, HttpResponse, Responder, get, web};

use crate::{
    server::{WebserverState, webserver::middleware::auth::AuthClient},
    shared::{UploadIdentity, status_response::StatusResponse},
};

#[get("/upload/status")]
pub async fn status(
    req: actix_web::HttpRequest,
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

    let id = UploadIdentity::new(user.settings.folder_id, upload_id);
    let chunk_map = web_data.chunk_map.lock().await;

    let total_chunks = match chunk_map.get_total(&id) {
        Some(v) => v,
        None => return HttpResponse::NotFound().finish(),
    };
    let completed_chunks = match chunk_map.get_complete(&id) {
        Some(v) => v,
        None => return HttpResponse::NotFound().finish(),
    };

    let res = StatusResponse {
        total_chunks: total_chunks as u64,
        completed_chunks,
    };

    match serde_json::to_string(&res) {
        Ok(v) => HttpResponse::Ok().body(v),
        Err(e) => HttpResponse::InternalServerError().body(format!("{e}")),
    }
}
