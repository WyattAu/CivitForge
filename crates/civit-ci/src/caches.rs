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

// ---------------------------------------------------------------------------
// Cache Hit Analysis
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheHitAnalysisResponse {
    pub id: String,
    pub cache_id: String,
    pub period_start: String,
    pub hit_count: i32,
    pub miss_count: i32,
    pub avg_hit_size_bytes: i64,
    pub total_size_bytes: i64,
    pub created_at: String,
}

#[derive(Debug, sqlx::FromRow)]
pub struct CacheHitAnalysisRow {
    pub id: Uuid,
    pub cache_id: Uuid,
    pub period_start: chrono::DateTime<chrono::Utc>,
    pub hit_count: i32,
    pub miss_count: i32,
    pub avg_hit_size_bytes: i64,
    pub total_size_bytes: i64,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl From<CacheHitAnalysisRow> for CacheHitAnalysisResponse {
    fn from(r: CacheHitAnalysisRow) -> Self {
        Self {
            id: r.id.to_string(),
            cache_id: r.cache_id.to_string(),
            period_start: r.period_start.to_rfc3339(),
            hit_count: r.hit_count,
            miss_count: r.miss_count,
            avg_hit_size_bytes: r.avg_hit_size_bytes,
            total_size_bytes: r.total_size_bytes,
            created_at: r.created_at.to_rfc3339(),
        }
    }
}

/// Record a cache hit analysis entry.
pub async fn record_cache_hit_analysis(
    pool: &sqlx::PgPool,
    cache_id: Uuid,
    hit_size_bytes: i64,
) -> std::result::Result<(), sqlx::Error> {
    let period_start = chrono::Utc::now()
        .date_naive()
        .and_hms_opt(0, 0, 0)
        .unwrap()
        .and_utc();

    sqlx::query(
        "INSERT INTO cache_hit_analysis (cache_id, period_start, hit_count, avg_hit_size_bytes, total_size_bytes) \
         VALUES ($1, $2, 1, $3, $3) \
         ON CONFLICT (cache_id, period_start) DO UPDATE \
         SET hit_count = cache_hit_analysis.hit_count + 1, \
             avg_hit_size_bytes = (cache_hit_analysis.avg_hit_size_bytes * cache_hit_analysis.hit_count + $3) / (cache_hit_analysis.hit_count + 1), \
             total_size_bytes = cache_hit_analysis.total_size_bytes + $3",
    )
    .bind(cache_id)
    .bind(period_start)
    .bind(hit_size_bytes)
    .execute(pool)
    .await?;
    Ok(())
}

/// Record a cache miss analysis entry.
pub async fn record_cache_miss_analysis(
    pool: &sqlx::PgPool,
    cache_id: Uuid,
    miss_size_bytes: i64,
) -> std::result::Result<(), sqlx::Error> {
    let period_start = chrono::Utc::now()
        .date_naive()
        .and_hms_opt(0, 0, 0)
        .unwrap()
        .and_utc();

    sqlx::query(
        "INSERT INTO cache_hit_analysis (cache_id, period_start, miss_count, total_size_bytes) \
         VALUES ($1, $2, 1, $3) \
         ON CONFLICT (cache_id, period_start) DO UPDATE \
         SET miss_count = cache_hit_analysis.miss_count + 1, \
             total_size_bytes = cache_hit_analysis.total_size_bytes + $3",
    )
    .bind(cache_id)
    .bind(period_start)
    .bind(miss_size_bytes)
    .execute(pool)
    .await?;
    Ok(())
}

/// Get hit analysis for a cache.
pub async fn get_cache_hit_analysis(
    pool: &sqlx::PgPool,
    cache_id: Uuid,
    limit: i64,
) -> std::result::Result<Vec<CacheHitAnalysisResponse>, sqlx::Error> {
    sqlx::query_as::<_, CacheHitAnalysisRow>(
        "SELECT * FROM cache_hit_analysis WHERE cache_id = $1 \
         ORDER BY period_start DESC LIMIT $2",
    )
    .bind(cache_id)
    .bind(limit)
    .fetch_all(pool)
    .await
    .map(|rows| rows.into_iter().map(|r| r.into()).collect())
}

/// Get aggregate hit analysis for a repo.
pub async fn get_repo_hit_analysis(
    pool: &sqlx::PgPool,
    repo_id: Uuid,
    days: i32,
) -> std::result::Result<serde_json::Value, sqlx::Error> {
    let row: (Option<i64>, Option<i64>, Option<i64>, Option<i64>) = sqlx::query_as(
        "SELECT \
            COALESCE(SUM(cha.hit_count), 0), \
            COALESCE(SUM(cha.miss_count), 0), \
            COALESCE(AVG(cha.avg_hit_size_bytes), 0), \
            COALESCE(SUM(cha.total_size_bytes), 0) \
         FROM cache_hit_analysis cha \
         JOIN pipeline_caches_v2 pc ON cha.cache_id = pc.id \
         WHERE pc.repo_id = $1 AND cha.period_start >= NOW() - INTERVAL '1 day' * $2",
    )
    .bind(repo_id)
    .bind(days)
    .fetch_one(pool)
    .await?;

    let total_hits = row.0.unwrap_or(0);
    let total_misses = row.1.unwrap_or(0);
    let avg_hit_size = row.2.unwrap_or(0);
    let total_size = row.3.unwrap_or(0);
    let total_requests = total_hits + total_misses;
    let hit_rate = if total_requests > 0 {
        total_hits as f64 / total_requests as f64
    } else {
        0.0
    };

    Ok(serde_json::json!({
        "repo_id": repo_id.to_string(),
        "period_days": days,
        "total_hits": total_hits,
        "total_misses": total_misses,
        "total_requests": total_requests,
        "hit_rate": hit_rate,
        "avg_hit_size_bytes": avg_hit_size,
        "total_size_bytes": total_size
    }))
}

/// Get performance insights for a cache.
pub async fn get_cache_performance_insights(
    pool: &sqlx::PgPool,
    cache_id: Uuid,
) -> std::result::Result<serde_json::Value, sqlx::Error> {
    let cache_row: Option<(String, i64, i32)> = sqlx::query_as(
        "SELECT key, size_bytes, hit_count FROM pipeline_caches_v2 WHERE id = $1",
    )
    .bind(cache_id)
    .fetch_optional(pool)
    .await?;

    let analysis_rows: Vec<(i32, i32, i64)> = sqlx::query_as(
        "SELECT hit_count, miss_count, avg_hit_size_bytes FROM cache_hit_analysis \
         WHERE cache_id = $1 ORDER BY period_start DESC LIMIT 30",
    )
    .bind(cache_id)
    .fetch_all(pool)
    .await?;

    let total_hits: i32 = analysis_rows.iter().map(|r| r.0).sum();
    let total_misses: i32 = analysis_rows.iter().map(|r| r.1).sum();
    let total_requests = total_hits + total_misses;
    let hit_rate = if total_requests > 0 {
        total_hits as f64 / total_requests as f64
    } else {
        0.0
    };

    let mut insights = Vec::new();
    if let Some((key, size, hits)) = cache_row {
        if hits == 0 {
            insights.push(serde_json::json!({
                "type": "unused",
                "message": "Cache has never been hit; consider removing it"
            }));
        }
        if size > 100_000_000 {
            insights.push(serde_json::json!({
                "type": "large_size",
                "message": format!("Cache is large ({} bytes); consider compression", size)
            }));
        }
        if hit_rate < 0.3 && total_requests > 10 {
            insights.push(serde_json::json!({
                "type": "low_hit_rate",
                "message": format!("Hit rate is {:.1}%; consider improving cache key strategy", hit_rate * 100.0)
            }));
        }

        Ok(serde_json::json!({
            "cache_id": cache_id.to_string(),
            "key": key,
            "size_bytes": size,
            "hit_count": hits,
            "total_hits_analyzed": total_hits,
            "total_misses_analyzed": total_misses,
            "hit_rate": hit_rate,
            "insights": insights
        }))
    } else {
        Ok(serde_json::json!({
            "cache_id": cache_id.to_string(),
            "error": "cache not found"
        }))
    }
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

// ---------------------------------------------------------------------------
// Cache Prediction Model V20
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachePredictionModelResponse {
    pub id: String,
    pub cache_key_pattern: String,
    pub predicted_hit_rate: f64,
    pub predicted_size_bytes: i64,
    pub predicted_ttl_seconds: i32,
    pub confidence: f64,
    pub last_trained_at: String,
}

#[derive(Debug, sqlx::FromRow)]
pub struct CachePredictionModelRow {
    pub id: Uuid,
    pub cache_key_pattern: String,
    pub predicted_hit_rate: f64,
    pub predicted_size_bytes: i64,
    pub predicted_ttl_seconds: i32,
    pub confidence: f64,
    pub last_trained_at: chrono::DateTime<chrono::Utc>,
}

impl From<CachePredictionModelRow> for CachePredictionModelResponse {
    fn from(r: CachePredictionModelRow) -> Self {
        Self {
            id: r.id.to_string(),
            cache_key_pattern: r.cache_key_pattern,
            predicted_hit_rate: r.predicted_hit_rate,
            predicted_size_bytes: r.predicted_size_bytes,
            predicted_ttl_seconds: r.predicted_ttl_seconds,
            confidence: r.confidence,
            last_trained_at: r.last_trained_at.to_rfc3339(),
        }
    }
}

/// Create or update a cache prediction model.
pub async fn upsert_cache_prediction_model(
    pool: &sqlx::PgPool,
    cache_key_pattern: &str,
    predicted_hit_rate: f64,
    predicted_size_bytes: i64,
    predicted_ttl_seconds: i32,
    confidence: f64,
) -> std::result::Result<CachePredictionModelResponse, sqlx::Error> {
    sqlx::query_as::<_, CachePredictionModelRow>(
        "INSERT INTO cache_prediction_model_v20 \
         (cache_key_pattern, predicted_hit_rate, predicted_size_bytes, predicted_ttl_seconds, confidence) \
         VALUES ($1, $2, $3, $4, $5) \
         ON CONFLICT (cache_key_pattern) DO UPDATE \
         SET predicted_hit_rate = $2, predicted_size_bytes = $3, \
             predicted_ttl_seconds = $4, confidence = $5, last_trained_at = NOW() \
         RETURNING *",
    )
    .bind(cache_key_pattern)
    .bind(predicted_hit_rate)
    .bind(predicted_size_bytes)
    .bind(predicted_ttl_seconds)
    .bind(confidence)
    .fetch_one(pool)
    .await
    .map(|r| r.into())
}

/// Get cache prediction for a key pattern.
pub async fn get_cache_prediction(
    pool: &sqlx::PgPool,
    cache_key_pattern: &str,
) -> std::result::Result<Option<CachePredictionModelResponse>, sqlx::Error> {
    sqlx::query_as::<_, CachePredictionModelRow>(
        "SELECT * FROM cache_prediction_model_v20 WHERE cache_key_pattern = $1",
    )
    .bind(cache_key_pattern)
    .fetch_optional(pool)
    .await
    .map(|r| r.map(|r| r.into()))
}

/// List cache prediction models.
pub async fn list_cache_prediction_models(
    pool: &sqlx::PgPool,
    limit: i64,
    offset: i64,
) -> std::result::Result<Vec<CachePredictionModelResponse>, sqlx::Error> {
    sqlx::query_as::<_, CachePredictionModelRow>(
        "SELECT * FROM cache_prediction_model_v20 \
         ORDER BY confidence DESC LIMIT $1 OFFSET $2",
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await
    .map(|rows| rows.into_iter().map(|r| r.into()).collect())
}

/// Delete a cache prediction model.
pub async fn delete_cache_prediction_model(
    pool: &sqlx::PgPool,
    model_id: Uuid,
) -> std::result::Result<bool, sqlx::Error> {
    let result = sqlx::query("DELETE FROM cache_prediction_model_v20 WHERE id = $1")
        .bind(model_id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

// ---------------------------------------------------------------------------
// Cache Warming Strategies V20
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheWarmingStrategyResponse {
    pub id: String,
    pub name: String,
    pub strategy_type: String,
    pub config: serde_json::Value,
    pub enabled: bool,
    pub hit_rate_improvement: f64,
    pub created_at: String,
}

#[derive(Debug, sqlx::FromRow)]
pub struct CacheWarmingStrategyRow {
    pub id: Uuid,
    pub name: String,
    pub strategy_type: String,
    pub config: serde_json::Value,
    pub enabled: bool,
    pub hit_rate_improvement: f64,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl From<CacheWarmingStrategyRow> for CacheWarmingStrategyResponse {
    fn from(r: CacheWarmingStrategyRow) -> Self {
        Self {
            id: r.id.to_string(),
            name: r.name,
            strategy_type: r.strategy_type,
            config: r.config,
            enabled: r.enabled,
            hit_rate_improvement: r.hit_rate_improvement,
            created_at: r.created_at.to_rfc3339(),
        }
    }
}

/// Create a cache warming strategy.
pub async fn create_warming_strategy(
    pool: &sqlx::PgPool,
    name: &str,
    strategy_type: &str,
    config: &serde_json::Value,
) -> std::result::Result<CacheWarmingStrategyResponse, sqlx::Error> {
    sqlx::query_as::<_, CacheWarmingStrategyRow>(
        "INSERT INTO cache_warming_strategies_v20 (name, strategy_type, config) \
         VALUES ($1, $2, $3) \
         RETURNING *",
    )
    .bind(name)
    .bind(strategy_type)
    .bind(config)
    .fetch_one(pool)
    .await
    .map(|r| r.into())
}

/// List cache warming strategies.
pub async fn list_warming_strategies(
    pool: &sqlx::PgPool,
    enabled_only: bool,
) -> std::result::Result<Vec<CacheWarmingStrategyResponse>, sqlx::Error> {
    let sql = if enabled_only {
        "SELECT * FROM cache_warming_strategies_v20 WHERE enabled = true ORDER BY name"
    } else {
        "SELECT * FROM cache_warming_strategies_v20 ORDER BY name"
    };
    sqlx::query_as::<_, CacheWarmingStrategyRow>(sql)
        .fetch_all(pool)
        .await
        .map(|rows| rows.into_iter().map(|r| r.into()).collect())
}

/// Update a cache warming strategy.
pub async fn update_warming_strategy(
    pool: &sqlx::PgPool,
    strategy_id: Uuid,
    name: Option<&str>,
    strategy_type: Option<&str>,
    config: Option<&serde_json::Value>,
    enabled: Option<bool>,
    hit_rate_improvement: Option<f64>,
) -> std::result::Result<CacheWarmingStrategyResponse, sqlx::Error> {
    sqlx::query_as::<_, CacheWarmingStrategyRow>(
        "UPDATE cache_warming_strategies_v20 \
         SET name = COALESCE($2, name), \
             strategy_type = COALESCE($3, strategy_type), \
             config = COALESCE($4, config), \
             enabled = COALESCE($5, enabled), \
             hit_rate_improvement = COALESCE($6, hit_rate_improvement) \
         WHERE id = $1 \
         RETURNING *",
    )
    .bind(strategy_id)
    .bind(name)
    .bind(strategy_type)
    .bind(config)
    .bind(enabled)
    .bind(hit_rate_improvement)
    .fetch_one(pool)
    .await
    .map(|r| r.into())
}

/// Delete a cache warming strategy.
pub async fn delete_warming_strategy(
    pool: &sqlx::PgPool,
    strategy_id: Uuid,
) -> std::result::Result<bool, sqlx::Error> {
    let result = sqlx::query("DELETE FROM cache_warming_strategies_v20 WHERE id = $1")
        .bind(strategy_id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

/// Get warming strategy summary.
pub async fn get_warming_strategy_summary(
    pool: &sqlx::PgPool,
) -> std::result::Result<serde_json::Value, sqlx::Error> {
    let row: (Option<i64>, Option<i64>, Option<f64>, Option<f64>) = sqlx::query_as(
        "SELECT \
            COUNT(*), \
            SUM(CASE WHEN enabled THEN 1 ELSE 0 END), \
            AVG(hit_rate_improvement), \
            MAX(hit_rate_improvement) \
         FROM cache_warming_strategies_v20",
    )
    .fetch_one(pool)
    .await?;

    Ok(serde_json::json!({
        "total_strategies": row.0.unwrap_or(0),
        "enabled_strategies": row.1.unwrap_or(0),
        "average_improvement": row.2.unwrap_or(0.0),
        "max_improvement": row.3.unwrap_or(0.0)
    }))
}

// ---------------------------------------------------------------------------
// Cache Cost Optimization V24
// ---------------------------------------------------------------------------

/// Get optimized cost analysis with recommendations.
pub async fn get_optimized_cost_analysis(
    pool: &sqlx::PgPool,
    repo_id: Uuid,
    cost_per_gb: f64,
) -> std::result::Result<serde_json::Value, sqlx::Error> {
    let row: (Option<i64>, Option<i64>, Option<i64>, Option<i64>) = sqlx::query_as(
        "SELECT \
            COALESCE(SUM(size_bytes), 0), \
            COALESCE(SUM(hit_count), 0), \
            COUNT(*), \
            COUNT(*) FILTER (WHERE expires_at < NOW()) \
         FROM pipeline_caches_v2 WHERE repo_id = $1",
    )
    .bind(repo_id)
    .fetch_one(pool)
    .await?;

    let total_size_bytes = row.0.unwrap_or(0);
    let total_hits = row.1.unwrap_or(0);
    let entry_count = row.2.unwrap_or(0);
    let expired_count = row.3.unwrap_or(0);
    let total_size_gb = total_size_bytes as f64 / (1024.0 * 1024.0 * 1024.0);
    let estimated_cost = total_size_gb * cost_per_gb;
    let cost_per_hit = if total_hits > 0 {
        estimated_cost / total_hits as f64
    } else {
        0.0
    };
    let wasted_cost = if total_size_bytes > 0 {
        estimated_cost * (expired_count as f64 / entry_count as f64)
    } else {
        0.0
    };

    Ok(serde_json::json!({
        "repo_id": repo_id.to_string(),
        "total_entries": entry_count,
        "total_size_bytes": total_size_bytes,
        "total_size_gb": total_size_gb,
        "total_hits": total_hits,
        "expired_entries": expired_count,
        "cost_per_gb": cost_per_gb,
        "estimated_monthly_cost": estimated_cost,
        "cost_per_hit": cost_per_hit,
        "wasted_cost_from_expired": wasted_cost,
        "optimization_recommendations": serde_json::json!({
            "evict_expired": expired_count > 0,
            "consider_compression": total_size_gb > 1.0,
            "review_hit_rate": if total_hits == 0 { "no hits recorded" } else { "active" }
        })
    }))
}

/// Get storage efficiency report.
pub async fn get_storage_efficiency_report(
    pool: &sqlx::PgPool,
    repo_id: Uuid,
) -> std::result::Result<serde_json::Value, sqlx::Error> {
    let rows: Vec<(String, i64, i32)> = sqlx::query_as(
        "SELECT key, size_bytes, hit_count FROM pipeline_caches_v2 WHERE repo_id = $1 ORDER BY hit_count DESC",
    )
    .bind(repo_id)
    .fetch_all(pool)
    .await?;

    let total_size: i64 = rows.iter().map(|r| r.1).sum();
    let total_hits: i32 = rows.iter().map(|r| r.2).sum();
    let high_value: Vec<&(String, i64, i32)> = rows.iter().filter(|r| r.2 > 10).collect();
    let low_value: Vec<&(String, i64, i32)> = rows.iter().filter(|r| r.2 <= 2).collect();

    Ok(serde_json::json!({
        "repo_id": repo_id.to_string(),
        "total_entries": rows.len(),
        "total_size_bytes": total_size,
        "high_value_caches": high_value.len(),
        "low_value_caches": low_value.len(),
        "efficiency_score": if rows.is_empty() { 0.0 } else { total_hits as f64 / rows.len() as f64 },
        "top_caches": rows.iter().take(5).map(|(k, s, h)| {
            serde_json::json!({
                "key": k,
                "size_bytes": s,
                "hit_count": h
            })
        }).collect::<Vec<_>>()
    }))
}

// ---------------------------------------------------------------------------
// Cache Performance Insights V24
// ---------------------------------------------------------------------------

/// Get advanced performance insights for caches in a repo.
pub async fn get_advanced_performance_insights(
    pool: &sqlx::PgPool,
    repo_id: Uuid,
) -> std::result::Result<serde_json::Value, sqlx::Error> {
    let row: (Option<i64>, Option<i64>, Option<f64>, Option<i64>) = sqlx::query_as(
        "SELECT \
            COUNT(*), \
            COALESCE(SUM(hit_count), 0), \
            COALESCE(AVG(hit_count), 0), \
            COALESCE(MAX(hit_count), 0) \
         FROM pipeline_caches_v2 WHERE repo_id = $1",
    )
    .bind(repo_id)
    .fetch_one(pool)
    .await?;

    let entry_count = row.0.unwrap_or(0);
    let total_hits = row.1.unwrap_or(0);
    let avg_hits = row.2.unwrap_or(0.0);
    let max_hits = row.3.unwrap_or(0);

    let analysis_rows: Vec<(i32, i32, i64)> = sqlx::query_as(
        "SELECT hit_count, miss_count, avg_hit_size_bytes \
         FROM cache_hit_analysis WHERE cache_id IN \
         (SELECT id FROM pipeline_caches_v2 WHERE repo_id = $1) \
         ORDER BY period_start DESC LIMIT 100",
    )
    .bind(repo_id)
    .fetch_all(pool)
    .await?;

    let total_analysis_hits: i32 = analysis_rows.iter().map(|r| r.0).sum();
    let total_analysis_misses: i32 = analysis_rows.iter().map(|r| r.1).sum();
    let total_requests = total_analysis_hits + total_analysis_misses;
    let hit_rate = if total_requests > 0 {
        total_analysis_hits as f64 / total_requests as f64
    } else {
        0.0
    };

    let mut insights = Vec::new();
    if entry_count == 0 {
        insights.push(serde_json::json!({
            "type": "no_caches",
            "message": "No caches found for this repository"
        }));
    } else if hit_rate < 0.5 && total_requests > 20 {
        insights.push(serde_json::json!({
            "type": "low_hit_rate",
            "message": format!("Hit rate is {:.1}%; consider reviewing cache key strategy", hit_rate * 100.0)
        }));
    }
    if max_hits > 100 {
        insights.push(serde_json::json!({
            "type": "hot_cache",
            "message": "Some caches have high hit counts; ensure they are optimized"
        }));
    }

    Ok(serde_json::json!({
        "repo_id": repo_id.to_string(),
        "total_entries": entry_count,
        "total_hits": total_hits,
        "average_hits_per_cache": avg_hits,
        "max_hits": max_hits,
        "analysis_total_hits": total_analysis_hits,
        "analysis_total_misses": total_analysis_misses,
        "overall_hit_rate": hit_rate,
        "insights": insights,
        "performance_score": if hit_rate > 0.8 { "excellent" } else if hit_rate > 0.6 { "good" } else if hit_rate > 0.4 { "fair" } else { "needs_improvement" }
    }))
}

/// Get cache trend analysis over time.
pub async fn get_cache_trend_analysis(
    pool: &sqlx::PgPool,
    repo_id: Uuid,
    days: i32,
) -> std::result::Result<serde_json::Value, sqlx::Error> {
    let rows: Vec<(chrono::DateTime<chrono::Utc>, i32, i32)> = sqlx::query_as(
        "SELECT cha.period_start, cha.hit_count, cha.miss_count \
         FROM cache_hit_analysis cha \
         JOIN pipeline_caches_v2 pc ON cha.cache_id = pc.id \
         WHERE pc.repo_id = $1 AND cha.period_start >= NOW() - INTERVAL '1 day' * $2 \
         ORDER BY cha.period_start",
    )
    .bind(repo_id)
    .bind(days)
    .fetch_all(pool)
    .await?;

    let total_hits: i64 = rows.iter().map(|r| r.1 as i64).sum();
    let total_misses: i64 = rows.iter().map(|r| r.2 as i64).sum();
    let total_requests = total_hits + total_misses;
    let hit_rate = if total_requests > 0 {
        total_hits as f64 / total_requests as f64
    } else {
        0.0
    };

    Ok(serde_json::json!({
        "repo_id": repo_id.to_string(),
        "period_days": days,
        "total_hits": total_hits,
        "total_misses": total_misses,
        "hit_rate": hit_rate,
        "data_points": rows.len(),
        "trend": if rows.len() < 2 {
            "insufficient_data"
        } else {
            let first_half = &rows[..rows.len()/2];
            let second_half = &rows[rows.len()/2..];
            let first_hits: i64 = first_half.iter().map(|r| r.1 as i64).sum();
            let second_hits: i64 = second_half.iter().map(|r| r.1 as i64).sum();
            if second_hits > first_hits * 2 { "improving" } else if second_hits < first_hits / 2 { "declining" } else { "stable" }
        }
    }))
}

// ---------------------------------------------------------------------------
// Cache Hit Analysis V18
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheHitAnalysisV18Response {
    pub id: String,
    pub cache_id: String,
    pub period_start: String,
    pub hit_count: i32,
    pub miss_count: i32,
    pub avg_hit_size_bytes: i64,
    pub total_size_bytes: i64,
    pub created_at: String,
}

#[derive(Debug, sqlx::FromRow)]
pub struct CacheHitAnalysisV18Row {
    pub id: uuid::Uuid,
    pub cache_id: uuid::Uuid,
    pub period_start: chrono::DateTime<chrono::Utc>,
    pub hit_count: i32,
    pub miss_count: i32,
    pub avg_hit_size_bytes: i64,
    pub total_size_bytes: i64,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl From<CacheHitAnalysisV18Row> for CacheHitAnalysisV18Response {
    fn from(r: CacheHitAnalysisV18Row) -> Self {
        Self {
            id: r.id.to_string(),
            cache_id: r.cache_id.to_string(),
            period_start: r.period_start.to_rfc3339(),
            hit_count: r.hit_count,
            miss_count: r.miss_count,
            avg_hit_size_bytes: r.avg_hit_size_bytes,
            total_size_bytes: r.total_size_bytes,
            created_at: r.created_at.to_rfc3339(),
        }
    }
}

/// Get hit analysis v18 for a cache.
pub async fn get_hit_analysis_v18(
    pool: &sqlx::PgPool,
    cache_id: uuid::Uuid,
) -> std::result::Result<Vec<CacheHitAnalysisV18Response>, sqlx::Error> {
    sqlx::query_as::<_, CacheHitAnalysisV18Row>(
        "SELECT * FROM cache_hit_analysis_v18 WHERE cache_id = $1 ORDER BY period_start DESC",
    )
    .bind(cache_id)
    .fetch_all(pool)
    .await
    .map(|rows| rows.into_iter().map(|r| r.into()).collect())
}

/// Record a hit/miss v18.
pub async fn record_hit_miss_v18(
    pool: &sqlx::PgPool,
    cache_id: uuid::Uuid,
    is_hit: bool,
    size_bytes: i64,
) -> std::result::Result<CacheHitAnalysisV18Response, sqlx::Error> {
    let now = chrono::Utc::now();
    let period_start = now - chrono::Duration::hours(1);

    sqlx::query_as::<_, CacheHitAnalysisV18Row>(
        "INSERT INTO cache_hit_analysis_v18 \
         (cache_id, period_start, hit_count, miss_count, avg_hit_size_bytes, total_size_bytes) \
         VALUES ($1, $2, $3, $4, $5, $6) \
         ON CONFLICT (cache_id, period_start) DO UPDATE SET \
            hit_count = cache_hit_analysis_v18.hit_count + $3, \
            miss_count = cache_hit_analysis_v18.miss_count + $4, \
            avg_hit_size_bytes = CASE WHEN $3 > 0 THEN \
                (cache_hit_analysis_v18.avg_hit_size_bytes + $5) / 2 \
            ELSE cache_hit_analysis_v18.avg_hit_size_bytes END, \
            total_size_bytes = cache_hit_analysis_v18.total_size_bytes + $6 \
         RETURNING *",
    )
    .bind(cache_id)
    .bind(period_start)
    .bind(if is_hit { 1 } else { 0 })
    .bind(if is_hit { 0 } else { 1 })
    .bind(size_bytes)
    .bind(size_bytes)
    .fetch_one(pool)
    .await
    .map(|r| r.into())
}

/// Get size history v18 for a cache.
pub async fn get_size_history_v18(
    pool: &sqlx::PgPool,
    cache_id: uuid::Uuid,
) -> std::result::Result<Vec<CacheHitAnalysisV18Response>, sqlx::Error> {
    sqlx::query_as::<_, CacheHitAnalysisV18Row>(
        "SELECT * FROM cache_hit_analysis_v18 WHERE cache_id = $1 \
         ORDER BY period_start DESC LIMIT 24",
    )
    .bind(cache_id)
    .fetch_all(pool)
    .await
    .map(|rows| rows.into_iter().map(|r| r.into()).collect())
}

/// Get cost optimizations v18 for a cache.
pub async fn get_cost_optimizations_v18(
    pool: &sqlx::PgPool,
    cache_id: uuid::Uuid,
    cost_per_gb: f64,
) -> std::result::Result<serde_json::Value, sqlx::Error> {
    let rows: Vec<CacheHitAnalysisV18Row> = sqlx::query_as(
        "SELECT * FROM cache_hit_analysis_v18 WHERE cache_id = $1 ORDER BY period_start DESC LIMIT 30",
    )
    .bind(cache_id)
    .fetch_all(pool)
    .await?;

    let total_hits: i64 = rows.iter().map(|r| r.hit_count as i64).sum();
    let total_misses: i64 = rows.iter().map(|r| r.miss_count as i64).sum();
    let total_size: i64 = rows.iter().map(|r| r.total_size_bytes).sum();
    let hit_rate = if total_hits + total_misses > 0 {
        total_hits as f64 / (total_hits + total_misses) as f64
    } else {
        0.0
    };
    let cost = (total_size as f64 / 1_073_741_824.0) * cost_per_gb;
    let wasted_cost = cost * (1.0 - hit_rate);

    Ok(serde_json::json!({
        "cache_id": cache_id.to_string(),
        "total_hits": total_hits,
        "total_misses": total_misses,
        "hit_rate": hit_rate,
        "total_size_bytes": total_size,
        "estimated_cost_usd": cost,
        "potential_savings_usd": wasted_cost
    }))
}

/// Get performance insights v18 for a cache.
pub async fn get_performance_insights_v18(
    pool: &sqlx::PgPool,
    cache_id: uuid::Uuid,
) -> std::result::Result<serde_json::Value, sqlx::Error> {
    let rows: Vec<CacheHitAnalysisV18Row> = sqlx::query_as(
        "SELECT * FROM cache_hit_analysis_v18 WHERE cache_id = $1 ORDER BY period_start DESC LIMIT 30",
    )
    .bind(cache_id)
    .fetch_all(pool)
    .await?;

    let total_hits: i64 = rows.iter().map(|r| r.hit_count as i64).sum();
    let total_misses: i64 = rows.iter().map(|r| r.miss_count as i64).sum();
    let avg_size: f64 = if !rows.is_empty() {
        rows.iter().map(|r| r.avg_hit_size_bytes as f64).sum::<f64>() / rows.len() as f64
    } else {
        0.0
    };

    Ok(serde_json::json!({
        "cache_id": cache_id.to_string(),
        "total_hits": total_hits,
        "total_misses": total_misses,
        "hit_rate": if total_hits + total_misses > 0 { total_hits as f64 / (total_hits + total_misses) as f64 } else { 0.0 },
        "average_hit_size_bytes": avg_size,
        "performance_score": if total_hits > total_misses * 2 { "excellent" } else if total_hits > total_misses { "good" } else { "needs_improvement" }
    }))
}
