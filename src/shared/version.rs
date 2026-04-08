/*
 * kystash - A simple image/file sharing server/client
 * Copyright (C) 2026 2kybe3 <kybe@kybe.xyz>
 */

use base64::{Engine, engine::general_purpose};
use serde::{Deserialize, Serialize};

const VERSION: &str = env!("CARGO_PKG_VERSION");
const SERVICE_NAME: &str = "kystash";
const MAGIC_STRING: &str = "uhhhh. this should be kystash.";

#[derive(Debug, Serialize, Deserialize)]
pub struct VersionResponse {
    version: String,
    service_name: String,
    magic_string: String,
    authorized: bool,
}

impl VersionResponse {
    pub fn new(authorized: bool) -> Self {
        Self {
            version: VERSION.into(),
            service_name: SERVICE_NAME.into(),
            magic_string: general_purpose::STANDARD.encode(MAGIC_STRING),
            authorized,
        }
    }

    pub fn verify(&self) -> bool {
        String::from_utf8_lossy(
            &general_purpose::STANDARD
                .decode(&self.magic_string)
                .unwrap_or_default(),
        ) == MAGIC_STRING
    }
}
