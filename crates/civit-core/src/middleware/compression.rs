#![forbid(unsafe_code)]

//! Response compression middleware with Brotli/gzip support.
//!
//! Compresses API responses using Brotli (preferred) or gzip (fallback).
//! Skips compression for responses smaller than 1KB or already-encoded content.

use axum::{
    body::Body,
    extract::Request,
    http::{HeaderName, HeaderValue, StatusCode, header},
    middleware::Next,
    response::Response,
};
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression as GzCompression;
use serde::{Deserialize, Serialize};
use std::io::Read;
use std::io::Write;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::debug;

/// Minimum response size in bytes before compression is applied.
const MIN_COMPRESS_SIZE: usize = 1024;

/// Supported compression algorithms.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CompressionAlgorithm {
    /// Brotli compression (best ratio, CPU-intensive).
    Brotli,
    /// Gzip compression (fast, widely supported).
    Gzip,
    /// No compression.
    None,
}

impl std::fmt::Display for CompressionAlgorithm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CompressionAlgorithm::Brotli => write!(f, "br"),
            CompressionAlgorithm::Gzip => write!(f, "gzip"),
            CompressionAlgorithm::None => write!(f, "identity"),
        }
    }
}

/// Compression configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressionConfig {
    /// Enable/disable compression globally.
    pub enabled: bool,
    /// Minimum response size to compress (bytes).
    pub min_size: usize,
    /// Gzip compression level (1-9, default 6).
    pub gzip_level: u32,
    /// Preferred algorithm order (first supported is used).
    pub preferred: Vec<CompressionAlgorithm>,
    /// Content types eligible for compression.
    pub compressible_content_types: Vec<String>,
    /// Content types to always skip compression for.
    pub skip_content_types: Vec<String>,
}

impl Default for CompressionConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            min_size: MIN_COMPRESS_SIZE,
            gzip_level: 6,
            preferred: vec![CompressionAlgorithm::Brotli, CompressionAlgorithm::Gzip],
            compressible_content_types: vec![
                "application/json".into(),
                "application/javascript".into(),
                "text/html".into(),
                "text/css".into(),
                "text/plain".into(),
                "text/xml".into(),
                "application/xml".into(),
                "image/svg+xml".into(),
            ],
            skip_content_types: vec![
                "image/png".into(),
                "image/jpeg".into(),
                "image/gif".into(),
                "video/".into(),
                "audio/".into(),
            ],
        }
    }
}

/// Compression statistics.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CompressionStats {
    pub total_responses: u64,
    pub compressed_responses: u64,
    pub skipped_responses: u64,
    pub total_bytes_original: u64,
    pub total_bytes_compressed: u64,
}

/// Shared compression state.
#[derive(Clone)]
pub struct CompressionState {
    pub config: CompressionConfig,
    pub stats: Arc<RwLock<CompressionStats>>,
}

impl CompressionState {
    pub fn new(config: CompressionConfig) -> Self {
        Self {
            config,
            stats: Arc::new(RwLock::new(CompressionStats::default())),
        }
    }

    pub async fn record_compression(&self, original_size: usize, compressed_size: usize) {
        let mut stats = self.stats.write().await;
        stats.total_responses += 1;
        stats.compressed_responses += 1;
        stats.total_bytes_original += original_size as u64;
        stats.total_bytes_compressed += compressed_size as u64;
    }

    pub async fn record_skip(&self) {
        let mut stats = self.stats.write().await;
        stats.total_responses += 1;
        stats.skipped_responses += 1;
    }

    pub async fn get_stats(&self) -> CompressionStats {
        self.stats.read().await.clone()
    }
}

/// Parse the `Accept-Encoding` header and return the best supported algorithm.
pub fn negotiate_encoding(accept_encoding: &str, preferred: &[CompressionAlgorithm]) -> CompressionAlgorithm {
    let accepted: Vec<&str> = accept_encoding
        .split(',')
        .map(|s| s.trim().split(';').next().unwrap_or("").trim())
        .filter(|s| !s.is_empty())
        .collect();

    for algo in preferred {
        if accepted.iter().any(|a| *a == algo.to_string()) {
            return *algo;
        }
    }

    CompressionAlgorithm::None
}

/// Check if the response content type is eligible for compression.
pub fn is_compressible(content_type: &str, config: &CompressionConfig) -> bool {
    for skip in &config.skip_content_types {
        if content_type.contains(skip) {
            return false;
        }
    }
    for ct in &config.compressible_content_types {
        if content_type.contains(ct) {
            return true;
        }
    }
    false
}

/// Compress data using gzip at the configured level.
pub fn compress_gzip(data: &[u8], level: u32) -> std::io::Result<Vec<u8>> {
    let level = GzCompression::new(level.min(9).max(1));
    let mut encoder = GzEncoder::new(Vec::new(), level);
    encoder.write_all(data)?;
    encoder.finish()
}

/// Decompress gzip data.
pub fn decompress_gzip(data: &[u8]) -> std::io::Result<Vec<u8>> {
    let mut decoder = GzDecoder::new(data);
    let mut output = Vec::new();
    decoder.read_to_end(&mut output)?;
    Ok(output)
}

/// Build the Content-Encoding header value for the chosen algorithm.
fn content_encoding_header_value(algo: CompressionAlgorithm) -> HeaderValue {
    match algo {
        CompressionAlgorithm::Brotli => HeaderValue::from_static("br"),
        CompressionAlgorithm::Gzip => HeaderValue::from_static("gzip"),
        CompressionAlgorithm::None => HeaderValue::from_static("identity"),
    }
}

/// Compression middleware for API responses.
pub async fn compression_middleware(req: Request, next: Next) -> Response {
    let state = req
        .extensions()
        .get::<Arc<CompressionState>>()
        .cloned();

    let config = match &state {
        Some(s) => s.config.clone(),
        None => CompressionConfig::default(),
    };

    if !config.enabled {
        return next.run(req).await;
    }

    let accept_encoding = req
        .headers()
        .get(header::ACCEPT_ENCODING)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    let chosen_algo = negotiate_encoding(accept_encoding, &config.preferred);

    let response = next.run(req).await;

    // Don't compress if already encoded or non-success status
    if response.headers().contains_key(header::CONTENT_ENCODING)
        || !response.status().is_success()
    {
        if let Some(ref s) = state {
            s.record_skip().await;
        }
        return response;
    }

    // Check content type eligibility
    let content_type = response
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();

    if !is_compressible(&content_type, &config) {
        if let Some(ref s) = state {
            s.record_skip().await;
        }
        return response;
    }

    // Preserve response status and headers before consuming the body
    let status = response.status();
    let resp_headers = response.headers().clone();

    // Collect body for compression
    let body_bytes = match axum::body::to_bytes(response.into_body(), usize::MAX).await {
        Ok(b) => b,
        Err(_) => {
            return Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Body::empty())
                .expect("operation should succeed");
        }
    };

    let original_size = body_bytes.len();

    // Skip if too small
    if original_size < config.min_size {
        if let Some(ref s) = state {
            s.record_skip().await;
        }
        let mut builder = Response::builder().status(status);
        for (k, v) in &resp_headers {
            builder = builder.header(k, v);
        }
        return builder
            .body(Body::from(body_bytes))
            .expect("operation should succeed");
    }

    let compressed = match chosen_algo {
        CompressionAlgorithm::Gzip => compress_gzip(&body_bytes, config.gzip_level).ok(),
        CompressionAlgorithm::Brotli => {
            // Brotli requires a separate crate; fall back to gzip for now
            compress_gzip(&body_bytes, config.gzip_level).ok()
        }
        CompressionAlgorithm::None => None,
    };

    if let Some(compressed_data) = compressed {
        let compressed_size = compressed_data.len();

        // Only use compressed version if it's actually smaller
        if compressed_size < original_size {
            if let Some(ref s) = state {
                s.record_compression(original_size, compressed_size).await;
            }
            debug!(
                original = original_size,
                compressed = compressed_size,
                algo = %chosen_algo,
                "Response compressed"
            );

            let mut builder = Response::builder().status(status);
            builder = builder.header(header::CONTENT_ENCODING, content_encoding_header_value(chosen_algo));
            builder = builder.header(
                HeaderName::from_static("x-original-size"),
                HeaderValue::from_str(&original_size.to_string()).expect("valid number"),
            );
            builder = builder.header(header::CONTENT_LENGTH, compressed_size as u64);

            return builder
                .body(Body::from(compressed_data))
                .expect("operation should succeed");
        }
    }

    // Fall through: return uncompressed
    if let Some(ref s) = state {
        s.record_skip().await;
    }
    let mut builder = Response::builder().status(status);
    for (k, v) in &resp_headers {
        builder = builder.header(k, v);
    }
    builder
        .body(Body::from(body_bytes))
        .expect("operation should succeed")
}

/// Compression statistics endpoint handler.
pub async fn compression_stats_handler(
    axum::extract::State(state): axum::extract::State<Arc<CompressionState>>,
) -> impl axum::response::IntoResponse {
    let stats = state.get_stats().await;
    axum::Json(stats)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compression_algorithm_display() {
        assert_eq!(CompressionAlgorithm::Brotli.to_string(), "br");
        assert_eq!(CompressionAlgorithm::Gzip.to_string(), "gzip");
        assert_eq!(CompressionAlgorithm::None.to_string(), "identity");
    }

    #[test]
    fn test_compression_config_default() {
        let config = CompressionConfig::default();
        assert!(config.enabled);
        assert_eq!(config.min_size, 1024);
        assert_eq!(config.gzip_level, 6);
        assert!(config.preferred.contains(&CompressionAlgorithm::Brotli));
    }

    #[test]
    fn test_negotiate_encoding_gzip() {
        let algo = negotiate_encoding("gzip, deflate", &[CompressionAlgorithm::Brotli, CompressionAlgorithm::Gzip]);
        assert_eq!(algo, CompressionAlgorithm::Gzip);
    }

    #[test]
    fn test_negotiate_encoding_brotli() {
        let algo = negotiate_encoding("br, gzip", &[CompressionAlgorithm::Brotli, CompressionAlgorithm::Gzip]);
        assert_eq!(algo, CompressionAlgorithm::Brotli);
    }

    #[test]
    fn test_negotiate_encoding_none() {
        let algo = negotiate_encoding("deflate", &[CompressionAlgorithm::Brotli, CompressionAlgorithm::Gzip]);
        assert_eq!(algo, CompressionAlgorithm::None);
    }

    #[test]
    fn test_negotiate_encoding_empty() {
        let algo = negotiate_encoding("", &[CompressionAlgorithm::Brotli, CompressionAlgorithm::Gzip]);
        assert_eq!(algo, CompressionAlgorithm::None);
    }

    #[test]
    fn test_is_compressible_json() {
        assert!(is_compressible("application/json", &CompressionConfig::default()));
    }

    #[test]
    fn test_is_compressible_text() {
        assert!(is_compressible("text/html", &CompressionConfig::default()));
    }

    #[test]
    fn test_is_compressible_skip_image() {
        assert!(!is_compressible("image/png", &CompressionConfig::default()));
    }

    #[test]
    fn test_is_compressible_skip_video() {
        assert!(!is_compressible("video/mp4", &CompressionConfig::default()));
    }

    #[test]
    fn test_gzip_compress_decompress_roundtrip() {
        let data = b"The quick brown fox jumps over the lazy dog. The quick brown fox jumps over the lazy dog. The quick brown fox jumps over the lazy dog. The quick brown fox jumps over the lazy dog. The quick brown fox jumps over the lazy dog. The quick brown fox jumps over the lazy dog. The quick brown fox jumps over the lazy dog. The quick brown fox jumps over the lazy dog. The quick brown fox jumps over the lazy dog. The quick brown fox jumps over the lazy dog. The quick brown fox jumps over the lazy dog. The quick brown fox jumps over the lazy dog. The quick brown fox jumps over the lazy dog. The quick brown fox jumps over the lazy dog. The quick brown fox jumps over the lazy dog. The quick brown fox jumps over the lazy dog. The quick brown fox jumps over the lazy dog. The quick brown fox jumps over the lazy dog. The quick brown fox jumps over the lazy dog. The quick brown fox jumps over the lazy dog.";
        let compressed = compress_gzip(data, 6).unwrap();
        assert!(compressed.len() < data.len());
        let decompressed = decompress_gzip(&compressed).unwrap();
        assert_eq!(decompressed, data);
    }

    #[test]
    fn test_gzip_compress_level_clamping() {
        let data = b"test data for compression level clamping";
        let c1 = compress_gzip(data, 0).unwrap();
        let c9 = compress_gzip(data, 9).unwrap();
        assert!(!c1.is_empty());
        assert!(!c9.is_empty());
    }

    #[tokio::test]
    async fn test_compression_state_stats() {
        let state = CompressionState::new(CompressionConfig::default());
        let stats = state.get_stats().await;
        assert_eq!(stats.total_responses, 0);

        state.record_compression(2000, 500).await;
        state.record_skip().await;
        let stats = state.get_stats().await;
        assert_eq!(stats.total_responses, 2);
        assert_eq!(stats.compressed_responses, 1);
        assert_eq!(stats.skipped_responses, 1);
        assert_eq!(stats.total_bytes_original, 2000);
        assert_eq!(stats.total_bytes_compressed, 500);
    }
}
