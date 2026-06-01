#![forbid(unsafe_code)]

use crate::dedup::chunkstore::ChunkStore;
use crate::oci::manifest::OciManifest;
use serde::{Deserialize, Serialize};

/// Tracks layer content across manifests for deduplication.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayerRecord {
    pub digest: String,
    pub size: u64,
    pub media_type: String,
    pub manifest_ids: Vec<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Statistics for the layer deduplication system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DedupStats {
    pub total_layers: usize,
    pub unique_layers: usize,
    pub deduplicated_layers: usize,
    pub space_savings_bytes: u64,
    pub dedup_ratio: f64,
}

/// Manages OCI layer deduplication across multiple manifests.
pub struct LayerDedupManager {
    layers: dashmap::DashMap<String, LayerRecord>,
    chunk_store: ChunkStore,
}

impl Default for LayerDedupManager {
    fn default() -> Self {
        Self::new()
    }
}

impl LayerDedupManager {
    pub fn new() -> Self {
        Self {
            layers: dashmap::DashMap::new(),
            chunk_store: ChunkStore::new(),
        }
    }

    /// Register a manifest and track its layers. Returns a deduplication report.
    pub fn register_manifest(&self, manifest_id: &str, manifest: &OciManifest) -> DedupReport {
        let mut dedup_count = 0usize;
        let mut space_savings = 0u64;
        let now = chrono::Utc::now();

        for layer in &manifest.layers {
            if let Some(mut record) = self.layers.get_mut(&layer.digest) {
                record.manifest_ids.push(manifest_id.to_string());
                dedup_count += 1;
                space_savings += layer.size;
            } else {
                let record = LayerRecord {
                    digest: layer.digest.clone(),
                    size: layer.size,
                    media_type: layer.media_type.clone(),
                    manifest_ids: vec![manifest_id.to_string()],
                    created_at: now,
                };
                self.layers.insert(layer.digest.clone(), record);
            }
        }

        DedupReport {
            manifest_id: manifest_id.to_string(),
            total_layers: manifest.layers.len(),
            deduplicated_layers: dedup_count,
            space_savings_bytes: space_savings,
        }
    }

    /// Check if a layer already exists in the dedup store.
    pub fn layer_exists(&self, digest: &str) -> bool {
        self.layers.contains_key(digest)
    }

    /// Get the reference count for a layer.
    pub fn layer_ref_count(&self, digest: &str) -> usize {
        self.layers
            .get(digest)
            .map(|r| r.manifest_ids.len())
            .unwrap_or(0)
    }

    /// Store a layer blob, deduplicating if it already exists.
    pub fn store_layer(&self, digest: &str, data: &[u8]) -> bool {
        if self.layer_exists(digest) {
            return false; // Already stored, no-op
        }
        self.chunk_store.put(data);
        true
    }

    /// Retrieve a layer blob.
    pub fn get_layer(&self, digest: &str) -> Option<Vec<u8>> {
        if !self.layer_exists(digest) {
            return None;
        }
        // In a real implementation, we would look up the chunk ID from the digest
        // For now, return None since we don't have a direct digest-to-chunk mapping
        self.chunk_store.get(digest)
    }

    /// Remove a manifest's references and garbage-collect orphaned layers.
    pub fn unregister_manifest(&self, manifest_id: &str) -> GarbageCollectResult {
        let mut orphaned = Vec::new();
        let mut remaining = Vec::new();
        for mut entry in self.layers.iter_mut() {
            let record = entry.value_mut();
            let before = record.manifest_ids.len();
            record.manifest_ids.retain(|id| id != manifest_id);
            if record.manifest_ids.is_empty() {
                orphaned.push(record.digest.clone());
            } else if record.manifest_ids.len() != before {
                remaining.push(record.digest.clone());
            }
        }
        let mut removed_layers = 0usize;
        for digest in &orphaned {
            if self.layers.remove(digest).is_some() {
                removed_layers += 1;
            }
        }
        GarbageCollectResult {
            removed_layers,
            orphaned_count: orphaned.len(),
            remaining_references: remaining.len(),
        }
    }

    /// Get deduplication statistics.
    pub fn stats(&self) -> DedupStats {
        let total_layers: usize = self.layers.iter().map(|r| r.manifest_ids.len()).sum();
        let unique_layers = self.layers.len();
        let deduplicated_layers = total_layers.saturating_sub(unique_layers);
        let space_savings_bytes: u64 = self
            .layers
            .iter()
            .filter(|r| r.manifest_ids.len() > 1)
            .map(|r| r.value().size * (r.manifest_ids.len() as u64 - 1))
            .sum();
        let total_bytes: u64 = self.layers.iter().map(|r| r.value().size).sum();
        let dedup_ratio = if total_bytes > 0 {
            space_savings_bytes as f64 / (total_bytes + space_savings_bytes) as f64
        } else {
            0.0
        };
        DedupStats {
            total_layers,
            unique_layers,
            deduplicated_layers,
            space_savings_bytes,
            dedup_ratio,
        }
    }

    /// List all unique layer digests.
    pub fn list_layers(&self) -> Vec<String> {
        self.layers.iter().map(|r| r.key().clone()).collect()
    }
}

/// Report from a single manifest registration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DedupReport {
    pub manifest_id: String,
    pub total_layers: usize,
    pub deduplicated_layers: usize,
    pub space_savings_bytes: u64,
}

/// Result of garbage collection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GarbageCollectResult {
    pub removed_layers: usize,
    pub orphaned_count: usize,
    pub remaining_references: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::oci::manifest::OciDescriptor;
    use std::collections::HashMap;

    fn make_layer(digest: &str, size: u64) -> OciDescriptor {
        OciDescriptor {
            media_type: "application/vnd.oci.image.layer.v1.tar+gzip".to_string(),
            digest: digest.to_string(),
            size,
            annotations: None,
        }
    }

    fn make_manifest(layers: Vec<OciDescriptor>) -> OciManifest {
        OciManifest {
            schema_version: 2,
            media_type: "application/vnd.oci.image.manifest.v1+json".to_string(),
            config: make_layer("sha256:config", 100),
            layers,
            annotations: HashMap::new(),
        }
    }

    #[test]
    fn test_register_first_manifest() {
        let mgr = LayerDedupManager::new();
        let manifest = make_manifest(vec![
            make_layer("sha256:layer1", 1000),
            make_layer("sha256:layer2", 2000),
        ]);
        let report = mgr.register_manifest("m1", &manifest);
        assert_eq!(report.total_layers, 2);
        assert_eq!(report.deduplicated_layers, 0);
        assert_eq!(mgr.stats().unique_layers, 2);
    }

    #[test]
    fn test_deduplication_across_manifests() {
        let mgr = LayerDedupManager::new();
        let shared_layer = make_layer("sha256:shared", 5000);
        let m1 = make_manifest(vec![
            shared_layer.clone(),
            make_layer("sha256:unique1", 1000),
        ]);
        let m2 = make_manifest(vec![
            shared_layer.clone(),
            make_layer("sha256:unique2", 2000),
        ]);
        mgr.register_manifest("m1", &m1);
        let report = mgr.register_manifest("m2", &m2);
        assert_eq!(report.deduplicated_layers, 1);
        assert_eq!(report.space_savings_bytes, 5000);
    }

    #[test]
    fn test_layer_exists() {
        let mgr = LayerDedupManager::new();
        let manifest = make_manifest(vec![make_layer("sha256:abc", 100)]);
        mgr.register_manifest("m1", &manifest);
        assert!(mgr.layer_exists("sha256:abc"));
        assert!(!mgr.layer_exists("sha256:nonexistent"));
    }

    #[test]
    fn test_layer_ref_count() {
        let mgr = LayerDedupManager::new();
        let shared = make_layer("sha256:shared", 100);
        let m1 = make_manifest(vec![shared.clone()]);
        let m2 = make_manifest(vec![shared.clone()]);
        let m3 = make_manifest(vec![shared.clone()]);
        mgr.register_manifest("m1", &m1);
        mgr.register_manifest("m2", &m2);
        mgr.register_manifest("m3", &m3);
        assert_eq!(mgr.layer_ref_count("sha256:shared"), 3);
    }

    #[test]
    fn test_store_layer_new() {
        let mgr = LayerDedupManager::new();
        let stored = mgr.store_layer("sha256:new", b"layer data");
        assert!(stored);
    }

    #[test]
    fn test_store_layer_existing() {
        let mgr = LayerDedupManager::new();
        let manifest = make_manifest(vec![make_layer("sha256:existing", 100)]);
        mgr.register_manifest("m1", &manifest);
        let stored = mgr.store_layer("sha256:existing", b"new data");
        assert!(!stored); // Already exists
    }

    #[test]
    fn test_unregister_manifest() {
        let mgr = LayerDedupManager::new();
        let shared = make_layer("sha256:shared", 100);
        let unique = make_layer("sha256:unique", 200);
        let m1 = make_manifest(vec![shared.clone(), unique.clone()]);
        let m2 = make_manifest(vec![shared.clone()]);
        mgr.register_manifest("m1", &m1);
        mgr.register_manifest("m2", &m2);
        let result = mgr.unregister_manifest("m1");
        assert_eq!(result.removed_layers, 1); // unique layer orphaned
        assert!(mgr.layer_exists("sha256:shared"));
        assert!(!mgr.layer_exists("sha256:unique"));
    }

    #[test]
    fn test_list_layers() {
        let mgr = LayerDedupManager::new();
        let manifest = make_manifest(vec![
            make_layer("sha256:a", 100),
            make_layer("sha256:b", 200),
        ]);
        mgr.register_manifest("m1", &manifest);
        let layers = mgr.list_layers();
        // 2 layers from manifest (config is not tracked as a layer in dedup)
        assert_eq!(layers.len(), 2);
    }

    #[test]
    fn test_stats() {
        let mgr = LayerDedupManager::new();
        let shared = make_layer("sha256:shared", 5000);
        let m1 = make_manifest(vec![shared.clone(), make_layer("sha256:u1", 1000)]);
        let m2 = make_manifest(vec![shared.clone(), make_layer("sha256:u2", 2000)]);
        mgr.register_manifest("m1", &m1);
        mgr.register_manifest("m2", &m2);
        let stats = mgr.stats();
        assert!(stats.unique_layers >= 3); // shared, u1, u2, config
        assert!(stats.deduplicated_layers >= 1); // shared layer deduped
        assert!(stats.space_savings_bytes >= 5000);
    }

    #[test]
    fn test_dedup_report_serialization() {
        let report = DedupReport {
            manifest_id: "m1".into(),
            total_layers: 5,
            deduplicated_layers: 2,
            space_savings_bytes: 10000,
        };
        let json = serde_json::to_string(&report).unwrap();
        let de: DedupReport = serde_json::from_str(&json).unwrap();
        assert_eq!(de.manifest_id, "m1");
    }

    #[test]
    fn test_dedup_stats_serialization() {
        let stats = DedupStats {
            total_layers: 10,
            unique_layers: 7,
            deduplicated_layers: 3,
            space_savings_bytes: 5000,
            dedup_ratio: 0.3,
        };
        let json = serde_json::to_string(&stats).unwrap();
        let de: DedupStats = serde_json::from_str(&json).unwrap();
        assert_eq!(de.unique_layers, 7);
    }
}
