#![forbid(unsafe_code)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use parking_lot::Mutex;
use uuid::Uuid;

/// A deduplicated error record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorRecord {
    pub id: Uuid,
    pub error_type: String,
    pub message: String,
    pub stack_trace: Option<String>,
    pub file: Option<String>,
    pub line: Option<i32>,
    pub user_id: Option<Uuid>,
    pub count: u64,
    pub first_seen_at: DateTime<Utc>,
    pub last_seen_at: DateTime<Utc>,
    pub resolved: bool,
}

/// Configuration for error tracking.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorTrackingConfig {
    #[serde(default = "default_et_enabled")]
    pub enabled: bool,
    #[serde(default = "default_et_max_errors")]
    pub max_errors: usize,
    #[serde(default = "default_et_max_stack_frames")]
    pub max_stack_frames: usize,
}

fn default_et_enabled() -> bool {
    true
}

fn default_et_max_errors() -> usize {
    10_000
}

fn default_et_max_stack_frames() -> usize {
    50
}

impl Default for ErrorTrackingConfig {
    fn default() -> Self {
        Self {
            enabled: default_et_enabled(),
            max_errors: default_et_max_errors(),
            max_stack_frames: default_et_max_stack_frames(),
        }
    }
}

/// Generate a deduplication key from error type and message.
fn error_fingerprint(error_type: &str, message: &str, stack_trace: Option<&str>) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(error_type.as_bytes());
    hasher.update(b"\0");
    hasher.update(message.as_bytes());
    if let Some(st) = stack_trace {
        hasher.update(b"\0");
        hasher.update(st.as_bytes());
    }
    hex::encode(hasher.finalize())
}

/// In-memory error tracker with deduplication.
pub struct ErrorTracker {
    config: ErrorTrackingConfig,
    errors: Mutex<HashMap<String, ErrorRecord>>,
}

impl ErrorTracker {
    pub fn new(config: ErrorTrackingConfig) -> Self {
        Self {
            config,
            errors: Mutex::new(HashMap::new()),
        }
    }

    /// Record an error event, deduplicating by type + message + stack trace.
    pub fn record_error(
        &self,
        error_type: &str,
        message: &str,
        stack_trace: Option<&str>,
        file: Option<&str>,
        line: Option<i32>,
        user_id: Option<Uuid>,
    ) -> ErrorRecord {
        if !self.config.enabled {
            return ErrorRecord {
                id: Uuid::nil(),
                error_type: error_type.to_string(),
                message: message.to_string(),
                stack_trace: stack_trace.map(|s| truncate_stack(s, self.config.max_stack_frames)),
                file: file.map(|s| s.to_string()),
                line,
                user_id,
                count: 0,
                first_seen_at: Utc::now(),
                last_seen_at: Utc::now(),
                resolved: false,
            };
        }

        let fingerprint = error_fingerprint(error_type, message, stack_trace);
        let now = Utc::now();

        let mut errors = self.errors.lock();

        if let Some(existing) = errors.get_mut(&fingerprint) {
            existing.count += 1;
            existing.last_seen_at = now;
            existing.clone()
        } else {
            // Evict oldest if at capacity
            if errors.len() >= self.config.max_errors
                && let Some(oldest_key) = errors
                    .iter()
                    .min_by_key(|(_, e)| e.first_seen_at)
                    .map(|(k, _)| k.clone())
                {
                    errors.remove(&oldest_key);
                }

            let record = ErrorRecord {
                id: Uuid::new_v4(),
                error_type: error_type.to_string(),
                message: message.to_string(),
                stack_trace: stack_trace.map(|s| truncate_stack(s, self.config.max_stack_frames)),
                file: file.map(|s| s.to_string()),
                line,
                user_id,
                count: 1,
                first_seen_at: now,
                last_seen_at: now,
                resolved: false,
            };
            errors.insert(fingerprint, record.clone());
            record
        }
    }

    /// Mark an error as resolved.
    pub fn resolve_error(&self, id: Uuid) -> bool {
        let mut errors = self.errors.lock();
        if let Some(record) = errors.values_mut().find(|e| e.id == id) {
            record.resolved = true;
            true
        } else {
            false
        }
    }

    /// Get all unresolved errors.
    pub fn unresolved_errors(&self) -> Vec<ErrorRecord> {
        let errors = self.errors.lock();
        errors
            .values()
            .filter(|e| !e.resolved)
            .cloned()
            .collect()
    }

    /// Get all errors.
    pub fn all_errors(&self) -> Vec<ErrorRecord> {
        let errors = self.errors.lock();
        errors.values().cloned().collect()
    }

    /// Get an error by ID.
    pub fn get_error(&self, id: Uuid) -> Option<ErrorRecord> {
        let errors = self.errors.lock();
        errors.values().find(|e| e.id == id).cloned()
    }

    /// Get errors for a specific user.
    pub fn errors_for_user(&self, user_id: Uuid) -> Vec<ErrorRecord> {
        let errors = self.errors.lock();
        errors
            .values()
            .filter(|e| e.user_id == Some(user_id))
            .cloned()
            .collect()
    }

    /// Get error count.
    pub fn error_count(&self) -> usize {
        self.errors.lock().len()
    }

    /// Get total occurrence count across all errors.
    pub fn total_occurrences(&self) -> u64 {
        self.errors.lock().values().map(|e| e.count).sum()
    }

    /// Export all errors and clear the buffer.
    pub fn export_errors(&self) -> Vec<ErrorRecord> {
        let mut errors = self.errors.lock();
        errors.drain().map(|(_, v)| v).collect()
    }

    /// Get a reference to the configuration.
    pub fn config(&self) -> &ErrorTrackingConfig {
        &self.config
    }

    /// Build a summary of errors grouped by type.
    pub fn summary(&self) -> ErrorSummary {
        let errors = self.errors.lock();
        let total = errors.len() as u64;
        let unresolved = errors.values().filter(|e| !e.resolved).count() as u64;
        let total_occurrences: u64 = errors.values().map(|e| e.count).sum();

        let mut by_type: HashMap<String, u64> = HashMap::new();
        for e in errors.values() {
            *by_type.entry(e.error_type.clone()).or_insert(0) += e.count;
        }

        let mut top_errors: Vec<ErrorRecord> = errors.values().cloned().collect();
        top_errors.sort_by_key(|b| std::cmp::Reverse(b.count));
        top_errors.truncate(10);

        ErrorSummary {
            total_unique_errors: total,
            unresolved_errors: unresolved,
            total_occurrences,
            errors_by_type: by_type,
            top_errors,
        }
    }
}

/// Truncate a stack trace to a maximum number of frames.
fn truncate_stack(stack: &str, max_frames: usize) -> String {
    let lines: Vec<&str> = stack.lines().collect();
    if lines.len() <= max_frames {
        stack.to_string()
    } else {
        let truncated: String = lines[..max_frames].join("\n");
        format!("{truncated}\n... ({} more frames)", lines.len() - max_frames)
    }
}

/// Summary of all tracked errors.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorSummary {
    pub total_unique_errors: u64,
    pub unresolved_errors: u64,
    pub total_occurrences: u64,
    pub errors_by_type: HashMap<String, u64>,
    pub top_errors: Vec<ErrorRecord>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = ErrorTrackingConfig::default();
        assert!(config.enabled);
        assert_eq!(config.max_errors, 10_000);
        assert_eq!(config.max_stack_frames, 50);
    }

    #[test]
    fn test_record_error_basic() {
        let tracker = ErrorTracker::new(ErrorTrackingConfig::default());
        let record = tracker.record_error("Panic", "unexpected null", None, None, None, None);
        assert_eq!(record.error_type, "Panic");
        assert_eq!(record.message, "unexpected null");
        assert_eq!(record.count, 1);
    }

    #[test]
    fn test_deduplication() {
        let tracker = ErrorTracker::new(ErrorTrackingConfig::default());
        let r1 = tracker.record_error("Panic", "boom", None, None, None, None);
        let r2 = tracker.record_error("Panic", "boom", None, None, None, None);
        assert_eq!(r1.id, r2.id);
        assert_eq!(r1.count + 1, r2.count);
        assert_eq!(tracker.error_count(), 1);
    }

    #[test]
    fn test_different_errors_not_deduped() {
        let tracker = ErrorTracker::new(ErrorTrackingConfig::default());
        tracker.record_error("Panic", "boom", None, None, None, None);
        tracker.record_error("Panic", "different", None, None, None, None);
        assert_eq!(tracker.error_count(), 2);
    }

    #[test]
    fn test_resolve_error() {
        let tracker = ErrorTracker::new(ErrorTrackingConfig::default());
        let record = tracker.record_error("Type", "msg", None, None, None, None);
        assert!(tracker.resolve_error(record.id));
        assert!(tracker.unresolved_errors().is_empty());
    }

    #[test]
    fn test_resolve_nonexistent() {
        let tracker = ErrorTracker::new(ErrorTrackingConfig::default());
        assert!(!tracker.resolve_error(Uuid::new_v4()));
    }

    #[test]
    fn test_errors_for_user() {
        let tracker = ErrorTracker::new(ErrorTrackingConfig::default());
        let user_id = Uuid::new_v4();
        tracker.record_error("Type", "msg", None, None, None, Some(user_id));
        tracker.record_error("Type", "msg2", None, None, None, None);
        let user_errors = tracker.errors_for_user(user_id);
        assert_eq!(user_errors.len(), 1);
    }

    #[test]
    fn test_max_errors_eviction() {
        let config = ErrorTrackingConfig {
            max_errors: 2,
            ..Default::default()
        };
        let tracker = ErrorTracker::new(config);
        tracker.record_error("T1", "m1", None, None, None, None);
        tracker.record_error("T2", "m2", None, None, None, None);
        tracker.record_error("T3", "m3", None, None, None, None);
        assert_eq!(tracker.error_count(), 2);
    }

    #[test]
    fn test_disabled_tracker() {
        let config = ErrorTrackingConfig {
            enabled: false,
            ..Default::default()
        };
        let tracker = ErrorTracker::new(config);
        let record = tracker.record_error("T", "m", None, None, None, None);
        assert_eq!(record.count, 0);
        assert_eq!(tracker.error_count(), 0);
    }

    #[test]
    fn test_export_clears() {
        let tracker = ErrorTracker::new(ErrorTrackingConfig::default());
        tracker.record_error("T", "m", None, None, None, None);
        let exported = tracker.export_errors();
        assert_eq!(exported.len(), 1);
        assert_eq!(tracker.error_count(), 0);
    }

    #[test]
    fn test_total_occurrences() {
        let tracker = ErrorTracker::new(ErrorTrackingConfig::default());
        tracker.record_error("T", "m", None, None, None, None);
        tracker.record_error("T", "m", None, None, None, None);
        tracker.record_error("T", "m", None, None, None, None);
        assert_eq!(tracker.total_occurrences(), 3);
    }

    #[test]
    fn test_summary() {
        let tracker = ErrorTracker::new(ErrorTrackingConfig::default());
        tracker.record_error("TypeA", "msg", None, None, None, None);
        tracker.record_error("TypeA", "msg", None, None, None, None);
        tracker.record_error("TypeB", "msg", None, None, None, None);
        let summary = tracker.summary();
        assert_eq!(summary.total_unique_errors, 2);
        assert_eq!(summary.total_occurrences, 3);
    }

    #[test]
    fn test_truncate_stack() {
        let stack = (0..100).map(|i| format!("frame-{i}")).collect::<Vec<_>>().join("\n");
        let truncated = truncate_stack(&stack, 5);
        assert!(truncated.contains("... (95 more frames)"));
    }

    #[test]
    fn test_truncate_stack_short() {
        let stack = "line1\nline2";
        let result = truncate_stack(&stack, 10);
        assert_eq!(result, stack);
    }

    #[test]
    fn test_get_error() {
        let tracker = ErrorTracker::new(ErrorTrackingConfig::default());
        let record = tracker.record_error("T", "m", None, None, None, None);
        let found = tracker.get_error(record.id);
        assert!(found.is_some());
        assert_eq!(found.unwrap().error_type, "T");
    }

    #[test]
    fn test_error_fingerprint_deterministic() {
        let f1 = error_fingerprint("T", "msg", Some("stack"));
        let f2 = error_fingerprint("T", "msg", Some("stack"));
        assert_eq!(f1, f2);
    }

    #[test]
    fn test_error_fingerprint_different() {
        let f1 = error_fingerprint("T1", "msg", None);
        let f2 = error_fingerprint("T2", "msg", None);
        assert_ne!(f1, f2);
    }
}
