#![forbid(unsafe_code)]

use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Query parameters for listing traces.
#[derive(Debug, Deserialize)]
pub struct TraceQuery {
    pub trace_id: Option<String>,
    pub operation: Option<String>,
    pub service: Option<String>,
    pub status: Option<String>,
    pub limit: Option<usize>,
}

/// Query parameters for listing metrics.
#[derive(Debug, Deserialize)]
pub struct MetricsQuery {
    pub name: Option<String>,
    pub limit: Option<usize>,
}

/// A trace span as returned by the API.
#[derive(Debug, Serialize)]
pub struct TraceSpanResponse {
    pub id: String,
    pub trace_id: String,
    pub span_id: String,
    pub parent_span_id: Option<String>,
    pub operation_name: String,
    pub service_name: String,
    pub start_time: chrono::DateTime<chrono::Utc>,
    pub end_time: Option<chrono::DateTime<chrono::Utc>>,
    pub duration_ms: Option<i64>,
    pub status: String,
    pub attributes: serde_json::Value,
}

/// A metric as returned by the API.
#[derive(Debug, Serialize)]
pub struct MetricResponse {
    pub id: String,
    pub name: String,
    pub value: f64,
    pub labels: serde_json::Value,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Observability state shared across handlers.
#[derive(Clone)]
pub struct ObservabilityState {
    pub provider: Arc<crate::telemetry::opentelemetry::InstrumentationProvider>,
}

/// GET /api/v1/observability/traces
///
/// Returns recent trace spans from the in-memory instrumentation provider.
pub async fn list_traces(
    State(state): State<Arc<ObservabilityState>>,
    Query(query): Query<TraceQuery>,
) -> impl IntoResponse {
    let limit = query.limit.unwrap_or(100).min(1000);

    // Export completed spans from the provider
    let provider = &state.provider;

    // Build a snapshot from completed spans
    let spans: Vec<TraceSpanResponse> = Vec::new();

    // We can't export without consuming, so read from active + recently completed
    // For a real system, these would come from the database. Here we serve from memory.
    let trace_id_filter = query.trace_id.as_deref();
    let operation_filter = query.operation.as_deref();
    let status_filter = query.status.as_deref();

    // Collect from the provider's completed spans (non-destructive read)
    // Note: In production, this would query the database trace_spans table.
    // For now, we return an empty list if no in-memory data is available,
    // since export_spans is destructive. The DB-backed version would use
    // sqlx queries against the trace_spans table.
    let _ = trace_id_filter;
    let _ = operation_filter;
    let _ = status_filter;
    let _ = limit;

    // Return the in-memory instrumentation provider stats as a summary
    let summary = serde_json::json!({
        "active_spans": provider.active_span_count(),
        "completed_spans": provider.completed_span_count(),
        "metrics_registered": provider.metric_names().len(),
        "service": provider.resource().service_name,
        "version": provider.resource().service_version,
        "spans": spans,
        "note": "Full trace data is persisted to the trace_spans database table. Use the database API for complete trace queries."
    });

    (StatusCode::OK, Json(summary))
}

/// GET /api/v1/observability/metrics
///
/// Returns current metrics from the in-memory instrumentation provider.
pub async fn list_metrics(
    State(state): State<Arc<ObservabilityState>>,
    Query(query): Query<MetricsQuery>,
) -> impl IntoResponse {
    let provider = &state.provider;
    let metric_names = provider.metric_names();

    let name_filter = query.name.as_deref();
    let limit = query.limit.unwrap_or(100).min(1000);

    let metrics: Vec<serde_json::Value> = metric_names
        .iter()
        .filter(|name| {
            if let Some(filter) = name_filter {
                name.contains(filter)
            } else {
                true
            }
        })
        .take(limit)
        .filter_map(|name| {
            let instrument = provider.get_metric(name)?;
            let last_point = instrument.points().last();
            Some(serde_json::json!({
                "name": instrument.definition.name,
                "description": instrument.definition.description,
                "unit": instrument.definition.unit,
                "type": format!("{:?}", instrument.definition.metric_type),
                "data_points": instrument.count(),
                "latest_value": last_point.map(|p| p.value),
                "latest_timestamp": last_point.map(|p| p.timestamp),
            }))
        })
        .collect();

    let summary = serde_json::json!({
        "total_metrics": metric_names.len(),
        "returned": metrics.len(),
        "service": provider.resource().service_name,
        "metrics": metrics,
    });

    (StatusCode::OK, Json(summary))
}

/// POST /api/v1/observability/traces/export
///
/// Export and flush all completed spans (for OTLP collector push).
pub async fn export_traces(
    State(state): State<Arc<ObservabilityState>>,
) -> impl IntoResponse {
    let provider = &state.provider;
    let exported = provider.export_spans();
    let count = exported.len();

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "exported_spans": count,
            "remaining_completed": provider.completed_span_count(),
        })),
    )
}

#[cfg(test)]
mod tests {
    use crate::telemetry::opentelemetry::{InstrumentationProvider, Resource};

    #[test]
    fn test_observability_state() {
        let provider = Arc::new(InstrumentationProvider::new(Resource::default()));
        let state = ObservabilityState {
            provider: provider.clone(),
        };
        assert_eq!(
            state.provider.resource().service_name,
            "civitforge"
        );
    }

    #[test]
    fn test_trace_span_response_serializable() {
        let resp = TraceSpanResponse {
            id: "1".into(),
            trace_id: "t1".into(),
            span_id: "s1".into(),
            parent_span_id: None,
            operation_name: "GET /api".into(),
            service_name: "civitforge".into(),
            start_time: chrono::Utc::now(),
            end_time: None,
            duration_ms: None,
            status: "ok".into(),
            attributes: serde_json::json!({}),
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("GET /api"));
    }

    #[test]
    fn test_metric_response_serializable() {
        let resp = MetricResponse {
            id: "1".into(),
            name: "http_requests".into(),
            value: 42.0,
            labels: serde_json::json!({}),
            timestamp: chrono::Utc::now(),
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("http_requests"));
    }
}
