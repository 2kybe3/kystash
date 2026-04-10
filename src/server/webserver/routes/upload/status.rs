/*
 * kystash - A simple image/file sharing server/client
 * Copyright (C) 2026 2kybe3 <kybe@kybe.xyz>
 */

use actix_web::{HttpMessage, HttpResponse, Responder, get, web};

use crate::{
    server::{WebserverState, webserver::middleware::auth::AuthClient},
    shared::UploadIdentity,
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
    match web_data.chunk_map.lock().await.to_string(&id) {
        Some(v) => HttpResponse::Found().body(v),
        None => HttpResponse::NotFound().finish(),
    }
}
