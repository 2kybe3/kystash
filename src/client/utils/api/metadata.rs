/*
 * kystash - A simple image/file sharing server/client
 * Copyright (C) 2026 2kybe3 <kybe@kybe.xyz>
 */

use crate::shared::{
    metadata::Metadata,
    upload_identity::{UploadId, UploadIdentity},
};

#[allow(unused)]
pub async fn get_metadata(
    identity: UploadIdentity,
    upload_id: &UploadId,
    token: &str,
    server_url: String,
) -> anyhow::Result<Option<Metadata>> {
    let client = reqwest::Client::new();
    let url = format!("{server_url}/metadata");

    let resp = client.get(&url).bearer_auth(token).send().await?;

    Ok(None)
}
