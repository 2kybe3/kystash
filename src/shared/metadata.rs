/*
 * kystash - A simple image/file sharing server/client
 * Copyright (C) 2026 2kybe3 <kybe@kybe.xyz>
 */

use std::{
    io::{self, SeekFrom},
    path::Path,
};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::{
    fs::File,
    io::{AsyncReadExt, AsyncSeekExt},
};

/// How many bytes we are gonna get for magic bytes file mime detection
const MAGIC_HEADER_SIZE: u64 = 8192;

#[derive(Error, Debug)]
pub enum MetadataError {
    #[error("failed to get metadata from file {0}")]
    IoError(io::Error),
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Metadata {
    /// A comment for a upload
    comment: Option<String>,
    /// The Date the file is uploaded
    uploaded_at: DateTime<Utc>,
    /// The Mime Type of the file
    mime_type: String,
    /// The Mime File Extension of the file
    mime_extension: Option<String>,
    /// The File Size in Bytes
    file_size: u64,
    /// If the code should be shown publicly
    is_public: bool,
    /// The code used to view the file (setting is_public should generate one if missing)
    code: Option<String>,
}

impl Metadata {
    pub fn new(mime_type: String, mime_extension: Option<String>, file_size: u64) -> Self {
        Self {
            comment: None,
            uploaded_at: Utc::now(),
            mime_type,
            mime_extension,
            file_size,
            is_public: false,
            code: None,
        }
    }

    pub async fn from_path(
        path: impl AsRef<Path>,
        file: Option<&mut File>,
    ) -> Result<Self, MetadataError> {
        let path = path.as_ref();

        let ext = path
            .extension()
            .and_then(|s| s.to_str())
            .map(|s| s.to_owned());

        let file = if let Some(file) = file {
            file
        } else {
            &mut File::open(&path).await.map_err(MetadataError::IoError)?
        };

        Self::from_file(file, ext).await
    }

    pub async fn from_file(file: &mut File, ext: Option<String>) -> Result<Self, MetadataError> {
        let size = file.metadata().await.map_err(MetadataError::IoError)?.len();

        file.seek(SeekFrom::Start(0))
            .await
            .map_err(MetadataError::IoError)?;
        let limit = std::cmp::min(size, MAGIC_HEADER_SIZE) as usize;
        let mut bytes = Vec::with_capacity(limit);
        file.take(8192)
            .read_to_end(&mut bytes)
            .await
            .map_err(MetadataError::IoError)?;

        if let Some(mime) = infer::get(&bytes) {
            return Ok(Self::new(
                mime.mime_type().to_owned(),
                Some(mime.extension().to_owned()),
                size,
            ));
        }

        if let Some(ref ext) = ext
            && let Some(mime) = mime_guess::from_ext(ext).first()
        {
            return Ok(Self::new(
                mime.essence_str().to_owned(),
                Some(ext.to_owned()),
                size,
            ));
        }

        if std::str::from_utf8(&bytes).is_ok() {
            return Ok(Self::new(
                "text/plain".to_string(),
                ext.or(Some("txt".to_string())),
                size,
            ));
        }

        Ok(Self::new("application/octet-stream".to_owned(), None, size))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use tokio::fs::File;

    pub async fn test_asset(file: &str) -> PathBuf {
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("assets")
            .join("tests")
            .join(file)
    }

    #[tokio::test(name = "metadata_txt")]
    pub async fn metadata_txt() -> anyhow::Result<()> {
        let path = test_asset("ipsum.txt").await;

        let metadata = Metadata::from_path(&path, None).await?;

        assert_eq!(metadata.mime_type, "text/plain");
        assert_eq!(metadata.mime_extension, Some("txt".to_owned()));

        Ok(())
    }

    #[tokio::test(name = "metadata_png")]
    pub async fn metadata_png() -> anyhow::Result<()> {
        let path = test_asset("tiny.png").await;

        let metadata = Metadata::from_path(&path, None).await?;

        assert_eq!(metadata.mime_type, "image/png");
        assert_eq!(metadata.mime_extension, Some("png".to_owned()));

        Ok(())
    }

    #[tokio::test(name = "metadata_png_no_ext")]
    pub async fn metadata_png_no_ext() -> anyhow::Result<()> {
        let path = test_asset("tiny.png").await;

        let mut file = File::open(&path).await?;

        let metadata = Metadata::from_file(&mut file, None).await?;

        assert_eq!(metadata.mime_type, "image/png");
        assert_eq!(metadata.mime_extension, Some("png".to_owned()));

        Ok(())
    }

    #[tokio::test(name = "metadata_default_private")]
    pub async fn metadata_default_private() {
        let metadata = Metadata::new("".into(), None, 69);
        assert!(!metadata.is_public);
    }
}
