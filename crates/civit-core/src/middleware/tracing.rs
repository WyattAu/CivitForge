#![forbid(unsafe_code)]

use axum::{
    extract::Request,
    middleware::Next,
    response::Response,
};
use std::sync::Arc;
use std::time::Instant;

/// State passed through request extensions for the tracing middleware.
#[derive(Clone)]
pub struct TracingState {
    pub provider: Arc<crate::telemetry::opentelemetry::InstrumentationProvider>,
}

/// Tracing middleware that records a span for every HTTP request.
///
/// Extracts trace context from `traceparent` header (W3C format) if present,
/// otherwise starts a new root span. Records method, URI, status code, and duration.
pub async fn tracing_middleware(req: Request, next: Next) -> Response {
    let state = req.extensions().get::<Arc<TracingState>>().cloned();

    let provider = match state {
        Some(s) => s.provider.clone(),
        None => return next.run(req).await,
    };

    let method = req.method().to_string();
    let uri = req.uri().path().to_string();

    // Extract or create trace context
    let parent_key = req
        .headers()
        .get("traceparent")
        .and_then(|v| v.to_str().ok())
        .and_then(|h| {
            let ctx = crate::telemetry::opentelemetry::TraceContext::from_w3c(h)?;
            // Look up parent span in provider
            let key = format!("{}:{}", ctx.trace_id, ctx.span_id);
            if provider.active_span_count() > 0 || provider.completed_span_count() > 0 {
                Some(key)
            } else {
                None
            }
        });

    let span_key = if let Some(parent) = parent_key {
        provider.start_child_span(&parent, format!("{method} {uri}"))
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

    let start = Instant::now();
    let response = next.run(req).await;
    let duration = start.elapsed();

    let status = response.status().as_u16();
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

    let duration_ms = duration.as_secs_f64() * 1000.0;
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
}
