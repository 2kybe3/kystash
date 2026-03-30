/*
 * kystash - A simple image/file sharing server/client
 * Copyright (C) 2026 2kybe3 <kybe@kybe.xyz>
 */

use actix_web::{HttpResponse, Responder, get, http::header, web};

use crate::server::WebserverState;

#[get("/")]
pub async fn root(data: web::Data<WebserverState>) -> impl Responder {
    HttpResponse::PermanentRedirect()
        .insert_header((header::LOCATION, data.cfg.webserver_root_redirect()))
        .finish()
}
