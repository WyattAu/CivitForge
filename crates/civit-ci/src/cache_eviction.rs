//! Cache Eviction Policies and Logs types and logic.

#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEvictionPolicyResponse {
    pub id: String,
    pub repo_id: String,
    pub name: String,
    pub policy_type: String,
    pub config: serde_json::Value,
    pub enabled: bool,
    pub created_at: String,
}

#[derive(Debug, sqlx::FromRow)]
pub struct CacheEvictionPolicyRow {
    pub id: Uuid,
    pub repo_id: Uuid,
    pub name: String,
    pub policy_type: String,
    pub config: serde_json::Value,
    pub enabled: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl From<CacheEvictionPolicyRow> for CacheEvictionPolicyResponse {
    fn from(r: CacheEvictionPolicyRow) -> Self {
        Self {
            id: r.id.to_string(),
            repo_id: r.repo_id.to_string(),
            name: r.name,
            policy_type: r.policy_type,
            config: r.config,
            enabled: r.enabled,
            created_at: r.created_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEvictionLogResponse {
    pub id: String,
    pub policy_id: String,
    pub cache_key: String,
    pub eviction_reason: String,
    pub evicted_at: String,
}

#[derive(Debug, sqlx::FromRow)]
pub struct CacheEvictionLogRow {
    pub id: Uuid,
    pub policy_id: Uuid,
    pub cache_key: String,
    pub eviction_reason: String,
    pub evicted_at: chrono::DateTime<chrono::Utc>,
}

impl From<CacheEvictionLogRow> for CacheEvictionLogResponse {
    fn from(r: CacheEvictionLogRow) -> Self {
        Self {
            id: r.id.to_string(),
            policy_id: r.policy_id.to_string(),
            cache_key: r.cache_key,
            eviction_reason: r.eviction_reason,
            evicted_at: r.evicted_at.to_rfc3339(),
        }
    }
}

/// Create a cache eviction policy.
pub async fn create_eviction_policy(
    pool: &sqlx::PgPool,
    repo_id: Uuid,
    name: &str,
    policy_type: &str,
    config: &serde_json::Value,
) -> std::result::Result<CacheEvictionPolicyResponse, sqlx::Error> {
    sqlx::query_as::<_, CacheEvictionPolicyRow>(
        "INSERT INTO cache_eviction_policies (repo_id, name, policy_type, config) \
         VALUES ($1, $2, $3, $4) \
         RETURNING *",
    )
    .bind(repo_id)
    .bind(name)
    .bind(policy_type)
    .bind(config)
    .fetch_one(pool)
    .await
    .map(|r| r.into())
}

/// List cache eviction policies for a repo.
pub async fn list_eviction_policies(
    pool: &sqlx::PgPool,
    repo_id: Uuid,
) -> std::result::Result<Vec<CacheEvictionPolicyResponse>, sqlx::Error> {
    sqlx::query_as::<_, CacheEvictionPolicyRow>(
        "SELECT * FROM cache_eviction_policies WHERE repo_id = $1 ORDER BY created_at DESC",
    )
    .bind(repo_id)
    .fetch_all(pool)
    .await
    .map(|rows| rows.into_iter().map(|r| r.into()).collect())
}

/// Update a cache eviction policy.
pub async fn update_eviction_policy(
    pool: &sqlx::PgPool,
    policy_id: Uuid,
    name: Option<&str>,
    policy_type: Option<&str>,
    config: Option<&serde_json::Value>,
    enabled: Option<bool>,
) -> std::result::Result<CacheEvictionPolicyResponse, sqlx::Error> {
    sqlx::query_as::<_, CacheEvictionPolicyRow>(
        "UPDATE cache_eviction_policies \
         SET name = COALESCE($2, name), \
             policy_type = COALESCE($3, policy_type), \
             config = COALESCE($4, config), \
             enabled = COALESCE($5, enabled) \
         WHERE id = $1 \
         RETURNING *",
    )
    .bind(policy_id)
    .bind(name)
    .bind(policy_type)
    .bind(config)
    .bind(enabled)
    .fetch_one(pool)
    .await
    .map(|r| r.into())
}

/// Delete a cache eviction policy.
pub async fn delete_eviction_policy(
    pool: &sqlx::PgPool,
    policy_id: Uuid,
) -> std::result::Result<bool, sqlx::Error> {
    let result = sqlx::query("DELETE FROM cache_eviction_policies WHERE id = $1")
        .bind(policy_id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

/// Log a cache eviction.
pub async fn log_eviction(
    pool: &sqlx::PgPool,
    policy_id: Uuid,
    cache_key: &str,
    eviction_reason: &str,
) -> std::result::Result<CacheEvictionLogResponse, sqlx::Error> {
    sqlx::query_as::<_, CacheEvictionLogRow>(
        "INSERT INTO cache_eviction_logs (policy_id, cache_key, eviction_reason) \
         VALUES ($1, $2, $3) \
         RETURNING *",
    )
    .bind(policy_id)
    .bind(cache_key)
    .bind(eviction_reason)
    .fetch_one(pool)
    .await
    .map(|r| r.into())
}

/// List eviction logs for a policy.
pub async fn list_eviction_logs(
    pool: &sqlx::PgPool,
    policy_id: Uuid,
    limit: i64,
    offset: i64,
) -> std::result::Result<Vec<CacheEvictionLogResponse>, sqlx::Error> {
    sqlx::query_as::<_, CacheEvictionLogRow>(
        "SELECT * FROM cache_eviction_logs WHERE policy_id = $1 \
         ORDER BY evicted_at DESC LIMIT $2 OFFSET $3",
    )
    .bind(policy_id)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await
    .map(|rows| rows.into_iter().map(|r| r.into()).collect())
}

/// Get cache eviction stats.
pub async fn get_eviction_stats(
    pool: &sqlx::PgPool,
    repo_id: Uuid,
) -> std::result::Result<serde_json::Value, sqlx::Error> {
    let row: (i64, i64) = sqlx::query_as(
        "SELECT COUNT(*), COUNT(*) FILTER (WHERE evicted_at > NOW() - INTERVAL '24 hours') \
         FROM cache_eviction_logs WHERE policy_id IN \
         (SELECT id FROM cache_eviction_policies WHERE repo_id = $1)",
    )
    .bind(repo_id)
    .fetch_one(pool)
    .await?;

    Ok(serde_json::json!({
        "total_evictions": row.0,
        "evictions_last_24h": row.1,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_eviction_policy_response_serialize() {
        let resp = CacheEvictionPolicyResponse {
            id: "00000000-0000-0000-0000-000000000001".to_string(),
            repo_id: "00000000-0000-0000-0000-000000000002".to_string(),
            name: "lru-eviction".to_string(),
            policy_type: "lru".to_string(),
            config: serde_json::json!({"max_size_mb": 1000}),
            enabled: true,
            created_at: "2025-01-01T00:00:00+00:00".to_string(),
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("lru-eviction"));
    }

    #[test]
    fn test_cache_eviction_log_response_serialize() {
        let resp = CacheEvictionLogResponse {
            id: "00000000-0000-0000-0000-000000000001".to_string(),
            policy_id: "00000000-0000-0000-0000-000000000002".to_string(),
            cache_key: "cargo-deps-v1".to_string(),
            eviction_reason: "lru_eviction".to_string(),
            evicted_at: "2025-01-01T00:00:00+00:00".to_string(),
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("lru_eviction"));
    }
}
