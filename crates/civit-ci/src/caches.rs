//! Pipeline Caches types.

#![forbid(unsafe_code)]

use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntryResponse {
    pub key: String,
    pub path: String,
    pub size_bytes: i64,
    pub created_at: String,
    pub expires_at: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateCacheRequest {
    pub key: String,
    pub path: String,
    #[serde(default)]
    pub size_bytes: i64,
    pub ttl_secs: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct CacheListParams {
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
    pub prefix: Option<String>,
}

pub fn default_limit() -> i64 {
    50
}

type CacheRow = (
    String,
    String,
    i64,
    chrono::DateTime<Utc>,
    Option<chrono::DateTime<Utc>>,
);

pub async fn list_caches_db(
    pool: &sqlx::PgPool,
    repo_id: Uuid,
    limit: i64,
    offset: i64,
    prefix: Option<&str>,
) -> std::result::Result<Vec<CacheEntryResponse>, sqlx::Error> {
    let rows: Vec<CacheRow> = if let Some(pfx) = prefix {
        sqlx::query_as(
            "SELECT key, path, size_bytes, created_at, expires_at FROM pipeline_caches
             WHERE repo_id = $1 AND key LIKE $2 ORDER BY created_at DESC LIMIT $3 OFFSET $4",
        )
        .bind(repo_id)
        .bind(format!("{pfx}%"))
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?
    } else {
        sqlx::query_as(
            "SELECT key, path, size_bytes, created_at, expires_at FROM pipeline_caches
             WHERE repo_id = $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3",
        )
        .bind(repo_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?
    };

    Ok(rows
        .into_iter()
        .map(
            |(key, path, size_bytes, created_at, expires_at)| CacheEntryResponse {
                key,
                path,
                size_bytes,
                created_at: created_at.to_rfc3339(),
                expires_at: expires_at.map(|e| e.to_rfc3339()),
            },
        )
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_entry_response_serialize() {
        let resp = CacheEntryResponse {
            key: "cargo-deps-v1".to_string(),
            path: "target/".to_string(),
            size_bytes: 1024 * 1024,
            created_at: "2025-01-01T00:00:00+00:00".to_string(),
            expires_at: Some("2025-01-08T00:00:00+00:00".to_string()),
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("cargo-deps-v1"));
    }

    #[test]
    fn test_create_cache_request_deserialize() {
        let json = r#"{"key": "deps-v1", "path": "node_modules/", "size_bytes": 2048}"#;
        let req: CreateCacheRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.key, "deps-v1");
        assert_eq!(req.path, "node_modules/");
    }

    #[test]
    fn test_cache_entry_no_expiry() {
        let resp = CacheEntryResponse {
            key: "temp-cache".into(),
            path: "/tmp/".into(),
            size_bytes: 512,
            created_at: "2025-01-01T00:00:00Z".into(),
            expires_at: None,
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("temp-cache"));
    }

    #[test]
    fn test_cache_list_params_defaults() {
        let json = r#"{}"#;
        let params: CacheListParams = serde_json::from_str(json).unwrap();
        assert_eq!(params.limit, 50);
        assert_eq!(params.offset, 0);
        assert!(params.prefix.is_none());
    }

    #[test]
    fn test_cache_list_params_with_prefix() {
        let json = r#"{"limit": 10, "offset": 0, "prefix": "cargo-"}"#;
        let params: CacheListParams = serde_json::from_str(json).unwrap();
        assert_eq!(params.prefix.as_deref(), Some("cargo-"));
    }

    #[test]
    fn test_create_cache_request_no_ttl() {
        let json = r#"{"key": "k", "path": "p"}"#;
        let req: CreateCacheRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.size_bytes, 0);
        assert!(req.ttl_secs.is_none());
    }

    #[test]
    fn test_create_cache_request_with_ttl() {
        let json = r#"{"key": "k", "path": "p", "ttl_secs": 3600}"#;
        let req: CreateCacheRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.ttl_secs, Some(3600));
    }

    #[test]
    fn test_cache_entry_large_size() {
        let resp = CacheEntryResponse {
            key: "big-cache".into(),
            path: "target/".into(),
            size_bytes: 5_000_000_000,
            created_at: "2025-01-01T00:00:00Z".into(),
            expires_at: None,
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("5000000000"));
    }

    #[test]
    fn test_cache_entry_zero_size() {
        let resp = CacheEntryResponse {
            key: "empty".into(),
            path: "empty/".into(),
            size_bytes: 0,
            created_at: "2025-01-01T00:00:00Z".into(),
            expires_at: None,
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("empty"));
    }

    #[test]
    fn test_cache_list_params_negative_offset() {
        let json = r#"{"limit": 10, "offset": -5}"#;
        let params: CacheListParams = serde_json::from_str(json).unwrap();
        assert_eq!(params.offset, -5);
    }

    #[test]
    fn test_cache_list_params_zero_limit() {
        let json = r#"{"limit": 0}"#;
        let params: CacheListParams = serde_json::from_str(json).unwrap();
        assert_eq!(params.limit, 0);
    }

    #[test]
    fn test_create_cache_request_negative_size() {
        let json = r#"{"key": "k", "path": "p", "size_bytes": -100}"#;
        let req: CreateCacheRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.size_bytes, -100);
    }

    #[test]
    fn test_create_cache_request_zero_ttl() {
        let json = r#"{"key": "k", "path": "p", "ttl_secs": 0}"#;
        let req: CreateCacheRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.ttl_secs, Some(0));
    }

    #[test]
    fn test_cache_entry_special_chars_in_key() {
        let resp = CacheEntryResponse {
            key: "key/with/slashes&special=chars".into(),
            path: "/tmp/".into(),
            size_bytes: 100,
            created_at: "2025-01-01T00:00:00Z".into(),
            expires_at: None,
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("key/with/slashes&special=chars"));
    }

    #[test]
    fn test_cache_entry_empty_path() {
        let resp = CacheEntryResponse {
            key: "k".into(),
            path: "".into(),
            size_bytes: 0,
            created_at: "2025-01-01T00:00:00Z".into(),
            expires_at: None,
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("\"path\":\"\""));
    }

    #[test]
    fn test_create_cache_request_special_chars() {
        let json = r#"{"key": "key with spaces!@#", "path": "/path/with spaces"}"#;
        let req: CreateCacheRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.key, "key with spaces!@#");
        assert_eq!(req.path, "/path/with spaces");
    }

    #[test]
    fn test_cache_entry_expires_at_format() {
        let resp = CacheEntryResponse {
            key: "k".into(),
            path: "p".into(),
            size_bytes: 0,
            created_at: "2025-01-01T00:00:00Z".into(),
            expires_at: Some("2025-12-31T23:59:59Z".into()),
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("2025-12-31T23:59:59Z"));
    }

    #[test]
    fn test_cache_entry_negative_size() {
        let resp = CacheEntryResponse {
            key: "k".into(),
            path: "p".into(),
            size_bytes: -1,
            created_at: "2025-01-01T00:00:00Z".into(),
            expires_at: None,
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("\"size_bytes\":-1"));
    }
}
