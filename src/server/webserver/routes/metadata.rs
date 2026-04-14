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
    _data: web::Data<WebserverState>,
) -> Result<HttpResponse, actix_web::Error> {
    let user = req
        .extensions()
        .get::<AuthClient>()
        .cloned()
        .ok_or(actix_web::error::ErrorInternalServerError("missing auth"))?;

    let upload_id = UploadId::try_from(req.headers())?;
    let _id = UploadIdentity::new(user.settings.folder_id, upload_id);

    todo!();
}

#[post("/metadata")]
pub async fn post_metadata(
    req: actix_web::HttpRequest,
    _data: web::Data<WebserverState>,
    _new_meta: web::Json<Metadata>,
) -> Result<HttpResponse, actix_web::Error> {
    let user = req
        .extensions()
        .get::<AuthClient>()
        .cloned()
        .ok_or(actix_web::error::ErrorInternalServerError("missing auth"))?;

    let upload_id = UploadId::try_from(req.headers())?;
    let _id = UploadIdentity::new(user.settings.folder_id, upload_id);

    todo!();
}
