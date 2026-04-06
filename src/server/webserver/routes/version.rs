/*
 * kystash - A simple image/file sharing server/client
 * Copyright (C) 2026 2kybe3 <kybe@kybe.xyz>
 */

use actix_web::{HttpMessage, HttpResponse, Responder, get};

use crate::{server::webserver::middleware::auth::AuthClient, shared::version::VersionResponse};

#[get("/version")]
pub async fn version(req: actix_web::HttpRequest) -> impl Responder {
    let user = req.extensions().get::<AuthClient>().cloned();

    match serde_json::to_string(&VersionResponse::new(user.is_some())) {
        Ok(v) => HttpResponse::Ok().body(v),
        Err(e) => HttpResponse::InternalServerError().body(format!("{e}")),
    }
}
