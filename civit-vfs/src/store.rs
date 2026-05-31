#![forbid(unsafe_code)]

use dashmap::DashMap;
use std::collections::HashMap;
use tracing::debug;

pub struct BlockStore {
    blocks: DashMap<String, Vec<u8>>,
    block_size: usize,
}

pub struct BlockRef {
    pub repo_id: String,
    pub commit_sha: String,
    pub path: String,
    pub size: u64,
    pub block_map: Vec<BlockEntry>,
}

pub struct BlockEntry {
    pub offset: u64,
    pub size: u32,
    pub block_id: String,
}

impl BlockStore {
    pub fn new(block_size: usize) -> Self {
        Self {
            blocks: DashMap::new(),
            block_size,
        }
    }

    fn content_hash(data: &[u8]) -> String {
        use sha2::{Digest, Sha256};
        let hash = Sha256::digest(data);
        hex::encode(hash)
    }

    pub fn put_block(&self, data: &[u8]) -> String {
        let id = Self::content_hash(data);
        self.blocks.insert(id.clone(), data.to_vec());
        debug!(block_id = %id, size = data.len(), "stored block");
        id
    }

    pub fn get_block(&self, id: &str) -> Option<Vec<u8>> {
        self.blocks.get(id).map(|b| b.value().clone())
    }

    pub fn delete_block(&self, id: &str) -> bool {
        if self.blocks.remove(id).is_some() {
            debug!(block_id = %id, "deleted block");
            true
        } else {
            false
        }
    }

    pub fn store_file(&self, file_ref: &BlockRef, data: &[u8]) -> Vec<BlockEntry> {
        let mut offset = 0u64;
        let mut block_map = Vec::new();

        while offset < data.len() as u64 {
            let end = std::cmp::min(offset + self.block_size as u64, data.len() as u64);
            let chunk = &data[offset as usize..end as usize];
            let block_id = self.put_block(chunk);
            block_map.push(BlockEntry {
                offset,
                size: (end - offset) as u32,
                block_id,
            });
            offset = end;
        }

        debug!(
            repo_id = %file_ref.repo_id,
            path = %file_ref.path,
            blocks = block_map.len(),
            "stored file in blocks"
        );

        block_map
    }

    pub fn read_file(
        &self,
        file_ref: &BlockRef,
        offset: u64,
        size: usize,
    ) -> Result<Vec<u8>, String> {
        let mut result = Vec::with_capacity(size);
        let mut remaining = size as u64;
        let mut file_offset = offset;

        for entry in &file_ref.block_map {
            if remaining == 0 {
                break;
            }
            let block_start = entry.offset;
            let block_end = block_start + entry.size as u64;

            if file_offset >= block_end {
                continue;
            }

            let read_start = file_offset.saturating_sub(block_start);
            let read_end = std::cmp::min(entry.size as u64, read_start + remaining) as usize;

            if let Some(block_data) = self.get_block(&entry.block_id) {
                let chunk = &block_data[read_start as usize..read_end];
                result.extend_from_slice(chunk);
                remaining -= chunk.len() as u64;
            }
            file_offset = block_end;
        }

        Ok(result)
    }

    pub fn sparse_checkout(
        &self,
        file_ref: &BlockRef,
        paths: &[String],
    ) -> HashMap<String, Vec<u8>> {
        let mut result = HashMap::new();
        // Reconstruct the full file content from blocks
        let mut file_data = Vec::new();
        for entry in &file_ref.block_map {
            if let Some(block_data) = self.get_block(&entry.block_id) {
                file_data.extend_from_slice(&block_data);
            }
        }
        // Return reconstructed data for each requested path
        for path in paths {
            let full_path = format!("{}{}", file_ref.path, path);
            if !file_data.is_empty() {
                result.insert(full_path, file_data.clone());
            }
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_store() -> BlockStore {
        BlockStore::new(16)
    }

    fn make_block_ref() -> BlockRef {
        BlockRef {
            repo_id: "repo-1".into(),
            commit_sha: "abc123".into(),
            path: "/src/".into(),
            size: 0,
            block_map: Vec::new(),
        }
    }

    #[test]
    fn test_put_and_get_block() {
        let store = make_store();
        let id = store.put_block(b"hello world");
        let data = store.get_block(&id).unwrap();
        assert_eq!(data, b"hello world");
    }

    #[test]
    fn test_put_block_deterministic_id() {
        let store = make_store();
        let id1 = store.put_block(b"test data");
        let id2 = store.put_block(b"test data");
        assert_eq!(id1, id2);
    }

    #[test]
    fn test_get_block_missing() {
        let store = make_store();
        assert!(store.get_block("nonexistent").is_none());
    }

    #[test]
    fn test_delete_block() {
        let store = make_store();
        let id = store.put_block(b"delete me");
        assert!(store.delete_block(&id));
        assert!(store.get_block(&id).is_none());
    }

    #[test]
    fn test_delete_block_missing() {
        let store = make_store();
        assert!(!store.delete_block("nonexistent"));
    }

    #[test]
    fn test_store_file_single_block() {
        let store = make_store();
        let file_ref = make_block_ref();
        let data = b"hello";
        let block_map = store.store_file(&file_ref, data);
        assert_eq!(block_map.len(), 1);
    }

    #[test]
    fn test_store_file_multiple_blocks() {
        let store = make_store();
        let file_ref = make_block_ref();
        let data = [0u8; 48];
        let block_map = store.store_file(&file_ref, &data);
        assert_eq!(block_map.len(), 3);
        assert_eq!(block_map[0].size, 16);
        assert_eq!(block_map[1].size, 16);
        assert_eq!(block_map[2].size, 16);
    }

    #[test]
    fn test_store_file_exact_block_size() {
        let store = make_store();
        let file_ref = make_block_ref();
        let data = [0u8; 16];
        let block_map = store.store_file(&file_ref, &data);
        assert_eq!(block_map.len(), 1);
        assert_eq!(block_map[0].size, 16);
    }

    #[test]
    fn test_read_file_whole() {
        let store = make_store();
        let file_ref = make_block_ref();
        let data = b"hello world from test";
        let block_map = store.store_file(&file_ref, data);
        let file_ref = BlockRef {
            block_map,
            ..file_ref
        };
        let result = store.read_file(&file_ref, 0, data.len()).unwrap();
        assert_eq!(result, data);
    }

    #[test]
    fn test_read_file_partial() {
        let store = make_store();
        let file_ref = make_block_ref();
        let data = b"hello world from test";
        let block_map = store.store_file(&file_ref, data);
        let file_ref = BlockRef {
            block_map,
            ..file_ref
        };
        let result = store.read_file(&file_ref, 6, 5).unwrap();
        assert_eq!(result, b"world");
    }

    #[test]
    fn test_read_file_cross_block() {
        let store = make_store();
        let file_ref = make_block_ref();
        let data = [0u8; 32];
        let block_map = store.store_file(&file_ref, &data);
        let file_ref = BlockRef {
            block_map,
            ..file_ref
        };
        let result = store.read_file(&file_ref, 14, 4).unwrap();
        assert_eq!(result.len(), 4);
    }

    #[test]
    fn test_sparse_checkout() {
        let store = make_store();
        let file_ref = make_block_ref();
        let data = b"some file content here";
        let block_map = store.store_file(&file_ref, data);
        let file_ref = BlockRef {
            block_map,
            ..file_ref
        };
        let paths = vec!["main.rs".into(), "lib.rs".into()];
        let result = store.sparse_checkout(&file_ref, &paths);
        assert_eq!(result.len(), 2);
        assert!(result.contains_key("/src/main.rs"));
        assert!(result.contains_key("/src/lib.rs"));
    }
}
