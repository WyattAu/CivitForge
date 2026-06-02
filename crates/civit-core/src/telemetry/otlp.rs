#![forbid(unsafe_code)]

//! OTLP HTTP/JSON exporter for traces and metrics.
//!
//! Implements the [OTLP/JSON](https://opentelemetry.io/docs/specs/otlp/#json-encoding)
//! format for HTTP transport. Uses `reqwest` (already a workspace dependency) to POST
//! spans and metrics to an OTLP-compatible collector (Jaeger, Tempo, Grafana, etc.).
//!
//! No new crate dependencies — reuses `reqwest`, `serde`, `serde_json`, `chrono`.

use super::opentelemetry::{
    InstrumentationProvider, MetricInstrument, MetricPoint, MetricType, OtSpan, OtelAttribute,
    Resource, SpanKind, SpanStatus,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

// ---------------------------------------------------------------------------
// OTLP/JSON data model (maps to protobuf, serialized as JSON)
// ---------------------------------------------------------------------------

/// Top-level OTLP export request body.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OtlpExportRequest {
    #[serde(rename = "resourceSpans")]
    pub resource_spans: Vec<OtlpResourceSpans>,
    #[serde(rename = "resourceMetrics", skip_serializing_if = "Vec::is_empty")]
    pub resource_metrics: Vec<OtlpResourceMetrics>,
}

/// Resource-level span container.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OtlpResourceSpans {
    pub resource: OtlpResource,
    #[serde(rename = "scopeSpans")]
    pub scope_spans: Vec<OtlpScopeSpans>,
    #[serde(rename = "schemaUrl", skip_serializing_if = "Option::is_none")]
    pub schema_url: Option<String>,
}

/// Instrumentation scope for spans.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OtlpScopeSpans {
    pub scope: OtlpInstrumentationScope,
    pub spans: Vec<OtlpSpanProto>,
    #[serde(rename = "schemaUrl", skip_serializing_if = "Option::is_none")]
    pub schema_url: Option<String>,
}

/// Instrumentation scope for metrics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OtlpScopeMetrics {
    pub scope: OtlpInstrumentationScope,
    pub metrics: Vec<OtlpMetricProto>,
    #[serde(rename = "schemaUrl", skip_serializing_if = "Option::is_none")]
    pub schema_url: Option<String>,
}

/// A single span in OTLP protobuf-mapped JSON format.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OtlpSpanProto {
    #[serde(rename = "traceId")]
    pub trace_id: String,
    #[serde(rename = "spanId")]
    pub span_id: String,
    #[serde(rename = "parentSpanId", skip_serializing_if = "Option::is_none")]
    pub parent_span_id: Option<String>,
    pub name: String,
    #[serde(rename = "kind")]
    pub kind: i32, // 0=Unspecified, 1=Internal, 2=Server, 3=Client, 4=Producer, 5=Consumer
    #[serde(rename = "startTimeUnixNano")]
    pub start_time_unix_nano: String,
    #[serde(rename = "endTimeUnixNano")]
    pub end_time_unix_nano: String,
    pub attributes: Vec<OtlpKeyValue>,
    #[serde(rename = "status")]
    pub status: OtlpStatus,
    pub events: Vec<OtlpSpanEventProto>,
    pub links: Vec<OtlpSpanLinkProto>,
}

/// Span status in OTLP format.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OtlpStatus {
    #[serde(rename = "code")]
    pub code: String, // "OK" | "ERROR" | "UNSET"
    #[serde(rename = "message", skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

/// Span event in OTLP format.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OtlpSpanEventProto {
    pub name: String,
    #[serde(rename = "timeUnixNano")]
    pub time_unix_nano: String,
    pub attributes: Vec<OtlpKeyValue>,
}

/// Span link in OTLP format.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OtlpSpanLinkProto {
    #[serde(rename = "traceId")]
    pub trace_id: String,
    #[serde(rename = "spanId")]
    pub span_id: String,
    #[serde(rename = "traceState", skip_serializing_if = "Option::is_none")]
    pub trace_state: Option<String>,
    pub attributes: Vec<OtlpKeyValue>,
    #[serde(rename = "droppedAttributesCount")]
    pub dropped_attributes_count: u32,
}

/// Key-value attribute in OTLP format.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OtlpKeyValue {
    pub key: String,
    pub value: OtlpAnyValue,
}

/// AnyValue in OTLP format.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum OtlpAnyValue {
    #[serde(rename = "stringValue")]
    String(String),
    #[serde(rename = "intValue")]
    Int(i64),
    #[serde(rename = "doubleValue")]
    Double(f64),
    #[serde(rename = "boolValue")]
    Bool(bool),
    #[serde(rename = "arrayValue")]
    Array(OtlpArrayValue),
}

/// Array of AnyValue elements.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OtlpArrayValue {
    pub values: Vec<OtlpAnyValue>,
}

/// Resource attributes in OTLP format.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OtlpResource {
    pub attributes: Vec<OtlpKeyValue>,
    #[serde(rename = "droppedAttributesCount")]
    pub dropped_attributes_count: u32,
}

/// Instrumentation scope in OTLP format.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OtlpInstrumentationScope {
    pub name: String,
    pub version: String,
}

// --- Metrics OTLP types ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OtlpResourceMetrics {
    pub resource: OtlpResource,
    #[serde(rename = "scopeMetrics")]
    pub scope_metrics: Vec<OtlpScopeMetrics>,
    #[serde(rename = "schemaUrl", skip_serializing_if = "Option::is_none")]
    pub schema_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OtlpMetricProto {
    pub name: String,
    pub description: String,
    pub unit: String,
    pub data: OtlpMetricData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "dataType", content = "data")]
pub enum OtlpMetricData {
    #[serde(rename = "intSum")]
    IntSum(OtlpSumProto),
    #[serde(rename = "doubleSum")]
    DoubleSum(OtlpSumProto),
    #[serde(rename = "doubleGauge")]
    DoubleGauge(OtlpGaugeProto),
    #[serde(rename = "histogram")]
    Histogram(OtlpHistogramProto),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OtlpSumProto {
    #[serde(rename = "dataPoints")]
    pub data_points: Vec<OtlpNumberDataPointProto>,
    #[serde(rename = "isMonotonic")]
    pub is_monotonic: bool,
    #[serde(rename = "aggregationTemporality")]
    pub aggregation_temporality: i32, // 0=Unspecified, 1=Delta, 2=Cumulative
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OtlpGaugeProto {
    #[serde(rename = "dataPoints")]
    pub data_points: Vec<OtlpNumberDataPointProto>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OtlpHistogramProto {
    #[serde(rename = "dataPoints")]
    pub data_points: Vec<OtlpHistogramDataPointProto>,
    #[serde(rename = "aggregationTemporality")]
    pub aggregation_temporality: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OtlpNumberDataPointProto {
    #[serde(rename = "asInt", skip_serializing_if = "Option::is_none")]
    pub as_int: Option<i64>,
    #[serde(rename = "asDouble", skip_serializing_if = "Option::is_none")]
    pub as_double: Option<f64>,
    #[serde(rename = "timeUnixNano")]
    pub time_unix_nano: String,
    pub attributes: Vec<OtlpKeyValue>,
    #[serde(rename = "exemplars", skip_serializing_if = "Vec::is_empty")]
    pub exemplars: Vec<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OtlpHistogramDataPointProto {
    pub count: u64,
    #[serde(rename = "sum", skip_serializing_if = "Option::is_none")]
    pub sum: Option<f64>,
    #[serde(rename = "min", skip_serializing_if = "Option::is_none")]
    pub min: Option<f64>,
    #[serde(rename = "max", skip_serializing_if = "Option::is_none")]
    pub max: Option<f64>,
    #[serde(rename = "bucketCounts", skip_serializing_if = "Vec::is_empty")]
    pub bucket_counts: Vec<u64>,
    #[serde(rename = "explicitBounds", skip_serializing_if = "Vec::is_empty")]
    pub explicit_bounds: Vec<f64>,
    #[serde(rename = "timeUnixNano")]
    pub time_unix_nano: String,
    pub attributes: Vec<OtlpKeyValue>,
    #[serde(rename = "exemplars", skip_serializing_if = "Vec::is_empty")]
    pub exemplars: Vec<serde_json::Value>,
}

// ---------------------------------------------------------------------------
// OTLP Exporter configuration
// ---------------------------------------------------------------------------

/// Configuration for the OTLP exporter.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OtlpExporterConfig {
    /// OTLP endpoint URL (e.g. "http://localhost:4318/v1/traces").
    pub endpoint: String,
    /// Service name (overrides Resource if set).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_name: Option<String>,
    /// Batch size before flushing (default: 512).
    pub batch_size: usize,
    /// Maximum time between flushes (default: 5s).
    pub flush_interval: Duration,
    /// Timeout for HTTP requests (default: 10s).
    pub request_timeout: Duration,
}

impl Default for OtlpExporterConfig {
    fn default() -> Self {
        Self {
            endpoint: std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT")
                .unwrap_or_else(|_| "http://localhost:4318/v1/traces".to_string()),
            service_name: None,
            batch_size: 512,
            flush_interval: Duration::from_secs(5),
            request_timeout: Duration::from_secs(10),
        }
    }
}

// ---------------------------------------------------------------------------
// OTLP Exporter
// ---------------------------------------------------------------------------

/// Exports OpenTelemetry data in OTLP/JSON format over HTTP.
///
/// Uses `reqwest::Client` to POST to an OTLP-compatible collector endpoint.
/// Spans are collected from an `InstrumentationProvider` and serialized
/// according to the OTLP JSON specification.
pub struct OtlpExporter {
    config: OtlpExporterConfig,
    client: reqwest::Client,
    exported_spans: AtomicU64,
    exported_metrics: AtomicU64,
    failed_exports: AtomicU64,
}

impl std::fmt::Debug for OtlpExporter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OtlpExporter")
            .field("endpoint", &self.config.endpoint)
            .field("batch_size", &self.config.batch_size)
            .field(
                "exported_spans",
                &self.exported_spans.load(Ordering::Relaxed),
            )
            .field(
                "exported_metrics",
                &self.exported_metrics.load(Ordering::Relaxed),
            )
            .field(
                "failed_exports",
                &self.failed_exports.load(Ordering::Relaxed),
            )
            .finish()
    }
}

impl OtlpExporter {
    /// Create a new OTLP exporter with the given configuration.
    pub fn new(config: OtlpExporterConfig) -> Self {
        let client = reqwest::Client::builder()
            .timeout(config.request_timeout)
            .build()
            .expect("failed to create reqwest client");
        Self {
            config,
            client,
            exported_spans: AtomicU64::new(0),
            exported_metrics: AtomicU64::new(0),
            failed_exports: AtomicU64::new(0),
        }
    }

    /// Export completed spans from the provider to the configured endpoint.
    pub async fn export_spans(
        &self,
        provider: &InstrumentationProvider,
    ) -> Result<ExportResult, ExportError> {
        let spans = provider.export_spans();
        let resource = provider.resource().clone();

        if spans.is_empty() {
            return Ok(ExportResult {
                spans_exported: 0,
                metrics_exported: 0,
            });
        }

        let request = build_otlp_request(&spans, &[], &resource, &self.config);

        let body = serde_json::to_vec(&request).map_err(|e| ExportError::Serialization {
            detail: e.to_string(),
        })?;

        let resp = self
            .client
            .post(&self.config.endpoint)
            .header("Content-Type", "application/json")
            .body(body)
            .send()
            .await
            .map_err(|e| ExportError::Http {
                detail: e.to_string(),
            })?;

        let status = resp.status();
        if status.is_success() {
            self.exported_spans
                .fetch_add(spans.len() as u64, Ordering::Relaxed);
            Ok(ExportResult {
                spans_exported: spans.len(),
                metrics_exported: 0,
            })
        } else {
            self.failed_exports.fetch_add(1, Ordering::Relaxed);
            Err(ExportError::Remote {
                status: status.as_u16(),
                detail: format!("OTLP collector returned {status}"),
            })
        }
    }

    /// Export metrics from the provider to the configured endpoint.
    pub async fn export_metrics(
        &self,
        provider: &InstrumentationProvider,
    ) -> Result<ExportResult, ExportError> {
        let resource = provider.resource().clone();
        let metric_names = provider.metric_names();

        if metric_names.is_empty() {
            return Ok(ExportResult {
                spans_exported: 0,
                metrics_exported: 0,
            });
        }

        let metrics: Vec<MetricInstrument> = metric_names
            .iter()
            .filter_map(|name| provider.get_metric(name))
            .collect();

        let request = build_otlp_request(&[], &metrics, &resource, &self.config);

        let body = serde_json::to_vec(&request).map_err(|e| ExportError::Serialization {
            detail: e.to_string(),
        })?;

        let resp = self
            .client
            .post(&self.config.endpoint)
            .header("Content-Type", "application/json")
            .body(body)
            .send()
            .await
            .map_err(|e| ExportError::Http {
                detail: e.to_string(),
            })?;

        let status = resp.status();
        if status.is_success() {
            self.exported_metrics
                .fetch_add(metrics.len() as u64, Ordering::Relaxed);
            Ok(ExportResult {
                spans_exported: 0,
                metrics_exported: metrics.len(),
            })
        } else {
            self.failed_exports.fetch_add(1, Ordering::Relaxed);
            Err(ExportError::Remote {
                status: status.as_u16(),
                detail: format!("OTLP collector returned {status}"),
            })
        }
    }

    /// Export both spans and metrics in a single request.
    pub async fn export(
        &self,
        provider: &InstrumentationProvider,
    ) -> Result<ExportResult, ExportError> {
        let spans = provider.export_spans();
        let resource = provider.resource().clone();

        let metrics: Vec<MetricInstrument> = provider
            .metric_names()
            .iter()
            .filter_map(|name| provider.get_metric(name))
            .collect();

        if spans.is_empty() && metrics.is_empty() {
            return Ok(ExportResult {
                spans_exported: 0,
                metrics_exported: 0,
            });
        }

        let request = build_otlp_request(&spans, &metrics, &resource, &self.config);

        let body = serde_json::to_vec(&request).map_err(|e| ExportError::Serialization {
            detail: e.to_string(),
        })?;

        let resp = self
            .client
            .post(&self.config.endpoint)
            .header("Content-Type", "application/json")
            .body(body)
            .send()
            .await
            .map_err(|e| ExportError::Http {
                detail: e.to_string(),
            })?;

        let status = resp.status();
        if status.is_success() {
            self.exported_spans
                .fetch_add(spans.len() as u64, Ordering::Relaxed);
            self.exported_metrics
                .fetch_add(metrics.len() as u64, Ordering::Relaxed);
            Ok(ExportResult {
                spans_exported: spans.len(),
                metrics_exported: metrics.len(),
            })
        } else {
            self.failed_exports.fetch_add(1, Ordering::Relaxed);
            Err(ExportError::Remote {
                status: status.as_u16(),
                detail: format!("OTLP collector returned {status}"),
            })
        }
    }

    /// Statistics counters.
    pub fn stats(&self) -> ExporterStats {
        ExporterStats {
            exported_spans: self.exported_spans.load(Ordering::Relaxed),
            exported_metrics: self.exported_metrics.load(Ordering::Relaxed),
            failed_exports: self.failed_exports.load(Ordering::Relaxed),
        }
    }

    /// Endpoint URL.
    pub fn endpoint(&self) -> &str {
        &self.config.endpoint
    }
}

// ---------------------------------------------------------------------------
// Result and error types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct ExportResult {
    pub spans_exported: usize,
    pub metrics_exported: usize,
}

#[derive(Debug, thiserror::Error)]
pub enum ExportError {
    #[error("serialization failed: {detail}")]
    Serialization { detail: String },
    #[error("HTTP request failed: {detail}")]
    Http { detail: String },
    #[error("remote collector error (HTTP {status}): {detail}")]
    Remote { status: u16, detail: String },
}

#[derive(Debug, Clone)]
pub struct ExporterStats {
    pub exported_spans: u64,
    pub exported_metrics: u64,
    pub failed_exports: u64,
}

// ---------------------------------------------------------------------------
// OTLP serialization helpers
// ---------------------------------------------------------------------------

/// Build an OTLP export request from spans, metrics, and resource.
fn build_otlp_request(
    spans: &[OtSpan],
    metrics: &[MetricInstrument],
    resource: &Resource,
    config: &OtlpExporterConfig,
) -> OtlpExportRequest {
    let service_name = config
        .service_name
        .as_deref()
        .unwrap_or(&resource.service_name);

    let otlp_resource = OtlpResource {
        attributes: vec![
            kv(
                "service.name",
                OtlpAnyValue::String(service_name.to_string()),
            ),
            kv(
                "service.version",
                OtlpAnyValue::String(resource.service_version.clone()),
            ),
            kv(
                "deployment.environment",
                OtlpAnyValue::String(resource.deployment_environment.clone()),
            ),
        ],
        dropped_attributes_count: 0,
    };

    let scope = OtlpInstrumentationScope {
        name: "civitforge".to_string(),
        version: "0.1.0".to_string(),
    };

    // Convert spans
    let otlp_spans: Vec<OtlpSpanProto> = spans.iter().map(convert_span).collect();

    let resource_spans = if !otlp_spans.is_empty() {
        vec![OtlpResourceSpans {
            resource: otlp_resource.clone(),
            scope_spans: vec![OtlpScopeSpans {
                scope: scope.clone(),
                spans: otlp_spans,
                schema_url: None,
            }],
            schema_url: None,
        }]
    } else {
        vec![]
    };

    // Convert metrics
    let otlp_metrics: Vec<OtlpMetricProto> = metrics.iter().map(convert_metric).collect();

    let resource_metrics = if !otlp_metrics.is_empty() {
        vec![OtlpResourceMetrics {
            resource: otlp_resource,
            scope_metrics: vec![OtlpScopeMetrics {
                scope,
                metrics: otlp_metrics,
                schema_url: None,
            }],
            schema_url: None,
        }]
    } else {
        vec![]
    };

    OtlpExportRequest {
        resource_spans,
        resource_metrics,
    }
}

/// Convert an `OtSpan` to OTLP protobuf-mapped JSON.
fn convert_span(span: &OtSpan) -> OtlpSpanProto {
    let start_nano = chrono_to_nano(span.start_time);
    let end_nano = span
        .end_time
        .map(chrono_to_nano)
        .unwrap_or(start_nano.clone());

    OtlpSpanProto {
        trace_id: span.trace_id.clone(),
        span_id: span.span_id.clone(),
        parent_span_id: span.parent_span_id.clone(),
        name: span.name.clone(),
        kind: span_kind_to_int(span.kind),
        start_time_unix_nano: start_nano,
        end_time_unix_nano: end_nano,
        attributes: convert_attributes(&span.attributes),
        status: convert_status(&span.status),
        events: span
            .events
            .iter()
            .map(|e| OtlpSpanEventProto {
                name: e.name.clone(),
                time_unix_nano: chrono_to_nano(e.timestamp),
                attributes: convert_attributes(&e.attributes),
            })
            .collect(),
        links: span
            .links
            .iter()
            .map(|l| OtlpSpanLinkProto {
                trace_id: l.trace_context.trace_id.clone(),
                span_id: l.trace_context.span_id.clone(),
                trace_state: l.trace_context.trace_state.clone(),
                attributes: convert_attributes(&l.attributes),
                dropped_attributes_count: 0,
            })
            .collect(),
    }
}

/// Convert a `MetricInstrument` to OTLP metric proto.
fn convert_metric(instrument: &MetricInstrument) -> OtlpMetricProto {
    OtlpMetricProto {
        name: instrument.definition.name.clone(),
        description: instrument.definition.description.clone(),
        unit: instrument.definition.unit.clone(),
        data: match instrument.definition.metric_type {
            MetricType::Counter => {
                let latest = instrument.points().last().cloned().unwrap_or(MetricPoint {
                    timestamp: chrono::Utc::now(),
                    value: 0.0,
                    attributes: HashMap::new(),
                });
                OtlpMetricData::DoubleSum(OtlpSumProto {
                    data_points: vec![OtlpNumberDataPointProto {
                        as_int: None,
                        as_double: Some(latest.value),
                        time_unix_nano: chrono_to_nano(latest.timestamp),
                        attributes: convert_attributes(&latest.attributes),
                        exemplars: vec![],
                    }],
                    is_monotonic: true,
                    aggregation_temporality: 2, // Cumulative
                })
            }
            MetricType::UpDownCounter => {
                let latest = instrument.points().last().cloned().unwrap_or(MetricPoint {
                    timestamp: chrono::Utc::now(),
                    value: 0.0,
                    attributes: HashMap::new(),
                });
                OtlpMetricData::DoubleSum(OtlpSumProto {
                    data_points: vec![OtlpNumberDataPointProto {
                        as_int: None,
                        as_double: Some(latest.value),
                        time_unix_nano: chrono_to_nano(latest.timestamp),
                        attributes: convert_attributes(&latest.attributes),
                        exemplars: vec![],
                    }],
                    is_monotonic: false,
                    aggregation_temporality: 2,
                })
            }
            MetricType::Gauge => {
                let latest = instrument.points().last().cloned().unwrap_or(MetricPoint {
                    timestamp: chrono::Utc::now(),
                    value: 0.0,
                    attributes: HashMap::new(),
                });
                OtlpMetricData::DoubleGauge(OtlpGaugeProto {
                    data_points: vec![OtlpNumberDataPointProto {
                        as_int: None,
                        as_double: Some(latest.value),
                        time_unix_nano: chrono_to_nano(latest.timestamp),
                        attributes: convert_attributes(&latest.attributes),
                        exemplars: vec![],
                    }],
                })
            }
            MetricType::Histogram => {
                let latest = instrument.points().last().cloned().unwrap_or(MetricPoint {
                    timestamp: chrono::Utc::now(),
                    value: 0.0,
                    attributes: HashMap::new(),
                });
                OtlpMetricData::Histogram(OtlpHistogramProto {
                    data_points: vec![OtlpHistogramDataPointProto {
                        count: if latest.value > 0.0 { 1 } else { 0 },
                        sum: Some(latest.value),
                        min: Some(latest.value),
                        max: Some(latest.value),
                        bucket_counts: vec![],
                        explicit_bounds: vec![],
                        time_unix_nano: chrono_to_nano(latest.timestamp),
                        attributes: convert_attributes(&latest.attributes),
                        exemplars: vec![],
                    }],
                    aggregation_temporality: 2,
                })
            }
        },
    }
}

// ---------------------------------------------------------------------------
// Conversion utilities
// ---------------------------------------------------------------------------

fn kv(key: &str, value: OtlpAnyValue) -> OtlpKeyValue {
    OtlpKeyValue {
        key: key.to_string(),
        value,
    }
}

fn convert_attributes(attrs: &HashMap<String, OtelAttribute>) -> Vec<OtlpKeyValue> {
    attrs
        .iter()
        .map(|(k, v)| OtlpKeyValue {
            key: k.clone(),
            value: match v {
                OtelAttribute::String(s) => OtlpAnyValue::String(s.clone()),
                OtelAttribute::Int(i) => OtlpAnyValue::Int(*i),
                OtelAttribute::Double(d) => OtlpAnyValue::Double(*d),
                OtelAttribute::Bool(b) => OtlpAnyValue::Bool(*b),
                OtelAttribute::StringArray(arr) => OtlpAnyValue::Array(OtlpArrayValue {
                    values: arr
                        .iter()
                        .map(|s| OtlpAnyValue::String(s.clone()))
                        .collect(),
                }),
                OtelAttribute::IntArray(arr) => OtlpAnyValue::Array(OtlpArrayValue {
                    values: arr.iter().map(|i| OtlpAnyValue::Int(*i)).collect(),
                }),
            },
        })
        .collect()
}

fn span_kind_to_int(kind: SpanKind) -> i32 {
    match kind {
        SpanKind::Internal => 1,
        SpanKind::Client => 3,
        SpanKind::Server => 2,
        SpanKind::Producer => 4,
        SpanKind::Consumer => 5,
    }
}

fn convert_status(status: &SpanStatus) -> OtlpStatus {
    match status {
        SpanStatus::Ok => OtlpStatus {
            code: "OK".to_string(),
            message: None,
        },
        SpanStatus::Error { message } => OtlpStatus {
            code: "ERROR".to_string(),
            message: Some(message.clone()),
        },
    }
}

/// Convert a `chrono::DateTime<Utc>` to nanoseconds since Unix epoch string.
fn chrono_to_nano(dt: chrono::DateTime<chrono::Utc>) -> String {
    dt.timestamp_nanos_opt()
        .map(|n| n.to_string())
        .unwrap_or_else(|| "0".to_string())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::telemetry::opentelemetry::MetricDefinition;

    fn make_resource() -> Resource {
        Resource {
            service_name: "test-service".into(),
            service_version: "0.1.0".into(),
            deployment_environment: "test".into(),
            attributes: HashMap::new(),
        }
    }

    fn make_span() -> OtSpan {
        let mut span = OtSpan::new(
            "test-span",
            "abcdef0123456789abcdef0123456789",
            "abcdef0123456789",
        );
        span.end();
        span
    }

    fn make_span_with_attrs() -> OtSpan {
        let mut span = OtSpan::new(
            "http-request",
            "abcdef0123456789abcdef0123456789",
            "0123456789abcdef",
        );
        span.set_attribute("http.method", OtelAttribute::String("GET".into()));
        span.set_attribute("http.status_code", OtelAttribute::Int(200));
        span.add_event("request.start", HashMap::new());
        span.end();
        span
    }

    fn make_config() -> OtlpExporterConfig {
        OtlpExporterConfig {
            endpoint: "http://localhost:4318/v1/traces".into(),
            service_name: Some("test-service".into()),
            batch_size: 10,
            flush_interval: Duration::from_secs(1),
            request_timeout: Duration::from_secs(5),
        }
    }

    #[test]
    fn test_convert_span_basic() {
        let span = make_span();
        let proto = convert_span(&span);
        assert_eq!(proto.name, "test-span");
        assert_eq!(proto.kind, 1); // Internal
        assert_eq!(proto.status.code, "OK");
        assert!(proto.start_time_unix_nano.parse::<i64>().is_ok());
    }

    #[test]
    fn test_convert_span_with_parent() {
        let span = OtSpan::new("child", "trace1", "span2").with_parent("span1");
        let proto = convert_span(&span);
        assert_eq!(proto.parent_span_id, Some("span1".into()));
    }

    #[test]
    fn test_convert_span_with_attributes() {
        let span = make_span_with_attrs();
        let proto = convert_span(&span);
        assert_eq!(proto.attributes.len(), 2);
        let method_attr = proto.attributes.iter().find(|a| a.key == "http.method");
        assert!(method_attr.is_some());
    }

    #[test]
    fn test_convert_span_with_events() {
        let span = make_span_with_attrs();
        let proto = convert_span(&span);
        assert_eq!(proto.events.len(), 1);
        assert_eq!(proto.events[0].name, "request.start");
    }

    #[test]
    fn test_convert_span_error_status() {
        let mut span = OtSpan::new("fail", "t", "s");
        span.set_status(SpanStatus::Error {
            message: "timeout".into(),
        });
        span.end();
        let proto = convert_span(&span);
        assert_eq!(proto.status.code, "ERROR");
        assert_eq!(proto.status.message, Some("timeout".into()));
    }

    #[test]
    fn test_span_kind_mapping() {
        assert_eq!(span_kind_to_int(SpanKind::Internal), 1);
        assert_eq!(span_kind_to_int(SpanKind::Server), 2);
        assert_eq!(span_kind_to_int(SpanKind::Client), 3);
        assert_eq!(span_kind_to_int(SpanKind::Producer), 4);
        assert_eq!(span_kind_to_int(SpanKind::Consumer), 5);
    }

    #[test]
    fn test_convert_attributes_string() {
        let mut attrs = HashMap::new();
        attrs.insert("key".into(), OtelAttribute::String("val".into()));
        let kv = convert_attributes(&attrs);
        assert_eq!(kv.len(), 1);
        assert_eq!(kv[0].key, "key");
    }

    #[test]
    fn test_convert_attributes_mixed() {
        let mut attrs = HashMap::new();
        attrs.insert("s".into(), OtelAttribute::String("hello".into()));
        attrs.insert("i".into(), OtelAttribute::Int(42));
        attrs.insert("d".into(), OtelAttribute::Double(2.71));
        attrs.insert("b".into(), OtelAttribute::Bool(true));
        let kv = convert_attributes(&attrs);
        assert_eq!(kv.len(), 4);
    }

    #[test]
    fn test_convert_attributes_array() {
        let mut attrs = HashMap::new();
        attrs.insert(
            "tags".into(),
            OtelAttribute::StringArray(vec!["a".into(), "b".into()]),
        );
        let kv = convert_attributes(&attrs);
        assert_eq!(kv.len(), 1);
    }

    #[test]
    fn test_chrono_to_nano() {
        let dt = chrono::Utc::now();
        let nano_str = chrono_to_nano(dt);
        assert!(nano_str.parse::<i64>().is_ok());
        assert!(nano_str.len() > 10);
    }

    #[test]
    fn test_build_otlp_request_empty() {
        let resource = make_resource();
        let config = make_config();
        let req = build_otlp_request(&[], &[], &resource, &config);
        assert!(req.resource_spans.is_empty());
        assert!(req.resource_metrics.is_empty());
    }

    #[test]
    fn test_build_otlp_request_with_spans() {
        let resource = make_resource();
        let config = make_config();
        let spans = vec![make_span()];
        let req = build_otlp_request(&spans, &[], &resource, &config);
        assert_eq!(req.resource_spans.len(), 1);
        assert_eq!(req.resource_spans[0].scope_spans[0].spans.len(), 1);
        assert_eq!(
            req.resource_spans[0].resource.attributes[0].key,
            "service.name"
        );
    }

    #[test]
    fn test_otlp_export_request_serialization() {
        let resource = make_resource();
        let config = make_config();
        let spans = vec![make_span(), make_span_with_attrs()];
        let req = build_otlp_request(&spans, &[], &resource, &config);
        let json = serde_json::to_string(&req).expect("serialize");
        assert!(json.contains("resourceSpans"));
        assert!(json.contains("http-request"));
    }

    #[test]
    fn test_otlp_export_request_with_metrics() {
        let resource = make_resource();
        let config = make_config();
        let mut instrument = MetricInstrument::new(MetricDefinition {
            name: "http.requests".into(),
            description: "Total HTTP requests".into(),
            unit: "1".into(),
            metric_type: MetricType::Counter,
        });
        instrument.record(42.0, HashMap::new());
        let instruments = vec![instrument];
        let req = build_otlp_request(&[], &instruments, &resource, &config);
        assert_eq!(req.resource_metrics.len(), 1);
        let json = serde_json::to_string(&req).expect("serialize");
        assert!(json.contains("resourceMetrics"));
        assert!(json.contains("http.requests"));
    }

    #[test]
    fn test_convert_metric_counter() {
        let mut instrument = MetricInstrument::new(MetricDefinition {
            name: "requests".into(),
            description: "desc".into(),
            unit: "1".into(),
            metric_type: MetricType::Counter,
        });
        instrument.record(10.0, HashMap::new());
        let proto = convert_metric(&instrument);
        assert_eq!(proto.name, "requests");
        assert!(matches!(proto.data, OtlpMetricData::DoubleSum(_)));
    }

    #[test]
    fn test_convert_metric_gauge() {
        let mut instrument = MetricInstrument::new(MetricDefinition {
            name: "temperature".into(),
            description: "desc".into(),
            unit: "C".into(),
            metric_type: MetricType::Gauge,
        });
        instrument.record(22.5, HashMap::new());
        let proto = convert_metric(&instrument);
        assert!(matches!(proto.data, OtlpMetricData::DoubleGauge(_)));
    }

    #[test]
    fn test_convert_metric_histogram() {
        let mut instrument = MetricInstrument::new(MetricDefinition {
            name: "latency".into(),
            description: "desc".into(),
            unit: "ms".into(),
            metric_type: MetricType::Histogram,
        });
        instrument.record(150.0, HashMap::new());
        let proto = convert_metric(&instrument);
        assert!(matches!(proto.data, OtlpMetricData::Histogram(_)));
    }

    #[test]
    fn test_otlp_exporter_new() {
        let config = make_config();
        let exporter = OtlpExporter::new(config);
        assert_eq!(exporter.endpoint(), "http://localhost:4318/v1/traces");
    }

    #[test]
    fn test_otlp_exporter_stats() {
        let exporter = OtlpExporter::new(make_config());
        let stats = exporter.stats();
        assert_eq!(stats.exported_spans, 0);
        assert_eq!(stats.exported_metrics, 0);
        assert_eq!(stats.failed_exports, 0);
    }

    #[test]
    fn test_otlp_exporter_debug() {
        let exporter = OtlpExporter::new(make_config());
        let debug = format!("{exporter:?}");
        assert!(debug.contains("OtlpExporter"));
        assert!(debug.contains("localhost:4318"));
    }

    #[test]
    fn test_otlp_config_default() {
        let config = OtlpExporterConfig::default();
        assert!(config.endpoint.contains("4318"));
        assert_eq!(config.batch_size, 512);
    }

    #[test]
    fn test_export_error_types() {
        let err = ExportError::Serialization {
            detail: "bad json".into(),
        };
        assert!(err.to_string().contains("serialization"));

        let err = ExportError::Http {
            detail: "connection refused".into(),
        };
        assert!(err.to_string().contains("HTTP"));

        let err = ExportError::Remote {
            status: 400,
            detail: "bad request".into(),
        };
        assert!(err.to_string().contains("400"));
    }

    #[test]
    fn test_otlp_resource_attributes() {
        let resource = make_resource();
        let config = make_config();
        let req = build_otlp_request(&[make_span()], &[], &resource, &config);
        let attrs = &req.resource_spans[0].resource.attributes;
        assert!(attrs.iter().any(|a| a.key == "service.name"));
        assert!(attrs.iter().any(|a| a.key == "service.version"));
        assert!(attrs.iter().any(|a| a.key == "deployment.environment"));
    }

    #[test]
    fn test_otlp_any_value_serialization() {
        let s = OtlpAnyValue::String("hello".into());
        let json = serde_json::to_string(&s).unwrap();
        assert!(json.contains("stringValue"));

        let i = OtlpAnyValue::Int(42);
        let json = serde_json::to_string(&i).unwrap();
        assert!(json.contains("intValue"));

        let d = OtlpAnyValue::Double(2.71);
        let json = serde_json::to_string(&d).unwrap();
        assert!(json.contains("doubleValue"));

        let b = OtlpAnyValue::Bool(true);
        let json = serde_json::to_string(&b).unwrap();
        assert!(json.contains("boolValue"));
    }

    #[test]
    fn test_otlp_status_serialization() {
        let ok = OtlpStatus {
            code: "OK".into(),
            message: None,
        };
        let json = serde_json::to_string(&ok).unwrap();
        assert!(json.contains("OK"));

        let err = OtlpStatus {
            code: "ERROR".into(),
            message: Some("timeout".into()),
        };
        let json = serde_json::to_string(&err).unwrap();
        assert!(json.contains("ERROR"));
        assert!(json.contains("timeout"));
    }

    #[test]
    fn test_export_result() {
        let result = ExportResult {
            spans_exported: 10,
            metrics_exported: 5,
        };
        assert_eq!(result.spans_exported, 10);
        assert_eq!(result.metrics_exported, 5);
    }
}
