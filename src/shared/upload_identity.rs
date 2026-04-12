/*
 * kystash - A simple image/file sharing server/client
 * Copyright (C) 2026 2kybe3 <kybe@kybe.xyz>
 */

use std::{
    fmt::{self, Display},
    io::SeekFrom,
    ops::{Deref, DerefMut},
};

use actix_web::{HttpResponse, ResponseError};
use tokio::{
    fs::File,
    io::{AsyncReadExt, AsyncSeekExt},
};
use tracing::warn;
use xxhash_rust::xxh3;

const UPLOAD_ID_LEN: usize = 16;

#[derive(Debug)]
pub struct UploadIdError;

impl fmt::Display for UploadIdError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Invalid Upload-ID")
    }
}

impl ResponseError for UploadIdError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::BadRequest().body(self.to_string())
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct UploadId(String);

impl UploadId {
    #[cfg(test)]
    pub fn new(str: impl Into<String>) -> Self {
        let str = str.into();
        Self(str)
    }

    pub fn from_file_name(file_name: &str) -> Option<Self> {
        let str = file_name.strip_suffix(".meta")?;
        #[cfg(not(test))]
        if str.len() != UPLOAD_ID_LEN {
            warn!("upload_id is not 16 char's long: {file_name}");
            return None;
        }
        Some(Self(str.to_owned()))
    }

    pub async fn from_file(file: &mut File) -> anyhow::Result<Self> {
        file.seek(SeekFrom::Start(0)).await?;

        let mut hasher = xxh3::Xxh3Builder::new().build();
        let mut buf = [0u8; 64 * 1024];

        loop {
            let n = file.read(&mut buf).await?;
            if n == 0 {
                break;
            }
            hasher.update(&buf[..n]);
        }

        Ok(Self(format!("{:016x}", hasher.digest())))
    }
}

impl Deref for UploadId {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for UploadId {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Display for UploadId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl TryFrom<&actix_web::http::header::HeaderMap> for UploadId {
    type Error = UploadIdError;

    fn try_from(value: &actix_web::http::header::HeaderMap) -> Result<Self, Self::Error> {
        let s = value
            .get("Upload-ID")
            .and_then(|s| s.to_str().ok())
            .ok_or(UploadIdError)?;

        if s.len() != UPLOAD_ID_LEN {
            return Err(UploadIdError);
        };

        Ok(Self(s.to_owned()))
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct UploadIdentity {
    pub folder_id: String,
    pub upload_id: UploadId,
}

impl UploadIdentity {
    pub fn new(folder_id: impl Into<String>, upload_id: UploadId) -> Self {
        Self {
            folder_id: folder_id.into(),
            upload_id,
        }
    }
}
