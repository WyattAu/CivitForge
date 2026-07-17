use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamplingRuleV7 {
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
pub struct CreateSamplingRuleV7 {
    pub service_name: String,
    pub endpoint: String,
    pub sample_rate: Option<f64>,
    pub max_traces_per_second: Option<i32>,
    pub priority: Option<i32>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateSamplingRuleV7 {
    pub service_name: Option<String>,
    pub endpoint: Option<String>,
    pub sample_rate: Option<f64>,
    pub max_traces_per_second: Option<i32>,
    pub priority: Option<i32>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceServiceDependencyV4 {
    pub id: Uuid,
    pub service_name: String,
    pub depends_on_service: String,
    pub call_count: i64,
    pub avg_duration_ms: f64,
    pub error_rate: f64,
    pub last_updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTraceServiceDependencyV4 {
    pub service_name: String,
    pub depends_on_service: String,
    pub call_count: Option<i64>,
    pub avg_duration_ms: Option<f64>,
    pub error_rate: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceDependencyGraphV4 {
    pub dependencies: Vec<TraceServiceDependencyV4>,
    pub total_services: i64,
    pub total_dependencies: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatencyAnalysisV4 {
    pub service_name: String,
    pub endpoint: String,
    pub avg_latency_ms: f64,
    pub p50_latency_ms: f64,
    pub p95_latency_ms: f64,
    pub p99_latency_ms: f64,
    pub sample_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorCorrelationV6 {
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
pub struct CapacityPlanningDataV4 {
    pub service_name: String,
    pub current_load: f64,
    pub projected_load: f64,
    pub recommended_capacity: f64,
    pub bottleneck_endpoints: Vec<String>,
    pub growth_rate: f64,
    pub time_to_capacity_hours: f64,
    pub utilization_score: f64,
    pub recommended_replicas: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatencyTrend {
    pub service_name: String,
    pub endpoint: String,
    pub timestamp: DateTime<Utc>,
    pub avg_latency_ms: f64,
    pub p95_latency_ms: f64,
    pub request_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorCorrelationSummary {
    pub service_name: String,
    pub endpoint: String,
    pub error_type: String,
    pub error_count: i64,
    pub first_seen_at: DateTime<Utc>,
    pub last_seen_at: DateTime<Utc>,
    pub affected_traces: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapacityRecommendation {
    pub service_name: String,
    pub current_capacity: f64,
    pub projected_peak: f64,
    pub recommended_capacity: f64,
    pub scale_up_threshold: f64,
    pub scale_down_threshold: f64,
    pub estimated_cost_impact: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamplingRuleV8 {
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
pub struct CreateSamplingRuleV8 {
    pub service_name: String,
    pub endpoint: String,
    pub sample_rate: Option<f64>,
    pub max_traces_per_second: Option<i32>,
    pub priority: Option<i32>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateSamplingRuleV8 {
    pub service_name: Option<String>,
    pub endpoint: Option<String>,
    pub sample_rate: Option<f64>,
    pub max_traces_per_second: Option<i32>,
    pub priority: Option<i32>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceServiceDependencyV5 {
    pub id: Uuid,
    pub service_name: String,
    pub depends_on_service: String,
    pub call_count: i64,
    pub avg_duration_ms: f64,
    pub error_rate: f64,
    pub last_updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTraceServiceDependencyV5 {
    pub service_name: String,
    pub depends_on_service: String,
    pub call_count: Option<i64>,
    pub avg_duration_ms: Option<f64>,
    pub error_rate: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceDependencyGraphV5 {
    pub dependencies: Vec<TraceServiceDependencyV5>,
    pub total_services: i64,
    pub total_dependencies: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatencyAnalysisV5 {
    pub service_name: String,
    pub endpoint: String,
    pub avg_latency_ms: f64,
    pub p50_latency_ms: f64,
    pub p95_latency_ms: f64,
    pub p99_latency_ms: f64,
    pub sample_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorCorrelationV7 {
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
pub struct CapacityPlanningDataV5 {
    pub service_name: String,
    pub current_load: f64,
    pub projected_load: f64,
    pub recommended_capacity: f64,
    pub bottleneck_endpoints: Vec<String>,
    pub growth_rate: f64,
    pub time_to_capacity_hours: f64,
    pub utilization_score: f64,
    pub recommended_replicas: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamplingRuleV9 {
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
pub struct CreateSamplingRuleV9 {
    pub service_name: String,
    pub endpoint: String,
    pub sample_rate: Option<f64>,
    pub max_traces_per_second: Option<i32>,
    pub priority: Option<i32>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateSamplingRuleV9 {
    pub service_name: Option<String>,
    pub endpoint: Option<String>,
    pub sample_rate: Option<f64>,
    pub max_traces_per_second: Option<i32>,
    pub priority: Option<i32>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceServiceDependencyV6 {
    pub id: Uuid,
    pub service_name: String,
    pub depends_on_service: String,
    pub call_count: i64,
    pub avg_duration_ms: f64,
    pub error_rate: f64,
    pub last_updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTraceServiceDependencyV6 {
    pub service_name: String,
    pub depends_on_service: String,
    pub call_count: Option<i64>,
    pub avg_duration_ms: Option<f64>,
    pub error_rate: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceDependencyGraphV6 {
    pub dependencies: Vec<TraceServiceDependencyV6>,
    pub total_services: i64,
    pub total_dependencies: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatencyAnalysisV6 {
    pub service_name: String,
    pub endpoint: String,
    pub avg_latency_ms: f64,
    pub p50_latency_ms: f64,
    pub p95_latency_ms: f64,
    pub p99_latency_ms: f64,
    pub sample_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorCorrelationV8 {
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
pub struct CapacityPlanningDataV6 {
    pub service_name: String,
    pub current_load: f64,
    pub projected_load: f64,
    pub recommended_capacity: f64,
    pub bottleneck_endpoints: Vec<String>,
    pub growth_rate: f64,
    pub time_to_capacity_hours: f64,
    pub utilization_score: f64,
    pub recommended_replicas: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamplingRuleV10 {
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
pub struct CreateSamplingRuleV10 {
    pub service_name: String,
    pub endpoint: String,
    pub sample_rate: Option<f64>,
    pub max_traces_per_second: Option<i32>,
    pub priority: Option<i32>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateSamplingRuleV10 {
    pub service_name: Option<String>,
    pub endpoint: Option<String>,
    pub sample_rate: Option<f64>,
    pub max_traces_per_second: Option<i32>,
    pub priority: Option<i32>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceServiceDependencyV7 {
    pub id: Uuid,
    pub service_name: String,
    pub depends_on_service: String,
    pub call_count: i64,
    pub avg_duration_ms: f64,
    pub error_rate: f64,
    pub last_updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTraceServiceDependencyV7 {
    pub service_name: String,
    pub depends_on_service: String,
    pub call_count: Option<i64>,
    pub avg_duration_ms: Option<f64>,
    pub error_rate: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceDependencyGraphV7 {
    pub dependencies: Vec<TraceServiceDependencyV7>,
    pub total_services: i64,
    pub total_dependencies: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatencyAnalysisV7 {
    pub service_name: String,
    pub endpoint: String,
    pub avg_latency_ms: f64,
    pub p50_latency_ms: f64,
    pub p95_latency_ms: f64,
    pub p99_latency_ms: f64,
    pub sample_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorCorrelationV9 {
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
pub struct CapacityPlanningDataV7 {
    pub service_name: String,
    pub current_load: f64,
    pub projected_load: f64,
    pub recommended_capacity: f64,
    pub bottleneck_endpoints: Vec<String>,
    pub growth_rate: f64,
    pub time_to_capacity_hours: f64,
    pub utilization_score: f64,
    pub recommended_replicas: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamplingRuleV11 {
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
pub struct CreateSamplingRuleV11 {
    pub service_name: String,
    pub endpoint: String,
    pub sample_rate: Option<f64>,
    pub max_traces_per_second: Option<i32>,
    pub priority: Option<i32>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateSamplingRuleV11 {
    pub service_name: Option<String>,
    pub endpoint: Option<String>,
    pub sample_rate: Option<f64>,
    pub max_traces_per_second: Option<i32>,
    pub priority: Option<i32>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceServiceDependencyV8 {
    pub id: Uuid,
    pub service_name: String,
    pub depends_on_service: String,
    pub call_count: i64,
    pub avg_duration_ms: f64,
    pub error_rate: f64,
    pub last_updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTraceServiceDependencyV8 {
    pub service_name: String,
    pub depends_on_service: String,
    pub call_count: Option<i64>,
    pub avg_duration_ms: Option<f64>,
    pub error_rate: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceDependencyGraphV8 {
    pub dependencies: Vec<TraceServiceDependencyV8>,
    pub total_services: i64,
    pub total_dependencies: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatencyAnalysisV8 {
    pub service_name: String,
    pub endpoint: String,
    pub avg_latency_ms: f64,
    pub p50_latency_ms: f64,
    pub p95_latency_ms: f64,
    pub p99_latency_ms: f64,
    pub sample_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapacityPlanningDataV8 {
    pub service_name: String,
    pub current_load: f64,
    pub projected_load: f64,
    pub recommended_capacity: f64,
    pub bottleneck_endpoints: Vec<String>,
    pub growth_rate: f64,
    pub time_to_capacity_hours: f64,
    pub utilization_score: f64,
    pub recommended_replicas: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatencyTrendV2 {
    pub service_name: String,
    pub endpoint: String,
    pub timestamp: DateTime<Utc>,
    pub avg_latency_ms: f64,
    pub p95_latency_ms: f64,
    pub request_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorCorrelationSummaryV2 {
    pub service_name: String,
    pub endpoint: String,
    pub error_type: String,
    pub error_count: i64,
    pub first_seen_at: DateTime<Utc>,
    pub last_seen_at: DateTime<Utc>,
    pub affected_traces: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapacityRecommendationV2 {
    pub service_name: String,
    pub current_capacity: f64,
    pub projected_peak: f64,
    pub recommended_capacity: f64,
    pub scale_up_threshold: f64,
    pub scale_down_threshold: f64,
    pub estimated_cost_impact: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamplingRuleV12 {
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
pub struct CreateSamplingRuleV12 {
    pub service_name: String,
    pub endpoint: String,
    pub sample_rate: Option<f64>,
    pub max_traces_per_second: Option<i32>,
    pub priority: Option<i32>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateSamplingRuleV12 {
    pub service_name: Option<String>,
    pub endpoint: Option<String>,
    pub sample_rate: Option<f64>,
    pub max_traces_per_second: Option<i32>,
    pub priority: Option<i32>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceServiceDependencyV9 {
    pub id: Uuid,
    pub service_name: String,
    pub depends_on_service: String,
    pub call_count: i64,
    pub avg_duration_ms: f64,
    pub error_rate: f64,
    pub last_updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTraceServiceDependencyV9 {
    pub service_name: String,
    pub depends_on_service: String,
    pub call_count: Option<i64>,
    pub avg_duration_ms: Option<f64>,
    pub error_rate: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceDependencyGraphV9 {
    pub dependencies: Vec<TraceServiceDependencyV9>,
    pub total_services: i64,
    pub total_dependencies: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamplingRuleV13 {
    pub id: uuid::Uuid,
    pub service_name: String,
    pub endpoint: String,
    pub sample_rate: f64,
    pub max_traces_per_second: i32,
    pub priority: i32,
    pub enabled: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSamplingRuleV13 {
    pub service_name: String,
    pub endpoint: String,
    pub sample_rate: Option<f64>,
    pub max_traces_per_second: Option<i32>,
    pub priority: Option<i32>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateSamplingRuleV13 {
    pub service_name: Option<String>,
    pub endpoint: Option<String>,
    pub sample_rate: Option<f64>,
    pub max_traces_per_second: Option<i32>,
    pub priority: Option<i32>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceServiceDependencyV10 {
    pub id: uuid::Uuid,
    pub service_name: String,
    pub depends_on_service: String,
    pub call_count: i64,
    pub avg_duration_ms: f64,
    pub error_rate: f64,
    pub last_updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTraceServiceDependencyV10 {
    pub service_name: String,
    pub depends_on_service: String,
    pub call_count: Option<i64>,
    pub avg_duration_ms: Option<f64>,
    pub error_rate: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceDependencyGraphV10 {
    pub dependencies: Vec<TraceServiceDependencyV10>,
    pub total_services: i64,
    pub total_dependencies: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatencyAnalysisV14 {
    pub service_name: String,
    pub endpoint: String,
    pub avg_latency_ms: f64,
    pub p50_latency_ms: f64,
    pub p95_latency_ms: f64,
    pub p99_latency_ms: f64,
    pub sample_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorCorrelationV10 {
    pub id: uuid::Uuid,
    pub trace_id: String,
    pub error_type: String,
    pub error_message: String,
    pub service_name: String,
    pub endpoint: String,
    pub span_id: Option<String>,
    pub correlated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapacityPlanningDataV10 {
    pub service_name: String,
    pub current_load: f64,
    pub projected_load: f64,
    pub recommended_capacity: f64,
    pub bottleneck_endpoints: Vec<String>,
    pub growth_rate: f64,
    pub time_to_capacity_hours: f64,
    pub utilization_score: f64,
    pub recommended_replicas: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamplingRuleV14 {
    pub id: uuid::Uuid,
    pub service_name: String,
    pub endpoint: String,
    pub sample_rate: f64,
    pub max_traces_per_second: i32,
    pub priority: i32,
    pub enabled: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSamplingRuleV14 {
    pub service_name: String,
    pub endpoint: String,
    pub sample_rate: Option<f64>,
    pub max_traces_per_second: Option<i32>,
    pub priority: Option<i32>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateSamplingRuleV14 {
    pub service_name: Option<String>,
    pub endpoint: Option<String>,
    pub sample_rate: Option<f64>,
    pub max_traces_per_second: Option<i32>,
    pub priority: Option<i32>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceServiceDependencyV11 {
    pub id: uuid::Uuid,
    pub service_name: String,
    pub depends_on_service: String,
    pub call_count: i64,
    pub avg_duration_ms: f64,
    pub error_rate: f64,
    pub last_updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTraceServiceDependencyV11 {
    pub service_name: String,
    pub depends_on_service: String,
    pub call_count: Option<i64>,
    pub avg_duration_ms: Option<f64>,
    pub error_rate: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceDependencyGraphV11 {
    pub dependencies: Vec<TraceServiceDependencyV11>,
    pub total_services: i64,
    pub total_dependencies: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatencyAnalysisV15 {
    pub service_name: String,
    pub endpoint: String,
    pub avg_latency_ms: f64,
    pub p50_latency_ms: f64,
    pub p95_latency_ms: f64,
    pub p99_latency_ms: f64,
    pub sample_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorCorrelationV11 {
    pub id: uuid::Uuid,
    pub trace_id: String,
    pub error_type: String,
    pub error_message: String,
    pub service_name: String,
    pub endpoint: String,
    pub span_id: Option<String>,
    pub correlated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapacityPlanningDataV11 {
    pub service_name: String,
    pub current_load: f64,
    pub projected_load: f64,
    pub recommended_capacity: f64,
    pub bottleneck_endpoints: Vec<String>,
    pub growth_rate: f64,
    pub time_to_capacity_hours: f64,
    pub utilization_score: f64,
    pub recommended_replicas: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamplingRuleV15 {
    pub id: uuid::Uuid,
    pub service_name: String,
    pub endpoint: String,
    pub sample_rate: f64,
    pub max_traces_per_second: i32,
    pub priority: i32,
    pub enabled: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSamplingRuleV15 {
    pub service_name: String,
    pub endpoint: String,
    pub sample_rate: Option<f64>,
    pub max_traces_per_second: Option<i32>,
    pub priority: Option<i32>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateSamplingRuleV15 {
    pub service_name: Option<String>,
    pub endpoint: Option<String>,
    pub sample_rate: Option<f64>,
    pub max_traces_per_second: Option<i32>,
    pub priority: Option<i32>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceServiceDependencyV12 {
    pub id: uuid::Uuid,
    pub service_name: String,
    pub depends_on_service: String,
    pub call_count: i64,
    pub avg_duration_ms: f64,
    pub error_rate: f64,
    pub last_updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTraceServiceDependencyV12 {
    pub service_name: String,
    pub depends_on_service: String,
    pub call_count: Option<i64>,
    pub avg_duration_ms: Option<f64>,
    pub error_rate: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceDependencyGraphV12 {
    pub dependencies: Vec<TraceServiceDependencyV12>,
    pub total_services: i64,
    pub total_dependencies: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatencyAnalysisV16 {
    pub service_name: String,
    pub endpoint: String,
    pub avg_latency_ms: f64,
    pub p50_latency_ms: f64,
    pub p95_latency_ms: f64,
    pub p99_latency_ms: f64,
    pub sample_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorCorrelationV12 {
    pub id: uuid::Uuid,
    pub trace_id: String,
    pub error_type: String,
    pub error_message: String,
    pub service_name: String,
    pub endpoint: String,
    pub span_id: Option<String>,
    pub correlated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapacityPlanningDataV12 {
    pub service_name: String,
    pub current_load: f64,
    pub projected_load: f64,
    pub recommended_capacity: f64,
    pub bottleneck_endpoints: Vec<String>,
    pub growth_rate: f64,
    pub time_to_capacity_hours: f64,
    pub utilization_score: f64,
    pub recommended_replicas: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamplingRuleV16 {
    pub id: uuid::Uuid,
    pub service_name: String,
    pub endpoint: String,
    pub sample_rate: f64,
    pub max_traces_per_second: i32,
    pub priority: i32,
    pub enabled: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSamplingRuleV16 {
    pub service_name: String,
    pub endpoint: String,
    pub sample_rate: Option<f64>,
    pub max_traces_per_second: Option<i32>,
    pub priority: Option<i32>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateSamplingRuleV16 {
    pub service_name: Option<String>,
    pub endpoint: Option<String>,
    pub sample_rate: Option<f64>,
    pub max_traces_per_second: Option<i32>,
    pub priority: Option<i32>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceServiceDependencyV13 {
    pub id: uuid::Uuid,
    pub service_name: String,
    pub depends_on_service: String,
    pub call_count: i64,
    pub avg_duration_ms: f64,
    pub error_rate: f64,
    pub last_updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTraceServiceDependencyV13 {
    pub service_name: String,
    pub depends_on_service: String,
    pub call_count: Option<i64>,
    pub avg_duration_ms: Option<f64>,
    pub error_rate: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceDependencyGraphV13 {
    pub dependencies: Vec<TraceServiceDependencyV13>,
    pub total_services: i64,
    pub total_dependencies: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatencyAnalysisV17 {
    pub service_name: String,
    pub endpoint: String,
    pub avg_latency_ms: f64,
    pub p50_latency_ms: f64,
    pub p95_latency_ms: f64,
    pub p99_latency_ms: f64,
    pub sample_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorCorrelationV13 {
    pub id: uuid::Uuid,
    pub trace_id: String,
    pub error_type: String,
    pub error_message: String,
    pub service_name: String,
    pub endpoint: String,
    pub span_id: Option<String>,
    pub correlated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapacityPlanningDataV13 {
    pub service_name: String,
    pub current_load: f64,
    pub projected_load: f64,
    pub recommended_capacity: f64,
    pub bottleneck_endpoints: Vec<String>,
    pub growth_rate: f64,
    pub time_to_capacity_hours: f64,
    pub utilization_score: f64,
    pub recommended_replicas: i32,
}

// V18: Sampling rules v17 and service dependencies v14

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamplingRuleV17 {
    pub id: uuid::Uuid,
    pub service_name: String,
    pub endpoint: String,
    pub sample_rate: f64,
    pub max_traces_per_second: i32,
    pub priority: i32,
    pub enabled: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSamplingRuleV17 {
    pub service_name: String,
    pub endpoint: String,
    pub sample_rate: Option<f64>,
    pub max_traces_per_second: Option<i32>,
    pub priority: Option<i32>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateSamplingRuleV17 {
    pub service_name: Option<String>,
    pub endpoint: Option<String>,
    pub sample_rate: Option<f64>,
    pub max_traces_per_second: Option<i32>,
    pub priority: Option<i32>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceServiceDependencyV14 {
    pub id: uuid::Uuid,
    pub service_name: String,
    pub depends_on_service: String,
    pub call_count: i64,
    pub avg_duration_ms: f64,
    pub error_rate: f64,
    pub last_updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTraceServiceDependencyV14 {
    pub service_name: String,
    pub depends_on_service: String,
    pub call_count: Option<i64>,
    pub avg_duration_ms: Option<f64>,
    pub error_rate: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceDependencyGraphV14 {
    pub dependencies: Vec<TraceServiceDependencyV14>,
    pub total_services: i64,
    pub total_dependencies: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatencyAnalysisV18 {
    pub service_name: String,
    pub endpoint: String,
    pub avg_latency_ms: f64,
    pub p50_latency_ms: f64,
    pub p95_latency_ms: f64,
    pub p99_latency_ms: f64,
    pub sample_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorCorrelationV14 {
    pub id: uuid::Uuid,
    pub trace_id: String,
    pub error_type: String,
    pub error_message: String,
    pub service_name: String,
    pub endpoint: String,
    pub span_id: Option<String>,
    pub correlated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapacityPlanningDataV14 {
    pub service_name: String,
    pub current_load: f64,
    pub projected_load: f64,
    pub recommended_capacity: f64,
    pub bottleneck_endpoints: Vec<String>,
    pub growth_rate: f64,
    pub time_to_capacity_hours: f64,
    pub utilization_score: f64,
    pub recommended_replicas: i32,
}

// V19: Enhanced distributed tracing with service dependency tracking

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamplingRuleV18 {
    pub id: uuid::Uuid,
    pub service_name: String,
    pub endpoint: String,
    pub sample_rate: f64,
    pub max_traces_per_second: i32,
    pub priority: i32,
    pub enabled: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSamplingRuleV18 {
    pub service_name: String,
    pub endpoint: String,
    pub sample_rate: Option<f64>,
    pub max_traces_per_second: Option<i32>,
    pub priority: Option<i32>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateSamplingRuleV18 {
    pub service_name: Option<String>,
    pub endpoint: Option<String>,
    pub sample_rate: Option<f64>,
    pub max_traces_per_second: Option<i32>,
    pub priority: Option<i32>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceServiceDependencyV15 {
    pub id: uuid::Uuid,
    pub service_name: String,
    pub depends_on_service: String,
    pub call_count: i64,
    pub avg_duration_ms: f64,
    pub error_rate: f64,
    pub last_updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTraceServiceDependencyV15 {
    pub service_name: String,
    pub depends_on_service: String,
    pub call_count: Option<i64>,
    pub avg_duration_ms: Option<f64>,
    pub error_rate: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceDependencyGraphV15 {
    pub dependencies: Vec<TraceServiceDependencyV15>,
    pub total_services: i64,
    pub total_dependencies: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatencyAnalysisV19 {
    pub service_name: String,
    pub endpoint: String,
    pub avg_latency_ms: f64,
    pub p50_latency_ms: f64,
    pub p95_latency_ms: f64,
    pub p99_latency_ms: f64,
    pub sample_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorCorrelationV15 {
    pub id: uuid::Uuid,
    pub trace_id: String,
    pub error_type: String,
    pub error_message: String,
    pub service_name: String,
    pub endpoint: String,
    pub span_id: Option<String>,
    pub correlated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapacityPlanningDataV15 {
    pub service_name: String,
    pub current_load: f64,
    pub projected_load: f64,
    pub recommended_capacity: f64,
    pub bottleneck_endpoints: Vec<String>,
    pub growth_rate: f64,
    pub time_to_capacity_hours: f64,
    pub utilization_score: f64,
    pub recommended_replicas: i32,
}

// V20: Enhanced distributed tracing with service dependency tracking and latency analysis

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamplingRuleV19 {
    pub id: uuid::Uuid,
    pub service_name: String,
    pub endpoint: String,
    pub sample_rate: f64,
    pub max_traces_per_second: i32,
    pub priority: i32,
    pub enabled: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSamplingRuleV19 {
    pub service_name: String,
    pub endpoint: String,
    pub sample_rate: Option<f64>,
    pub max_traces_per_second: Option<i32>,
    pub priority: Option<i32>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateSamplingRuleV19 {
    pub service_name: Option<String>,
    pub endpoint: Option<String>,
    pub sample_rate: Option<f64>,
    pub max_traces_per_second: Option<i32>,
    pub priority: Option<i32>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceServiceDependencyV16 {
    pub id: uuid::Uuid,
    pub service_name: String,
    pub depends_on_service: String,
    pub call_count: i64,
    pub avg_duration_ms: f64,
    pub error_rate: f64,
    pub last_updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTraceServiceDependencyV16 {
    pub service_name: String,
    pub depends_on_service: String,
    pub call_count: Option<i64>,
    pub avg_duration_ms: Option<f64>,
    pub error_rate: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceDependencyGraphV16 {
    pub dependencies: Vec<TraceServiceDependencyV16>,
    pub total_services: i64,
    pub total_dependencies: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatencyAnalysisV20 {
    pub service_name: String,
    pub endpoint: String,
    pub avg_latency_ms: f64,
    pub p50_latency_ms: f64,
    pub p95_latency_ms: f64,
    pub p99_latency_ms: f64,
    pub sample_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorCorrelationV16 {
    pub id: uuid::Uuid,
    pub trace_id: String,
    pub error_type: String,
    pub error_message: String,
    pub service_name: String,
    pub endpoint: String,
    pub span_id: Option<String>,
    pub correlated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapacityPlanningDataV16 {
    pub service_name: String,
    pub current_load: f64,
    pub projected_load: f64,
    pub recommended_capacity: f64,
    pub bottleneck_endpoints: Vec<String>,
    pub growth_rate: f64,
    pub time_to_capacity_hours: f64,
    pub utilization_score: f64,
    pub recommended_replicas: i32,
}

// V21: Enhanced distributed tracing v21 with advanced sampling rules v20 and service dependencies v17

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamplingRuleV20 {
    pub id: uuid::Uuid,
    pub service_name: String,
    pub endpoint: String,
    pub sample_rate: f64,
    pub max_traces_per_second: i32,
    pub priority: i32,
    pub enabled: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSamplingRuleV20 {
    pub service_name: String,
    pub endpoint: String,
    pub sample_rate: Option<f64>,
    pub max_traces_per_second: Option<i32>,
    pub priority: Option<i32>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateSamplingRuleV20 {
    pub service_name: Option<String>,
    pub endpoint: Option<String>,
    pub sample_rate: Option<f64>,
    pub max_traces_per_second: Option<i32>,
    pub priority: Option<i32>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceServiceDependencyV17 {
    pub id: uuid::Uuid,
    pub service_name: String,
    pub depends_on_service: String,
    pub call_count: i64,
    pub avg_duration_ms: f64,
    pub error_rate: f64,
    pub last_updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTraceServiceDependencyV17 {
    pub service_name: String,
    pub depends_on_service: String,
    pub call_count: Option<i64>,
    pub avg_duration_ms: Option<f64>,
    pub error_rate: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceDependencyGraphV17 {
    pub dependencies: Vec<TraceServiceDependencyV17>,
    pub total_services: i64,
    pub total_dependencies: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatencyAnalysisV21 {
    pub service_name: String,
    pub endpoint: String,
    pub avg_latency_ms: f64,
    pub p50_latency_ms: f64,
    pub p95_latency_ms: f64,
    pub p99_latency_ms: f64,
    pub sample_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorCorrelationV17 {
    pub id: uuid::Uuid,
    pub trace_id: String,
    pub error_type: String,
    pub error_message: String,
    pub service_name: String,
    pub endpoint: String,
    pub span_id: Option<String>,
    pub correlated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapacityPlanningDataV17 {
    pub service_name: String,
    pub current_load: f64,
    pub projected_load: f64,
    pub recommended_capacity: f64,
    pub bottleneck_endpoints: Vec<String>,
    pub growth_rate: f64,
    pub time_to_capacity_hours: f64,
    pub utilization_score: f64,
    pub recommended_replicas: i32,
}

// V22: Enhanced distributed tracing v22 with sampling rules v21 and service dependencies v18

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamplingRuleV21 {
    pub id: uuid::Uuid,
    pub service_name: String,
    pub endpoint: String,
    pub sample_rate: f64,
    pub max_traces_per_second: i32,
    pub priority: i32,
    pub enabled: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSamplingRuleV21 {
    pub service_name: String,
    pub endpoint: String,
    pub sample_rate: Option<f64>,
    pub max_traces_per_second: Option<i32>,
    pub priority: Option<i32>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateSamplingRuleV21 {
    pub service_name: Option<String>,
    pub endpoint: Option<String>,
    pub sample_rate: Option<f64>,
    pub max_traces_per_second: Option<i32>,
    pub priority: Option<i32>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceServiceDependencyV18 {
    pub id: uuid::Uuid,
    pub service_name: String,
    pub depends_on_service: String,
    pub call_count: i64,
    pub avg_duration_ms: f64,
    pub error_rate: f64,
    pub last_updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTraceServiceDependencyV18 {
    pub service_name: String,
    pub depends_on_service: String,
    pub call_count: Option<i64>,
    pub avg_duration_ms: Option<f64>,
    pub error_rate: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceDependencyGraphV18 {
    pub dependencies: Vec<TraceServiceDependencyV18>,
    pub total_services: i64,
    pub total_dependencies: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatencyAnalysisV22 {
    pub service_name: String,
    pub endpoint: String,
    pub avg_latency_ms: f64,
    pub p50_latency_ms: f64,
    pub p95_latency_ms: f64,
    pub p99_latency_ms: f64,
    pub sample_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorCorrelationV18 {
    pub id: uuid::Uuid,
    pub trace_id: String,
    pub error_type: String,
    pub error_message: String,
    pub service_name: String,
    pub endpoint: String,
    pub span_id: Option<String>,
    pub correlated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapacityPlanningDataV18 {
    pub service_name: String,
    pub current_load: f64,
    pub projected_load: f64,
    pub recommended_capacity: f64,
    pub bottleneck_endpoints: Vec<String>,
    pub growth_rate: f64,
    pub time_to_capacity_hours: f64,
    pub utilization_score: f64,
    pub recommended_replicas: i32,
}

// V23 types (from distributed_tracing_v23.rs)

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
    pub severity_counts: HashMap<String, i64>,
    pub service_counts: HashMap<String, i64>,
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

// V24 types (from distributed_tracing_v24.rs)

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
    pub failure_type_counts: HashMap<String, i64>,
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
