/*
 * kystash - A simple image/file sharing server/client
 * Copyright (C) 2026 2kybe3 <kybe@kybe.xyz>
 */

use actix_web::{HttpMessage, HttpResponse, Responder, get, web};

use crate::server::{WebserverState, webserver::middleware::auth::AuthClient};

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

    let id = (user.settings.folder_id.to_string(), upload_id.to_owned());
    let chunk_map = &web_data.chunk_map;

    let bv = chunk_map.lock().await;
    let bv = bv.get(&id);

    match bv {
        None => HttpResponse::NotFound().finish(),
        Some(v) => {
            let data: String = v
                .iter()
                .map(|b| if *b { "1" } else { "0" })
                .collect::<Vec<_>>()
                .join("");

            HttpResponse::Found().body(data)
        }
    }
}
