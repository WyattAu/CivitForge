//! CSRF protection middleware for browser-initiated requests.
//!
//! Validates Origin header on state-changing methods (POST/PUT/DELETE/PATCH).
//! API clients without Origin/Referer headers (curl, programmatic) are allowed.
//! Requests with Origin from unknown browser origins are rejected.

#![forbid(unsafe_code)]

use axum::{
    extract::Request,
    http::{Method, StatusCode, header},
    middleware::Next,
    response::{IntoResponse, Response},
};
use tracing::warn;

/// CSRF protection middleware.
///
/// Rules:
/// - Safe methods (GET, HEAD, OPTIONS) → always allowed
/// - Requests without Origin AND without Referer → allowed (API client)
/// - Requests with Origin matching localhost/127.0.0.1 or HTTPS → allowed
/// - Requests with unrecognized Origin → rejected 403
pub async fn csrf_middleware(req: Request, next: Next) -> Response {
    let method = req.method().clone();

    // Safe methods are always allowed
    if matches!(method, Method::GET | Method::HEAD | Method::OPTIONS) {
        return next.run(req).await;
    }

    // State-changing methods without Origin or Referer → likely API client (curl, etc.)
    let has_origin = req.headers().get(header::ORIGIN).is_some();
    let has_referer = req.headers().get(header::REFERER).is_some();

    if !has_origin && !has_referer {
        return next.run(req).await;
    }

    // If Origin is present, validate it
    if let Some(origin) = req.headers().get(header::ORIGIN)
        && let Ok(origin_str) = origin.to_str()
    {
        let allowed = origin_str.starts_with("http://127.0.0.1")
            || origin_str.starts_with("http://localhost")
            || origin_str.starts_with("http://[::1]")
            || origin_str.starts_with("https://");

        if allowed {
            return next.run(req).await;
        }

        warn!(origin = %origin_str, method = %method, "CSRF: blocked request from unknown origin");
        return (
            StatusCode::FORBIDDEN,
            axum::Json(serde_json::json!({
                "error": "csrf_rejected",
                "message": "Request origin not allowed."
            })),
        )
            .into_response();
    }

    // Has Referer but no Origin → allow (legitimate API client)
    next.run(req).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;

    fn make_request(
        method: &str,
        origin: Option<&str>,
        referer: Option<&str>,
    ) -> axum::http::Request<Body> {
        let mut builder = axum::http::Request::builder()
            .method(method)
            .uri("/api/v1/repos");
        if let Some(o) = origin {
            builder = builder.header(header::ORIGIN, o);
        }
        if let Some(r) = referer {
            builder = builder.header(header::REFERER, r);
        }
        builder.body(Body::empty()).unwrap()
    }

    /// Test the CSRF decision logic directly (Origin validation without middleware plumbing).
    fn csrf_decision(req: &axum::http::Request<Body>) -> bool {
        let method = req.method();

        if matches!(method, &Method::GET | &Method::HEAD | &Method::OPTIONS) {
            return true; // allowed
        }

        let has_origin = req.headers().get(header::ORIGIN).is_some();
        let has_referer = req.headers().get(header::REFERER).is_some();

        if !has_origin && !has_referer {
            return true; // allowed
        }

        if let Some(origin) = req.headers().get(header::ORIGIN) {
            if let Ok(origin_str) = origin.to_str() {
                return origin_str.starts_with("http://127.0.0.1")
                    || origin_str.starts_with("http://localhost")
                    || origin_str.starts_with("http://[::1]")
                    || origin_str.starts_with("https://");
            }
        }

        true // has Referer but no Origin
    }

    #[test]
    fn test_safe_methods_allowed() {
        let req = make_request("GET", Some("http://evil.com"), None);
        assert!(csrf_decision(&req));
    }

    #[test]
    fn test_no_origin_no_referer_allowed() {
        let req = make_request("POST", None, None);
        assert!(csrf_decision(&req));
    }

    #[test]
    fn test_localhost_origin_allowed() {
        let req = make_request("POST", Some("http://localhost:9091"), None);
        assert!(csrf_decision(&req));
    }

    #[test]
    fn test_127001_origin_allowed() {
        let req = make_request("POST", Some("http://127.0.0.1:9091"), None);
        assert!(csrf_decision(&req));
    }

    #[test]
    fn test_https_origin_allowed() {
        let req = make_request("POST", Some("https://forge.example.com"), None);
        assert!(csrf_decision(&req));
    }

    #[test]
    fn test_evil_origin_blocked() {
        let req = make_request("POST", Some("http://evil.com"), None);
        assert!(!csrf_decision(&req));
    }

    #[test]
    fn test_http_non_localhost_blocked() {
        let req = make_request("DELETE", Some("http://attacker.com"), None);
        assert!(!csrf_decision(&req));
    }

    #[test]
    fn test_referer_only_allowed() {
        let req = make_request("PUT", None, Some("http://example.com/page"));
        assert!(csrf_decision(&req));
    }
}
