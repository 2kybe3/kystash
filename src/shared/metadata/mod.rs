/*
 * kystash - A simple image/file sharing server/client
 * Copyright (C) 2026 2kybe3 <kybe@kybe.xyz>
 */

pub mod store;

use std::{
    io::{self, SeekFrom},
    path::Path,
};

use anyhow::anyhow;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::{
    fs::{File, OpenOptions},
    io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt},
};

/// How many bytes we are gonna get for magic bytes file mime detection
const MAGIC_HEADER_SIZE: u64 = 8192;

#[derive(Debug, Deserialize, Serialize, Hash, PartialEq)]
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
    ) -> Result<Self, io::Error> {
        let path = path.as_ref();

        let ext = path
            .extension()
            .and_then(|s| s.to_str())
            .map(|s| s.to_owned());

        let file = if let Some(file) = file {
            file
        } else {
            &mut File::open(&path).await?
        };

        Self::gen_from_file(file, ext).await
    }

    pub async fn load(path: impl AsRef<Path>) -> Result<Self, io::Error> {
        let mut buf = Vec::new();
        OpenOptions::new()
            .read(true)
            .open(path)
            .await?
            .read_to_end(&mut buf)
            .await?;

        Ok(serde_json::from_str(&String::from_utf8_lossy(&buf))?)
    }

    pub async fn save(&self, path: impl AsRef<Path>) -> anyhow::Result<()> {
        tokio::fs::create_dir_all(
            path.as_ref()
                .parent()
                .ok_or(anyhow!("paren't doesnt exist"))?,
        )
        .await?;
        Ok(OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(path)
            .await?
            .write_all(serde_json::to_string(self)?.as_bytes())
            .await?)
    }

    pub async fn gen_from_file(file: &mut File, ext: Option<String>) -> Result<Self, io::Error> {
        let size = file.metadata().await?.len();

        file.seek(SeekFrom::Start(0)).await?;
        let limit = std::cmp::min(size, MAGIC_HEADER_SIZE) as usize;
        let mut bytes = Vec::with_capacity(limit);
        file.take(8192).read_to_end(&mut bytes).await?;

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
    use crate::utils;
    use std::{env::temp_dir, fs, path::PathBuf};
    use tokio::fs::File;

    pub fn test_asset(file: &str) -> PathBuf {
        utils::tests::test_asset_folder()
            .join("metadata")
            .join(file)
    }

    #[tokio::test(name = "metadata_txt")]
    pub async fn metadata_txt() -> anyhow::Result<()> {
        let path = test_asset("ipsum.txt");

        let metadata = Metadata::from_path(&path, None).await?;

        assert_eq!(metadata.mime_type, "text/plain");
        assert_eq!(metadata.mime_extension, Some("txt".to_owned()));

        Ok(())
    }

    #[tokio::test(name = "metadata_png")]
    pub async fn metadata_png() -> anyhow::Result<()> {
        let path = test_asset("tiny.png");

        let metadata = Metadata::from_path(&path, None).await?;

        assert_eq!(metadata.mime_type, "image/png");
        assert_eq!(metadata.mime_extension, Some("png".to_owned()));

        Ok(())
    }

    #[tokio::test(name = "metadata_png_no_ext")]
    pub async fn metadata_png_no_ext() -> anyhow::Result<()> {
        let path = test_asset("tiny.png");

        let mut file = File::open(&path).await?;

        let metadata = Metadata::gen_from_file(&mut file, None).await?;

        assert_eq!(metadata.mime_type, "image/png");
        assert_eq!(metadata.mime_extension, Some("png".to_owned()));

        Ok(())
    }

    #[tokio::test(name = "metadata_default_private")]
    pub async fn metadata_default_private() {
        let metadata = Metadata::new("".into(), None, 69);
        assert!(!metadata.is_public);
    }

    #[tokio::test(name = "metadata-save-and-load")]
    pub async fn metadata_save_and_load() -> anyhow::Result<()> {
        let metadata = Metadata::new("".into(), None, 69);
        let file = temp_dir().join("test.meta");
        metadata.save(&file).await?;

        assert!(fs::exists(&file)?);

        let new = Metadata::load(file).await?;

        assert_eq!(metadata, new);

        Ok(())
    }
}
