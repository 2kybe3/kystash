/*
 * kystash - A simple image/file sharing server/client
 * Copyright (C) 2026 2kybe3 <kybe@kybe.xyz>
 */

use actix_web::{HttpMessage, HttpResponse, get, post, web};

use crate::{
    server::{WebserverState, webserver::middleware::auth::AuthClient},
    shared::{
        metadata::Metadata,
        upload_identity::{UploadId, UploadIdentity},
    },
};

#[get("/metadata")]
pub async fn get_metadata(
    req: actix_web::HttpRequest,
    data: web::Data<WebserverState>,
) -> Result<HttpResponse, actix_web::Error> {
    let user = req
        .extensions()
        .get::<AuthClient>()
        .cloned()
        .ok_or(actix_web::error::ErrorInternalServerError("missing auth"))?;

    let upload_id = UploadId::try_from(req.headers())?;
    let id = UploadIdentity::new(user.settings.folder_id, upload_id);

    let map = data
        .metadata_store
        .lock()
        .await
        .get_identity(&id)
        .cloned()
        .ok_or(actix_web::error::ErrorNotFound("metadata not found"))?;

    let str = serde_json::to_string(&map)
        .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;

    Ok(HttpResponse::Ok().body(str))
}

#[post("/metadata")]
pub async fn post_metadata(
    req: actix_web::HttpRequest,
    data: web::Data<WebserverState>,
    new_meta: web::Json<Metadata>,
) -> Result<HttpResponse, actix_web::Error> {
    let user = req
        .extensions()
        .get::<AuthClient>()
        .cloned()
        .ok_or(actix_web::error::ErrorInternalServerError("missing auth"))?;

    let upload_id = UploadId::try_from(req.headers())?;
    let id = UploadIdentity::new(user.settings.folder_id, upload_id);

    let mut store = data.metadata_store.lock().await;

    let old_meta = store
        .get_identity(&id)
        .cloned()
        .ok_or(actix_web::error::ErrorNotFound(""))?;
    let res = Metadata::change_allowed(&old_meta, &new_meta, &store);
    if !res.0 {
        return Err(actix_web::error::ErrorBadRequest(res.1.unwrap().to_owned()));
    }

    if let Err(e) = store.set_metadata(id, new_meta.0) {
        return Err(actix_web::error::ErrorInternalServerError(e.to_string()));
    }

    Ok(HttpResponse::Ok().finish())
}
