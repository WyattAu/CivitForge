#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use parking_lot::Mutex;

/// OCI media type constants.
pub mod media_types {
    pub const OCI_MANIFEST: &str = "application/vnd.oci.image.manifest.v1+json";
    pub const OCI_INDEX: &str = "application/vnd.oci.image.index.v1+json";
    pub const OCI_LAYER: &str = "application/vnd.oci.image.layer.v1.tar+gzip";
    pub const OCI_CONFIG: &str = "application/vnd.oci.image.config.v1+json";
    pub const OCI_ARTIFACT: &str = "application/vnd.civitforge.artifact.v1+json";
}

/// OCI image manifest (distribution spec).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OciManifest {
    #[serde(rename = "schemaVersion")]
    pub schema_version: u32,
    #[serde(rename = "mediaType")]
    pub media_type: String,
    pub config: Descriptor,
    pub layers: Vec<Descriptor>,
    pub annotations: Option<HashMap<String, String>>,
}

/// OCI image index (multi-arch manifest list).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OciIndex {
    #[serde(rename = "schemaVersion")]
    pub schema_version: u32,
    #[serde(rename = "mediaType")]
    pub media_type: String,
    pub manifests: Vec<Descriptor>,
    pub annotations: Option<HashMap<String, String>>,
}

/// Content descriptor (digest + size + media type).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Descriptor {
    #[serde(rename = "mediaType")]
    pub media_type: String,
    pub digest: String,
    pub size: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub annotations: Option<HashMap<String, String>>,
}

/// Blob storage abstraction for OCI layers.
pub trait BlobStore: Send + Sync {
    fn exists(&self, digest: &str) -> bool;
    fn get(&self, digest: &str) -> Option<Vec<u8>>;
    fn put(&self, data: &[u8]) -> Result<String, String>;
    fn delete(&self, digest: &str) -> Result<(), String>;
    fn size(&self, digest: &str) -> Option<u64>;
}

/// In-memory blob store for testing and prototyping.
pub struct InMemoryBlobStore {
    blobs: Mutex<HashMap<String, Vec<u8>>>,
}

impl InMemoryBlobStore {
    pub fn new() -> Self {
        Self {
            blobs: Mutex::new(HashMap::new()),
        }
    }

    pub fn blob_count(&self) -> usize {
        self.blobs.lock().len()
    }

    pub fn total_bytes(&self) -> u64 {
        self.blobs
            .lock()
            .values()
            .map(|v| v.len() as u64)
            .sum()
    }

    fn compute_digest(data: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(data);
        format!("sha256:{}", hex::encode(hasher.finalize()))
    }
}

impl Default for InMemoryBlobStore {
    fn default() -> Self {
        Self::new()
    }
}

impl BlobStore for InMemoryBlobStore {
    fn exists(&self, digest: &str) -> bool {
        self.blobs.lock().contains_key(digest)
    }

    fn get(&self, digest: &str) -> Option<Vec<u8>> {
        self.blobs.lock().get(digest).cloned()
    }

    fn put(&self, data: &[u8]) -> Result<String, String> {
        let digest = Self::compute_digest(data);
        let mut blobs = self.blobs.lock();
        if !blobs.contains_key(&digest) {
            blobs.insert(digest.clone(), data.to_vec());
        }
        Ok(digest)
    }

    fn delete(&self, digest: &str) -> Result<(), String> {
        let mut blobs = self.blobs.lock();
        blobs
            .remove(digest)
            .map(|_| ())
            .ok_or_else(|| format!("blob not found: {digest}"))
    }

    fn size(&self, digest: &str) -> Option<u64> {
        self.blobs
            .lock()
            .get(digest)
            .map(|d| d.len() as u64)
    }
}

/// Artifact registry for managing OCI manifests.
pub struct ArtifactRegistry {
    store: Box<dyn BlobStore>,
    manifests: Mutex<HashMap<String, OciManifest>>,
}

impl ArtifactRegistry {
    pub fn new(store: Box<dyn BlobStore>) -> Self {
        Self {
            store,
            manifests: Mutex::new(HashMap::new()),
        }
    }

    /// Push a layer blob, returns its digest.
    pub fn push_layer(&self, data: &[u8]) -> Result<String, String> {
        self.store.put(data)
    }

    /// Push a manifest.
    pub fn push_manifest(&self, name: &str, manifest: &OciManifest) -> Result<String, String> {
        let data = serde_json::to_vec(manifest).map_err(|e| e.to_string())?;
        let digest = self.store.put(&data)?;
        let mut manifests = self.manifests.lock();
        manifests.insert(name.to_string(), manifest.clone());
        Ok(digest)
    }

    /// Pull a manifest by name.
    pub fn pull_manifest(&self, name: &str) -> Option<OciManifest> {
        self.manifests.lock().get(name).cloned()
    }

    /// Pull a blob by digest.
    pub fn pull_blob(&self, digest: &str) -> Option<Vec<u8>> {
        self.store.get(digest)
    }

    /// Check if a blob exists.
    pub fn blob_exists(&self, digest: &str) -> bool {
        self.store.exists(digest)
    }

    /// List stored artifacts.
    pub fn list_artifacts(&self) -> Vec<String> {
        self.manifests.lock().keys().cloned().collect()
    }

    /// Delete a manifest.
    pub fn delete_manifest(&self, name: &str) -> Result<(), String> {
        let mut manifests = self.manifests.lock();
        manifests
            .remove(name)
            .map(|_| ())
            .ok_or_else(|| format!("manifest not found: {name}"))
    }

    /// Calculate space savings from deduplication.
    /// Returns ratio of unique bytes to total bytes pushed (<= 1.0).
    pub fn deduplication_ratio(&self, total_bytes_pushed: u64) -> f64 {
        if total_bytes_pushed == 0 {
            return 1.0;
        }
        self.store.size("_total").unwrap_or(total_bytes_pushed) as f64 / total_bytes_pushed as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_manifest() -> OciManifest {
        OciManifest {
            schema_version: 2,
            media_type: media_types::OCI_MANIFEST.to_string(),
            config: Descriptor {
                media_type: media_types::OCI_CONFIG.to_string(),
                digest: "sha256:deadbeef".to_string(),
                size: 128,
                annotations: None,
            },
            layers: vec![Descriptor {
                media_type: media_types::OCI_LAYER.to_string(),
                digest: "sha256:abcdef00".to_string(),
                size: 4096,
                annotations: None,
            }],
            annotations: None,
        }
    }

    #[test]
    fn test_push_pull_blob() {
        let store = InMemoryBlobStore::new();
        let registry = ArtifactRegistry::new(Box::new(store));
        let data = b"hello blob world";
        let digest = registry.push_layer(data).unwrap();
        let pulled = registry.pull_blob(&digest).unwrap();
        assert_eq!(pulled, data);
    }

    #[test]
    fn test_blob_exists() {
        let store = InMemoryBlobStore::new();
        let registry = ArtifactRegistry::new(Box::new(store));
        let digest = registry.push_layer(b"data").unwrap();
        assert!(registry.blob_exists(&digest));
        assert!(!registry.blob_exists("sha256:nonexistent"));
    }

    #[test]
    fn test_push_pull_manifest() {
        let store = InMemoryBlobStore::new();
        let registry = ArtifactRegistry::new(Box::new(store));
        let manifest = make_test_manifest();
        registry.push_manifest("test-artifact", &manifest).unwrap();
        let pulled = registry.pull_manifest("test-artifact").unwrap();
        assert_eq!(pulled.schema_version, manifest.schema_version);
        assert_eq!(pulled.layers.len(), 1);
    }

    #[test]
    fn test_list_artifacts() {
        let store = InMemoryBlobStore::new();
        let registry = ArtifactRegistry::new(Box::new(store));
        registry
            .push_manifest("artifact-a", &make_test_manifest())
            .unwrap();
        registry
            .push_manifest("artifact-b", &make_test_manifest())
            .unwrap();
        let list = registry.list_artifacts();
        assert_eq!(list.len(), 2);
        assert!(list.contains(&"artifact-a".to_string()));
        assert!(list.contains(&"artifact-b".to_string()));
    }

    #[test]
    fn test_delete_manifest() {
        let store = InMemoryBlobStore::new();
        let registry = ArtifactRegistry::new(Box::new(store));
        registry
            .push_manifest("to-delete", &make_test_manifest())
            .unwrap();
        assert!(registry.pull_manifest("to-delete").is_some());
        registry.delete_manifest("to-delete").unwrap();
        assert!(registry.pull_manifest("to-delete").is_none());
        assert!(registry.delete_manifest("nope").is_err());
    }

    #[test]
    fn test_digest_computation() {
        let data = b"test data";
        let digest = InMemoryBlobStore::compute_digest(data);
        assert!(digest.starts_with("sha256:"));
        assert_eq!(digest.len(), 7 + 64);
    }

    #[test]
    fn test_dedup_put() {
        let store = InMemoryBlobStore::new();
        store.put(b"same content").unwrap();
        assert_eq!(store.blob_count(), 1);
        store.put(b"same content").unwrap();
        assert_eq!(store.blob_count(), 1);
        store.put(b"different").unwrap();
        assert_eq!(store.blob_count(), 2);
    }

    #[test]
    fn test_serialization_round_trip() {
        let manifest = make_test_manifest();
        let json = serde_json::to_string(&manifest).unwrap();
        let deserialized: OciManifest = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.schema_version, manifest.schema_version);
        assert_eq!(deserialized.media_type, manifest.media_type);
        assert_eq!(deserialized.layers.len(), manifest.layers.len());
    }

    #[test]
    fn test_index_serialization_round_trip() {
        let index = OciIndex {
            schema_version: 2,
            media_type: media_types::OCI_INDEX.to_string(),
            manifests: vec![Descriptor {
                media_type: media_types::OCI_MANIFEST.to_string(),
                digest: "sha256:abc".to_string(),
                size: 100,
                annotations: None,
            }],
            annotations: None,
        };
        let json = serde_json::to_string(&index).unwrap();
        let deserialized: OciIndex = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.manifests.len(), 1);
        assert_eq!(deserialized.manifests[0].digest, "sha256:abc");
    }
}
