/*
 * kystash - A simple image/file sharing server/client
 * Copyright (C) 2026 2kybe3 <kybe@kybe.xyz>
 */

use crate::{shared::metadata::Metadata, utils};
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
}

impl MetadataStore {
    pub fn get(&self, id: MetadataId) -> &Metadata {
        &self.store[id]
    }

    pub fn get_or_insert(&mut self, meta: Metadata) -> anyhow::Result<MetadataId> {
        let hash = xxh3_128(&postcard::to_stdvec(&meta)?);

        match self.meta_to_id.entry(hash) {
            Entry::Occupied(mut o) => {
                let id = *o.get();
                if self.store[id] == meta {
                    Ok(id)
                } else {
                    error!("hash collision ocured: {meta:?} with hash {hash}");
                    utils::error::fatal_error_no_exit();
                    self.store.push(meta);
                    let new_id = self.store.len();
                    o.insert(new_id);
                    Ok(new_id)
                }
            }
            Entry::Vacant(v) => {
                let id = self.store.len();
                self.store.push(meta);
                v.insert(id);
                Ok(id)
            }
        }
    }

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

            let mut files = fs::read_dir(&folder_path).await?;
            while let Some(file_entry) = files.next_entry().await? {
                let file_type = file_entry.file_type().await?;
                if !file_type.is_file() {
                    continue;
                }

                let path = file_entry.path();

                if let Some(ext) = path.extension()
                    && ext == "meta"
                {
                    store.add_meta_file(path).await?;
                }
            }
        }

        Ok(store)
    }

    pub async fn add_meta_file(&mut self, path: impl AsRef<Path>) -> anyhow::Result<()> {
        trace!("adding metadata file {} to store", path.as_ref().display());
        let meta = Metadata::load(&path).await?;
        self.get_or_insert(meta)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test(name = "metadata_store_load")]
    pub async fn metadata_store_load() -> anyhow::Result<()> {
        let _ = MetadataStore::load_from_upload_store(
            crate::utils::tests::test_asset_folder().join("metadata_store"),
        )
        .await?;
        Ok(())
    }
}
