#![forbid(unsafe_code)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::info;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SyncCheckpoint {
    pub instance_id: String,
    pub last_synced_revision: String,
    pub checkpoint_time: DateTime<Utc>,
    pub synced_entities: Vec<String>,
}

impl SyncCheckpoint {
    pub fn new(instance_id: String, revision: String) -> Self {
        Self {
            instance_id,
            last_synced_revision: revision,
            checkpoint_time: Utc::now(),
            synced_entities: Vec::new(),
        }
    }

    pub fn with_entities(mut self, entities: Vec<String>) -> Self {
        self.synced_entities = entities;
        self
    }

    pub fn is_empty(&self) -> bool {
        self.last_synced_revision.is_empty()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ConflictStrategy {
    LastWriteWins,
    FirstWriteWins,
    Merge,
    Manual,
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ConflictEntry {
    pub entity_id: String,
    pub local_value: serde_json::Value,
    pub remote_value: serde_json::Value,
    pub resolved_value: Option<serde_json::Value>,
    pub strategy: ConflictStrategy,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct ConflictResolution {
    strategy: ConflictStrategy,
    resolution_log: Vec<ConflictEntry>,
}

impl ConflictResolution {
    pub fn new(strategy: ConflictStrategy) -> Self {
        Self {
            strategy,
            resolution_log: Vec::new(),
        }
    }

    pub fn resolve(
        &mut self,
        entity_id: &str,
        local: serde_json::Value,
        remote: serde_json::Value,
        timestamp: DateTime<Utc>,
    ) -> serde_json::Value {
        let resolved = match self.strategy {
            ConflictStrategy::LastWriteWins => remote.clone(),
            ConflictStrategy::FirstWriteWins => local.clone(),
            ConflictStrategy::Merge => merge_values(&local, &remote),
            ConflictStrategy::Manual => serde_json::Value::Null,
            ConflictStrategy::None => remote.clone(),
        };

        let entry = ConflictEntry {
            entity_id: entity_id.to_string(),
            local_value: local,
            remote_value: remote,
            resolved_value: Some(resolved.clone()),
            strategy: self.strategy.clone(),
            timestamp,
        };

        info!(
            entity_id = %entity_id,
            strategy = ?self.strategy,
            "resolved conflict"
        );

        self.resolution_log.push(entry);
        resolved
    }

    pub fn get_resolution_log(&self) -> &[ConflictEntry] {
        &self.resolution_log
    }
}

fn merge_values(local: &serde_json::Value, remote: &serde_json::Value) -> serde_json::Value {
    match (local, remote) {
        (serde_json::Value::Object(local_map), serde_json::Value::Object(remote_map)) => {
            let mut merged = local_map.clone();
            for (key, value) in remote_map {
                merged.insert(key.clone(), value.clone());
            }
            serde_json::Value::Object(merged)
        }
        (_, remote) => remote.clone(),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SyncDelta {
    pub entity_id: String,
    pub old_revision: String,
    pub new_revision: String,
    pub delta_data: Vec<u8>,
    pub new_data: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PartitionStatus {
    pub partition_id: String,
    pub nodes: Vec<String>,
    pub detected_at: DateTime<Utc>,
    pub healed_at: Option<DateTime<Utc>>,
    pub is_active: bool,
}

#[derive(Debug)]
pub struct IncrementalSyncEngine {
    checkpoints: std::sync::Mutex<HashMap<String, SyncCheckpoint>>,
    deltas: std::sync::Mutex<Vec<SyncDelta>>,
    max_deltas: usize,
}

impl IncrementalSyncEngine {
    pub fn new(max_deltas: usize) -> Self {
        Self {
            checkpoints: std::sync::Mutex::new(HashMap::new()),
            deltas: std::sync::Mutex::new(Vec::new()),
            max_deltas,
        }
    }

    pub fn save_checkpoint(&self, checkpoint: SyncCheckpoint) {
        let mut cps = self.checkpoints.lock().unwrap();
        cps.insert(checkpoint.instance_id.clone(), checkpoint);
    }

    pub fn get_checkpoint(&self, instance_id: &str) -> Option<SyncCheckpoint> {
        let cps = self.checkpoints.lock().unwrap();
        cps.get(instance_id).cloned()
    }

    pub fn record_delta(&self, delta: SyncDelta) {
        let mut deltas = self.deltas.lock().unwrap();
        deltas.push(delta);
        while deltas.len() > self.max_deltas {
            deltas.remove(0);
        }
    }

    pub fn compute_and_record_delta(
        &self,
        entity_id: &str,
        old: &[u8],
        new: &[u8],
        old_rev: &str,
        new_rev: &str,
    ) {
        let delta_data = DeltaCompressor::compute_delta(old, new);
        let delta = SyncDelta {
            entity_id: entity_id.to_string(),
            old_revision: old_rev.to_string(),
            new_revision: new_rev.to_string(),
            delta_data,
            new_data: new.to_vec(),
        };
        self.record_delta(delta);
    }

    pub fn get_deltas_since(&self, revision: &str) -> Vec<SyncDelta> {
        let deltas = self.deltas.lock().unwrap();
        deltas
            .iter()
            .filter(|d| d.old_revision.as_str() >= revision)
            .cloned()
            .collect()
    }

    pub fn delta_count(&self) -> usize {
        self.deltas.lock().unwrap().len()
    }

    pub fn checkpoint_count(&self) -> usize {
        self.checkpoints.lock().unwrap().len()
    }

    pub fn recompute_deltas(&self, base: &[u8], new: &[u8]) -> Vec<u8> {
        DeltaCompressor::compute_delta(base, new)
    }
}

impl Default for IncrementalSyncEngine {
    fn default() -> Self {
        Self::new(1000)
    }
}

pub struct PartitionTracker {
    partitions: std::sync::Mutex<Vec<PartitionStatus>>,
}

impl PartitionTracker {
    pub fn new() -> Self {
        Self {
            partitions: std::sync::Mutex::new(Vec::new()),
        }
    }

    pub fn detect_partition(&self, partition_id: String, nodes: Vec<String>) {
        let status = PartitionStatus {
            partition_id,
            nodes,
            detected_at: Utc::now(),
            healed_at: None,
            is_active: true,
        };
        let mut partitions = self.partitions.lock().unwrap();
        partitions.push(status);
    }

    pub fn heal_partition(&self, partition_id: &str) -> bool {
        let mut partitions = self.partitions.lock().unwrap();
        if let Some(p) = partitions
            .iter_mut()
            .find(|p| p.partition_id == partition_id && p.is_active)
        {
            p.healed_at = Some(Utc::now());
            p.is_active = false;
            return true;
        }
        false
    }

    pub fn active_partitions(&self) -> Vec<PartitionStatus> {
        let partitions = self.partitions.lock().unwrap();
        partitions.iter().filter(|p| p.is_active).cloned().collect()
    }

    pub fn all_partitions(&self) -> Vec<PartitionStatus> {
        let partitions = self.partitions.lock().unwrap();
        partitions.clone()
    }

    pub fn partition_count(&self) -> usize {
        let partitions = self.partitions.lock().unwrap();
        partitions.len()
    }

    pub fn active_count(&self) -> usize {
        let partitions = self.partitions.lock().unwrap();
        partitions.iter().filter(|p| p.is_active).count()
    }
}

impl Default for PartitionTracker {
    fn default() -> Self {
        Self::new()
    }
}

pub struct BandwidthOptimizer {
    compression_enabled: std::sync::atomic::AtomicBool,
    compression_level: std::sync::atomic::AtomicI32,
    min_delta_size: usize,
}

impl BandwidthOptimizer {
    pub fn new(min_delta_size: usize) -> Self {
        Self {
            compression_enabled: std::sync::atomic::AtomicBool::new(true),
            compression_level: std::sync::atomic::AtomicI32::new(3),
            min_delta_size,
        }
    }

    pub fn optimize_transfer(&self, delta_data: &[u8], new_data: &[u8]) -> Vec<u8> {
        if !self
            .compression_enabled
            .load(std::sync::atomic::Ordering::Relaxed)
        {
            return delta_data.to_vec();
        }
        if delta_data.len() < self.min_delta_size {
            return delta_data.to_vec();
        }
        if delta_data.len() < new_data.len() {
            delta_data.to_vec()
        } else {
            new_data.to_vec()
        }
    }

    pub fn set_compression_enabled(&self, enabled: bool) {
        self.compression_enabled
            .store(enabled, std::sync::atomic::Ordering::Relaxed);
    }

    pub fn is_compression_enabled(&self) -> bool {
        self.compression_enabled
            .load(std::sync::atomic::Ordering::Relaxed)
    }

    pub fn set_compression_level(&self, level: i32) {
        self.compression_level
            .store(level, std::sync::atomic::Ordering::Relaxed);
    }

    pub fn compression_level(&self) -> i32 {
        self.compression_level
            .load(std::sync::atomic::Ordering::Relaxed)
    }
}

impl Default for BandwidthOptimizer {
    fn default() -> Self {
        Self::new(64)
    }
}

pub struct DeltaCompressor;

impl DeltaCompressor {
    pub fn compute_delta(old: &[u8], new: &[u8]) -> Vec<u8> {
        let mut delta = Vec::new();
        let mut i = 0;

        if old.is_empty() && new.is_empty() {
            return delta;
        }

        if old.is_empty() {
            delta.push(0);
            delta.extend_from_slice(&(new.len() as u32).to_le_bytes());
            delta.extend_from_slice(new);
            return delta;
        }

        let mut old_idx = 0;

        while i < new.len() {
            let mut best_len = 0;
            let mut best_offset = 0;

            for j in 0..old.len().saturating_sub(best_len) {
                let mut len = 0;
                while i + len < new.len() && j + len < old.len() && new[i + len] == old[j + len] {
                    len += 1;
                }
                if len > best_len {
                    best_len = len;
                    best_offset = j;
                }
            }

            if best_len >= 4 {
                if old_idx < i {
                    let insert_len = i - old_idx;
                    delta.push(0);
                    delta.extend_from_slice(&(insert_len as u32).to_le_bytes());
                    delta.extend_from_slice(&new[old_idx..i]);
                }
                delta.push(1);
                delta.extend_from_slice(&(best_offset as u32).to_le_bytes());
                delta.extend_from_slice(&(best_len as u32).to_le_bytes());
                i += best_len;
                old_idx = i;
            } else {
                i += 1;
            }
        }

        if old_idx < new.len() {
            let insert_len = new.len() - old_idx;
            delta.push(0);
            delta.extend_from_slice(&(insert_len as u32).to_le_bytes());
            delta.extend_from_slice(&new[old_idx..]);
        }

        delta
    }

    pub fn apply_delta(base: &[u8], delta: &[u8]) -> Vec<u8> {
        let mut result = Vec::new();
        let mut idx = 0;

        while idx < delta.len() {
            let op = delta[idx];
            idx += 1;

            match op {
                0 => {
                    if idx + 4 > delta.len() {
                        break;
                    }
                    let len = u32::from_le_bytes([
                        delta[idx],
                        delta[idx + 1],
                        delta[idx + 2],
                        delta[idx + 3],
                    ]) as usize;
                    idx += 4;
                    if idx + len > delta.len() {
                        break;
                    }
                    result.extend_from_slice(&delta[idx..idx + len]);
                    idx += len;
                }
                1 => {
                    if idx + 8 > delta.len() {
                        break;
                    }
                    let offset = u32::from_le_bytes([
                        delta[idx],
                        delta[idx + 1],
                        delta[idx + 2],
                        delta[idx + 3],
                    ]) as usize;
                    idx += 4;
                    let len = u32::from_le_bytes([
                        delta[idx],
                        delta[idx + 1],
                        delta[idx + 2],
                        delta[idx + 3],
                    ]) as usize;
                    idx += 4;
                    let end = (offset + len).min(base.len());
                    if offset < base.len() {
                        result.extend_from_slice(&base[offset..end]);
                    }
                }
                _ => {}
            }
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sync_checkpoint_new() {
        let cp = SyncCheckpoint::new("inst-1".into(), "rev-abc".into());
        assert_eq!(cp.instance_id, "inst-1");
        assert_eq!(cp.last_synced_revision, "rev-abc");
        assert!(cp.synced_entities.is_empty());
        assert!(!cp.is_empty());
    }

    #[test]
    fn test_sync_checkpoint_with_entities() {
        let cp = SyncCheckpoint::new("inst-1".into(), "rev-1".into())
            .with_entities(vec!["repo-a".into(), "repo-b".into()]);
        assert_eq!(cp.synced_entities.len(), 2);
    }

    #[test]
    fn test_sync_checkpoint_empty_revision() {
        let cp = SyncCheckpoint::new("inst-1".into(), "".into());
        assert!(cp.is_empty());
    }

    #[test]
    fn test_conflict_resolution_last_write_wins() {
        let mut resolver = ConflictResolution::new(ConflictStrategy::LastWriteWins);
        let local = serde_json::json!({"value": 1});
        let remote = serde_json::json!({"value": 2});
        let resolved = resolver.resolve("entity-1", local, remote, Utc::now());
        assert_eq!(resolved, serde_json::json!({"value": 2}));
        assert_eq!(resolver.get_resolution_log().len(), 1);
    }

    #[test]
    fn test_conflict_resolution_first_write_wins() {
        let mut resolver = ConflictResolution::new(ConflictStrategy::FirstWriteWins);
        let local = serde_json::json!({"value": 1});
        let remote = serde_json::json!({"value": 2});
        let resolved = resolver.resolve("entity-1", local, remote, Utc::now());
        assert_eq!(resolved, serde_json::json!({"value": 1}));
    }

    #[test]
    fn test_conflict_resolution_merge() {
        let mut resolver = ConflictResolution::new(ConflictStrategy::Merge);
        let local = serde_json::json!({"a": 1});
        let remote = serde_json::json!({"b": 2});
        let resolved = resolver.resolve("entity-1", local, remote, Utc::now());
        assert_eq!(resolved, serde_json::json!({"a": 1, "b": 2}));
    }

    #[test]
    fn test_conflict_resolution_manual() {
        let mut resolver = ConflictResolution::new(ConflictStrategy::Manual);
        let local = serde_json::json!({"value": 1});
        let remote = serde_json::json!({"value": 2});
        let resolved = resolver.resolve("entity-1", local, remote, Utc::now());
        assert_eq!(resolved, serde_json::Value::Null);
    }

    #[test]
    fn test_conflict_resolution_multiple() {
        let mut resolver = ConflictResolution::new(ConflictStrategy::LastWriteWins);
        resolver.resolve("e1", serde_json::json!(1), serde_json::json!(2), Utc::now());
        resolver.resolve("e2", serde_json::json!(3), serde_json::json!(4), Utc::now());
        assert_eq!(resolver.get_resolution_log().len(), 2);
        assert_eq!(resolver.get_resolution_log()[0].entity_id, "e1");
        assert_eq!(resolver.get_resolution_log()[1].entity_id, "e2");
    }

    #[test]
    fn test_delta_empty_inputs() {
        let delta = DeltaCompressor::compute_delta(&[], &[]);
        let result = DeltaCompressor::apply_delta(&[], &delta);
        assert!(result.is_empty());
    }

    #[test]
    fn test_delta_insert_only() {
        let delta = DeltaCompressor::compute_delta(&[], b"hello world");
        let result = DeltaCompressor::apply_delta(&[], &delta);
        assert_eq!(result, b"hello world");
    }

    #[test]
    fn test_delta_no_change() {
        let data = b"unchanged data here";
        let delta = DeltaCompressor::compute_delta(data, data);
        let result = DeltaCompressor::apply_delta(data, &delta);
        assert_eq!(result, data);
    }

    #[test]
    fn test_delta_small_change() {
        let old = b"the quick brown fox";
        let new = b"the slow brown fox";
        let delta = DeltaCompressor::compute_delta(old, new);
        let result = DeltaCompressor::apply_delta(old, &delta);
        assert_eq!(result, new);
    }

    #[test]
    fn test_delta_append() {
        let old = b"hello";
        let new = b"hello world";
        let delta = DeltaCompressor::compute_delta(old, new);
        let result = DeltaCompressor::apply_delta(old, &delta);
        assert_eq!(result, new);
    }

    #[test]
    fn test_delta_compression_ratio() {
        let old = b"abcdefghijabcdefghijabcdefghijabcdefghijabcdefghij";
        let new = b"abcdefghijabcdefghijabcdefghijabcdefghijXXXXXX";
        let delta = DeltaCompressor::compute_delta(old, new);
        assert!(delta.len() < new.len());
        let result = DeltaCompressor::apply_delta(old, &delta);
        assert_eq!(result, new);
    }

    #[test]
    fn test_merge_values_objects() {
        let local = serde_json::json!({"a": 1, "b": 2});
        let remote = serde_json::json!({"b": 3, "c": 4});
        let merged = merge_values(&local, &remote);
        assert_eq!(merged["a"], 1);
        assert_eq!(merged["b"], 3);
        assert_eq!(merged["c"], 4);
    }

    #[test]
    fn test_merge_values_non_objects() {
        let local = serde_json::json!("string");
        let remote = serde_json::json!(42);
        let merged = merge_values(&local, &remote);
        assert_eq!(merged, 42);
    }

    #[test]
    fn test_incremental_sync_save_and_get_checkpoint() {
        let engine = IncrementalSyncEngine::new(100);
        let cp = SyncCheckpoint::new("inst-a".into(), "rev-1".into());
        engine.save_checkpoint(cp.clone());
        let retrieved = engine.get_checkpoint("inst-a").unwrap();
        assert_eq!(retrieved.instance_id, "inst-a");
        assert_eq!(retrieved.last_synced_revision, "rev-1");
    }

    #[test]
    fn test_incremental_sync_checkpoint_not_found() {
        let engine = IncrementalSyncEngine::new(100);
        assert!(engine.get_checkpoint("nonexistent").is_none());
    }

    #[test]
    fn test_incremental_sync_record_delta() {
        let engine = IncrementalSyncEngine::new(100);
        let delta = SyncDelta {
            entity_id: "e1".into(),
            old_revision: "rev-1".into(),
            new_revision: "rev-2".into(),
            delta_data: vec![1, 2, 3],
            new_data: vec![4, 5, 6],
        };
        engine.record_delta(delta);
        assert_eq!(engine.delta_count(), 1);
    }

    #[test]
    fn test_incremental_sync_max_deltas_eviction() {
        let engine = IncrementalSyncEngine::new(2);
        engine.record_delta(SyncDelta {
            entity_id: "e1".into(),
            old_revision: "r1".into(),
            new_revision: "r2".into(),
            delta_data: vec![1],
            new_data: vec![2],
        });
        engine.record_delta(SyncDelta {
            entity_id: "e2".into(),
            old_revision: "r2".into(),
            new_revision: "r3".into(),
            delta_data: vec![3],
            new_data: vec![4],
        });
        engine.record_delta(SyncDelta {
            entity_id: "e3".into(),
            old_revision: "r3".into(),
            new_revision: "r4".into(),
            delta_data: vec![5],
            new_data: vec![6],
        });
        assert_eq!(engine.delta_count(), 2);
    }

    #[test]
    fn test_incremental_sync_compute_and_record() {
        let engine = IncrementalSyncEngine::new(100);
        engine.compute_and_record_delta("e1", b"hello", b"hello world", "r1", "r2");
        assert_eq!(engine.delta_count(), 1);
    }

    #[test]
    fn test_incremental_sync_get_deltas_since() {
        let engine = IncrementalSyncEngine::new(100);
        engine.record_delta(SyncDelta {
            entity_id: "e1".into(),
            old_revision: "r1".into(),
            new_revision: "r2".into(),
            delta_data: vec![],
            new_data: vec![],
        });
        engine.record_delta(SyncDelta {
            entity_id: "e2".into(),
            old_revision: "r2".into(),
            new_revision: "r3".into(),
            delta_data: vec![],
            new_data: vec![],
        });
        engine.record_delta(SyncDelta {
            entity_id: "e3".into(),
            old_revision: "r3".into(),
            new_revision: "r4".into(),
            delta_data: vec![],
            new_data: vec![],
        });
        let since = engine.get_deltas_since("r2");
        assert_eq!(since.len(), 2);
    }

    #[test]
    fn test_partition_tracker_detect_and_heal() {
        let tracker = PartitionTracker::new();
        tracker.detect_partition("p-1".into(), vec!["inst-a".into(), "inst-b".into()]);
        assert_eq!(tracker.active_count(), 1);
        assert!(tracker.heal_partition("p-1"));
        assert_eq!(tracker.active_count(), 0);
    }

    #[test]
    fn test_partition_tracker_heal_nonexistent() {
        let tracker = PartitionTracker::new();
        assert!(!tracker.heal_partition("nonexistent"));
    }

    #[test]
    fn test_partition_tracker_multiple_partitions() {
        let tracker = PartitionTracker::new();
        tracker.detect_partition("p-1".into(), vec!["a".into()]);
        tracker.detect_partition("p-2".into(), vec!["b".into(), "c".into()]);
        assert_eq!(tracker.partition_count(), 2);
        tracker.heal_partition("p-1");
        assert_eq!(tracker.active_count(), 1);
    }

    #[test]
    fn test_partition_tracker_all_partitions() {
        let tracker = PartitionTracker::new();
        tracker.detect_partition("p-1".into(), vec!["a".into()]);
        tracker.heal_partition("p-1");
        let all = tracker.all_partitions();
        assert_eq!(all.len(), 1);
        assert!(!all[0].is_active);
    }

    #[test]
    fn test_bandwidth_optimizer_compresses_when_beneficial() {
        let opt = BandwidthOptimizer::new(0);
        let delta = vec![1, 2, 3, 4, 5];
        let new = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let result = opt.optimize_transfer(&delta, &new);
        assert_eq!(result, delta);
    }

    #[test]
    fn test_bandwidth_optimizer_uses_full_when_delta_larger() {
        let opt = BandwidthOptimizer::new(0);
        let delta = vec![0u8; 200];
        let new = vec![0u8; 50];
        let result = opt.optimize_transfer(&delta, &new);
        assert_eq!(result, new);
    }

    #[test]
    fn test_bandwidth_optimizer_skip_small_deltas() {
        let opt = BandwidthOptimizer::new(100);
        let delta = vec![1, 2, 3];
        let new = vec![1, 2, 3, 4, 5];
        let result = opt.optimize_transfer(&delta, &new);
        assert_eq!(result, delta);
    }

    #[test]
    fn test_bandwidth_optimizer_toggle_compression() {
        let opt = BandwidthOptimizer::new(0);
        assert!(opt.is_compression_enabled());
        opt.set_compression_enabled(false);
        assert!(!opt.is_compression_enabled());
    }

    #[test]
    fn test_bandwidth_optimizer_compression_level() {
        let opt = BandwidthOptimizer::new(0);
        opt.set_compression_level(10);
        assert_eq!(opt.compression_level(), 10);
    }

    #[test]
    fn test_incremental_sync_recompute_deltas() {
        let engine = IncrementalSyncEngine::new(100);
        let delta = engine.recompute_deltas(b"hello", b"hello world");
        let result = DeltaCompressor::apply_delta(b"hello", &delta);
        assert_eq!(result, b"hello world");
    }

    #[test]
    fn test_sync_delta_serialization() {
        let delta = SyncDelta {
            entity_id: "e1".into(),
            old_revision: "r1".into(),
            new_revision: "r2".into(),
            delta_data: vec![1, 2, 3],
            new_data: vec![4, 5, 6],
        };
        let json = serde_json::to_string(&delta).unwrap();
        let deser: SyncDelta = serde_json::from_str(&json).unwrap();
        assert_eq!(deser.entity_id, "e1");
        assert_eq!(deser.old_revision, "r1");
    }

    #[test]
    fn test_partition_status_construction() {
        let status = PartitionStatus {
            partition_id: "p-1".into(),
            nodes: vec!["a".into(), "b".into()],
            detected_at: Utc::now(),
            healed_at: None,
            is_active: true,
        };
        assert!(status.is_active);
        assert!(status.healed_at.is_none());
    }
}
