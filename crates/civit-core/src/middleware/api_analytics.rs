#![forbid(unsafe_code)]

//! API analytics middleware to record all requests.
//!
//! Records endpoint, method, status code, response time, user ID, IP address,
//! user agent, and request/response sizes to the api_analytics table.

use axum::{
    extract::Request,
    middleware::Next,
    response::Response,
};
use std::net::IpAddr;
use std::time::Instant;
use tracing::error;

/// Extract client IP from request. Checks `X-Forwarded-For`, `X-Real-Ip`,
/// then falls back to loopback.
fn extract_client_ip(req: &Request) -> IpAddr {
    if let Some(forwarded) = req.headers().get("x-forwarded-for")
        && let Ok(val) = forwarded.to_str()
        && let Some(first_ip) = val.split(',').next().map(|s| s.trim())
        && let Ok(ip) = first_ip.parse::<IpAddr>()
    {
        return ip;
    }

    if let Some(real_ip) = req.headers().get("x-real-ip")
        && let Ok(val) = real_ip.to_str()
        && let Ok(ip) = val.parse::<IpAddr>()
    {
        return ip;
    }

    IpAddr::V4(std::net::Ipv4Addr::LOCALHOST)
}

/// Extract user ID from JWT claims if present.
fn extract_user_id(req: &Request) -> Option<uuid::Uuid> {
    if let Some(auth_header) = req.headers().get("authorization") {
        if let Ok(_auth_str) = auth_header.to_str() {
            if let Some(token) = auth_header.to_str().ok().and_then(|s| s.strip_prefix("Bearer ")) {
                if let Some(jwt_service) = req.extensions().get::<std::sync::Arc<civit_auth::jwt::JwtService>>() {
                    if let Ok(claims) = jwt_service.validate_token(token) {
                        return claims.sub.parse::<uuid::Uuid>().ok();
                    }
                }
            }
        }
    }
    None
}

/// API analytics middleware function for axum.
pub async fn api_analytics_middleware(req: Request, next: Next) -> Response {
    let start = Instant::now();
    let method = req.method().clone();
    let uri = req.uri().clone();
    let endpoint = uri.path().to_string();
    let request_size = req
        .headers()
        .get("content-length")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.parse::<i32>().ok())
        .unwrap_or(0);
    let ip_address = extract_client_ip(&req);
    let user_agent = req
        .headers()
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());
    let user_id = extract_user_id(&req);
    let db = req.extensions().get::<std::sync::Arc<crate::db::DbRepository>>().cloned();

    let response = next.run(req).await;

    let duration = start.elapsed();
    let response_time_ms = duration.as_millis() as i32;
    let status_code = response.status().as_u16() as i32;
    let response_size = response
        .headers()
        .get("content-length")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.parse::<i32>().ok())
        .unwrap_or(0);

    if let Some(db) = db {
        let endpoint_clone = endpoint.clone();
        let method_str = method.to_string();
        let ua = user_agent.clone();
        let ip = ip_address.to_string();

        tokio::spawn(async move {
            if let Err(e) = db.record_api_analytic(
                &endpoint_clone,
                &method_str,
                status_code,
                response_time_ms,
                user_id,
                Some(&ip),
                ua.as_deref(),
                request_size,
                response_size,
            ).await {
                error!("Failed to record API analytic: {}", e);
            }
        });
    }

    response
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_client_ip_forwarded() {
        let req = Request::builder()
            .header("x-forwarded-for", "4.3.2.1, 1.2.3.4")
            .body(axum::body::Body::empty())
            .unwrap();
        assert_eq!(
            extract_client_ip(&req),
            "4.3.2.1".parse::<IpAddr>().unwrap()
        );
    }

    #[test]
    fn test_extract_client_ip_real_ip() {
        let req = Request::builder()
            .header("X-Real-Ip", "10.0.0.1")
            .body(axum::body::Body::empty())
            .unwrap();
        assert_eq!(
            extract_client_ip(&req),
            "10.0.0.1".parse::<IpAddr>().unwrap()
        );
    }

    #[test]
    fn test_extract_client_ip_fallback() {
        let req = Request::builder()
            .body(axum::body::Body::empty())
            .unwrap();
        assert_eq!(
            extract_client_ip(&req),
            IpAddr::V4(std::net::Ipv4Addr::LOCALHOST)
        );
    }
}