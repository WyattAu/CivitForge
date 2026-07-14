use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamplingRule {
    pub id: Uuid,
    pub service_name: String,
    pub endpoint: String,
    pub sample_rate: f64,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSamplingRule {
    pub service_name: String,
    pub endpoint: String,
    pub sample_rate: Option<f64>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateSamplingRule {
    pub service_name: Option<String>,
    pub endpoint: Option<String>,
    pub sample_rate: Option<f64>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceAnalysis {
    pub trace_id: String,
    pub service_name: String,
    pub endpoint: String,
    pub duration_ms: f64,
    pub span_count: i64,
    pub error_count: i64,
    pub sampled: bool,
    pub analyzed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatencyRecord {
    pub id: Uuid,
    pub trace_id: String,
    pub service_name: String,
    pub endpoint: String,
    pub latency_ms: f64,
    pub recorded_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorCorrelation {
    pub trace_id: String,
    pub error_type: String,
    pub error_message: String,
    pub service_name: String,
    pub endpoint: String,
    pub span_id: Option<String>,
    pub correlated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceAnalysisFilter {
    pub service_name: Option<String>,
    pub endpoint: Option<String>,
    pub min_duration_ms: Option<f64>,
    pub max_duration_ms: Option<f64>,
    pub has_errors: Option<bool>,
    pub since: Option<DateTime<Utc>>,
    pub until: Option<DateTime<Utc>>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceAnalysisResult {
    pub analyses: Vec<TraceAnalysis>,
    pub total_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamplingRuleStats {
    pub total_rules: i64,
    pub enabled_rules: i64,
    pub avg_sample_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatencyStats {
    pub avg_latency_ms: f64,
    pub p50_latency_ms: f64,
    pub p95_latency_ms: f64,
    pub p99_latency_ms: f64,
    pub sample_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamplingRuleV2 {
    pub id: Uuid,
    pub service_name: String,
    pub endpoint: String,
    pub sample_rate: f64,
    pub max_traces_per_second: i32,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSamplingRuleV2 {
    pub service_name: String,
    pub endpoint: String,
    pub sample_rate: Option<f64>,
    pub max_traces_per_second: Option<i32>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateSamplingRuleV2 {
    pub service_name: Option<String>,
    pub endpoint: Option<String>,
    pub sample_rate: Option<f64>,
    pub max_traces_per_second: Option<i32>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceDependency {
    pub id: Uuid,
    pub parent_trace_id: String,
    pub child_trace_id: String,
    pub dependency_type: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTraceDependency {
    pub parent_trace_id: String,
    pub child_trace_id: String,
    pub dependency_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatencyAnalysis {
    pub id: Uuid,
    pub trace_id: String,
    pub service_name: String,
    pub endpoint: String,
    pub latency_ms: f64,
    pub recorded_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorCorrelationV3 {
    pub id: Uuid,
    pub trace_id: String,
    pub error_type: String,
    pub error_message: String,
    pub service_name: String,
    pub endpoint: String,
    pub span_id: Option<String>,
    pub correlated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceDependencyStats {
    pub total_dependencies: i64,
    pub unique_parent_traces: i64,
    pub unique_child_traces: i64,
}
