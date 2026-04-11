/*
 * kystash - A simple image/file sharing server/client
 * Copyright (C) 2026 2kybe3 <kybe@kybe.xyz>
 */

use actix_web::{HttpMessage, HttpResponse, Responder, get, web};

use crate::{
    server::{WebserverState, webserver::middleware::auth::AuthClient},
    shared::UploadIdentity,
};

#[get("/metadata")]
pub async fn get_metadata(
    req: actix_web::HttpRequest,
    data: web::Data<WebserverState>,
) -> impl Responder {
    let user = match req.extensions().get::<AuthClient>().cloned() {
        Some(v) => v,
        None => return HttpResponse::InternalServerError().finish(),
    };

    let upload_id = match req.headers().get("Upload-ID").and_then(|s| s.to_str().ok()) {
        Some(v) => v,
        None => return HttpResponse::BadRequest().body("Invalid Upload-ID"),
    };

    let id = UploadIdentity::new(user.settings.folder_id, upload_id.to_owned());

    let map = match data.metadata_store.lock().await.get_identity(&id).cloned() {
        Some(v) => v,
        None => return HttpResponse::NotFound().finish(),
    };

    let str = match serde_json::to_string(&map) {
        Ok(v) => v,
        Err(e) => return HttpResponse::InternalServerError().body(format!("{e}")),
    };

    HttpResponse::Ok().body(str)
}
