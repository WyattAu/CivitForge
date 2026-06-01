#![forbid(unsafe_code)]

use crate::dedup::cdc::compute_digest;
use crate::dedup::chunkstore::ChunkStore;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Configuration for the S3-compatible object store.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct S3Config {
    pub endpoint: String,
    pub region: String,
    pub bucket: String,
    pub access_key: Option<String>,
    pub secret_key: Option<String>,
    pub prefix: String,
    pub max_concurrent_uploads: usize,
    pub part_size: usize,
}

impl Default for S3Config {
    fn default() -> Self {
        Self {
            endpoint: "http://localhost:9000".to_string(),
            region: "us-east-1".to_string(),
            bucket: "civitforge".to_string(),
            access_key: None,
            secret_key: None,
            prefix: String::new(),
            max_concurrent_uploads: 4,
            part_size: 8 * 1024 * 1024, // 8 MB
        }
    }
}

/// Object metadata returned from S3 operations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectMeta {
    pub key: String,
    pub size: u64,
    pub etag: String,
    pub last_modified: chrono::DateTime<chrono::Utc>,
    pub content_type: String,
    pub checksum: String,
}

/// Result of a multipart upload initiation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultipartUpload {
    pub upload_id: String,
    pub key: String,
    pub parts: Vec<MultipartPart>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultipartPart {
    pub part_number: u32,
    pub etag: String,
    pub size: u64,
}

/// Statistics for the object store.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectStoreStats {
    pub object_count: usize,
    pub total_size: u64,
    pub bucket: String,
}

/// Result of an upload operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadResult {
    pub key: String,
    pub etag: String,
    pub size: u64,
    pub deduplicated: bool,
}

/// Trait for object store operations. Implementations can back this with
/// real S3, MinIO, or in-memory storage for testing.
pub trait ObjectStore: Send + Sync {
    /// Upload an object. Returns the ETag and whether it was deduplicated.
    fn put_object(
        &self,
        key: &str,
        data: &[u8],
        content_type: &str,
    ) -> anyhow::Result<UploadResult>;

    /// Download an object by key.
    fn get_object(&self, key: &str) -> anyhow::Result<Vec<u8>>;

    /// Check if an object exists.
    fn object_exists(&self, key: &str) -> bool;

    /// Delete an object.
    fn delete_object(&self, key: &str) -> anyhow::Result<bool>;

    /// Get object metadata without downloading.
    fn head_object(&self, key: &str) -> anyhow::Result<ObjectMeta>;

    /// List objects with an optional prefix filter.
    fn list_objects(&self, prefix: &str) -> anyhow::Result<Vec<ObjectMeta>>;

    /// Get store statistics.
    fn stats(&self) -> anyhow::Result<ObjectStoreStats>;
}

/// In-memory object store implementation for testing and development.
pub struct InMemoryObjectStore {
    config: S3Config,
    objects: dashmap::DashMap<String, InMemoryObject>,
}

#[derive(Debug, Clone)]
struct InMemoryObject {
    data: Vec<u8>,
    etag: String,
    last_modified: chrono::DateTime<chrono::Utc>,
    content_type: String,
    checksum: String,
}

impl InMemoryObjectStore {
    pub fn new(config: S3Config) -> Self {
        Self {
            config,
            objects: dashmap::DashMap::new(),
        }
    }

    fn full_key(&self, key: &str) -> String {
        if self.config.prefix.is_empty() {
            key.to_string()
        } else {
            format!("{}/{key}", self.config.prefix)
        }
    }
}

impl ObjectStore for InMemoryObjectStore {
    fn put_object(
        &self,
        key: &str,
        data: &[u8],
        content_type: &str,
    ) -> anyhow::Result<UploadResult> {
        let full_key = self.full_key(key);
        let etag = compute_digest(data);
        let checksum = hex::encode(Sha256::digest(data));
        let deduplicated = self.objects.contains_key(&full_key);
        self.objects.insert(
            full_key.clone(),
            InMemoryObject {
                data: data.to_vec(),
                etag: etag.clone(),
                last_modified: chrono::Utc::now(),
                content_type: content_type.to_string(),
                checksum,
            },
        );
        Ok(UploadResult {
            key: full_key,
            etag,
            size: data.len() as u64,
            deduplicated,
        })
    }

    fn get_object(&self, key: &str) -> anyhow::Result<Vec<u8>> {
        let full_key = self.full_key(key);
        self.objects
            .get(&full_key)
            .map(|o| o.data.clone())
            .ok_or_else(|| anyhow::anyhow!("Object not found: {key}"))
    }

    fn object_exists(&self, key: &str) -> bool {
        self.objects.contains_key(&self.full_key(key))
    }

    fn delete_object(&self, key: &str) -> anyhow::Result<bool> {
        let full_key = self.full_key(key);
        Ok(self.objects.remove(&full_key).is_some())
    }

    fn head_object(&self, key: &str) -> anyhow::Result<ObjectMeta> {
        let full_key = self.full_key(key);
        let obj = self
            .objects
            .get(&full_key)
            .ok_or_else(|| anyhow::anyhow!("Object not found: {key}"))?;
        Ok(ObjectMeta {
            key: full_key,
            size: obj.data.len() as u64,
            etag: obj.etag.clone(),
            last_modified: obj.last_modified,
            content_type: obj.content_type.clone(),
            checksum: obj.checksum.clone(),
        })
    }

    fn list_objects(&self, prefix: &str) -> anyhow::Result<Vec<ObjectMeta>> {
        let full_prefix = self.full_key(prefix);
        let mut results = Vec::new();
        for entry in self.objects.iter() {
            if entry.key().starts_with(&full_prefix) {
                results.push(ObjectMeta {
                    key: entry.key().clone(),
                    size: entry.data.len() as u64,
                    etag: entry.etag.clone(),
                    last_modified: entry.last_modified,
                    content_type: entry.content_type.clone(),
                    checksum: entry.checksum.clone(),
                });
            }
        }
        Ok(results)
    }

    fn stats(&self) -> anyhow::Result<ObjectStoreStats> {
        let object_count = self.objects.len();
        let total_size: u64 = self.objects.iter().map(|o| o.data.len() as u64).sum();
        Ok(ObjectStoreStats {
            object_count,
            total_size,
            bucket: self.config.bucket.clone(),
        })
    }
}

/// Bridges the dedup ChunkStore with an ObjectStore for uploading
/// deduplicated chunks to S3/MinIO.
pub struct DedupObjectBridge {
    chunk_store: ChunkStore,
    object_store: Box<dyn ObjectStore>,
    prefix: String,
}

impl DedupObjectBridge {
    pub fn new(chunk_store: ChunkStore, object_store: Box<dyn ObjectStore>, prefix: &str) -> Self {
        Self {
            chunk_store,
            object_store,
            prefix: prefix.to_string(),
        }
    }

    /// Upload a file through the dedup store and then to object storage.
    pub fn upload_file(
        &self,
        file_id: &str,
        data: &[u8],
        content_type: &str,
    ) -> anyhow::Result<UploadResult> {
        let manifest = self.chunk_store.store_file(file_id, file_id, data);
        // Upload chunks individually for dedup
        let mut total_size = 0u64;
        for (i, chunk_id) in manifest.chunk_ids.iter().enumerate() {
            let key = format!("{}/{}/{}", self.prefix, file_id, i);
            if let Some(chunk_data) = self.chunk_store.get(chunk_id) {
                let result = self
                    .object_store
                    .put_object(&key, &chunk_data, content_type)?;
                total_size += result.size;
            }
        }
        // Upload manifest
        let manifest_json = serde_json::to_vec(&manifest)?;
        let manifest_key = format!("{}/{}/manifest.json", self.prefix, file_id);
        let manifest_result =
            self.object_store
                .put_object(&manifest_key, &manifest_json, "application/json")?;
        Ok(UploadResult {
            key: format!("{}/{file_id}", self.prefix),
            etag: manifest_result.etag,
            size: total_size + manifest_result.size,
            deduplicated: manifest.chunk_ids.len() < (data.len() / 8192).max(1),
        })
    }

    /// Reconstruct a file from object storage through the dedup store.
    pub fn download_file(&self, file_id: &str) -> anyhow::Result<Vec<u8>> {
        let manifest_key = format!("{}/{}/manifest.json", self.prefix, file_id);
        let manifest_data = self.object_store.get_object(&manifest_key)?;
        let manifest: crate::dedup::chunkstore::ChunkManifest =
            serde_json::from_slice(&manifest_data)?;
        self.chunk_store
            .reconstruct(&manifest)
            .ok_or_else(|| anyhow::anyhow!("Failed to reconstruct file: {file_id}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_store() -> InMemoryObjectStore {
        InMemoryObjectStore::new(S3Config::default())
    }

    #[test]
    fn test_put_and_get_object() {
        let store = test_store();
        store
            .put_object("test/key", b"hello world", "text/plain")
            .unwrap();
        let data = store.get_object("test/key").unwrap();
        assert_eq!(data, b"hello world");
    }

    #[test]
    fn test_put_with_prefix() {
        let config = S3Config {
            prefix: "chunks".into(),
            ..Default::default()
        };
        let store = InMemoryObjectStore::new(config);
        store
            .put_object("file1", b"data", "application/octet-stream")
            .unwrap();
        assert!(store.object_exists("file1"));
        assert!(store.get_object("file1").is_ok());
    }

    #[test]
    fn test_object_exists() {
        let store = test_store();
        assert!(!store.object_exists("missing"));
        store.put_object("present", b"data", "text/plain").unwrap();
        assert!(store.object_exists("present"));
    }

    #[test]
    fn test_delete_object() {
        let store = test_store();
        store
            .put_object("to-delete", b"data", "text/plain")
            .unwrap();
        assert!(store.delete_object("to-delete").unwrap());
        assert!(!store.object_exists("to-delete"));
        assert!(!store.delete_object("to-delete").unwrap());
    }

    #[test]
    fn test_head_object() {
        let store = test_store();
        store
            .put_object("meta-test", b"content", "application/json")
            .unwrap();
        let meta = store.head_object("meta-test").unwrap();
        assert_eq!(meta.size, 7);
        assert_eq!(meta.content_type, "application/json");
    }

    #[test]
    fn test_list_objects() {
        let store = test_store();
        store.put_object("repo/a.txt", b"a", "text/plain").unwrap();
        store.put_object("repo/b.txt", b"b", "text/plain").unwrap();
        store.put_object("other/c.txt", b"c", "text/plain").unwrap();
        let objects = store.list_objects("repo/").unwrap();
        assert_eq!(objects.len(), 2);
    }

    #[test]
    fn test_stats() {
        let store = test_store();
        store.put_object("a", b"hello", "text/plain").unwrap();
        store.put_object("b", b"world!!", "text/plain").unwrap();
        let stats = store.stats().unwrap();
        assert_eq!(stats.object_count, 2);
        assert_eq!(stats.total_size, 12);
    }

    #[test]
    fn test_dedup_upload() {
        let chunk_store = ChunkStore::new();
        let object_store = Box::new(test_store());
        let bridge = DedupObjectBridge::new(chunk_store, object_store, "uploads");
        let data = b"The quick brown fox jumps over the lazy dog".repeat(100);
        let result = bridge
            .upload_file("file1", &data, "application/octet-stream")
            .unwrap();
        assert!(result.size > 0);
    }

    #[test]
    fn test_dedup_download_roundtrip() {
        let chunk_store = ChunkStore::new();
        let object_store = Box::new(test_store());
        let bridge = DedupObjectBridge::new(chunk_store, object_store, "uploads");
        let data = b"reconstruct test data that should round trip correctly".repeat(50);
        bridge
            .upload_file("roundtrip-file", &data, "application/octet-stream")
            .unwrap();
        let reconstructed = bridge.download_file("roundtrip-file").unwrap();
        assert_eq!(reconstructed, data);
    }

    #[test]
    fn test_object_meta_serialization() {
        let meta = ObjectMeta {
            key: "test/key".into(),
            size: 100,
            etag: "abc123".into(),
            last_modified: chrono::Utc::now(),
            content_type: "text/plain".into(),
            checksum: "sha256:xyz".into(),
        };
        let json = serde_json::to_string(&meta).unwrap();
        let de: ObjectMeta = serde_json::from_str(&json).unwrap();
        assert_eq!(de.key, "test/key");
    }

    #[test]
    fn test_s3_config_default() {
        let config = S3Config::default();
        assert_eq!(config.endpoint, "http://localhost:9000");
        assert_eq!(config.region, "us-east-1");
        assert_eq!(config.part_size, 8 * 1024 * 1024);
    }

    #[test]
    fn test_multipart_upload_serialization() {
        let upload = MultipartUpload {
            upload_id: "up-123".into(),
            key: "test/key".into(),
            parts: vec![MultipartPart {
                part_number: 1,
                etag: "e1".into(),
                size: 100,
            }],
        };
        let json = serde_json::to_string(&upload).unwrap();
        let de: MultipartUpload = serde_json::from_str(&json).unwrap();
        assert_eq!(de.upload_id, "up-123");
    }

    #[test]
    fn test_upload_result_serialization() {
        let result = UploadResult {
            key: "k".into(),
            etag: "e".into(),
            size: 50,
            deduplicated: false,
        };
        let json = serde_json::to_string(&result).unwrap();
        let de: UploadResult = serde_json::from_str(&json).unwrap();
        assert!(!de.deduplicated);
    }
}
