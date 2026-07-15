//! Cache Hit Analysis v8: Advanced cache analysis with size tracking v8,
//! cost optimization v8, and performance insights v8.

#![forbid(unsafe_code)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheHitAnalysisV8 {
    pub id: Uuid,
    pub cache_id: Uuid,
    pub period_start: DateTime<Utc>,
    pub hit_count: i32,
    pub miss_count: i32,
    pub avg_hit_size_bytes: i64,
    pub total_size_bytes: i64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheSizeTrackingV8 {
    pub id: Uuid,
    pub cache_id: Uuid,
    pub measured_at: DateTime<Utc>,
    pub size_bytes: i64,
    pub item_count: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheCostOptimizationV8 {
    pub id: Uuid,
    pub cache_id: Uuid,
    pub period_start: DateTime<Utc>,
    pub estimated_savings_bytes: i64,
    pub recommended_actions: serde_json::Value,
    pub applied_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachePerformanceInsightsV8 {
    pub id: Uuid,
    pub cache_id: Uuid,
    pub period_start: DateTime<Utc>,
    pub hit_rate: f64,
    pub avg_hit_latency_ms: i64,
    pub avg_miss_latency_ms: i64,
    pub eviction_count: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordHitMissRequestV8 {
    pub cache_id: Uuid,
    pub hit: bool,
    pub size_bytes: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordSizeRequestV8 {
    pub cache_id: Uuid,
    pub size_bytes: i64,
    pub item_count: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateAnalyticsRequestV8 {
    pub cache_id: Uuid,
    pub period_start: DateTime<Utc>,
}

#[derive(Debug, sqlx::FromRow)]
struct AnalysisRowV8 {
    id: Uuid,
    cache_id: Uuid,
    period_start: DateTime<Utc>,
    hit_count: i32,
    miss_count: i32,
    avg_hit_size_bytes: i64,
    total_size_bytes: i64,
    created_at: DateTime<Utc>,
}

impl From<AnalysisRowV8> for CacheHitAnalysisV8 {
    fn from(row: AnalysisRowV8) -> Self {
        CacheHitAnalysisV8 {
            id: row.id,
            cache_id: row.cache_id,
            period_start: row.period_start,
            hit_count: row.hit_count,
            miss_count: row.miss_count,
            avg_hit_size_bytes: row.avg_hit_size_bytes,
            total_size_bytes: row.total_size_bytes,
            created_at: row.created_at,
        }
    }
}

#[derive(Debug, sqlx::FromRow)]
struct SizeRowV8 {
    id: Uuid,
    cache_id: Uuid,
    measured_at: DateTime<Utc>,
    size_bytes: i64,
    item_count: i32,
    created_at: DateTime<Utc>,
}

impl From<SizeRowV8> for CacheSizeTrackingV8 {
    fn from(row: SizeRowV8) -> Self {
        CacheSizeTrackingV8 {
            id: row.id,
            cache_id: row.cache_id,
            measured_at: row.measured_at,
            size_bytes: row.size_bytes,
            item_count: row.item_count,
            created_at: row.created_at,
        }
    }
}

#[derive(Debug, sqlx::FromRow)]
struct CostRowV8 {
    id: Uuid,
    cache_id: Uuid,
    period_start: DateTime<Utc>,
    estimated_savings_bytes: i64,
    recommended_actions: serde_json::Value,
    applied_at: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
}

impl From<CostRowV8> for CacheCostOptimizationV8 {
    fn from(row: CostRowV8) -> Self {
        CacheCostOptimizationV8 {
            id: row.id,
            cache_id: row.cache_id,
            period_start: row.period_start,
            estimated_savings_bytes: row.estimated_savings_bytes,
            recommended_actions: row.recommended_actions,
            applied_at: row.applied_at,
            created_at: row.created_at,
        }
    }
}

#[derive(Debug, sqlx::FromRow)]
struct PerformanceRowV8 {
    id: Uuid,
    cache_id: Uuid,
    period_start: DateTime<Utc>,
    hit_rate: f64,
    avg_hit_latency_ms: i64,
    avg_miss_latency_ms: i64,
    eviction_count: i32,
    created_at: DateTime<Utc>,
}

impl From<PerformanceRowV8> for CachePerformanceInsightsV8 {
    fn from(row: PerformanceRowV8) -> Self {
        CachePerformanceInsightsV8 {
            id: row.id,
            cache_id: row.cache_id,
            period_start: row.period_start,
            hit_rate: row.hit_rate,
            avg_hit_latency_ms: row.avg_hit_latency_ms,
            avg_miss_latency_ms: row.avg_miss_latency_ms,
            eviction_count: row.eviction_count,
            created_at: row.created_at,
        }
    }
}

pub struct CacheHitAnalysisServiceV8 {
    pool: PgPool,
}

impl CacheHitAnalysisServiceV8 {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn record_hit_miss(
        &self,
        request: RecordHitMissRequestV8,
    ) -> Result<CacheHitAnalysisV8, sqlx::Error> {
        let size_bytes = request.size_bytes.unwrap_or(0);
        let hit_increment = if request.hit { 1 } else { 0 };
        let miss_increment = if request.hit { 0 } else { 1 };

        let row = sqlx::query_as::<_, AnalysisRowV8>(
            "INSERT INTO cache_hit_analysis_v8
             (cache_id, period_start, hit_count, miss_count, avg_hit_size_bytes, total_size_bytes, created_at)
             VALUES ($1, date_trunc('hour', NOW()), $2, $3, $4, $5, NOW())
             ON CONFLICT (cache_id, period_start) DO UPDATE SET
                 hit_count = cache_hit_analysis_v8.hit_count + EXCLUDED.hit_count,
                 miss_count = cache_hit_analysis_v8.miss_count + EXCLUDED.miss_count,
                 avg_hit_size_bytes = CASE
                     WHEN EXCLUDED.hit_count > 0 THEN
                         (cache_hit_analysis_v8.avg_hit_size_bytes * cache_hit_analysis_v8.hit_count + $4 * $2) /
                         (cache_hit_analysis_v8.hit_count + $2)
                     ELSE cache_hit_analysis_v8.avg_hit_size_bytes
                 END,
                 total_size_bytes = cache_hit_analysis_v8.total_size_bytes + $5
             RETURNING id, cache_id, period_start, hit_count, miss_count, avg_hit_size_bytes, total_size_bytes, created_at",
        )
        .bind(request.cache_id)
        .bind(hit_increment)
        .bind(miss_increment)
        .bind(size_bytes)
        .bind(size_bytes)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    pub async fn get_analysis(
        &self,
        cache_id: Uuid,
    ) -> Result<Vec<CacheHitAnalysisV8>, sqlx::Error> {
        let rows = sqlx::query_as::<_, AnalysisRowV8>(
            "SELECT id, cache_id, period_start, hit_count, miss_count, avg_hit_size_bytes, total_size_bytes, created_at
             FROM cache_hit_analysis_v8
             WHERE cache_id = $1
             ORDER BY period_start DESC",
        )
        .bind(cache_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn record_size(
        &self,
        request: RecordSizeRequestV8,
    ) -> Result<CacheSizeTrackingV8, sqlx::Error> {
        let row = sqlx::query_as::<_, SizeRowV8>(
            "INSERT INTO cache_size_tracking_v8
             (cache_id, measured_at, size_bytes, item_count, created_at)
             VALUES ($1, NOW(), $2, $3, NOW())
             RETURNING id, cache_id, measured_at, size_bytes, item_count, created_at",
        )
        .bind(request.cache_id)
        .bind(request.size_bytes)
        .bind(request.item_count)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    pub async fn get_size_history(
        &self,
        cache_id: Uuid,
        limit: i64,
    ) -> Result<Vec<CacheSizeTrackingV8>, sqlx::Error> {
        let rows = sqlx::query_as::<_, SizeRowV8>(
            "SELECT id, cache_id, measured_at, size_bytes, item_count, created_at
             FROM cache_size_tracking_v8
             WHERE cache_id = $1
             ORDER BY measured_at DESC
             LIMIT $2",
        )
        .bind(cache_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn generate_cost_optimization(
        &self,
        cache_id: Uuid,
        period_start: DateTime<Utc>,
    ) -> Result<CacheCostOptimizationV8, sqlx::Error> {
        let row = sqlx::query_as::<_, CostRowV8>(
            "INSERT INTO cache_cost_optimization_v8
             (cache_id, period_start, estimated_savings_bytes, recommended_actions, created_at)
             SELECT
                 $1 as cache_id,
                 $2 as period_start,
                 GREATEST(total_size_bytes - (total_size_bytes * hit_count / GREATEST(hit_count + miss_count, 1)), 0) as estimated_savings_bytes,
                 CASE
                     WHEN miss_count > hit_count THEN
                         jsonb_build_array('Consider increasing cache size', 'Review cache key strategy')
                     WHEN avg_hit_size_bytes > 1048576 THEN
                         jsonb_build_array('Large items detected', 'Consider item compression')
                     ELSE
                         jsonb_build_array('Cache performance is optimal')
                 END as recommended_actions,
                 NOW() as created_at
             FROM cache_hit_analysis_v8
             WHERE cache_id = $1 AND period_start = $2
             RETURNING id, cache_id, period_start, estimated_savings_bytes, recommended_actions, applied_at, created_at",
        )
        .bind(cache_id)
        .bind(period_start)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    pub async fn get_cost_optimizations(
        &self,
        cache_id: Uuid,
    ) -> Result<Vec<CacheCostOptimizationV8>, sqlx::Error> {
        let rows = sqlx::query_as::<_, CostRowV8>(
            "SELECT id, cache_id, period_start, estimated_savings_bytes, recommended_actions, applied_at, created_at
             FROM cache_cost_optimization_v8
             WHERE cache_id = $1
             ORDER BY period_start DESC",
        )
        .bind(cache_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn generate_performance_insights(
        &self,
        cache_id: Uuid,
        period_start: DateTime<Utc>,
    ) -> Result<CachePerformanceInsightsV8, sqlx::Error> {
        let row = sqlx::query_as::<_, PerformanceRowV8>(
            "INSERT INTO cache_performance_insights_v8
             (cache_id, period_start, hit_rate, avg_hit_latency_ms, avg_miss_latency_ms, eviction_count, created_at)
             SELECT
                 $1 as cache_id,
                 $2 as period_start,
                 CASE
                     WHEN (hit_count + miss_count) > 0 THEN
                         hit_count::NUMERIC / (hit_count + miss_count)
                     ELSE 0
                 END::NUMERIC(5,4) as hit_rate,
                 0::BIGINT as avg_hit_latency_ms,
                 0::BIGINT as avg_miss_latency_ms,
                 0 as eviction_count,
                 NOW() as created_at
             FROM cache_hit_analysis_v8
             WHERE cache_id = $1 AND period_start = $2
             RETURNING id, cache_id, period_start, hit_rate, avg_hit_latency_ms, avg_miss_latency_ms, eviction_count, created_at",
        )
        .bind(cache_id)
        .bind(period_start)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    pub async fn get_performance_insights(
        &self,
        cache_id: Uuid,
    ) -> Result<Vec<CachePerformanceInsightsV8>, sqlx::Error> {
        let rows = sqlx::query_as::<_, PerformanceRowV8>(
            "SELECT id, cache_id, period_start, hit_rate, avg_hit_latency_ms, avg_miss_latency_ms, eviction_count, created_at
             FROM cache_performance_insights_v8
             WHERE cache_id = $1
             ORDER BY period_start DESC",
        )
        .bind(cache_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analysis_v8_serialize() {
        let analysis = CacheHitAnalysisV8 {
            id: Uuid::new_v4(),
            cache_id: Uuid::new_v4(),
            period_start: Utc::now(),
            hit_count: 100,
            miss_count: 10,
            avg_hit_size_bytes: 1024,
            total_size_bytes: 102400,
            created_at: Utc::now(),
        };
        let json = serde_json::to_string(&analysis).unwrap();
        assert!(json.contains("100"));
        assert!(json.contains("1024"));
    }

    #[test]
    fn test_record_hit_miss_request_v8_deserialize() {
        let json = r#"{"cache_id": "550e8400-e29b-41d4-a716-446655440000", "hit": true, "size_bytes": 2048}"#;
        let req: RecordHitMissRequestV8 = serde_json::from_str(json).unwrap();
        assert!(req.hit);
        assert_eq!(req.size_bytes, Some(2048));
    }

    #[test]
    fn test_size_tracking_v8_serialize() {
        let tracking = CacheSizeTrackingV8 {
            id: Uuid::new_v4(),
            cache_id: Uuid::new_v4(),
            measured_at: Utc::now(),
            size_bytes: 1048576,
            item_count: 50,
            created_at: Utc::now(),
        };
        let json = serde_json::to_string(&tracking).unwrap();
        assert!(json.contains("1048576"));
        assert!(json.contains("50"));
    }

    #[test]
    fn test_cost_optimization_v8_serialize() {
        let c = CacheCostOptimizationV8 {
            id: Uuid::new_v4(),
            cache_id: Uuid::new_v4(),
            period_start: Utc::now(),
            estimated_savings_bytes: 2048,
            recommended_actions: serde_json::json!(["Increase cache size"]),
            applied_at: None,
            created_at: Utc::now(),
        };
        let json = serde_json::to_string(&c).unwrap();
        assert!(json.contains("2048"));
    }
}
