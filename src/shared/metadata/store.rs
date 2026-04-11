/*
 * kystash - A simple image/file sharing server/client
 * Copyright (C) 2026 2kybe3 <kybe@kybe.xyz>
 */

use crate::{
    shared::{UploadIdentity, metadata::Metadata},
    utils,
};
use anyhow::anyhow;
use std::{
    collections::{HashMap, hash_map::Entry},
    path::Path,
};
use tokio::fs;
use tracing::{error, info, trace, warn};
use xxhash_rust::xxh3::xxh3_128;

type MetadataId = usize;

#[derive(Debug, Default)]
pub struct MetadataStore {
    store: Vec<Metadata>,                  // id -> meta
    meta_to_id: HashMap<u128, MetadataId>, // hash -> id
    identity_to_id: HashMap<UploadIdentity, MetadataId>,
}

impl MetadataStore {
    pub fn get(&self, id: MetadataId) -> &Metadata {
        &self.store[id]
    }

    /// Get's the current metadata of a identity
    pub fn get_identity(&self, identity: &UploadIdentity) -> Option<&Metadata> {
        let id = self.identity_to_id.get(identity)?;
        let metadata = self.get(*id);
        Some(metadata)
    }

    /// Sets the metadata for a identity
    pub fn set_metadata(&mut self, identity: UploadIdentity, meta: Metadata) -> anyhow::Result<()> {
        let hash = xxh3_128(&postcard::to_stdvec(&meta)?);

        let id = match self.meta_to_id.entry(hash) {
            Entry::Occupied(o) => {
                let id = *o.get();
                if self.store[id] == meta {
                    id
                } else {
                    error!("hash collision ocured: {meta:?} with hash {hash}");
                    utils::error::fatal_error();
                }
            }
            Entry::Vacant(v) => {
                let id = self.store.len();
                self.store.push(meta);
                v.insert(id);
                id
            }
        };

        self.identity_to_id.insert(identity, id);

        Ok(())
    }

    // TODO: iprove error handling here
    pub async fn load_from_upload_store(path: impl AsRef<Path>) -> anyhow::Result<MetadataStore> {
        info!("starting generating store from upload directory");

        let mut store = MetadataStore::default();

        let mut folder = fs::read_dir(&path).await?;
        while let Some(folder_entry) = folder.next_entry().await? {
            let folder_type = folder_entry.file_type().await?;
            if !folder_type.is_dir() {
                warn!("polluted upload folder");
                continue;
            }

            let folder_path = folder_entry.path();
            let folder_name = folder_entry.file_name();
            let folder_name = folder_name
                .to_str()
                .ok_or(anyhow!("failed to convert folder_name to str"))?;

            let mut files = fs::read_dir(&folder_path).await?;
            while let Some(file_entry) = files.next_entry().await? {
                let file_type = file_entry.file_type().await?;
                if !file_type.is_file() {
                    continue;
                }

                let file_path = file_entry.path();
                let file_name = file_entry.file_name();
                let file_name = file_name
                    .to_str()
                    .ok_or(anyhow!("failed to convert file_name to str"))?
                    .strip_suffix(".meta")
                    .ok_or(anyhow!("file_name doesn't end with meta"))?;

                if let Some(ext) = file_path.extension()
                    && ext == "meta"
                {
                    store
                        .add_meta_file(
                            file_path.as_ref(),
                            UploadIdentity::new(folder_name, file_name),
                        )
                        .await?;
                }
            }
        }

        Ok(store)
    }

    pub async fn add_meta_file(
        &mut self,
        path: &Path,
        identity: UploadIdentity,
    ) -> anyhow::Result<()> {
        trace!("adding metadata file {} to store", path.display());
        let meta = Metadata::load(path).await?;
        self.set_metadata(identity, meta)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test(name = "metadata_store")]
    pub async fn metadata_store() -> anyhow::Result<()> {
        eprintln!(">> Stage 1 : load metadata");
        let mut res = MetadataStore::load_from_upload_store(
            crate::utils::tests::test_asset_folder().join("metadata_store"),
        )
        .await?;
        eprintln!(">> Stage 2 : Verify loaded data");

        // Exist
        assert!(res.get_identity(&UploadIdentity::new("1", "1")).is_some());
        assert!(res.get_identity(&UploadIdentity::new("1", "2")).is_some());
        assert!(res.get_identity(&UploadIdentity::new("1", "3")).is_some());

        // Doesn't exist
        assert!(res.get_identity(&UploadIdentity::new("1", "4")).is_none());

        // Exist
        assert!(res.get_identity(&UploadIdentity::new("2", "1")).is_some());
        assert!(res.get_identity(&UploadIdentity::new("2", "2")).is_some());
        assert!(res.get_identity(&UploadIdentity::new("2", "3")).is_some());

        // Doesn't exist
        assert!(res.get_identity(&UploadIdentity::new("1", "5")).is_none());
        assert!(res.get_identity(&UploadIdentity::new("3", "3")).is_none());

        assert!(res.get_identity(&UploadIdentity::new("", "")).is_none());
        assert!(res.get_identity(&UploadIdentity::new("", "test")).is_none());

        eprintln!(">> Stage 3 : Update Entry");

        // Update Metadata of a UploadIdentity
        let new_set_meta = Metadata::new("test".into(), Some(".test".into()), 69);

        let meta = res
            .get_identity(&UploadIdentity::new("1", "1"))
            .unwrap()
            .clone();
        res.set_metadata(UploadIdentity::new("1", "1"), new_set_meta.clone())?;
        let new_meta = res.get_identity(&UploadIdentity::new("1", "1")).unwrap();

        assert_ne!(meta, *new_meta);
        assert_eq!(new_set_meta, *new_meta);

        Ok(())
    }
}
