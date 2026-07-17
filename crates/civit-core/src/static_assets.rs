#![forbid(unsafe_code)]

//! Static asset optimization with content-hash fingerprinting and cache headers.
//!
//! Provides asset fingerprinting (content hash in filenames), immutable cache headers,
//! ETag support, and compression hints for static files.

use axum::{
    body::Body,
    extract::Request,
    http::{HeaderMap, HeaderName, HeaderValue, StatusCode, header},
    middleware::Next,
    response::Response,
};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Map of original filename -> fingerprinted filename.
#[derive(Debug, Clone, Default)]
pub struct AssetFingerprintMap {
    inner: Arc<RwLock<HashMap<String, String>>>,
}

impl AssetFingerprintMap {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Compute a content hash for a file and register the fingerprinted name.
    pub async fn fingerprint_file(&self, original: &str, content: &[u8]) -> String {
        let hash = compute_content_hash(content);
        let ext = Path::new(original)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");
        let stem = Path::new(original)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or(original);
        let fingerprinted = if ext.is_empty() {
            format!("{stem}-{hash}")
        } else {
            format!("{stem}-{hash}.{ext}")
        };
        self.inner
            .write()
            .await
            .insert(original.to_string(), fingerprinted.clone());
        fingerprinted
    }

    /// Look up the fingerprinted filename for an original name.
    pub async fn resolve(&self, original: &str) -> Option<String> {
        self.inner.read().await.get(original).cloned()
    }

    /// Build a full fingerprint map from a directory of static assets.
    pub async fn build_from_dir(&self, dir: &Path) -> std::io::Result<usize> {
        let mut count = 0usize;
        if !dir.is_dir() {
            return Ok(count);
        }
        let mut entries = tokio::fs::read_dir(dir).await?;
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.is_file() {
                let content = tokio::fs::read(&path).await?;
                let name = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("");
                if !name.is_empty() {
                    self.fingerprint_file(name, &content).await;
                    count += 1;
                }
            }
        }
        Ok(count)
    }
}

/// Compute a SHA-256 hex digest of the given bytes.
pub fn compute_content_hash(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    format!("{:x}", hasher.finalize())
}

/// Compute an ETag value from content hash (weak validator).
pub fn compute_etag(data: &[u8]) -> String {
    let hash = compute_content_hash(data);
    format!("W/\"{hash}\"")
}

/// Build cache-control header value for immutable static assets.
pub fn immutable_cache_control() -> &'static str {
    "public, max-age=31536000, immutable"
}

/// Build cache-control header value for HTML documents (short TTL, revalidate).
pub fn html_cache_control() -> &'static str {
    "public, max-age=0, must-revalidate"
}

/// Determine the appropriate cache-control header based on file extension.
pub fn cache_control_for_path(path: &str) -> &'static str {
    match Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
    {
        "js" | "css" | "woff2" | "woff" | "ttf" | "eot" | "svg" | "png" | "jpg" | "jpeg"
        | "gif" | "webp" | "avif" | "ico" | "map" => immutable_cache_control(),
        "html" | "htm" => html_cache_control(),
        _ => "public, max-age=3600, must-revalidate",
    }
}

/// Check if a path looks like a fingerprinted asset (contains a hex hash segment).
pub fn is_fingerprinted(path: &str) -> bool {
    let name = Path::new(path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("");
    if let Some(dash_pos) = name.rfind('-') {
        let hash_part = &name[dash_pos + 1..];
        if let Some(dot_pos) = hash_part.rfind('.') {
            let candidate = &hash_part[..dot_pos];
            return candidate.len() == 64 && candidate.chars().all(|c| c.is_ascii_hexdigit());
        }
        return hash_part.len() == 64 && hash_part.chars().all(|c| c.is_ascii_hexdigit());
    }
    false
}

/// State for the static asset middleware.
#[derive(Clone)]
pub struct StaticAssetState {
    pub fingerprints: AssetFingerprintMap,
    pub root_dir: PathBuf,
}

/// Middleware that adds cache headers, ETag, and content-encoding hints for static assets.
pub async fn static_asset_middleware(req: Request, next: Next) -> Response {
    let path = req.uri().path().to_string();

    if path.starts_with("/api/") {
        return next.run(req).await;
    }

    let mut response = next.run(req).await;

    if response.status().is_success() {
        let headers = response.headers_mut();
        let cache_ctl = cache_control_for_path(&path);
        let val = HeaderValue::from_static(cache_ctl);
        headers.insert(header::CACHE_CONTROL, val);

        if is_fingerprinted(&path) {
            headers.insert(
                HeaderName::from_static("x-immutable"),
                HeaderValue::from_static("true"),
            );
        }

        headers.insert(header::VARY, HeaderValue::from_static("Accept-Encoding"));
    }

    response
}

/// Compute ETag from response body bytes (call after collecting body).
pub fn add_etag_header(headers: &mut HeaderMap, body: &[u8]) {
    let etag = compute_etag(body);
    let val = HeaderValue::from_str(&etag).expect("etag is valid header value");
    headers.insert(header::ETAG, val);
}

/// Check if the request's `If-None-Match` matches the given ETag.
pub fn matches_not_modified(req_headers: &HeaderMap, etag: &str) -> bool {
    req_headers
        .get(header::IF_NONE_MATCH)
        .and_then(|v| v.to_str().ok())
        .map(|v| v == etag)
        .unwrap_or(false)
}

/// Build a 304 Not Modified response with standard headers.
pub fn not_modified_response() -> Response {
    Response::builder()
        .status(StatusCode::NOT_MODIFIED)
        .body(Body::empty())
        .expect("operation should succeed")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_content_hash_deterministic() {
        let data = b"hello world";
        let h1 = compute_content_hash(data);
        let h2 = compute_content_hash(data);
        assert_eq!(h1, h2);
        assert_eq!(h1.len(), 64);
    }

    #[test]
    fn test_compute_content_hash_different_inputs() {
        let h1 = compute_content_hash(b"hello");
        let h2 = compute_content_hash(b"world");
        assert_ne!(h1, h2);
    }

    #[test]
    fn test_compute_etag_format() {
        let etag = compute_etag(b"test");
        assert!(etag.starts_with("W/\""));
        assert!(etag.ends_with('"'));
    }

    #[test]
    fn test_immutable_cache_control() {
        assert_eq!(immutable_cache_control(), "public, max-age=31536000, immutable");
    }

    #[test]
    fn test_html_cache_control() {
        assert_eq!(html_cache_control(), "public, max-age=0, must-revalidate");
    }

    #[test]
    fn test_cache_control_for_path_js() {
        assert_eq!(cache_control_for_path("/assets/app.js"), immutable_cache_control());
    }

    #[test]
    fn test_cache_control_for_path_css() {
        assert_eq!(cache_control_for_path("/styles/main.css"), immutable_cache_control());
    }

    #[test]
    fn test_cache_control_for_path_html() {
        assert_eq!(cache_control_for_path("/index.html"), html_cache_control());
    }

    #[test]
    fn test_cache_control_for_path_unknown() {
        assert_eq!(
            cache_control_for_path("/some/random.txt"),
            "public, max-age=3600, must-revalidate"
        );
    }

    #[test]
    fn test_is_fingerprinted_true() {
        assert!(is_fingerprinted(
            "/assets/app-a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2.js"
        ));
    }

    #[test]
    fn test_is_fingerprinted_false_plain() {
        assert!(!is_fingerprinted("/assets/app.js"));
    }

    #[test]
    fn test_is_fingerprinted_false_short_hash() {
        assert!(!is_fingerprinted("/assets/app-abc123.js"));
    }

    #[test]
    fn test_matches_not_modified() {
        let mut headers = HeaderMap::new();
        headers.insert(
            header::IF_NONE_MATCH,
            HeaderValue::from_static("W/\"abc123\""),
        );
        assert!(matches_not_modified(&headers, "W/\"abc123\""));
        assert!(!matches_not_modified(&headers, "W/\"def456\""));
    }

    #[test]
    fn test_matches_not_modified_missing_header() {
        let headers = HeaderMap::new();
        assert!(!matches_not_modified(&headers, "W/\"abc123\""));
    }

    #[tokio::test]
    async fn test_asset_fingerprint_map_set_and_resolve() {
        let map = AssetFingerprintMap::new();
        let fp = map.fingerprint_file("app.js", b"content").await;
        assert!(fp.starts_with("app-"));
        assert!(fp.ends_with(".js"));

        let resolved = map.resolve("app.js").await;
        assert_eq!(resolved, Some(fp));
    }

    #[tokio::test]
    async fn test_asset_fingerprint_map_resolve_missing() {
        let map = AssetFingerprintMap::new();
        assert!(map.resolve("missing.js").await.is_none());
    }

    #[tokio::test]
    async fn test_asset_fingerprint_map_build_from_dir() {
        let dir = tempfile::tempdir().unwrap();
        tokio::fs::write(dir.path().join("test.txt"), b"hello").await.unwrap();
        let map = AssetFingerprintMap::new();
        let count = map.build_from_dir(dir.path()).await.unwrap();
        assert_eq!(count, 1);
        let resolved = map.resolve("test.txt").await;
        assert!(resolved.is_some());
    }

    #[test]
    fn test_not_modified_response_status() {
        let resp = not_modified_response();
        assert_eq!(resp.status(), StatusCode::NOT_MODIFIED);
    }

    #[test]
    fn test_add_etag_header() {
        let mut headers = HeaderMap::new();
        add_etag_header(&mut headers, b"content");
        assert!(headers.contains_key(header::ETAG));
    }
}
