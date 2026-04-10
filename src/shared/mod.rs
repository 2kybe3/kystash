/*
 * kystash - A simple image/file sharing server/client
 * Copyright (C) 2026 2kybe3 <kybe@kybe.xyz>
 */

pub mod metadata;
pub mod version;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct UploadIdentity {
    pub folder_id: String,
    pub upload_id: String,
}

impl UploadIdentity {
    pub fn new(folder_id: impl Into<String>, upload_id: impl Into<String>) -> Self {
        Self {
            folder_id: folder_id.into(),
            upload_id: upload_id.into(),
        }
    }
}
