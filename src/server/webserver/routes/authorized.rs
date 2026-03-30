/*
 * kystash - A simple image/file sharing server/client
 * Copyright (C) 2026 2kybe3 <kybe@kybe.xyz>
 */

use actix_web::{HttpMessage, HttpResponse, Responder, get};
use tracing::debug;

use crate::server::webserver::middleware::auth::AuthClient;

#[get("/authorized")]
pub async fn authorized(req: actix_web::HttpRequest) -> impl Responder {
    let user = match req.extensions().get::<AuthClient>().cloned() {
        Some(v) => v,
        None => return HttpResponse::InternalServerError().finish(),
    };

    debug!("{} authorized with settings {:?}", user.name, user.settings);
    HttpResponse::Ok().finish()
}
