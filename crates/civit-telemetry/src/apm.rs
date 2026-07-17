#![forbid(unsafe_code)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;

/// An APM transaction representing a single request or task.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApmTransaction {
    pub transaction_id: String,
    pub name: String,
    pub transaction_type: String,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub duration_ms: Option<i64>,
    pub result: String,
    pub context: serde_json::Value,
}

/// An APM span representing a unit of work within a transaction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApmSpan {
    pub span_id: String,
    pub transaction_id: String,
    pub parent_id: Option<String>,
    pub name: String,
    pub span_type: String,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub duration_ms: Option<i64>,
    pub context: serde_json::Value,
}

/// Configuration for APM.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApmConfig {
    #[serde(default = "default_apm_enabled")]
    pub enabled: bool,
    #[serde(default = "default_apm_max_transactions")]
    pub max_transactions: usize,
    #[serde(default = "default_apm_max_spans_per_transaction")]
    pub max_spans_per_transaction: usize,
    #[serde(default = "default_apm_sample_rate")]
    pub sample_rate: f64,
}

fn default_apm_enabled() -> bool {
    true
}

fn default_apm_max_transactions() -> usize {
    1_000
}

fn default_apm_max_spans_per_transaction() -> usize {
    100
}

fn default_apm_sample_rate() -> f64 {
    1.0
}

impl Default for ApmConfig {
    fn default() -> Self {
        Self {
            enabled: default_apm_enabled(),
            max_transactions: default_apm_max_transactions(),
            max_spans_per_transaction: default_apm_max_spans_per_transaction(),
            sample_rate: default_apm_sample_rate(),
        }
    }
}

/// In-memory APM recorder.
pub struct ApmRecorder {
    config: ApmConfig,
    tx_counter: AtomicU64,
    span_counter: AtomicU64,
    sampling_counter: AtomicU64,
    transactions: Mutex<Vec<ApmTransaction>>,
    spans: Mutex<Vec<ApmSpan>>,
}

impl ApmRecorder {
    pub fn new(config: ApmConfig) -> Self {
        Self {
            config,
            tx_counter: AtomicU64::new(1),
            span_counter: AtomicU64::new(1),
            sampling_counter: AtomicU64::new(0),
            transactions: Mutex::new(Vec::new()),
            spans: Mutex::new(Vec::new()),
        }
    }

    /// Check whether this transaction should be sampled.
    pub fn should_sample(&self) -> bool {
        if !self.config.enabled || self.config.sample_rate >= 1.0 {
            return self.config.enabled;
        }
        let count = self.sampling_counter.fetch_add(1, Ordering::Relaxed);
        let threshold = (self.config.sample_rate * 1000.0) as u64;
        (count % 1000) < threshold
    }

    /// Start a new transaction.
    pub fn start_transaction(&self, name: &str, transaction_type: &str) -> ApmTransaction {
        let id = format!(
            "{:016x}",
            self.tx_counter.fetch_add(1, Ordering::Relaxed)
        );
        ApmTransaction {
            transaction_id: id,
            name: name.to_string(),
            transaction_type: transaction_type.to_string(),
            start_time: Utc::now(),
            end_time: None,
            duration_ms: None,
            result: "success".to_string(),
            context: serde_json::Value::Object(serde_json::Map::new()),
        }
    }

    /// Start a span within a transaction.
    pub fn start_span(
        &self,
        transaction_id: &str,
        parent_id: Option<&str>,
        name: &str,
        span_type: &str,
    ) -> ApmSpan {
        let id = format!(
            "{:016x}",
            self.span_counter.fetch_add(1, Ordering::Relaxed)
        );
        ApmSpan {
            span_id: id,
            transaction_id: transaction_id.to_string(),
            parent_id: parent_id.map(|s| s.to_string()),
            name: name.to_string(),
            span_type: span_type.to_string(),
            start_time: Utc::now(),
            end_time: None,
            duration_ms: None,
            context: serde_json::Value::Object(serde_json::Map::new()),
        }
    }

    /// End a transaction and record it.
    pub fn end_transaction(&self, mut tx: ApmTransaction, result: &str) {
        let now = Utc::now();
        tx.end_time = Some(now);
        tx.duration_ms = Some((now - tx.start_time).num_milliseconds());
        tx.result = result.to_string();
        let mut txns = self.transactions.lock().unwrap();
        if txns.len() >= self.config.max_transactions {
            txns.remove(0);
        }
        txns.push(tx);
    }

    /// End a span and record it.
    pub fn end_span(&self, mut span: ApmSpan) {
        let now = Utc::now();
        span.end_time = Some(now);
        span.duration_ms = Some((now - span.start_time).num_milliseconds());
        let mut spans = self.spans.lock().unwrap();
        // Enforce per-transaction limit by removing oldest spans from this transaction
        let count_here = spans
            .iter()
            .filter(|s| s.transaction_id == span.transaction_id)
            .count();
        if count_here >= self.config.max_spans_per_transaction {
            if let Some(pos) = spans
                .iter()
                .position(|s| s.transaction_id == span.transaction_id)
            {
                spans.remove(pos);
            }
        }
        spans.push(span);
    }

    /// Export all completed transactions.
    pub fn export_transactions(&self) -> Vec<ApmTransaction> {
        let mut txns = self.transactions.lock().unwrap();
        std::mem::take(&mut *txns)
    }

    /// Export all completed spans.
    pub fn export_spans(&self) -> Vec<ApmSpan> {
        let mut spans = self.spans.lock().unwrap();
        std::mem::take(&mut *spans)
    }

    /// Get the count of buffered transactions.
    pub fn transaction_count(&self) -> usize {
        self.transactions.lock().unwrap().len()
    }

    /// Get the count of buffered spans.
    pub fn span_count(&self) -> usize {
        self.spans.lock().unwrap().len()
    }

    /// Get a reference to the configuration.
    pub fn config(&self) -> &ApmConfig {
        &self.config
    }

    /// Calculate performance statistics for a given transaction name.
    pub fn stats_for_transaction(&self, name: &str) -> TransactionStats {
        let txns = self.transactions.lock().unwrap();
        let matching: Vec<&ApmTransaction> = txns
            .iter()
            .filter(|t| t.name == name && t.duration_ms.is_some())
            .collect();

        if matching.is_empty() {
            return TransactionStats::default();
        }

        let durations: Vec<i64> = matching.iter().map(|t| t.duration_ms.unwrap()).collect();
        let count = durations.len() as u64;
        let sum: i64 = durations.iter().sum();
        let min = *durations.iter().min().unwrap();
        let max = *durations.iter().max().unwrap();
        let avg = sum as f64 / count as f64;

        let error_count = matching
            .iter()
            .filter(|t| t.result != "success")
            .count() as u64;

        TransactionStats {
            name: name.to_string(),
            count,
            total_duration_ms: sum,
            avg_duration_ms: avg,
            min_duration_ms: min,
            max_duration_ms: max,
            error_count,
            success_rate: if count > 0 {
                ((count - error_count) as f64 / count as f64) * 100.0
            } else {
                100.0
            },
        }
    }
}

/// Performance statistics for a transaction.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TransactionStats {
    pub name: String,
    pub count: u64,
    pub total_duration_ms: i64,
    pub avg_duration_ms: f64,
    pub min_duration_ms: i64,
    pub max_duration_ms: i64,
    pub error_count: u64,
    pub success_rate: f64,
}

/// APM dashboard data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApmDashboard {
    pub total_transactions: u64,
    pub total_spans: u64,
    pub avg_duration_ms: f64,
    pub error_rate: f64,
    pub top_slow_transactions: Vec<TransactionStats>,
}

impl ApmRecorder {
    /// Build a dashboard summary.
    pub fn dashboard(&self) -> ApmDashboard {
        let txns = self.transactions.lock().unwrap();
        let spans = self.spans.lock().unwrap();

        let total_transactions = txns.len() as u64;
        let total_spans = spans.len() as u64;

        let durations: Vec<i64> = txns.iter().filter_map(|t| t.duration_ms).collect();
        let avg_duration_ms = if durations.is_empty() {
            0.0
        } else {
            durations.iter().sum::<i64>() as f64 / durations.len() as f64
        };

        let errors = txns.iter().filter(|t| t.result != "success").count() as u64;
        let error_rate = if total_transactions > 0 {
            (errors as f64 / total_transactions as f64) * 100.0
        } else {
            0.0
        };

        // Group by name and compute stats
        let mut name_counts: HashMap<String, Vec<i64>> = HashMap::new();
        for txn in txns.iter() {
            if let Some(dur) = txn.duration_ms {
                name_counts
                    .entry(txn.name.clone())
                    .or_default()
                    .push(dur);
            }
        }

        let mut top_slow: Vec<TransactionStats> = name_counts
            .into_iter()
            .map(|(name, durs)| {
                let count = durs.len() as u64;
                let total: i64 = durs.iter().sum();
                TransactionStats {
                    name,
                    count,
                    total_duration_ms: total,
                    avg_duration_ms: total as f64 / count as f64,
                    min_duration_ms: *durs.iter().min().unwrap_or(&0),
                    max_duration_ms: *durs.iter().max().unwrap_or(&0),
                    error_count: 0,
                    success_rate: 100.0,
                }
            })
            .collect();
        top_slow.sort_by(|a, b| {
            b.avg_duration_ms
                .partial_cmp(&a.avg_duration_ms)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        top_slow.truncate(10);

        ApmDashboard {
            total_transactions,
            total_spans,
            avg_duration_ms,
            error_rate,
            top_slow_transactions: top_slow,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = ApmConfig::default();
        assert!(config.enabled);
        assert_eq!(config.max_transactions, 1_000);
        assert_eq!(config.max_spans_per_transaction, 100);
    }

    #[test]
    fn test_start_transaction() {
        let recorder = ApmRecorder::new(ApmConfig::default());
        let tx = recorder.start_transaction("GET /api", "request");
        assert_eq!(tx.name, "GET /api");
        assert_eq!(tx.transaction_type, "request");
        assert_eq!(tx.result, "success");
        assert!(tx.end_time.is_none());
    }

    #[test]
    fn test_end_transaction() {
        let recorder = ApmRecorder::new(ApmConfig::default());
        let tx = recorder.start_transaction("work", "task");
        recorder.end_transaction(tx, "success");
        assert_eq!(recorder.transaction_count(), 1);
    }

    #[test]
    fn test_export_clears() {
        let recorder = ApmRecorder::new(ApmConfig::default());
        let tx = recorder.start_transaction("work", "task");
        recorder.end_transaction(tx, "success");
        let exported = recorder.export_transactions();
        assert_eq!(exported.len(), 1);
        assert_eq!(recorder.transaction_count(), 0);
    }

    #[test]
    fn test_start_span() {
        let recorder = ApmRecorder::new(ApmConfig::default());
        let tx = recorder.start_transaction("req", "request");
        let span = recorder.start_span(&tx.transaction_id, None, "db.query", "db");
        assert_eq!(span.transaction_id, tx.transaction_id);
        assert!(span.parent_id.is_none());
    }

    #[test]
    fn test_end_span() {
        let recorder = ApmRecorder::new(ApmConfig::default());
        let tx = recorder.start_transaction("req", "request");
        let span = recorder.start_span(&tx.transaction_id, None, "db.query", "db");
        recorder.end_span(span);
        assert_eq!(recorder.span_count(), 1);
    }

    #[test]
    fn test_max_transactions_eviction() {
        let config = ApmConfig {
            max_transactions: 2,
            ..Default::default()
        };
        let recorder = ApmRecorder::new(config);
        for i in 0..4 {
            let tx = recorder.start_transaction(&format!("tx-{i}"), "task");
            recorder.end_transaction(tx, "success");
        }
        assert_eq!(recorder.transaction_count(), 2);
    }

    #[test]
    fn test_sampling_disabled() {
        let config = ApmConfig {
            enabled: false,
            ..Default::default()
        };
        let recorder = ApmRecorder::new(config);
        assert!(!recorder.should_sample());
    }

    #[test]
    fn test_sampling_always() {
        let recorder = ApmRecorder::new(ApmConfig::default());
        for _ in 0..100 {
            assert!(recorder.should_sample());
        }
    }

    #[test]
    fn test_stats_for_transaction() {
        let recorder = ApmRecorder::new(ApmConfig::default());
        for _ in 0..5 {
            let tx = recorder.start_transaction("api.call", "request");
            recorder.end_transaction(tx, "success");
        }
        let stats = recorder.stats_for_transaction("api.call");
        assert_eq!(stats.count, 5);
        assert_eq!(stats.name, "api.call");
    }

    #[test]
    fn test_stats_empty() {
        let recorder = ApmRecorder::new(ApmConfig::default());
        let stats = recorder.stats_for_transaction("nonexistent");
        assert_eq!(stats.count, 0);
    }

    #[test]
    fn test_dashboard() {
        let recorder = ApmRecorder::new(ApmConfig::default());
        let tx = recorder.start_transaction("work", "task");
        recorder.end_transaction(tx, "success");
        let dashboard = recorder.dashboard();
        assert_eq!(dashboard.total_transactions, 1);
    }
}
