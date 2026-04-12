/*
 * kystash - A simple image/file sharing server/client
 * Copyright (C) 2026 2kybe3 <kybe@kybe.xyz>
 */

use actix_web::{HttpMessage, HttpResponse, get, web};

use crate::{
    server::{WebserverState, webserver::middleware::auth::AuthClient},
    shared::{
        status_response::StatusResponse,
        upload_identity::{UploadId, UploadIdentity},
    },
};

#[get("/upload/status")]
pub async fn status(
    req: actix_web::HttpRequest,
    web_data: web::Data<WebserverState>,
) -> Result<HttpResponse, actix_web::Error> {
    let user = req
        .extensions()
        .get::<AuthClient>()
        .cloned()
        .ok_or(actix_web::error::ErrorInternalServerError("missing auth"))?;

    let upload_id = UploadId::try_from(req.headers())?;
    let id = UploadIdentity::new(user.settings.folder_id, upload_id);

    let chunk_map = web_data.chunk_map.lock().await;

    let total_chunks = chunk_map
        .get_total(&id)
        .ok_or(actix_web::error::ErrorNotFound(""))?;
    let completed_chunks = chunk_map
        .get_complete(&id)
        .ok_or(actix_web::error::ErrorNotFound(""))?;

    let res = StatusResponse {
        total_chunks: total_chunks as u64,
        completed_chunks,
    };

    match serde_json::to_string(&res) {
        Ok(v) => Ok(HttpResponse::Ok().body(v)),
        Err(e) => Err(actix_web::error::ErrorInternalServerError(e.to_string())),
    }
}
