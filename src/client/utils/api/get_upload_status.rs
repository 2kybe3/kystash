/*
 * kystash - A simple image/file sharing server/client
 * Copyright (C) 2026 2kybe3 <kybe@kybe.xyz>
 */

use bitvec::vec::BitVec;
use reqwest::StatusCode;
use tracing::{error, warn};

use crate::shared::status_response::StatusResponse;

pub async fn get_upload_status(
    client: &reqwest::Client,
    chunk_count: u64,
    server_url: &str,
    upload_id: &str,
    token: &str,
) -> Option<BitVec> {
    let url = format!("{server_url}/upload/status");

    let resp = match client
        .get(&url)
        .bearer_auth(token)
        .header("Upload-ID", upload_id)
        .send()
        .await
    {
        Ok(v) => v,
        Err(e) => {
            warn!("can't reach {url} to get upload status. {e}");
            return None;
        }
    };

    match resp.status() {
        StatusCode::OK => {}
        StatusCode::NOT_FOUND => {
            warn!("no status map found.");
            return None;
        }
        s => {
            warn!("unexpected status code {s}");
            return None;
        }
    }

    let status: StatusResponse = match resp.json().await {
        Ok(v) => v,
        Err(e) => {
            error!("failed to parse JSON response: {e}");
            return None;
        }
    };

    if status.total_chunks != chunk_count {
        warn!(
            "chunk count mismatch: server={} client={}",
            status.total_chunks, chunk_count
        );
    }

    let mut bv = BitVec::repeat(false, chunk_count as usize);

    for idx in status.completed_chunks {
        if (idx as u64) < chunk_count {
            bv.set(idx, true);
        } else {
            warn!("server returned out-of-bounds chunk index {idx}");
        }
    }

    Some(bv)
}
