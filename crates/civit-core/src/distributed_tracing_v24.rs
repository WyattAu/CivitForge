#![forbid(unsafe_code)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceServiceHealthV20 {
    pub id: Uuid,
    pub service_name: String,
    pub health_score: f64,
    pub error_rate: f64,
    pub avg_latency_ms: f64,
    pub throughput_rps: f64,
    pub checked_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTraceServiceHealthV20 {
    pub service_name: String,
    pub health_score: Option<f64>,
    pub error_rate: Option<f64>,
    pub avg_latency_ms: Option<f64>,
    pub throughput_rps: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceCascadeFailureDetectionV20 {
    pub id: Uuid,
    pub source_service: String,
    pub affected_service: String,
    pub failure_type: String,
    pub cascade_depth: i32,
    pub detected_at: DateTime<Utc>,
    pub resolved_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTraceCascadeFailureDetectionV20 {
    pub source_service: String,
    pub affected_service: String,
    pub failure_type: String,
    pub cascade_depth: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceCircuitBreakerStateV24 {
    pub id: Uuid,
    pub service_name: String,
    pub state: String,
    pub failure_count: i32,
    pub success_count: i32,
    pub last_failure_at: Option<DateTime<Utc>>,
    pub last_state_change_at: DateTime<Utc>,
    pub timeout_seconds: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTraceCircuitBreakerStateV24 {
    pub service_name: String,
    pub state: Option<String>,
    pub timeout_seconds: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceSelfHealingSuggestionV24 {
    pub id: Uuid,
    pub service_name: String,
    pub suggestion_type: String,
    pub description: String,
    pub confidence: f64,
    pub applied: bool,
    pub created_at: DateTime<Utc>,
    pub applied_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTraceSelfHealingSuggestionV24 {
    pub service_name: String,
    pub suggestion_type: String,
    pub description: String,
    pub confidence: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceServiceHealthSummaryV20 {
    pub total_services: i64,
    pub healthy_services: i64,
    pub degraded_services: i64,
    pub unhealthy_services: i64,
    pub avg_health_score: f64,
    pub services: Vec<TraceServiceHealthV20>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceCascadeFailureSummaryV20 {
    pub total_cascades: i64,
    pub unresolved_count: i64,
    pub max_cascade_depth: i32,
    pub affected_services: Vec<String>,
    pub failure_type_counts: std::collections::HashMap<String, i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceCircuitBreakerSummaryV24 {
    pub total_circuit_breakers: i64,
    pub open_count: i64,
    pub half_open_count: i64,
    pub closed_count: i64,
    pub services: Vec<TraceCircuitBreakerStateV24>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceSelfHealingSummaryV24 {
    pub total_suggestions: i64,
    pub applied_count: i64,
    pub pending_count: i64,
    pub avg_confidence: f64,
    pub suggestions: Vec<TraceSelfHealingSuggestionV24>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceServiceHealthRequestV20 {
    pub service_name: Option<String>,
    pub since: Option<DateTime<Utc>>,
    pub limit: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceCascadeFailureRequestV20 {
    pub source_service: Option<String>,
    pub affected_service: Option<String>,
    pub failure_type: Option<String>,
    pub since: Option<DateTime<Utc>>,
    pub limit: Option<i64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trace_service_health_v20_serialization() {
        let health = TraceServiceHealthV20 {
            id: Uuid::new_v4(),
            service_name: "civitforge".to_string(),
            health_score: 0.95,
            error_rate: 0.02,
            avg_latency_ms: 45.0,
            throughput_rps: 150.0,
            checked_at: Utc::now(),
        };
        let json = serde_json::to_string(&health).unwrap();
        assert!(json.contains("civitforge"));
        assert!(json.contains("0.95"));
    }

    #[test]
    fn test_trace_cascade_failure_detection_v20_serialization() {
        let cascade = TraceCascadeFailureDetectionV20 {
            id: Uuid::new_v4(),
            source_service: "civitforge".to_string(),
            affected_service: "civit-git".to_string(),
            failure_type: "timeout".to_string(),
            cascade_depth: 2,
            detected_at: Utc::now(),
            resolved_at: None,
        };
        let json = serde_json::to_string(&cascade).unwrap();
        assert!(json.contains("civit-git"));
        assert!(json.contains("timeout"));
    }

    #[test]
    fn test_trace_circuit_breaker_state_v24() {
        let cb = TraceCircuitBreakerStateV24 {
            id: Uuid::new_v4(),
            service_name: "civitforge".to_string(),
            state: "half_open".to_string(),
            failure_count: 5,
            success_count: 2,
            last_failure_at: Some(Utc::now()),
            last_state_change_at: Utc::now(),
            timeout_seconds: 30,
        };
        assert_eq!(cb.state, "half_open");
        assert_eq!(cb.failure_count, 5);
    }

    #[test]
    fn test_trace_self_healing_suggestion_v24() {
        let suggestion = TraceSelfHealingSuggestionV24 {
            id: Uuid::new_v4(),
            service_name: "civitforge".to_string(),
            suggestion_type: "restart".to_string(),
            description: "Service appears unhealthy, suggest restart".to_string(),
            confidence: 0.85,
            applied: false,
            created_at: Utc::now(),
            applied_at: None,
        };
        assert_eq!(suggestion.suggestion_type, "restart");
        assert!(!suggestion.applied);
    }

    #[test]
    fn test_trace_service_health_summary_v20() {
        let summary = TraceServiceHealthSummaryV20 {
            total_services: 5,
            healthy_services: 3,
            degraded_services: 1,
            unhealthy_services: 1,
            avg_health_score: 0.78,
            services: vec![],
        };
        assert_eq!(summary.total_services, 5);
        assert_eq!(summary.healthy_services, 3);
    }
}
