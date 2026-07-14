#![forbid(unsafe_code)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use std::time::Instant;

/// A recorded distributed trace span stored in memory and exported to the database.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceSpan {
    pub trace_id: String,
    pub span_id: String,
    pub parent_span_id: Option<String>,
    pub operation_name: String,
    pub service_name: String,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub duration_ms: Option<i64>,
    pub status: String,
    pub attributes: HashMap<String, serde_json::Value>,
    pub events: Vec<TraceEvent>,
}

/// An event recorded during a trace span.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceEvent {
    pub name: String,
    pub timestamp: DateTime<Utc>,
    pub attributes: HashMap<String, serde_json::Value>,
}

/// Configuration for distributed tracing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistributedTracingConfig {
    #[serde(default = "default_service_name")]
    pub service_name: String,
    #[serde(default = "default_max_spans")]
    pub max_spans: usize,
    #[serde(default = "default_sample_rate")]
    pub sample_rate: f64,
    #[serde(default)]
    pub export_to_jaeger: bool,
    #[serde(default = "default_jaeger_endpoint")]
    pub jaeger_endpoint: String,
    #[serde(default)]
    pub export_to_zipkin: bool,
    #[serde(default = "default_zipkin_endpoint")]
    pub zipkin_endpoint: String,
}

fn default_service_name() -> String {
    "civitforge".to_string()
}

fn default_max_spans() -> usize {
    10_000
}

fn default_sample_rate() -> f64 {
    1.0
}

fn default_jaeger_endpoint() -> String {
    "http://localhost:14268/api/traces".to_string()
}

fn default_zipkin_endpoint() -> String {
    "http://localhost:9411/api/v2/spans".to_string()
}

impl Default for DistributedTracingConfig {
    fn default() -> Self {
        Self {
            service_name: default_service_name(),
            max_spans: default_max_spans(),
            sample_rate: default_sample_rate(),
            export_to_jaeger: false,
            jaeger_endpoint: default_jaeger_endpoint(),
            export_to_zipkin: false,
            zipkin_endpoint: default_zipkin_endpoint(),
        }
    }
}

/// Generate a W3C Trace Context trace ID (32 hex chars).
pub fn generate_trace_id() -> String {
    uuid::Uuid::new_v4().to_string().replace('-', "")
}

/// Generate a W3C Trace Context span ID (16 hex chars).
pub fn generate_span_id() -> String {
    let bytes: [u8; 8] = [
        rand::random(),
        rand::random(),
        rand::random(),
        rand::random(),
        rand::random(),
        rand::random(),
        rand::random(),
        rand::random(),
    ];
    hex::encode(bytes)
}

/// Parse a W3C `traceparent` header into trace_id, span_id, and trace_flags.
pub fn parse_traceparent(header: &str) -> Option<(String, String, u8)> {
    let parts: Vec<&str> = header.split('-').collect();
    if parts.len() >= 4 && parts[0] == "00" {
        let trace_flags = u8::from_str_radix(parts[3], 16).ok()?;
        Some((
            parts[1].to_string(),
            parts[2].to_string(),
            trace_flags,
        ))
    } else {
        None
    }
}

/// Format a W3C `traceparent` header from components.
pub fn format_traceparent(trace_id: &str, span_id: &str, trace_flags: u8) -> String {
    format!("00-{trace_id}-{span_id}-{trace_flags:02x}")
}

/// In-memory distributed trace recorder.
pub struct DistributedTracer {
    config: DistributedTracingConfig,
    spans: Mutex<Vec<TraceSpan>>,
    sampling_counter: AtomicU64,
}

impl DistributedTracer {
    pub fn new(config: DistributedTracingConfig) -> Self {
        Self {
            config,
            spans: Mutex::new(Vec::new()),
            sampling_counter: AtomicU64::new(0),
        }
    }

    /// Check whether this request should be sampled.
    pub fn should_sample(&self) -> bool {
        if self.config.sample_rate >= 1.0 {
            return true;
        }
        let count = self.sampling_counter.fetch_add(1, Ordering::Relaxed);
        let threshold = (self.config.sample_rate * 1000.0) as u64;
        (count % 1000) < threshold
    }

    /// Start a new root span.
    pub fn start_span(&self, operation_name: &str) -> TraceSpan {
        let trace_id = generate_trace_id();
        let span_id = generate_span_id();
        TraceSpan {
            trace_id,
            span_id,
            parent_span_id: None,
            operation_name: operation_name.to_string(),
            service_name: self.config.service_name.clone(),
            start_time: Utc::now(),
            end_time: None,
            duration_ms: None,
            status: "ok".to_string(),
            attributes: HashMap::new(),
            events: Vec::new(),
        }
    }

    /// Start a child span from a parent span's trace context.
    pub fn start_child_span(
        &self,
        parent: &TraceSpan,
        operation_name: &str,
    ) -> TraceSpan {
        let span_id = generate_span_id();
        TraceSpan {
            trace_id: parent.trace_id.clone(),
            span_id,
            parent_span_id: Some(parent.span_id.clone()),
            operation_name: operation_name.to_string(),
            service_name: self.config.service_name.clone(),
            start_time: Utc::now(),
            end_time: None,
            duration_ms: None,
            status: "ok".to_string(),
            attributes: HashMap::new(),
            events: Vec::new(),
        }
    }

    /// End a span and record it.
    pub fn end_span(&self, mut span: TraceSpan) {
        let now = Utc::now();
        span.end_time = Some(now);
        span.duration_ms = Some((now - span.start_time).num_milliseconds());
        let mut spans = self.spans.lock().unwrap();
        if spans.len() >= self.config.max_spans {
            spans.remove(0);
        }
        spans.push(span);
    }

    /// Record a span event.
    pub fn record_event(
        span: &mut TraceSpan,
        name: &str,
        attributes: HashMap<String, serde_json::Value>,
    ) {
        span.events.push(TraceEvent {
            name: name.to_string(),
            timestamp: Utc::now(),
            attributes,
        });
    }

    /// Set a span status to error.
    pub fn set_error(span: &mut TraceSpan, message: &str) {
        span.status = "error".to_string();
        let mut attrs = HashMap::new();
        attrs.insert(
            "error.message".to_string(),
            serde_json::Value::String(message.to_string()),
        );
        Self::record_event(span, "exception", attrs);
    }

    /// Export all completed spans and clear the buffer.
    pub fn export_spans(&self) -> Vec<TraceSpan> {
        let mut spans = self.spans.lock().unwrap();
        std::mem::take(&mut *spans)
    }

    /// Get the current count of buffered spans.
    pub fn span_count(&self) -> usize {
        self.spans.lock().unwrap().len()
    }

    /// Get a reference to the configuration.
    pub fn config(&self) -> &DistributedTracingConfig {
        &self.config
    }
}

/// Build Jaeger-compatible JSON payload from a completed span.
pub fn to_jaeger_payload(span: &TraceSpan) -> serde_json::Value {
    let tags: Vec<serde_json::Value> = span
        .attributes
        .iter()
        .map(|(k, v)| {
            serde_json::json!({
                "key": k,
                "value": v,
                "vType": "string",
            })
        })
        .collect();

    let logs: Vec<serde_json::Value> = span
        .events
        .iter()
        .map(|e| {
            serde_json::json!({
                "timestamp": e.timestamp.timestamp_micros(),
                "fields": e.attributes.iter().map(|(k, v)| {
                    serde_json::json!({"key": k, "value": v})
                }).collect::<Vec<_>>(),
            })
        })
        .collect();

    serde_json::json!({
        "traceID": span.trace_id,
        "spanID": span.span_id,
        "parentSpanID": span.parent_span_id,
        "operationName": span.operation_name,
        "service_name": span.service_name,
        "startTime": span.start_time.timestamp_micros(),
        "duration": span.duration_ms.unwrap_or(0) as u64 * 1000,
        "tags": tags,
        "logs": logs,
        "references": [],
    })
}

/// Build Zipkin-compatible JSON payload from a completed span.
pub fn to_zipkin_payload(span: &TraceSpan) -> serde_json::Value {
    let annotations: Vec<serde_json::Value> = span
        .events
        .iter()
        .map(|e| {
            serde_json::json!({
                "timestamp": e.timestamp.timestamp_micros(),
                "value": e.name,
            })
        })
        .collect();

    let binary_annotations: Vec<serde_json::Value> = span
        .attributes
        .iter()
        .map(|(k, v)| {
            serde_json::json!({
                "key": k,
                "value": v,
            })
        })
        .collect();

    serde_json::json!({
        "traceId": span.trace_id,
        "id": span.span_id,
        "parentId": span.parent_span_id,
        "name": span.operation_name,
        "timestamp": span.start_time.timestamp_micros(),
        "duration": span.duration_ms.unwrap_or(0) as u64 * 1000,
        "annotations": annotations,
        "binaryAnnotations": binary_annotations,
        "endpoint": {
            "serviceName": span.service_name,
        },
    })
}

/// Get the W3C `tracestate` header name.
pub const TRACEPARENT_HEADER: &str = "traceparent";
/// Get the W3C `tracestate` header name.
pub const TRACESTATE_HEADER: &str = "tracestate";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_trace_id() {
        let id = generate_trace_id();
        assert_eq!(id.len(), 32);
        assert!(id.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_generate_span_id() {
        let id = generate_span_id();
        assert_eq!(id.len(), 16);
        assert!(id.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_generate_unique_trace_ids() {
        let ids: Vec<String> = (0..100).map(|_| generate_trace_id()).collect();
        let unique: std::collections::HashSet<&str> = ids.iter().map(|s| s.as_str()).collect();
        assert_eq!(unique.len(), 100);
    }

    #[test]
    fn test_parse_traceparent_valid() {
        let header = "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01";
        let (trace_id, span_id, flags) = parse_traceparent(header).unwrap();
        assert_eq!(trace_id, "4bf92f3577b34da6a3ce929d0e0e4736");
        assert_eq!(span_id, "00f067aa0ba902b7");
        assert_eq!(flags, 1);
    }

    #[test]
    fn test_parse_traceparent_invalid() {
        assert!(parse_traceparent("invalid").is_none());
    }

    #[test]
    fn test_format_traceparent() {
        let header = format_traceparent("trace123", "span456", 0x01);
        assert_eq!(header, "00-trace123-span456-01");
    }

    #[test]
    fn test_traceparent_roundtrip() {
        let trace_id = "abcdef0123456789abcdef0123456789";
        let span_id = "0123456789abcdef";
        let header = format_traceparent(trace_id, span_id, 0x01);
        let (t, s, f) = parse_traceparent(&header).unwrap();
        assert_eq!(t, trace_id);
        assert_eq!(s, span_id);
        assert_eq!(f, 1);
    }

    #[test]
    fn test_tracer_start_span() {
        let tracer = DistributedTracer::new(DistributedTracingConfig::default());
        let span = tracer.start_span("http.request");
        assert_eq!(span.operation_name, "http.request");
        assert_eq!(span.service_name, "civitforge");
        assert!(span.parent_span_id.is_none());
        assert!(span.end_time.is_none());
    }

    #[test]
    fn test_tracer_child_span() {
        let tracer = DistributedTracer::new(DistributedTracingConfig::default());
        let parent = tracer.start_span("parent");
        let child = tracer.start_child_span(&parent, "child");
        assert_eq!(child.trace_id, parent.trace_id);
        assert_eq!(child.parent_span_id, Some(parent.span_id.clone()));
    }

    #[test]
    fn test_tracer_end_span() {
        let tracer = DistributedTracer::new(DistributedTracingConfig::default());
        let span = tracer.start_span("work");
        tracer.end_span(span);
        assert_eq!(tracer.span_count(), 1);
    }

    #[test]
    fn test_tracer_export_clears() {
        let tracer = DistributedTracer::new(DistributedTracingConfig::default());
        let span = tracer.start_span("work");
        tracer.end_span(span);
        let exported = tracer.export_spans();
        assert_eq!(exported.len(), 1);
        assert_eq!(tracer.span_count(), 0);
    }

    #[test]
    fn test_tracer_max_spans_eviction() {
        let config = DistributedTracingConfig {
            max_spans: 2,
            ..Default::default()
        };
        let tracer = DistributedTracer::new(config);
        for i in 0..4 {
            let span = tracer.start_span(&format!("span-{i}"));
            tracer.end_span(span);
        }
        assert_eq!(tracer.span_count(), 2);
    }

    #[test]
    fn test_tracer_set_error() {
        let mut span = TraceSpan {
            trace_id: "t".into(),
            span_id: "s".into(),
            parent_span_id: None,
            operation_name: "test".into(),
            service_name: "svc".into(),
            start_time: Utc::now(),
            end_time: None,
            duration_ms: None,
            status: "ok".into(),
            attributes: HashMap::new(),
            events: Vec::new(),
        };
        DistributedTracer::set_error(&mut span, "boom");
        assert_eq!(span.status, "error");
        assert_eq!(span.events.len(), 1);
    }

    #[test]
    fn test_tracer_record_event() {
        let mut span = TraceSpan {
            trace_id: "t".into(),
            span_id: "s".into(),
            parent_span_id: None,
            operation_name: "test".into(),
            service_name: "svc".into(),
            start_time: Utc::now(),
            end_time: None,
            duration_ms: None,
            status: "ok".into(),
            attributes: HashMap::new(),
            events: Vec::new(),
        };
        let mut attrs = HashMap::new();
        attrs.insert("k".into(), serde_json::json!("v"));
        DistributedTracer::record_event(&mut span, "my-event", attrs);
        assert_eq!(span.events.len(), 1);
        assert_eq!(span.events[0].name, "my-event");
    }

    #[test]
    fn test_jaeger_payload() {
        let span = TraceSpan {
            trace_id: "trace1".into(),
            span_id: "span1".into(),
            parent_span_id: None,
            operation_name: "test".into(),
            service_name: "civitforge".into(),
            start_time: Utc::now(),
            end_time: Some(Utc::now()),
            duration_ms: Some(42),
            status: "ok".into(),
            attributes: HashMap::new(),
            events: Vec::new(),
        };
        let payload = to_jaeger_payload(&span);
        assert_eq!(payload["traceID"], "trace1");
        assert_eq!(payload["spanID"], "span1");
        assert_eq!(payload["operationName"], "test");
    }

    #[test]
    fn test_zipkin_payload() {
        let span = TraceSpan {
            trace_id: "trace1".into(),
            span_id: "span1".into(),
            parent_span_id: None,
            operation_name: "test".into(),
            service_name: "civitforge".into(),
            start_time: Utc::now(),
            end_time: Some(Utc::now()),
            duration_ms: Some(42),
            status: "ok".into(),
            attributes: HashMap::new(),
            events: Vec::new(),
        };
        let payload = to_zipkin_payload(&span);
        assert_eq!(payload["traceId"], "trace1");
        assert_eq!(payload["id"], "span1");
        assert_eq!(payload["name"], "test");
    }

    #[test]
    fn test_default_config() {
        let config = DistributedTracingConfig::default();
        assert_eq!(config.service_name, "civitforge");
        assert_eq!(config.max_spans, 10_000);
        assert!((config.sample_rate - 1.0).abs() < f64::EPSILON);
        assert!(!config.export_to_jaeger);
        assert!(!config.export_to_zipkin);
    }

    #[test]
    fn test_sampling_always_sample() {
        let tracer = DistributedTracer::new(DistributedTracingConfig::default());
        for _ in 0..100 {
            assert!(tracer.should_sample());
        }
    }
}
