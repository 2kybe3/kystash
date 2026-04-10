/*
 * kystash - A simple image/file sharing server/client
 * Copyright (C) 2026 2kybe3 <kybe@kybe.xyz>
 */

use std::{
    collections::HashMap,
    ops::{Deref, DerefMut},
};

use bitvec::vec::BitVec;
use tracing::warn;

use crate::shared::UploadIdentity;

pub struct ChunkMap(HashMap<UploadIdentity, BitVec>);

impl ChunkMap {
    pub fn new() -> Self {
        ChunkMap(HashMap::new())
    }

    pub fn set_finished_chunk(&mut self, identity: &UploadIdentity, index: usize, total: usize) {
        if let Some(bv) = self.0.get_mut(identity) {
            if bv.len() != total {
                warn!("size of chunk map changed. resizing.");
                bv.resize(total, false);
            }
            bv.set(index, true);
        } else {
            let mut bv = BitVec::repeat(false, total);
            bv.set(index, true);
            self.0.insert(identity.clone(), bv);
        }
    }

    #[allow(unused)]
    pub fn is_finished(&self, identity: &UploadIdentity) -> bool {
        let Some(vec) = self.0.get(identity) else {
            return false;
        };

        vec.iter().filter(|v| *v == false).count() == 0
    }
}

impl From<HashMap<UploadIdentity, BitVec>> for ChunkMap {
    fn from(value: HashMap<UploadIdentity, BitVec>) -> Self {
        ChunkMap(value)
    }
}

impl Deref for ChunkMap {
    type Target = HashMap<UploadIdentity, BitVec>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ChunkMap {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test(name = "chunk_map_finished")]
    pub async fn chunk_map_finished() {
        let mut map = ChunkMap::new();
        let identity = UploadIdentity::new("test", "test");
        for i in 0..10 {
            map.set_finished_chunk(&identity, i as usize, 10);
        }
        assert!(map.is_finished(&identity))
    }

    #[tokio::test(name = "chunk_map_random_resize_finished")]
    pub async fn chunk_map_random_resize_finished() {
        let mut map = ChunkMap::new();
        let identity = UploadIdentity::new("test", "test");
        for i in 0..10 {
            map.set_finished_chunk(&identity, i as usize, 10);
        }
        assert!(map.is_finished(&identity));
        map.set_finished_chunk(&identity, 10, 20);
        assert!(!map.is_finished(&identity));
        for i in 11..20 {
            map.set_finished_chunk(&identity, i as usize, 20);
        }
        assert!(map.is_finished(&identity));
    }
}
