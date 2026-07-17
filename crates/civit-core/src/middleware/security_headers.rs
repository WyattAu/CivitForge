#![forbid(unsafe_code)]

use axum::http::{HeaderValue, header};
use tower_http::set_header::SetResponseHeaderLayer;

pub fn security_headers() -> SetResponseHeaderLayer<HeaderValue> {
    SetResponseHeaderLayer::overriding(
        header::CONTENT_SECURITY_POLICY,
        HeaderValue::from_static(
            "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; font-src 'self';",
        ),
    )
}
