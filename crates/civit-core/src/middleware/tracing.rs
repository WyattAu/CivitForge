#![forbid(unsafe_code)]

use axum::{
    extract::Request,
    middleware::Next,
    response::Response,
};
use std::sync::Arc;
use std::time::Instant;

use otelkit::otel::opentelemetry::propagation::Extractor;
use otelkit::otel::opentelemetry::trace::TraceContextExt;
use tracing::Instrument;

/// State passed through request extensions for the tracing middleware.
#[derive(Clone)]
pub struct TracingState {
    pub provider: Arc<crate::telemetry::opentelemetry::InstrumentationProvider>,
}

/// Adapter letting the global OTel propagator read inbound HTTP headers.
struct HeaderExtractor<'a>(&'a axum::http::HeaderMap);

impl Extractor for HeaderExtractor<'_> {
    fn get(&self, key: &str) -> Option<&str> {
        self.0.get(key).and_then(|v| v.to_str().ok())
    }

    fn keys(&self) -> Vec<&str> {
        self.0.keys().map(|k| k.as_str()).collect()
    }
}

/// HTTP metric instruments.
///
/// Created once against the global meter provider otelkit installs; before
/// that exists they are no-ops, which is correct — dropping telemetry at
/// boot beats blocking the first request on it.
struct HttpMetrics {
    requests: otelkit::otel::opentelemetry::metrics::Counter<u64>,
    duration: otelkit::otel::opentelemetry::metrics::Histogram<f64>,
}

fn http_metrics() -> &'static HttpMetrics {
    static METRICS: std::sync::OnceLock<HttpMetrics> = std::sync::OnceLock::new();
    METRICS.get_or_init(|| {
        let meter = otelkit::otel::opentelemetry::global::meter("civitforge");
        HttpMetrics {
            requests: meter
                .u64_counter("http_server_requests_total")
                .with_description("HTTP requests handled")
                .build(),
            duration: meter
                .f64_histogram("http_server_request_duration_ms")
                .with_description("HTTP request duration in milliseconds")
                .with_unit("ms")
                .build(),
        }
    })
}

/// Extract the inbound `traceparent` into an OTel parent context.
fn extract_parent(
    headers: &axum::http::HeaderMap,
) -> Option<otelkit::otel::opentelemetry::Context> {
    let ctx = otelkit::otel::opentelemetry::global::get_text_map_propagator(|prop| {
        prop.extract(&HeaderExtractor(headers))
    });
    // An empty context means "no valid inbound header"; treat it as root so
    // `set_parent` is not called with a context that carries no span id.
    ctx.span().span_context().is_valid().then_some(ctx)
}

/// Tracing middleware that records a span for every HTTP request.
///
/// Two paths, deliberately:
/// 1. An SDK span (`tracing` span picked up by otelkit's
///    tracing-opentelemetry layer) carrying the inbound W3C parent, so
///    exported spans join the caller's trace.
/// 2. The in-process `InstrumentationProvider` counters that
///    `/api/v1/observability` reads. Retained until otelkit's Prometheus
///    meter replaces them.
pub async fn tracing_middleware(req: Request, next: Next) -> Response {
    // Extension-based lookup, kept for callers that route through a scope
    // with TracingState installed. The router uses
    // `tracing_middleware_with_state`, because nothing ever inserted this
    // extension: the middleware was dead code and every counter it feeds saw
    // zero requests.
    let provider = req
        .extensions()
        .get::<Arc<TracingState>>()
        .map(|s| s.provider.clone());
    match provider {
        Some(p) => trace_request(p, req, next).await,
        None => next.run(req).await,
    }
}

/// State-based variant the router installs.
///
/// The provider lives on `AppState` — one instance shared with the
/// observability endpoints — so counters aggregated for the UI and the
/// rollout gate are the same numbers.
pub async fn tracing_middleware_with_state(
    axum::extract::State(state): axum::extract::State<crate::api::AppState>,
    req: Request,
    next: Next,
) -> Response {
    trace_request(state.telemetry_provider.clone(), req, next).await
}

async fn trace_request(
    provider: Arc<crate::telemetry::opentelemetry::InstrumentationProvider>,
    req: Request,
    next: Next,
) -> Response {
    let method = req.method().to_string();
    let uri = req.uri().path().to_string();

    // Legacy in-process span, unchanged: observability endpoints and the
    // E2E suite assert its counters.
    let parent_key = req
        .headers()
        .get("traceparent")
        .and_then(|v| v.to_str().ok())
        .and_then(|h| {
            let ctx = crate::telemetry::opentelemetry::TraceContext::from_w3c(h)?;
            let key = format!("{}:{}", ctx.trace_id, ctx.span_id);
            if provider.active_span_count() > 0 || provider.completed_span_count() > 0 {
                Some(key)
            } else {
                None
            }
        });

    let span_key = if let Some(parent) = parent_key {
        provider
            .start_child_span(&parent, format!("{method} {uri}"))
            .unwrap_or_else(|| provider.start_span(format!("{method} {uri}")))
    } else {
        provider.start_span(format!("{method} {uri}"))
    };

    provider.set_attribute(
        &span_key,
        "http.method",
        crate::telemetry::opentelemetry::OtelAttribute::String(method.clone()),
    );
    provider.set_attribute(
        &span_key,
        "http.uri",
        crate::telemetry::opentelemetry::OtelAttribute::String(uri.clone()),
    );
    provider.set_attribute(
        &span_key,
        "http.kind",
        crate::telemetry::opentelemetry::OtelAttribute::String("server".into()),
    );

    let parent_ctx = extract_parent(req.headers());
    let span = tracing::info_span!(
        "http.server.request",
        otel.kind = "server",
        http.request.method = %method,
        url.path = %uri,
        http.response.status_code = tracing::field::Empty,
        http.server.request.duration_ms = tracing::field::Empty,
    );
    if let Some(ctx) = parent_ctx {
        use otelkit::otel::tracing_opentelemetry::OpenTelemetrySpanExt;
        if let Err(e) = span.set_parent(ctx) {
            // No global tracer provider installed (no OTLP endpoint
            // configured). The request still gets a local span; only
            // OTel export is unavailable.
            tracing::debug!("otel parent context not attached: {e}");
        }
    }

    let start = Instant::now();
    let response = next.run(req).instrument(span.clone()).await;
    let duration = start.elapsed();

    let status = response.status().as_u16();
    let duration_ms = duration.as_secs_f64() * 1000.0;

    span.record("http.response.status_code", status);
    span.record("http.server.request.duration_ms", duration_ms);
    if status >= 400 {
        span.record("otel.status_code", "ERROR");
    }

    provider.set_attribute(
        &span_key,
        "http.status_code",
        crate::telemetry::opentelemetry::OtelAttribute::Int(status as i64),
    );

    if status >= 400 {
        provider.set_status(
            &span_key,
            crate::telemetry::opentelemetry::SpanStatus::Error {
                message: format!("HTTP {status}"),
            },
        );
    }

    provider.set_attribute(
        &span_key,
        "http.duration_ms",
        crate::telemetry::opentelemetry::OtelAttribute::Double(duration_ms),
    );

    provider.end_span(&span_key);

    // Real OTel instruments: the previous calls here targeted hand-rolled
    // provider counters that were never registered, so every write was a
    // silent no-op and metrics_registered stayed 0. Attributes follow the
    // OTel HTTP semantic conventions.
    let metrics = http_metrics();
    let attrs = [
        otelkit::otel::opentelemetry::KeyValue::new("http.request.method", method.clone()),
        otelkit::otel::opentelemetry::KeyValue::new(
            "http.response.status_code",
            i64::from(status),
        ),
    ];
    metrics.requests.add(1, &attrs);
    metrics.duration.record(duration_ms, &attrs);

    // Also record via the global tracing_setup functions
    crate::telemetry::tracing_setup::record_http_request(duration);

    // Feed the rolling health window. This is the only place the status code
    // is known, so it is the only place an error rate can come from — and
    // flag-kit's health gate is gated on exactly that.
    crate::telemetry::global_health_window().record(duration, status);

    response
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::telemetry::opentelemetry::{InstrumentationProvider, Resource};

    #[test]
    fn test_tracing_state_clone() {
        let provider = Arc::new(InstrumentationProvider::new(Resource::default()));
        let state = TracingState {
            provider: provider.clone(),
        };
        let state2 = state.clone();
        assert_eq!(
            state.provider.resource().service_name,
            state2.provider.resource().service_name
        );
    }

    #[test]
    fn test_provider_creates_spans() {
        let provider = InstrumentationProvider::new(Resource::default());
        let key = provider.start_span("GET /api/test");
        provider.set_attribute(
            &key,
            "http.method",
            crate::telemetry::opentelemetry::OtelAttribute::String("GET".into()),
        );
        provider.end_span(&key);
        assert_eq!(provider.completed_span_count(), 1);
    }

    /// W3C extraction: a well-formed `traceparent` yields a remote parent
    /// with the caller's trace id, a garbage header yields no parent (the
    /// span must start a fresh root rather than link to nothing).
    #[test]
    fn extracts_w3c_parent_context() {
        otelkit::otel::opentelemetry::global::set_text_map_propagator(
            otelkit::otel::opentelemetry_sdk::propagation::TraceContextPropagator::new(),
        );

        let mut headers = axum::http::HeaderMap::new();
        headers.insert(
            "traceparent",
            "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01"
                .parse()
                .unwrap(),
        );
        let ctx = extract_parent(&headers).expect("valid traceparent");
        let binding = ctx.span();
        let sc = binding.span_context();
        assert_eq!(sc.trace_id().to_string(), "4bf92f3577b34da6a3ce929d0e0e4736");
        assert!(sc.is_remote());

        let mut bad = axum::http::HeaderMap::new();
        bad.insert("traceparent", "not-a-traceparent".parse().unwrap());
        assert!(extract_parent(&bad).is_none());
    }
}