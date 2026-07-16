#![forbid(unsafe_code)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogRetentionPolicyV19 {
    pub id: Uuid,
    pub service: String,
    pub level: String,
    pub retention_days: i32,
    pub archive_after_days: Option<i32>,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateLogRetentionPolicyV19 {
    pub service: String,
    pub level: String,
    pub retention_days: Option<i32>,
    pub archive_after_days: Option<i32>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateLogRetentionPolicyV19 {
    pub service: Option<String>,
    pub level: Option<String>,
    pub retention_days: Option<i32>,
    pub archive_after_days: Option<i32>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogArchiveV19 {
    pub id: Uuid,
    pub service: String,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub entry_count: i64,
    pub size_bytes: i64,
    pub archive_path: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateLogArchiveV19 {
    pub service: String,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub entry_count: i64,
    pub size_bytes: i64,
    pub archive_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogArchiveStatsV19 {
    pub total_archives: i64,
    pub total_size_bytes: i64,
    pub total_entries: i64,
    pub service_counts: std::collections::HashMap<String, i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogLifecycleEventV19 {
    pub id: Uuid,
    pub log_id: Uuid,
    pub event_type: String,
    pub details: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogSearchOptimizationV22 {
    pub index_name: String,
    pub index_size_bytes: i64,
    pub query_count: i64,
    pub avg_query_time_ms: f64,
    pub last_optimized_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogSearchPerformanceV22 {
    pub total_queries: i64,
    pub avg_response_time_ms: f64,
    pub p95_response_time_ms: f64,
    pub cache_hit_rate: f64,
    pub index_efficiency: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogRetentionStatsV19 {
    pub active_policies: i64,
    pub total_entries_managed: i64,
    pub entries_archived: i64,
    pub entries_deleted: i64,
    pub last_cleanup_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogArchiveRequestV19 {
    pub service: Option<String>,
    pub before_date: DateTime<Utc>,
    pub dry_run: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogArchiveResultV19 {
    pub entries_archived: i64,
    pub size_bytes: i64,
    pub archive_path: String,
    pub duration_ms: i64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_retention_policy_v19_serialization() {
        let policy = LogRetentionPolicyV19 {
            id: Uuid::new_v4(),
            service: "civitforge".to_string(),
            level: "info".to_string(),
            retention_days: 30,
            archive_after_days: Some(7),
            enabled: true,
            created_at: Utc::now(),
        };
        let json = serde_json::to_string(&policy).unwrap();
        assert!(json.contains("civitforge"));
        assert!(json.contains("info"));
    }

    #[test]
    fn test_log_archive_v19_serialization() {
        let archive = LogArchiveV19 {
            id: Uuid::new_v4(),
            service: "civitforge".to_string(),
            period_start: Utc::now(),
            period_end: Utc::now(),
            entry_count: 1000,
            size_bytes: 1024000,
            archive_path: "/archives/2024/01/01.log.gz".to_string(),
            created_at: Utc::now(),
        };
        let json = serde_json::to_string(&archive).unwrap();
        assert!(json.contains("/archives/2024/01/01.log.gz"));
    }

    #[test]
    fn test_create_log_retention_policy_v19() {
        let req = CreateLogRetentionPolicyV19 {
            service: "civitforge".to_string(),
            level: "error".to_string(),
            retention_days: Some(90),
            archive_after_days: Some(14),
            enabled: Some(true),
        };
        assert_eq!(req.service, "civitforge");
        assert_eq!(req.level, "error");
        assert_eq!(req.retention_days, Some(90));
    }

    #[test]
    fn test_log_archive_stats_v19() {
        let mut service_counts = std::collections::HashMap::new();
        service_counts.insert("civitforge".to_string(), 500);
        service_counts.insert("git-server".to_string(), 300);
        
        let stats = LogArchiveStatsV19 {
            total_archives: 10,
            total_size_bytes: 10240000,
            total_entries: 800,
            service_counts,
        };
        assert_eq!(stats.total_archives, 10);
        assert_eq!(stats.service_counts.len(), 2);
    }

    #[test]
    fn test_log_search_performance_v22() {
        let perf = LogSearchPerformanceV22 {
            total_queries: 10000,
            avg_response_time_ms: 15.5,
            p95_response_time_ms: 45.2,
            cache_hit_rate: 0.85,
            index_efficiency: 0.92,
        };
        assert_eq!(perf.total_queries, 10000);
        assert!(perf.cache_hit_rate > 0.8);
    }
}
