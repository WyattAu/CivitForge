#![forbid(unsafe_code)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
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
}
