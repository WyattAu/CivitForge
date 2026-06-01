#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

/// OpenTelemetry trace context propagation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceContext {
    pub trace_id: String,
    pub span_id: String,
    pub trace_flags: u8,
    pub trace_state: Option<String>,
}

impl TraceContext {
    pub fn new(trace_id: impl Into<String>, span_id: impl Into<String>) -> Self {
        Self {
            trace_id: trace_id.into(),
            span_id: span_id.into(),
            trace_flags: 0x01, // sampled
            trace_state: None,
        }
    }

    pub fn from_w3c(header: &str) -> Option<Self> {
        // Format: 00-{trace_id}-{span_id}-{flags}
        let parts: Vec<&str> = header.split('-').collect();
        if parts.len() >= 4 && parts[0] == "00" {
            Some(Self {
                trace_id: parts[1].to_string(),
                span_id: parts[2].to_string(),
                trace_flags: u8::from_str_radix(parts[3], 16).unwrap_or(0),
                trace_state: None,
            })
        } else if parts.len() >= 3 {
            // Fallback: {trace_id}-{span_id}-{flags}
            Some(Self {
                trace_id: parts[0].to_string(),
                span_id: parts[1].to_string(),
                trace_flags: u8::from_str_radix(parts[2], 16).unwrap_or(0),
                trace_state: None,
            })
        } else {
            None
        }
    }

    pub fn to_w3c(&self) -> String {
        format!(
            "00-{}-{}-{:02x}",
            self.trace_id, self.span_id, self.trace_flags
        )
    }

    /// Generate a new child span context from this trace.
    pub fn child(&self, span_id: impl Into<String>) -> Self {
        Self {
            trace_id: self.trace_id.clone(),
            span_id: span_id.into(),
            trace_flags: self.trace_flags,
            trace_state: self.trace_state.clone(),
        }
    }
}

/// Span kind for OpenTelemetry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpanKind {
    Internal,
    Client,
    Server,
    Producer,
    Consumer,
}

/// Status of a completed span.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SpanStatus {
    Ok,
    Error { message: String },
}

/// An OpenTelemetry span with attributes, events, and timing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OtSpan {
    pub trace_id: String,
    pub span_id: String,
    pub parent_span_id: Option<String>,
    pub name: String,
    pub kind: SpanKind,
    pub start_time: chrono::DateTime<chrono::Utc>,
    pub end_time: Option<chrono::DateTime<chrono::Utc>>,
    pub status: SpanStatus,
    pub attributes: HashMap<String, OtelAttribute>,
    pub events: Vec<OtelEvent>,
    pub links: Vec<OtelLink>,
}

impl OtSpan {
    pub fn new(
        name: impl Into<String>,
        trace_id: impl Into<String>,
        span_id: impl Into<String>,
    ) -> Self {
        Self {
            trace_id: trace_id.into(),
            span_id: span_id.into(),
            parent_span_id: None,
            name: name.into(),
            kind: SpanKind::Internal,
            start_time: chrono::Utc::now(),
            end_time: None,
            status: SpanStatus::Ok,
            attributes: HashMap::new(),
            events: Vec::new(),
            links: Vec::new(),
        }
    }

    pub fn with_parent(mut self, parent_id: impl Into<String>) -> Self {
        self.parent_span_id = Some(parent_id.into());
        self
    }

    pub fn with_kind(mut self, kind: SpanKind) -> Self {
        self.kind = kind;
        self
    }

    pub fn set_attribute(&mut self, key: impl Into<String>, value: OtelAttribute) {
        self.attributes.insert(key.into(), value);
    }

    pub fn add_event(
        &mut self,
        name: impl Into<String>,
        attributes: HashMap<String, OtelAttribute>,
    ) {
        self.events.push(OtelEvent {
            name: name.into(),
            timestamp: chrono::Utc::now(),
            attributes,
        });
    }

    pub fn add_link(
        &mut self,
        trace_context: TraceContext,
        attributes: HashMap<String, OtelAttribute>,
    ) {
        self.links.push(OtelLink {
            trace_context,
            attributes,
        });
    }

    pub fn set_status(&mut self, status: SpanStatus) {
        self.status = status;
    }

    pub fn end(&mut self) {
        self.end_time = Some(chrono::Utc::now());
    }

    pub fn is_recording(&self) -> bool {
        self.end_time.is_none()
    }

    pub fn duration_ms(&self) -> Option<f64> {
        self.end_time
            .map(|end| (end - self.start_time).num_milliseconds() as f64)
    }

    pub fn attribute(&self, key: &str) -> Option<&OtelAttribute> {
        self.attributes.get(key)
    }
}

/// An attribute value in OpenTelemetry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum OtelAttribute {
    String(String),
    Int(i64),
    Double(f64),
    Bool(bool),
    StringArray(Vec<String>),
    IntArray(Vec<i64>),
}

impl OtelAttribute {
    pub fn type_name(&self) -> &str {
        match self {
            Self::String(_) | Self::StringArray(_) => "string",
            Self::Int(_) | Self::IntArray(_) => "int",
            Self::Double(_) => "double",
            Self::Bool(_) => "bool",
        }
    }
}

/// An event recorded during a span.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OtelEvent {
    pub name: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub attributes: HashMap<String, OtelAttribute>,
}

/// A link to another span.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OtelLink {
    pub trace_context: TraceContext,
    pub attributes: HashMap<String, OtelAttribute>,
}

/// A metric measurement point.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricPoint {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub value: f64,
    pub attributes: HashMap<String, OtelAttribute>,
}

/// Metric types supported by the instrumentation layer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MetricType {
    Counter,
    Gauge,
    Histogram,
    UpDownCounter,
}

/// A metric definition.
#[derive(Debug, Clone)]
pub struct MetricDefinition {
    pub name: String,
    pub description: String,
    pub unit: String,
    pub metric_type: MetricType,
}

impl MetricDefinition {
    pub fn counter(
        name: impl Into<String>,
        description: impl Into<String>,
        unit: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            unit: unit.into(),
            metric_type: MetricType::Counter,
        }
    }

    pub fn gauge(
        name: impl Into<String>,
        description: impl Into<String>,
        unit: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            unit: unit.into(),
            metric_type: MetricType::Gauge,
        }
    }

    pub fn histogram(
        name: impl Into<String>,
        description: impl Into<String>,
        unit: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            unit: unit.into(),
            metric_type: MetricType::Histogram,
        }
    }
}

/// A metric instrument for recording measurements.
pub struct MetricInstrument {
    pub definition: MetricDefinition,
    points: Vec<MetricPoint>,
}

impl MetricInstrument {
    pub fn new(definition: MetricDefinition) -> Self {
        Self {
            definition,
            points: Vec::new(),
        }
    }

    pub fn record(&mut self, value: f64, attributes: HashMap<String, OtelAttribute>) {
        self.points.push(MetricPoint {
            timestamp: chrono::Utc::now(),
            value,
            attributes,
        });
    }

    pub fn increment(&mut self, attributes: HashMap<String, OtelAttribute>) {
        let last_value = self
            .points
            .iter()
            .rev()
            .find(|p| p.attributes == attributes)
            .map(|p| p.value)
            .unwrap_or(0.0);
        self.record(last_value + 1.0, attributes);
    }

    pub fn points(&self) -> &[MetricPoint] {
        &self.points
    }

    pub fn count(&self) -> usize {
        self.points.len()
    }

    pub fn sum(&self) -> f64 {
        self.points.iter().map(|p| p.value).sum()
    }
}

/// Central OpenTelemetry instrumentation provider.
pub struct InstrumentationProvider {
    span_id_counter: AtomicU64,
    active_spans: dashmap::DashMap<String, OtSpan>,
    completed_spans: dashmap::DashMap<String, OtSpan>,
    metrics: dashmap::DashMap<String, MetricInstrument>,
    resource: Resource,
}

/// Resource attributes describing the service.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resource {
    pub service_name: String,
    pub service_version: String,
    pub deployment_environment: String,
    pub attributes: HashMap<String, String>,
}

impl Default for Resource {
    fn default() -> Self {
        Self {
            service_name: "civitforge".to_string(),
            service_version: env!("CARGO_PKG_VERSION").to_string(),
            deployment_environment: "development".to_string(),
            attributes: HashMap::new(),
        }
    }
}

impl InstrumentationProvider {
    pub fn new(resource: Resource) -> Self {
        Self {
            span_id_counter: AtomicU64::new(1),
            active_spans: dashmap::DashMap::new(),
            completed_spans: dashmap::DashMap::new(),
            metrics: dashmap::DashMap::new(),
            resource,
        }
    }

    fn next_span_id(&self) -> String {
        format!(
            "{:016x}",
            self.span_id_counter.fetch_add(1, Ordering::Relaxed)
        )
    }

    fn new_trace_id() -> String {
        uuid::Uuid::new_v4().to_string().replace('-', "")[..32].to_string()
    }

    /// Start a new root span.
    pub fn start_span(&self, name: impl Into<String>) -> String {
        let trace_id = Self::new_trace_id();
        let span_id = self.next_span_id();
        let span = OtSpan::new(name, &trace_id, &span_id);
        let key = format!("{trace_id}:{span_id}");
        self.active_spans.insert(key.clone(), span);
        key
    }

    /// Start a child span.
    pub fn start_child_span(&self, parent_key: &str, name: impl Into<String>) -> Option<String> {
        let parent = self.active_spans.get(parent_key)?;
        let span_id = self.next_span_id();
        let mut span = OtSpan::new(name, &parent.trace_id, &span_id).with_parent(&parent.span_id);
        span.kind = parent.kind.clone();
        let key = format!("{}:{span_id}", parent.trace_id);
        self.active_spans.insert(key.clone(), span);
        Some(key)
    }

    /// Set an attribute on an active span.
    pub fn set_attribute(&self, span_key: &str, key: impl Into<String>, value: OtelAttribute) {
        if let Some(mut span) = self.active_spans.get_mut(span_key) {
            span.set_attribute(key, value);
        }
    }

    /// Record an event on an active span.
    pub fn add_event(
        &self,
        span_key: &str,
        name: impl Into<String>,
        attributes: HashMap<String, OtelAttribute>,
    ) {
        if let Some(mut span) = self.active_spans.get_mut(span_key) {
            span.add_event(name, attributes);
        }
    }

    /// Set the status of an active span.
    pub fn set_status(&self, span_key: &str, status: SpanStatus) {
        if let Some(mut span) = self.active_spans.get_mut(span_key) {
            span.set_status(status);
        }
    }

    /// End an active span and move it to completed.
    pub fn end_span(&self, span_key: &str) -> Option<OtSpan> {
        let (_key, mut span) = self.active_spans.remove(span_key)?;
        span.end();
        let completed = span.clone();
        self.completed_spans.insert(span_key.to_string(), span);
        Some(completed)
    }

    /// Create or get a metric instrument.
    pub fn register_metric(&self, definition: MetricDefinition) {
        self.metrics
            .entry(definition.name.clone())
            .or_insert_with(|| MetricInstrument::new(definition));
    }

    /// Record a metric value.
    pub fn record_metric(
        &self,
        name: &str,
        value: f64,
        attributes: HashMap<String, OtelAttribute>,
    ) {
        if let Some(mut instrument) = self.metrics.get_mut(name) {
            instrument.record(value, attributes);
        }
    }

    /// Increment a counter metric.
    pub fn increment_metric(&self, name: &str, attributes: HashMap<String, OtelAttribute>) {
        if let Some(mut instrument) = self.metrics.get_mut(name) {
            instrument.increment(attributes);
        }
    }

    /// Get active span count.
    pub fn active_span_count(&self) -> usize {
        self.active_spans.len()
    }

    /// Get completed span count.
    pub fn completed_span_count(&self) -> usize {
        self.completed_spans.len()
    }

    /// Export all completed spans (clears the buffer).
    pub fn export_spans(&self) -> Vec<OtSpan> {
        let spans: Vec<OtSpan> = self
            .completed_spans
            .iter()
            .map(|r| r.value().clone())
            .collect();
        self.completed_spans.clear();
        spans
    }

    /// Get resource attributes.
    pub fn resource(&self) -> &Resource {
        &self.resource
    }

    /// Get all registered metric names.
    pub fn metric_names(&self) -> Vec<String> {
        self.metrics.iter().map(|r| r.key().clone()).collect()
    }

    /// Get metric instrument by name.
    pub fn get_metric(&self, name: &str) -> Option<MetricInstrument> {
        self.metrics.get(name).map(|i| MetricInstrument {
            definition: i.definition.clone(),
            points: i.points.clone(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trace_context_new() {
        let ctx = TraceContext::new("trace123", "span456");
        assert_eq!(ctx.trace_id, "trace123");
        assert_eq!(ctx.span_id, "span456");
        assert_eq!(ctx.trace_flags, 0x01);
    }

    #[test]
    fn test_trace_context_w3c_roundtrip() {
        let ctx = TraceContext::new("abcdef0123456789abcdef0123456789", "abcdef0123456789");
        let w3c = ctx.to_w3c();
        assert!(w3c.starts_with("00-"));
        let parsed = TraceContext::from_w3c(&w3c).unwrap();
        assert_eq!(parsed.trace_id, ctx.trace_id);
        assert_eq!(parsed.span_id, ctx.span_id);
    }

    #[test]
    fn test_trace_context_from_w3c() {
        let header = "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01";
        let ctx = TraceContext::from_w3c(header).unwrap();
        assert_eq!(ctx.trace_id, "4bf92f3577b34da6a3ce929d0e0e4736");
        assert_eq!(ctx.span_id, "00f067aa0ba902b7");
        assert_eq!(ctx.trace_flags, 0x01);
    }

    #[test]
    fn test_trace_context_from_w3c_invalid() {
        assert!(TraceContext::from_w3c("invalid").is_none());
    }

    #[test]
    fn test_trace_context_child() {
        let parent = TraceContext::new("trace1", "span1");
        let child = parent.child("span2");
        assert_eq!(child.trace_id, "trace1");
        assert_eq!(child.span_id, "span2");
    }

    #[test]
    fn test_span_new() {
        let span = OtSpan::new("test-span", "trace1", "span1");
        assert_eq!(span.name, "test-span");
        assert!(span.end_time.is_none());
        assert!(span.is_recording());
    }

    #[test]
    fn test_span_with_parent() {
        let span = OtSpan::new("child", "trace1", "span2").with_parent("span1");
        assert_eq!(span.parent_span_id, Some("span1".to_string()));
    }

    #[test]
    fn test_span_attributes() {
        let mut span = OtSpan::new("test", "t", "s");
        span.set_attribute("key", OtelAttribute::String("value".into()));
        span.set_attribute("count", OtelAttribute::Int(42));
        assert_eq!(span.attributes.len(), 2);
        assert!(matches!(
            span.attribute("key"),
            Some(OtelAttribute::String(_))
        ));
    }

    #[test]
    fn test_span_events() {
        let mut span = OtSpan::new("test", "t", "s");
        let mut attrs = HashMap::new();
        attrs.insert(
            "event_key".to_string(),
            OtelAttribute::String("event_val".into()),
        );
        span.add_event("test-event", attrs);
        assert_eq!(span.events.len(), 1);
        assert_eq!(span.events[0].name, "test-event");
    }

    #[test]
    fn test_span_links() {
        let mut span = OtSpan::new("test", "t", "s");
        let ctx = TraceContext::new("other_trace", "other_span");
        span.add_link(ctx, HashMap::new());
        assert_eq!(span.links.len(), 1);
    }

    #[test]
    fn test_span_end() {
        let mut span = OtSpan::new("test", "t", "s");
        span.end();
        assert!(!span.is_recording());
        assert!(span.end_time.is_some());
        assert!(span.duration_ms().is_some());
    }

    #[test]
    fn test_span_duration_none() {
        let span = OtSpan::new("test", "t", "s");
        assert!(span.duration_ms().is_none());
    }

    #[test]
    fn test_span_status() {
        let mut span = OtSpan::new("test", "t", "s");
        span.set_status(SpanStatus::Error {
            message: "test error".into(),
        });
        assert!(matches!(&span.status, SpanStatus::Error { message } if message == "test error"));
    }

    #[test]
    fn test_otel_attribute_type_name() {
        assert_eq!(OtelAttribute::String("x".into()).type_name(), "string");
        assert_eq!(OtelAttribute::Int(1).type_name(), "int");
        assert_eq!(OtelAttribute::Double(1.0).type_name(), "double");
        assert_eq!(OtelAttribute::Bool(true).type_name(), "bool");
    }

    #[test]
    fn test_instrumentation_start_span() {
        let provider = InstrumentationProvider::new(Resource::default());
        let key = provider.start_span("http-request");
        assert!(provider.active_span_count() > 0);
        let completed = provider.end_span(&key).unwrap();
        assert_eq!(completed.name, "http-request");
        assert!(completed.end_time.is_some());
    }

    #[test]
    fn test_instrumentation_child_span() {
        let provider = InstrumentationProvider::new(Resource::default());
        let parent_key = provider.start_span("parent");
        let child_key = provider.start_child_span(&parent_key, "child").unwrap();
        provider.end_span(&child_key);
        provider.end_span(&parent_key);
        assert_eq!(provider.completed_span_count(), 2);
    }

    #[test]
    fn test_instrumentation_child_span_no_parent() {
        let provider = InstrumentationProvider::new(Resource::default());
        assert!(provider.start_child_span("nonexistent", "child").is_none());
    }

    #[test]
    fn test_instrumentation_set_attribute() {
        let provider = InstrumentationProvider::new(Resource::default());
        let key = provider.start_span("test");
        provider.set_attribute(&key, "http.method", OtelAttribute::String("GET".into()));
        provider.end_span(&key);
    }

    #[test]
    fn test_instrumentation_add_event() {
        let provider = InstrumentationProvider::new(Resource::default());
        let key = provider.start_span("test");
        let mut attrs = HashMap::new();
        attrs.insert("k".to_string(), OtelAttribute::Int(1));
        provider.add_event(&key, "exception", attrs);
        let span = provider.end_span(&key).unwrap();
        assert_eq!(span.events.len(), 1);
    }

    #[test]
    fn test_instrumentation_set_status() {
        let provider = InstrumentationProvider::new(Resource::default());
        let key = provider.start_span("test");
        provider.set_status(
            &key,
            SpanStatus::Error {
                message: "fail".into(),
            },
        );
        let span = provider.end_span(&key).unwrap();
        assert!(matches!(&span.status, SpanStatus::Error { .. }));
    }

    #[test]
    fn test_instrumentation_export() {
        let provider = InstrumentationProvider::new(Resource::default());
        let k1 = provider.start_span("s1");
        let k2 = provider.start_span("s2");
        provider.end_span(&k1);
        provider.end_span(&k2);
        assert_eq!(provider.completed_span_count(), 2);
        let exported = provider.export_spans();
        assert_eq!(exported.len(), 2);
        assert_eq!(provider.completed_span_count(), 0);
    }

    #[test]
    fn test_metric_definition_counter() {
        let def = MetricDefinition::counter("requests", "Total requests", "1");
        assert!(matches!(def.metric_type, MetricType::Counter));
    }

    #[test]
    fn test_metric_definition_gauge() {
        let def = MetricDefinition::gauge("temperature", "Current temp", "C");
        assert!(matches!(def.metric_type, MetricType::Gauge));
    }

    #[test]
    fn test_metric_definition_histogram() {
        let def = MetricDefinition::histogram("duration", "Request duration", "ms");
        assert!(matches!(def.metric_type, MetricType::Histogram));
    }

    #[test]
    fn test_metric_instrument_record() {
        let mut instrument = MetricInstrument::new(MetricDefinition::counter("c", "desc", "1"));
        let mut attrs = HashMap::new();
        attrs.insert("path".to_string(), OtelAttribute::String("/api".into()));
        instrument.record(5.0, attrs);
        assert_eq!(instrument.count(), 1);
        assert_eq!(instrument.sum(), 5.0);
    }

    #[test]
    fn test_metric_instrument_increment() {
        let mut instrument = MetricInstrument::new(MetricDefinition::counter("c", "desc", "1"));
        let attrs = HashMap::new();
        instrument.increment(attrs.clone());
        instrument.increment(attrs.clone());
        instrument.increment(attrs.clone());
        assert_eq!(instrument.count(), 3);
        // Last recorded value should be 3.0 (1.0, 2.0, 3.0)
        let last_val = instrument.points().last().map(|p| p.value).unwrap_or(0.0);
        assert_eq!(last_val, 3.0);
    }

    #[test]
    fn test_instrumentation_register_metric() {
        let provider = InstrumentationProvider::new(Resource::default());
        provider.register_metric(MetricDefinition::counter("test_counter", "desc", "1"));
        let mut attrs = HashMap::new();
        attrs.insert("k".to_string(), OtelAttribute::String("v".into()));
        provider.increment_metric("test_counter", attrs);
        let names = provider.metric_names();
        assert!(names.contains(&"test_counter".to_string()));
    }

    #[test]
    fn test_instrumentation_resource() {
        let provider = InstrumentationProvider::new(Resource::default());
        assert_eq!(provider.resource().service_name, "civitforge");
    }

    #[test]
    fn test_span_serialization() {
        let mut span = OtSpan::new("test", "trace1", "span1");
        span.end();
        let json = serde_json::to_string(&span).unwrap();
        let de: OtSpan = serde_json::from_str(&json).unwrap();
        assert_eq!(de.name, "test");
    }

    #[test]
    fn test_metric_point_serialization() {
        let point = MetricPoint {
            timestamp: chrono::Utc::now(),
            value: 42.0,
            attributes: HashMap::new(),
        };
        let json = serde_json::to_string(&point).unwrap();
        let de: MetricPoint = serde_json::from_str(&json).unwrap();
        assert!((de.value - 42.0).abs() < 0.001);
    }

    #[test]
    fn test_resource_serialization() {
        let resource = Resource::default();
        let json = serde_json::to_string(&resource).unwrap();
        let de: Resource = serde_json::from_str(&json).unwrap();
        assert_eq!(de.service_name, "civitforge");
    }
}
