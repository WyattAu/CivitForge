#![forbid(unsafe_code)]

use crate::dedup::cdc::{CdcConfig, FastCdc, compute_digest};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkManifest {
    pub file_id: String,
    pub file_name: String,
    pub total_size: u64,
    pub chunk_ids: Vec<String>,
    pub chunk_sizes: Vec<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoreStats {
    pub chunk_count: usize,
    pub total_bytes: u64,
    pub dedup_savings: u64,
    pub reference_count: usize,
    pub dedup_ratio: f64,
}

pub struct ChunkStore {
    chunks: DashMap<String, Vec<u8>>,
    references: DashMap<String, u64>,
    total_bytes: AtomicU64,
    dedup_savings: AtomicU64,
}

impl Default for ChunkStore {
    fn default() -> Self {
        Self::new()
    }
}

impl ChunkStore {
    pub fn new() -> Self {
        Self {
            chunks: DashMap::new(),
            references: DashMap::new(),
            total_bytes: AtomicU64::new(0),
            dedup_savings: AtomicU64::new(0),
        }
    }

    pub fn put(&self, data: &[u8]) -> String {
        let id = compute_digest(data);
        if let Some(mut entry) = self.chunks.get_mut(&id) {
            *entry.value_mut() = data.to_vec();
            self.references
                .entry(id.clone())
                .and_modify(|r| *r += 1)
                .or_insert(1);
            self.dedup_savings
                .fetch_add(data.len() as u64, Ordering::Relaxed);
            return id;
        }
        self.chunks.insert(id.clone(), data.to_vec());
        self.references
            .entry(id.clone())
            .and_modify(|r| *r += 1)
            .or_insert(1);
        self.total_bytes
            .fetch_add(data.len() as u64, Ordering::Relaxed);
        id
    }

    /// Store data keyed by a caller-specified id (for layer dedup where the
    /// layer digest is the canonical key).
    pub fn put_direct(&self, id: &str, data: &[u8]) {
        if let Some(mut entry) = self.chunks.get_mut(id) {
            *entry.value_mut() = data.to_vec();
            self.references
                .entry(id.to_string())
                .and_modify(|r| *r += 1)
                .or_insert(1);
            self.dedup_savings
                .fetch_add(data.len() as u64, Ordering::Relaxed);
        } else {
            self.chunks.insert(id.to_string(), data.to_vec());
            self.references
                .entry(id.to_string())
                .and_modify(|r| *r += 1)
                .or_insert(1);
            self.total_bytes
                .fetch_add(data.len() as u64, Ordering::Relaxed);
        }
    }

    pub fn get(&self, id: &str) -> Option<Vec<u8>> {
        self.chunks.get(id).map(|v| v.value().clone())
    }

    pub fn delete(&self, id: &str) -> bool {
        if let Some((_, data)) = self.chunks.remove(id) {
            self.references.remove(id);
            self.total_bytes
                .fetch_sub(data.len() as u64, Ordering::Relaxed);
            true
        } else {
            false
        }
    }

    pub fn store_file(&self, file_id: &str, file_name: &str, data: &[u8]) -> ChunkManifest {
        let cdc = FastCdc::new(CdcConfig::default());
        let result = cdc.chunk_data(data);
        let mut chunk_ids = Vec::with_capacity(result.chunks.len());
        let mut chunk_sizes = Vec::with_capacity(result.chunks.len());

        for chunk in &result.chunks {
            let chunk_data = &data[chunk.offset as usize..(chunk.offset as usize + chunk.size)];
            let id = self.put(chunk_data);
            chunk_ids.push(id);
            chunk_sizes.push(chunk.size);
        }

        ChunkManifest {
            file_id: file_id.to_string(),
            file_name: file_name.to_string(),
            total_size: result.total_size,
            chunk_ids,
            chunk_sizes,
        }
    }

    pub fn reconstruct(&self, manifest: &ChunkManifest) -> Option<Vec<u8>> {
        let mut buf = Vec::with_capacity(manifest.total_size as usize);
        for (id, &size) in manifest.chunk_ids.iter().zip(manifest.chunk_sizes.iter()) {
            let data = self.get(id)?;
            buf.extend_from_slice(&data[..size]);
        }
        Some(buf)
    }

    pub fn garbage_collect(&self, referenced_ids: HashSet<String>) -> usize {
        let all_keys: Vec<String> = self.chunks.iter().map(|e| e.key().clone()).collect();
        let mut removed = 0;
        for key in all_keys {
            if !referenced_ids.contains(&key) {
                if let Some((_, data)) = self.chunks.remove(&key) {
                    self.total_bytes
                        .fetch_sub(data.len() as u64, Ordering::Relaxed);
                    self.references.remove(&key);
                    removed += 1;
                }
            }
        }
        removed
    }

    pub fn stats(&self) -> StoreStats {
        let chunk_count = self.chunks.len();
        let total_bytes = self.total_bytes.load(Ordering::Relaxed);
        let dedup_savings = self.dedup_savings.load(Ordering::Relaxed);
        let reference_count = self.references.len();
        let dedup_ratio = if total_bytes > 0 {
            dedup_savings as f64 / (total_bytes + dedup_savings) as f64
        } else {
            0.0
        };
        StoreStats {
            chunk_count,
            total_bytes,
            dedup_savings,
            reference_count,
            dedup_ratio,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_put_and_get() {
        let store = ChunkStore::new();
        let data = b"hello world";
        let id = store.put(data);
        let retrieved = store.get(&id).unwrap();
        assert_eq!(retrieved, data);
    }

    #[test]
    fn test_deduplication() {
        let store = ChunkStore::new();
        let data = b"hello world";
        let id1 = store.put(data);
        let id2 = store.put(data);
        assert_eq!(id1, id2);
        let stats = store.stats();
        assert!(stats.dedup_savings > 0);
    }

    #[test]
    fn test_delete() {
        let store = ChunkStore::new();
        let id = store.put(b"test data");
        assert!(store.delete(&id));
        assert!(store.get(&id).is_none());
        assert!(!store.delete(&id));
    }

    #[test]
    fn test_store_file_and_reconstruct() {
        let store = ChunkStore::new();
        let data = b"The quick brown fox jumps over the lazy dog".repeat(100);
        let manifest = store.store_file("file1", "test.txt", &data);
        assert_eq!(manifest.total_size, data.len() as u64);
        let reconstructed = store.reconstruct(&manifest).unwrap();
        assert_eq!(reconstructed, data);
    }

    #[test]
    fn test_garbage_collect() {
        let store = ChunkStore::new();
        let id_a = store.put(b"keep this");
        let id_b = store.put(b"remove this");
        let referenced: HashSet<String> = [id_a.clone()].into_iter().collect();
        let removed = store.garbage_collect(referenced);
        assert_eq!(removed, 1);
        assert!(store.get(&id_a).is_some());
        assert!(store.get(&id_b).is_none());
    }

    #[test]
    fn test_stats() {
        let store = ChunkStore::new();
        store.put(b"hello");
        store.put(b"world");
        let stats = store.stats();
        assert_eq!(stats.chunk_count, 2);
        assert_eq!(stats.total_bytes, 10);
    }

    #[test]
    fn test_empty_reconstruct() {
        let store = ChunkStore::new();
        let manifest = ChunkManifest {
            file_id: "empty".into(),
            file_name: "empty.txt".into(),
            total_size: 0,
            chunk_ids: vec![],
            chunk_sizes: vec![],
        };
        let result = store.reconstruct(&manifest).unwrap();
        assert!(result.is_empty());
    }
}
