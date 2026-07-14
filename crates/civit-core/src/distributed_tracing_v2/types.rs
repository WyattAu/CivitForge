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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamplingRuleV3 {
    pub id: Uuid,
    pub service_name: String,
    pub endpoint: String,
    pub sample_rate: f64,
    pub max_traces_per_second: i32,
    pub priority: i32,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSamplingRuleV3 {
    pub service_name: String,
    pub endpoint: String,
    pub sample_rate: Option<f64>,
    pub max_traces_per_second: Option<i32>,
    pub priority: Option<i32>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateSamplingRuleV3 {
    pub service_name: Option<String>,
    pub endpoint: Option<String>,
    pub sample_rate: Option<f64>,
    pub max_traces_per_second: Option<i32>,
    pub priority: Option<i32>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceMapEntry {
    pub id: Uuid,
    pub service_name: String,
    pub endpoint: String,
    pub call_count: i64,
    pub avg_duration_ms: f64,
    pub error_rate: f64,
    pub last_updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceMap {
    pub services: Vec<ServiceMapEntry>,
    pub total_services: i64,
    pub total_endpoints: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceDependency {
    pub from_service: String,
    pub to_service: String,
    pub call_count: i64,
    pub avg_duration_ms: f64,
    pub error_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyAnalysis {
    pub dependencies: Vec<ServiceDependency>,
    pub total_dependencies: i64,
    pub critical_paths: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapacityPlanningData {
    pub service_name: String,
    pub current_load: f64,
    pub projected_load: f64,
    pub recommended_capacity: f64,
    pub bottleneck_endpoints: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamplingRuleV4 {
    pub id: Uuid,
    pub service_name: String,
    pub endpoint: String,
    pub sample_rate: f64,
    pub max_traces_per_second: i32,
    pub priority: i32,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSamplingRuleV4 {
    pub service_name: String,
    pub endpoint: String,
    pub sample_rate: Option<f64>,
    pub max_traces_per_second: Option<i32>,
    pub priority: Option<i32>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateSamplingRuleV4 {
    pub service_name: Option<String>,
    pub endpoint: Option<String>,
    pub sample_rate: Option<f64>,
    pub max_traces_per_second: Option<i32>,
    pub priority: Option<i32>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceServiceDependency {
    pub id: Uuid,
    pub service_name: String,
    pub depends_on_service: String,
    pub call_count: i64,
    pub avg_duration_ms: f64,
    pub error_rate: f64,
    pub last_updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTraceServiceDependency {
    pub service_name: String,
    pub depends_on_service: String,
    pub call_count: Option<i64>,
    pub avg_duration_ms: Option<f64>,
    pub error_rate: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceDependencyGraph {
    pub dependencies: Vec<TraceServiceDependency>,
    pub total_services: i64,
    pub total_dependencies: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamplingRuleV5 {
    pub id: Uuid,
    pub service_name: String,
    pub endpoint: String,
    pub sample_rate: f64,
    pub max_traces_per_second: i32,
    pub priority: i32,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSamplingRuleV5 {
    pub service_name: String,
    pub endpoint: String,
    pub sample_rate: Option<f64>,
    pub max_traces_per_second: Option<i32>,
    pub priority: Option<i32>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateSamplingRuleV5 {
    pub service_name: Option<String>,
    pub endpoint: Option<String>,
    pub sample_rate: Option<f64>,
    pub max_traces_per_second: Option<i32>,
    pub priority: Option<i32>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceServiceDependencyV2 {
    pub id: Uuid,
    pub service_name: String,
    pub depends_on_service: String,
    pub call_count: i64,
    pub avg_duration_ms: f64,
    pub error_rate: f64,
    pub last_updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTraceServiceDependencyV2 {
    pub service_name: String,
    pub depends_on_service: String,
    pub call_count: Option<i64>,
    pub avg_duration_ms: Option<f64>,
    pub error_rate: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceDependencyGraphV2 {
    pub dependencies: Vec<TraceServiceDependencyV2>,
    pub total_services: i64,
    pub total_dependencies: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatencyAnalysisV2 {
    pub service_name: String,
    pub endpoint: String,
    pub avg_latency_ms: f64,
    pub p50_latency_ms: f64,
    pub p95_latency_ms: f64,
    pub p99_latency_ms: f64,
    pub sample_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorCorrelationV4 {
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
pub struct CapacityPlanningDataV2 {
    pub service_name: String,
    pub current_load: f64,
    pub projected_load: f64,
    pub recommended_capacity: f64,
    pub bottleneck_endpoints: Vec<String>,
    pub growth_rate: f64,
    pub time_to_capacity_hours: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamplingRuleV6 {
    pub id: Uuid,
    pub service_name: String,
    pub endpoint: String,
    pub sample_rate: f64,
    pub max_traces_per_second: i32,
    pub priority: i32,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSamplingRuleV6 {
    pub service_name: String,
    pub endpoint: String,
    pub sample_rate: Option<f64>,
    pub max_traces_per_second: Option<i32>,
    pub priority: Option<i32>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateSamplingRuleV6 {
    pub service_name: Option<String>,
    pub endpoint: Option<String>,
    pub sample_rate: Option<f64>,
    pub max_traces_per_second: Option<i32>,
    pub priority: Option<i32>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceServiceDependencyV3 {
    pub id: Uuid,
    pub service_name: String,
    pub depends_on_service: String,
    pub call_count: i64,
    pub avg_duration_ms: f64,
    pub error_rate: f64,
    pub last_updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTraceServiceDependencyV3 {
    pub service_name: String,
    pub depends_on_service: String,
    pub call_count: Option<i64>,
    pub avg_duration_ms: Option<f64>,
    pub error_rate: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceDependencyGraphV3 {
    pub dependencies: Vec<TraceServiceDependencyV3>,
    pub total_services: i64,
    pub total_dependencies: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatencyAnalysisV3 {
    pub service_name: String,
    pub endpoint: String,
    pub avg_latency_ms: f64,
    pub p50_latency_ms: f64,
    pub p95_latency_ms: f64,
    pub p99_latency_ms: f64,
    pub sample_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorCorrelationV5 {
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
pub struct CapacityPlanningDataV3 {
    pub service_name: String,
    pub current_load: f64,
    pub projected_load: f64,
    pub recommended_capacity: f64,
    pub bottleneck_endpoints: Vec<String>,
    pub growth_rate: f64,
    pub time_to_capacity_hours: f64,
    pub utilization_score: f64,
}
