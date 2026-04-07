/*
 * kystash - A simple image/file sharing server/client
 * Copyright (C) 2026 2kybe3 <kybe@kybe.xyz>
 */

use std::io::SeekFrom;

use tokio::{
    fs::File,
    io::{AsyncReadExt, AsyncSeekExt},
};
use xxhash_rust::xxh3;

pub async fn get_upload_id(file: &mut File) -> anyhow::Result<String> {
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

    Ok(format!("{:016x}", hasher.digest()))
}
