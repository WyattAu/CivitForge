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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntryV2Response {
    pub id: String,
    pub key: String,
    pub path: String,
    pub size_bytes: i64,
    pub hit_count: i32,
    pub last_hit_at: Option<String>,
    pub expires_at: String,
    pub created_at: String,
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

#[derive(Debug, Serialize)]
pub struct CacheStats {
    pub total_entries: i64,
    pub total_size_bytes: i64,
    pub total_hits: i64,
    pub average_hit_count: f64,
    pub expired_entries: i64,
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

type CacheV2Row = (
    Uuid,
    String,
    String,
    i64,
    i32,
    Option<chrono::DateTime<Utc>>,
    chrono::DateTime<Utc>,
    chrono::DateTime<Utc>,
);

/// List caches v2 for a repo.
pub async fn list_caches_v2(
    pool: &sqlx::PgPool,
    repo_id: Uuid,
    limit: i64,
    offset: i64,
    prefix: Option<&str>,
) -> std::result::Result<Vec<CacheEntryV2Response>, sqlx::Error> {
    let rows: Vec<CacheV2Row> = if let Some(pfx) = prefix {
        sqlx::query_as(
            "SELECT id, key, path, size_bytes, hit_count, last_hit_at, expires_at, created_at FROM pipeline_caches_v2
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
            "SELECT id, key, path, size_bytes, hit_count, last_hit_at, expires_at, created_at FROM pipeline_caches_v2
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
            |(id, key, path, size_bytes, hit_count, last_hit_at, expires_at, created_at)| {
                CacheEntryV2Response {
                    id: id.to_string(),
                    key,
                    path,
                    size_bytes,
                    hit_count,
                    last_hit_at: last_hit_at.map(|t| t.to_rfc3339()),
                    expires_at: expires_at.to_rfc3339(),
                    created_at: created_at.to_rfc3339(),
                }
            },
        )
        .collect())
}

/// Get cache hit statistics for a repo.
pub async fn get_cache_stats(
    pool: &sqlx::PgPool,
    repo_id: Uuid,
) -> std::result::Result<CacheStats, sqlx::Error> {
    let row: (Option<i64>, Option<i64>, Option<i64>, Option<f64>, Option<i64>) = sqlx::query_as(
        "SELECT
            COUNT(*) as total_entries,
            COALESCE(SUM(size_bytes), 0) as total_size_bytes,
            COALESCE(SUM(hit_count), 0) as total_hits,
            COALESCE(AVG(hit_count), 0) as average_hit_count,
            COUNT(*) FILTER (WHERE expires_at < NOW()) as expired_entries
         FROM pipeline_caches_v2 WHERE repo_id = $1",
    )
    .bind(repo_id)
    .fetch_one(pool)
    .await?;

    Ok(CacheStats {
        total_entries: row.0.unwrap_or(0),
        total_size_bytes: row.1.unwrap_or(0),
        total_hits: row.2.unwrap_or(0),
        average_hit_count: row.3.unwrap_or(0.0),
        expired_entries: row.4.unwrap_or(0),
    })
}

/// Record a cache hit.
pub async fn record_cache_hit(
    pool: &sqlx::PgPool,
    repo_id: Uuid,
    key: &str,
) -> std::result::Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE pipeline_caches_v2 SET hit_count = hit_count + 1, last_hit_at = NOW()
         WHERE repo_id = $1 AND key = $2",
    )
    .bind(repo_id)
    .bind(key)
    .execute(pool)
    .await?;
    Ok(())
}

/// Invalidate (delete) expired caches for a repo.
pub async fn invalidate_expired_caches(
    pool: &sqlx::PgPool,
    repo_id: Uuid,
) -> std::result::Result<i64, sqlx::Error> {
    let result = sqlx::query(
        "DELETE FROM pipeline_caches_v2 WHERE repo_id = $1 AND expires_at < NOW()",
    )
    .bind(repo_id)
    .execute(pool)
    .await?;
    Ok(result.rows_affected() as i64)
}

/// Invalidate (delete) a specific cache by key.
pub async fn invalidate_cache(
    pool: &sqlx::PgPool,
    repo_id: Uuid,
    key: &str,
) -> std::result::Result<bool, sqlx::Error> {
    let result = sqlx::query("DELETE FROM pipeline_caches_v2 WHERE repo_id = $1 AND key = $2")
        .bind(repo_id)
        .bind(key)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

// ---------------------------------------------------------------------------
// Cache Strategies
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStrategyResponse {
    pub id: String,
    pub repo_id: String,
    pub name: String,
    pub strategy_type: String,
    pub config: serde_json::Value,
    pub enabled: bool,
    pub created_at: String,
}

#[derive(Debug, sqlx::FromRow)]
pub struct CacheStrategyRow {
    pub id: Uuid,
    pub repo_id: Uuid,
    pub name: String,
    pub strategy_type: String,
    pub config: serde_json::Value,
    pub enabled: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl From<CacheStrategyRow> for CacheStrategyResponse {
    fn from(r: CacheStrategyRow) -> Self {
        Self {
            id: r.id.to_string(),
            repo_id: r.repo_id.to_string(),
            name: r.name,
            strategy_type: r.strategy_type,
            config: r.config,
            enabled: r.enabled,
            created_at: r.created_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheAnalyticsResponse {
    pub id: String,
    pub cache_id: String,
    pub hit_count: i32,
    pub miss_count: i32,
    pub size_bytes: i64,
    pub last_accessed_at: Option<String>,
    pub created_at: String,
}

#[derive(Debug, sqlx::FromRow)]
pub struct CacheAnalyticsRow {
    pub id: Uuid,
    pub cache_id: Uuid,
    pub hit_count: i32,
    pub miss_count: i32,
    pub size_bytes: i64,
    pub last_accessed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl From<CacheAnalyticsRow> for CacheAnalyticsResponse {
    fn from(r: CacheAnalyticsRow) -> Self {
        Self {
            id: r.id.to_string(),
            cache_id: r.cache_id.to_string(),
            hit_count: r.hit_count,
            miss_count: r.miss_count,
            size_bytes: r.size_bytes,
            last_accessed_at: r.last_accessed_at.map(|t| t.to_rfc3339()),
            created_at: r.created_at.to_rfc3339(),
        }
    }
}

/// Create a cache strategy.
pub async fn create_cache_strategy(
    pool: &sqlx::PgPool,
    repo_id: Uuid,
    name: &str,
    strategy_type: &str,
    config: &serde_json::Value,
) -> std::result::Result<CacheStrategyResponse, sqlx::Error> {
    sqlx::query_as::<_, CacheStrategyRow>(
        "INSERT INTO cache_strategies (repo_id, name, strategy_type, config) \
         VALUES ($1, $2, $3, $4) \
         RETURNING *",
    )
    .bind(repo_id)
    .bind(name)
    .bind(strategy_type)
    .bind(config)
    .fetch_one(pool)
    .await
    .map(|r| r.into())
}

/// List cache strategies for a repo.
pub async fn list_cache_strategies(
    pool: &sqlx::PgPool,
    repo_id: Uuid,
) -> std::result::Result<Vec<CacheStrategyResponse>, sqlx::Error> {
    sqlx::query_as::<_, CacheStrategyRow>(
        "SELECT * FROM cache_strategies WHERE repo_id = $1 ORDER BY created_at DESC",
    )
    .bind(repo_id)
    .fetch_all(pool)
    .await
    .map(|rows| rows.into_iter().map(|r| r.into()).collect())
}

/// Get a cache strategy by ID.
pub async fn get_cache_strategy(
    pool: &sqlx::PgPool,
    strategy_id: Uuid,
) -> std::result::Result<Option<CacheStrategyResponse>, sqlx::Error> {
    sqlx::query_as::<_, CacheStrategyRow>(
        "SELECT * FROM cache_strategies WHERE id = $1",
    )
    .bind(strategy_id)
    .fetch_optional(pool)
    .await
    .map(|r| r.map(|r| r.into()))
}

/// Update a cache strategy.
pub async fn update_cache_strategy(
    pool: &sqlx::PgPool,
    strategy_id: Uuid,
    name: Option<&str>,
    strategy_type: Option<&str>,
    config: Option<&serde_json::Value>,
    enabled: Option<bool>,
) -> std::result::Result<CacheStrategyResponse, sqlx::Error> {
    sqlx::query_as::<_, CacheStrategyRow>(
        "UPDATE cache_strategies \
         SET name = COALESCE($2, name), \
             strategy_type = COALESCE($3, strategy_type), \
             config = COALESCE($4, config), \
             enabled = COALESCE($5, enabled) \
         WHERE id = $1 \
         RETURNING *",
    )
    .bind(strategy_id)
    .bind(name)
    .bind(strategy_type)
    .bind(config)
    .bind(enabled)
    .fetch_one(pool)
    .await
    .map(|r| r.into())
}

/// Delete a cache strategy.
pub async fn delete_cache_strategy(
    pool: &sqlx::PgPool,
    strategy_id: Uuid,
) -> std::result::Result<bool, sqlx::Error> {
    let result = sqlx::query("DELETE FROM cache_strategies WHERE id = $1")
        .bind(strategy_id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

/// Record a cache hit in analytics.
pub async fn record_cache_hit_analytics(
    pool: &sqlx::PgPool,
    cache_id: Uuid,
    size_bytes: i64,
) -> std::result::Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO cache_analytics (cache_id, hit_count, size_bytes, last_accessed_at) \
         VALUES ($1, 1, $2, NOW()) \
         ON CONFLICT (cache_id) DO UPDATE \
         SET hit_count = cache_analytics.hit_count + 1, \
             last_accessed_at = NOW()",
    )
    .bind(cache_id)
    .bind(size_bytes)
    .execute(pool)
    .await?;
    Ok(())
}

/// Record a cache miss in analytics.
pub async fn record_cache_miss_analytics(
    pool: &sqlx::PgPool,
    cache_id: Uuid,
    size_bytes: i64,
) -> std::result::Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO cache_analytics (cache_id, miss_count, size_bytes) \
         VALUES ($1, 1, $2) \
         ON CONFLICT (cache_id) DO UPDATE \
         SET miss_count = cache_analytics.miss_count + 1",
    )
    .bind(cache_id)
    .bind(size_bytes)
    .execute(pool)
    .await?;
    Ok(())
}

/// Get analytics for a specific cache.
pub async fn get_cache_analytics(
    pool: &sqlx::PgPool,
    cache_id: Uuid,
) -> std::result::Result<Option<CacheAnalyticsResponse>, sqlx::Error> {
    sqlx::query_as::<_, CacheAnalyticsRow>(
        "SELECT * FROM cache_analytics WHERE cache_id = $1",
    )
    .bind(cache_id)
    .fetch_optional(pool)
    .await
    .map(|r| r.map(|r| r.into()))
}

/// Get cache optimization report (hit rates, sizes, recommendations).
pub async fn get_cache_optimization_report(
    pool: &sqlx::PgPool,
    repo_id: Uuid,
) -> std::result::Result<serde_json::Value, sqlx::Error> {
    let caches: Vec<(Uuid, String, i64, i32)> = sqlx::query_as(
        "SELECT id, key, size_bytes, hit_count FROM pipeline_caches_v2 WHERE repo_id = $1",
    )
    .bind(repo_id)
    .fetch_all(pool)
    .await?;

    let total_size: i64 = caches.iter().map(|c| c.2).sum();
    let total_hits: i32 = caches.iter().map(|c| c.3).sum();

    let mut recommendations = Vec::new();
    for (id, key, size, hits) in &caches {
        if *hits == 0 {
            recommendations.push(serde_json::json!({
                "cache_id": id.to_string(),
                "key": key,
                "recommendation": "unused_cache",
                "message": "Cache has never been hit; consider removing it"
            }));
        } else if *size > 100_000_000 {
            recommendations.push(serde_json::json!({
                "cache_id": id.to_string(),
                "key": key,
                "recommendation": "large_cache",
                "message": format!("Cache is large ({} bytes); consider compression or pruning", size)
            }));
        }
    }

    Ok(serde_json::json!({
        "repo_id": repo_id.to_string(),
        "total_entries": caches.len(),
        "total_size_bytes": total_size,
        "total_hits": total_hits,
        "average_hit_rate": if caches.is_empty() { 0.0 } else { total_hits as f64 / caches.len() as f64 },
        "recommendations": recommendations
    }))
}

/// Get cache cost analysis (estimated storage costs).
pub async fn get_cache_cost_analysis(
    pool: &sqlx::PgPool,
    repo_id: Uuid,
    cost_per_gb: f64,
) -> std::result::Result<serde_json::Value, sqlx::Error> {
    let row: (Option<i64>, Option<i64>, Option<i64>) = sqlx::query_as(
        "SELECT \
            COALESCE(SUM(size_bytes), 0), \
            COALESCE(SUM(hit_count), 0), \
            COUNT(*) \
         FROM pipeline_caches_v2 WHERE repo_id = $1",
    )
    .bind(repo_id)
    .fetch_one(pool)
    .await?;

    let total_size_bytes = row.0.unwrap_or(0);
    let total_hits = row.1.unwrap_or(0);
    let entry_count = row.2.unwrap_or(0);
    let total_size_gb = total_size_bytes as f64 / (1024.0 * 1024.0 * 1024.0);
    let estimated_cost = total_size_gb * cost_per_gb;
    let cost_per_hit = if total_hits > 0 {
        estimated_cost / total_hits as f64
    } else {
        0.0
    };

    Ok(serde_json::json!({
        "repo_id": repo_id.to_string(),
        "total_entries": entry_count,
        "total_size_bytes": total_size_bytes,
        "total_size_gb": total_size_gb,
        "total_hits": total_hits,
        "cost_per_gb": cost_per_gb,
        "estimated_monthly_cost": estimated_cost,
        "cost_per_hit": cost_per_hit
    }))
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
