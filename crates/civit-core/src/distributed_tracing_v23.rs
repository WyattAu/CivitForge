#![forbid(unsafe_code)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceAnomalyDetectionV19 {
    pub id: Uuid,
    pub service_name: String,
    pub endpoint: String,
    pub anomaly_type: String,
    pub severity: String,
    pub detected_at: DateTime<Utc>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub details: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTraceAnomalyDetectionV19 {
    pub service_name: String,
    pub endpoint: String,
    pub anomaly_type: String,
    pub severity: Option<String>,
    pub details: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TracePerformanceBaselineV19 {
    pub id: Uuid,
    pub service_name: String,
    pub endpoint: String,
    pub p50_latency_ms: f64,
    pub p95_latency_ms: f64,
    pub p99_latency_ms: f64,
    pub sample_count: i32,
    pub last_updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTracePerformanceBaselineV19 {
    pub service_name: String,
    pub endpoint: String,
    pub p50_latency_ms: f64,
    pub p95_latency_ms: f64,
    pub p99_latency_ms: f64,
    pub sample_count: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateTracePerformanceBaselineV19 {
    pub p50_latency_ms: Option<f64>,
    pub p95_latency_ms: Option<f64>,
    pub p99_latency_ms: Option<f64>,
    pub sample_count: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceAnomalyAlertV23 {
    pub id: Uuid,
    pub anomaly_id: Uuid,
    pub alert_type: String,
    pub severity: String,
    pub message: String,
    pub acknowledged: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceRootCauseAnalysisV23 {
    pub id: Uuid,
    pub trace_id: String,
    pub service_name: String,
    pub endpoint: String,
    pub root_cause: String,
    pub confidence: f64,
    pub contributing_factors: Vec<String>,
    pub recommendations: Vec<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceAnomalyStatsV19 {
    pub total_anomalies: i64,
    pub unresolved_count: i64,
    pub severity_counts: std::collections::HashMap<String, i64>,
    pub service_counts: std::collections::HashMap<String, i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TracePerformanceStatsV19 {
    pub total_baselines: i64,
    pub avg_p50_latency_ms: f64,
    pub avg_p95_latency_ms: f64,
    pub avg_p99_latency_ms: f64,
    pub services_with_baselines: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceAnomalyDetectionRequestV23 {
    pub service_name: Option<String>,
    pub endpoint: Option<String>,
    pub severity: Option<String>,
    pub since: Option<DateTime<Utc>>,
    pub until: Option<DateTime<Utc>>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceRootCauseRequestV23 {
    pub trace_id: String,
    pub include_recommendations: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceAlertIntegrationV23 {
    pub anomaly_id: Uuid,
    pub alert_channel: String,
    pub notification_sent: bool,
    pub acknowledged_by: Option<String>,
    pub acknowledged_at: Option<DateTime<Utc>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trace_anomaly_detection_v19_serialization() {
        let anomaly = TraceAnomalyDetectionV19 {
            id: Uuid::new_v4(),
            service_name: "civitforge".to_string(),
            endpoint: "/api/v1/repos".to_string(),
            anomaly_type: "latency_spike".to_string(),
            severity: "warning".to_string(),
            detected_at: Utc::now(),
            resolved_at: None,
            details: serde_json::json!({"p99_latency_ms": 500.0}),
        };
        let json = serde_json::to_string(&anomaly).unwrap();
        assert!(json.contains("latency_spike"));
        assert!(json.contains("warning"));
    }

    #[test]
    fn test_trace_performance_baseline_v19_serialization() {
        let baseline = TracePerformanceBaselineV19 {
            id: Uuid::new_v4(),
            service_name: "civitforge".to_string(),
            endpoint: "/api/v1/repos".to_string(),
            p50_latency_ms: 50.0,
            p95_latency_ms: 150.0,
            p99_latency_ms: 300.0,
            sample_count: 1000,
            last_updated_at: Utc::now(),
        };
        let json = serde_json::to_string(&baseline).unwrap();
        assert!(json.contains("50.0"));
        assert!(json.contains("150.0"));
    }

    #[test]
    fn test_trace_root_cause_analysis_v23() {
        let rca = TraceRootCauseAnalysisV23 {
            id: Uuid::new_v4(),
            trace_id: "trace-123".to_string(),
            service_name: "civitforge".to_string(),
            endpoint: "/api/v1/repos".to_string(),
            root_cause: "Database connection pool exhausted".to_string(),
            confidence: 0.85,
            contributing_factors: vec![
                "High concurrent requests".to_string(),
                "Slow query detected".to_string(),
            ],
            recommendations: vec![
                "Increase connection pool size".to_string(),
                "Optimize slow queries".to_string(),
            ],
            created_at: Utc::now(),
        };
        assert_eq!(rca.contributing_factors.len(), 2);
        assert_eq!(rca.recommendations.len(), 2);
        assert!(rca.confidence > 0.8);
    }

    #[test]
    fn test_trace_anomaly_stats_v19() {
        let mut severity_counts = std::collections::HashMap::new();
        severity_counts.insert("warning".to_string(), 5);
        severity_counts.insert("critical".to_string(), 2);
        
        let mut service_counts = std::collections::HashMap::new();
        service_counts.insert("civitforge".to_string(), 7);
        
        let stats = TraceAnomalyStatsV19 {
            total_anomalies: 10,
            unresolved_count: 3,
            severity_counts,
            service_counts,
        };
        assert_eq!(stats.total_anomalies, 10);
        assert_eq!(stats.unresolved_count, 3);
    }

    #[test]
    fn test_trace_alert_integration_v23() {
        let alert = TraceAlertIntegrationV23 {
            anomaly_id: Uuid::new_v4(),
            alert_channel: "slack".to_string(),
            notification_sent: true,
            acknowledged_by: Some("admin".to_string()),
            acknowledged_at: Some(Utc::now()),
        };
        assert!(alert.notification_sent);
        assert!(alert.acknowledged_by.is_some());
    }
}
