//! Cache Warming Rules and Logs types and logic.

#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheWarmingRuleResponse {
    pub id: String,
    pub repo_id: String,
    pub name: String,
    pub trigger_type: String,
    pub cache_keys: Vec<String>,
    pub enabled: bool,
    pub created_at: String,
}

#[derive(Debug, sqlx::FromRow)]
pub struct CacheWarmingRuleRow {
    pub id: Uuid,
    pub repo_id: Uuid,
    pub name: String,
    pub trigger_type: String,
    pub cache_keys: Vec<String>,
    pub enabled: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl From<CacheWarmingRuleRow> for CacheWarmingRuleResponse {
    fn from(r: CacheWarmingRuleRow) -> Self {
        Self {
            id: r.id.to_string(),
            repo_id: r.repo_id.to_string(),
            name: r.name,
            trigger_type: r.trigger_type,
            cache_keys: r.cache_keys,
            enabled: r.enabled,
            created_at: r.created_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateCacheWarmingRuleRequest {
    pub name: String,
    pub trigger_type: String,
    #[serde(default)]
    pub cache_keys: Vec<String>,
    #[serde(default = "default_true")]
    pub enabled: bool,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Deserialize)]
pub struct UpdateCacheWarmingRuleRequest {
    pub name: Option<String>,
    pub trigger_type: Option<String>,
    pub cache_keys: Option<Vec<String>>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheWarmingLogResponse {
    pub id: String,
    pub rule_id: String,
    pub cache_keys: Vec<String>,
    pub status: String,
    pub duration_ms: i32,
    pub error_message: Option<String>,
    pub created_at: String,
}

#[derive(Debug, sqlx::FromRow)]
pub struct CacheWarmingLogRow {
    pub id: Uuid,
    pub rule_id: Uuid,
    pub cache_keys: Vec<String>,
    pub status: String,
    pub duration_ms: i32,
    pub error_message: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl From<CacheWarmingLogRow> for CacheWarmingLogResponse {
    fn from(r: CacheWarmingLogRow) -> Self {
        Self {
            id: r.id.to_string(),
            rule_id: r.rule_id.to_string(),
            cache_keys: r.cache_keys,
            status: r.status,
            duration_ms: r.duration_ms,
            error_message: r.error_message,
            created_at: r.created_at.to_rfc3339(),
        }
    }
}

// ---------------------------------------------------------------------------
// DB operations
// ---------------------------------------------------------------------------

/// Create a new cache warming rule.
pub async fn create_warming_rule(
    pool: &sqlx::PgPool,
    repo_id: Uuid,
    name: &str,
    trigger_type: &str,
    cache_keys: &[String],
    enabled: bool,
) -> std::result::Result<CacheWarmingRuleResponse, sqlx::Error> {
    sqlx::query_as::<_, CacheWarmingRuleRow>(
        "INSERT INTO cache_warming_rules (repo_id, name, trigger_type, cache_keys, enabled) \
         VALUES ($1, $2, $3, $4, $5) \
         RETURNING *",
    )
    .bind(repo_id)
    .bind(name)
    .bind(trigger_type)
    .bind(cache_keys)
    .bind(enabled)
    .fetch_one(pool)
    .await
    .map(|r| r.into())
}

/// Get a warming rule by ID.
pub async fn get_warming_rule(
    pool: &sqlx::PgPool,
    rule_id: Uuid,
) -> std::result::Result<Option<CacheWarmingRuleResponse>, sqlx::Error> {
    sqlx::query_as::<_, CacheWarmingRuleRow>(
        "SELECT * FROM cache_warming_rules WHERE id = $1",
    )
    .bind(rule_id)
    .fetch_optional(pool)
    .await
    .map(|r| r.map(|r| r.into()))
}

/// Update a warming rule.
pub async fn update_warming_rule(
    pool: &sqlx::PgPool,
    rule_id: Uuid,
    name: Option<&str>,
    trigger_type: Option<&str>,
    cache_keys: Option<&[String]>,
    enabled: Option<bool>,
) -> std::result::Result<CacheWarmingRuleResponse, sqlx::Error> {
    sqlx::query_as::<_, CacheWarmingRuleRow>(
        "UPDATE cache_warming_rules \
         SET name = COALESCE($2, name), \
             trigger_type = COALESCE($3, trigger_type), \
             cache_keys = COALESCE($4, cache_keys), \
             enabled = COALESCE($5, enabled) \
         WHERE id = $1 \
         RETURNING *",
    )
    .bind(rule_id)
    .bind(name)
    .bind(trigger_type)
    .bind(cache_keys)
    .bind(enabled)
    .fetch_one(pool)
    .await
    .map(|r| r.into())
}

/// Delete a warming rule.
pub async fn delete_warming_rule(
    pool: &sqlx::PgPool,
    rule_id: Uuid,
) -> std::result::Result<bool, sqlx::Error> {
    let result = sqlx::query("DELETE FROM cache_warming_rules WHERE id = $1")
        .bind(rule_id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

/// List warming rules for a repository.
pub async fn list_warming_rules(
    pool: &sqlx::PgPool,
    repo_id: Uuid,
) -> std::result::Result<Vec<CacheWarmingRuleResponse>, sqlx::Error> {
    sqlx::query_as::<_, CacheWarmingRuleRow>(
        "SELECT * FROM cache_warming_rules WHERE repo_id = $1 ORDER BY name ASC",
    )
    .bind(repo_id)
    .fetch_all(pool)
    .await
    .map(|rows| rows.into_iter().map(|r| r.into()).collect())
}

/// Get rules by trigger type.
pub async fn get_rules_by_trigger(
    pool: &sqlx::PgPool,
    repo_id: Uuid,
    trigger_type: &str,
) -> std::result::Result<Vec<CacheWarmingRuleResponse>, sqlx::Error> {
    sqlx::query_as::<_, CacheWarmingRuleRow>(
        "SELECT * FROM cache_warming_rules \
         WHERE repo_id = $1 AND trigger_type = $2 AND enabled = true",
    )
    .bind(repo_id)
    .bind(trigger_type)
    .fetch_all(pool)
    .await
    .map(|rows| rows.into_iter().map(|r| r.into()).collect())
}

/// Record a warming log entry.
pub async fn record_warming_log(
    pool: &sqlx::PgPool,
    rule_id: Uuid,
    cache_keys: &[String],
    status: &str,
    duration_ms: i32,
    error_message: Option<&str>,
) -> std::result::Result<CacheWarmingLogResponse, sqlx::Error> {
    sqlx::query_as::<_, CacheWarmingLogRow>(
        "INSERT INTO cache_warming_logs (rule_id, cache_keys, status, duration_ms, error_message) \
         VALUES ($1, $2, $3, $4, $5) \
         RETURNING *",
    )
    .bind(rule_id)
    .bind(cache_keys)
    .bind(status)
    .bind(duration_ms)
    .bind(error_message)
    .fetch_one(pool)
    .await
    .map(|r| r.into())
}

/// List warming logs for a rule.
pub async fn list_warming_logs(
    pool: &sqlx::PgPool,
    rule_id: Uuid,
    limit: i64,
    offset: i64,
) -> std::result::Result<Vec<CacheWarmingLogResponse>, sqlx::Error> {
    sqlx::query_as::<_, CacheWarmingLogRow>(
        "SELECT * FROM cache_warming_logs \
         WHERE rule_id = $1 \
         ORDER BY created_at DESC \
         LIMIT $2 OFFSET $3",
    )
    .bind(rule_id)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await
    .map(|rows| rows.into_iter().map(|r| r.into()).collect())
}

/// Get warming statistics for a repository.
pub async fn get_warming_stats(
    pool: &sqlx::PgPool,
    repo_id: Uuid,
) -> std::result::Result<serde_json::Value, sqlx::Error> {
    let rule_count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM cache_warming_rules WHERE repo_id = $1",
    )
    .bind(repo_id)
    .fetch_one(pool)
    .await?;

    let log_stats: (i64, i64, f64) = sqlx::query_as(
        "SELECT \
            COUNT(*), \
            COUNT(*) FILTER (WHERE status = 'success'), \
            COALESCE(AVG(duration_ms), 0) \
         FROM cache_warming_logs l \
         JOIN cache_warming_rules r ON l.rule_id = r.id \
         WHERE r.repo_id = $1",
    )
    .bind(repo_id)
    .fetch_one(pool)
    .await?;

    Ok(serde_json::json!({
        "repo_id": repo_id.to_string(),
        "total_rules": rule_count.0,
        "total_executions": log_stats.0,
        "successful_executions": log_stats.1,
        "average_duration_ms": log_stats.2
    }))
}

/// Preheat cache for specific keys.
pub async fn preheat_cache(
    pool: &sqlx::PgPool,
    repo_id: Uuid,
    cache_keys: &[String],
) -> std::result::Result<serde_json::Value, sqlx::Error> {
    let mut warmed = 0;
    let mut skipped = 0;

    for key in cache_keys {
        let result = sqlx::query(
            "INSERT INTO pipeline_caches (repo_id, key, path, size_bytes, created_at, expires_at) \
             VALUES ($1, $2, $2, 0, NOW(), NOW() + INTERVAL '1 hour') \
             ON CONFLICT (repo_id, key) DO NOTHING",
        )
        .bind(repo_id)
        .bind(key)
        .execute(pool)
        .await?;

        if result.rows_affected() > 0 {
            warmed += 1;
        } else {
            skipped += 1;
        }
    }

    Ok(serde_json::json!({
        "warmed": warmed,
        "skipped": skipped,
        "total": cache_keys.len()
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_warming_rule_response_serialize() {
        let resp = CacheWarmingRuleResponse {
            id: "00000000-0000-0000-0000-000000000001".to_string(),
            repo_id: "00000000-0000-0000-0000-000000000002".to_string(),
            name: "pre-build warm".to_string(),
            trigger_type: "push".to_string(),
            cache_keys: vec!["node_modules".to_string(), "target".to_string()],
            enabled: true,
            created_at: "2025-01-01T00:00:00+00:00".to_string(),
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("pre-build warm"));
    }

    #[test]
    fn test_warming_log_response_serialize() {
        let resp = CacheWarmingLogResponse {
            id: "00000000-0000-0000-0000-000000000001".to_string(),
            rule_id: "00000000-0000-0000-0000-000000000002".to_string(),
            cache_keys: vec!["node_modules".to_string()],
            status: "success".to_string(),
            duration_ms: 1234,
            error_message: None,
            created_at: "2025-01-01T00:00:00+00:00".to_string(),
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("success"));
    }

    #[test]
    fn test_create_warming_rule_request() {
        let json = r#"{"name": "pre-deploy", "trigger_type": "deploy", "cache_keys": ["dist", "node_modules"]}"#;
        let req: CreateCacheWarmingRuleRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.name, "pre-deploy");
        assert_eq!(req.trigger_type, "deploy");
        assert!(req.enabled);
    }

    #[test]
    fn test_update_warming_rule_request() {
        let json = r#"{"name": "updated", "enabled": false}"#;
        let req: UpdateCacheWarmingRuleRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.name.as_deref(), Some("updated"));
        assert_eq!(req.enabled, Some(false));
    }
}
