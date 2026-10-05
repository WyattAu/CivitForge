#![forbid(unsafe_code)]

use axum::{
    extract::Request,
    middleware::Next,
    response::Response,
};
use std::sync::Arc;
use std::time::Instant;

use opentelemetry::propagation::Extractor;
use opentelemetry::trace::TraceContextExt;
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

/// Extract the inbound `traceparent` into an OTel parent context.
fn extract_parent(headers: &axum::http::HeaderMap) -> Option<opentelemetry::Context> {
    let ctx = opentelemetry::global::get_text_map_propagator(|prop| {
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
    let state = req.extensions().get::<Arc<TracingState>>().cloned();

    let provider = match state {
        Some(s) => s.provider.clone(),
        None => return next.run(req).await,
    };

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
        use tracing_opentelemetry::OpenTelemetrySpanExt;
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

    // Record metrics
    let mut labels = std::collections::HashMap::new();
    labels.insert(
        "method".to_string(),
        crate::telemetry::opentelemetry::OtelAttribute::String(method),
    );
    labels.insert(
        "status".to_string(),
        crate::telemetry::opentelemetry::OtelAttribute::Int(status as i64),
    );
    provider.increment_metric("http_requests_total", labels.clone());
    provider.record_metric("http_request_duration_ms", duration_ms, labels);

    // Also record via the global tracing_setup functions
    crate::telemetry::tracing_setup::record_http_request(duration);

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
        opentelemetry::global::set_text_map_propagator(
            opentelemetry_sdk::propagation::TraceContextPropagator::new(),
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