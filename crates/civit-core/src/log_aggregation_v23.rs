#![forbid(unsafe_code)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogIndexOptimizationV20 {
    pub id: Uuid,
    pub index_name: String,
    pub table_name: String,
    pub columns: Vec<String>,
    pub query_pattern: Option<String>,
    pub improvement_percent: Option<f64>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateLogIndexOptimizationV20 {
    pub index_name: String,
    pub table_name: String,
    pub columns: Vec<String>,
    pub query_pattern: Option<String>,
    pub improvement_percent: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogCompressionStatsV20 {
    pub id: Uuid,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub original_bytes: i64,
    pub compressed_bytes: i64,
    pub compression_ratio: f64,
    pub entry_count: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateLogCompressionStatsV20 {
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub original_bytes: i64,
    pub compressed_bytes: i64,
    pub compression_ratio: f64,
    pub entry_count: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogIndexOptimizationSummaryV20 {
    pub total_optimizations: i64,
    pub avg_improvement_percent: f64,
    pub tables_optimized: Vec<String>,
    pub last_optimized_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogCompressionSummaryV20 {
    pub total_periods: i64,
    pub total_original_bytes: i64,
    pub total_compressed_bytes: i64,
    pub avg_compression_ratio: f64,
    pub total_entries_compressed: i64,
    pub last_compressed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogQueryPerformanceV23 {
    pub query_pattern: String,
    pub avg_execution_time_ms: f64,
    pub p95_execution_time_ms: f64,
    pub total_executions: i64,
    pub suggested_index: Option<String>,
    pub estimated_improvement_percent: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogStorageOptimizationV23 {
    pub table_name: String,
    pub current_size_bytes: i64,
    pub estimated_optimizable_bytes: i64,
    pub optimization_suggestions: Vec<String>,
    pub last_analyzed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogIndexOptimizationRequestV20 {
    pub table_name: Option<String>,
    pub limit: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogCompressionRequestV20 {
    pub since: Option<DateTime<Utc>>,
    pub until: Option<DateTime<Utc>>,
    pub limit: Option<i64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_index_optimization_v20_serialization() {
        let opt = LogIndexOptimizationV20 {
            id: Uuid::new_v4(),
            index_name: "idx_log_entries_service".to_string(),
            table_name: "log_entries_v7".to_string(),
            columns: vec!["service".to_string(), "created_at".to_string()],
            query_pattern: Some("SELECT * FROM log_entries_v7 WHERE service = $1".to_string()),
            improvement_percent: Some(45.5),
            created_at: Utc::now(),
        };
        let json = serde_json::to_string(&opt).unwrap();
        assert!(json.contains("idx_log_entries_service"));
        assert!(json.contains("log_entries_v7"));
    }

    #[test]
    fn test_log_compression_stats_v20_serialization() {
        let stats = LogCompressionStatsV20 {
            id: Uuid::new_v4(),
            period_start: Utc::now(),
            period_end: Utc::now(),
            original_bytes: 1024000,
            compressed_bytes: 512000,
            compression_ratio: 0.5,
            entry_count: 5000,
            created_at: Utc::now(),
        };
        let json = serde_json::to_string(&stats).unwrap();
        assert!(json.contains("1024000"));
        assert!(json.contains("512000"));
    }

    #[test]
    fn test_create_log_index_optimization_v20() {
        let req = CreateLogIndexOptimizationV20 {
            index_name: "idx_log_entries_level".to_string(),
            table_name: "log_entries_v7".to_string(),
            columns: vec!["level".to_string()],
            query_pattern: None,
            improvement_percent: Some(30.0),
        };
        assert_eq!(req.index_name, "idx_log_entries_level");
        assert_eq!(req.columns.len(), 1);
    }

    #[test]
    fn test_log_query_performance_v23() {
        let perf = LogQueryPerformanceV23 {
            query_pattern: "SELECT * FROM log_entries_v7 WHERE service = $1".to_string(),
            avg_execution_time_ms: 12.5,
            p95_execution_time_ms: 45.0,
            total_executions: 10000,
            suggested_index: Some("idx_log_entries_v7_service".to_string()),
            estimated_improvement_percent: 60.0,
        };
        assert_eq!(perf.total_executions, 10000);
        assert!(perf.suggested_index.is_some());
    }

    #[test]
    fn test_log_storage_optimization_v23() {
        let opt = LogStorageOptimizationV23 {
            table_name: "log_entries_v7".to_string(),
            current_size_bytes: 102400000,
            estimated_optimizable_bytes: 30720000,
            optimization_suggestions: vec![
                "Add index on service column".to_string(),
                "Enable compression for older entries".to_string(),
            ],
            last_analyzed_at: Utc::now(),
        };
        assert_eq!(opt.optimization_suggestions.len(), 2);
    }
}
