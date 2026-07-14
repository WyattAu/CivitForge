use super::types::*;
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

pub struct PerformanceTestStore {
    pool: PgPool,
}

impl PerformanceTestStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn add_test_config(
        &self,
        test_id: Uuid,
        req: CreateTestConfigRequest,
    ) -> Result<TestConfigEntry, sqlx::Error> {
        let id = Uuid::new_v4();
        let now = Utc::now();

        sqlx::query(
            r#"INSERT INTO performance_test_configs (id, test_id, config_key, config_value, created_at)
               VALUES ($1, $2, $3, $4, $5)"#,
        )
        .bind(id)
        .bind(test_id)
        .bind(&req.config_key)
        .bind(&req.config_value)
        .bind(now)
        .execute(&self.pool)
        .await?;

        Ok(TestConfigEntry {
            id,
            test_id,
            config_key: req.config_key,
            config_value: req.config_value,
            created_at: now,
        })
    }

    pub async fn get_test_configs(
        &self,
        test_id: Uuid,
    ) -> Result<Vec<TestConfigEntry>, sqlx::Error> {
        let rows = sqlx::query_as::<_, TestConfigRow>(
            r#"SELECT id, test_id, config_key, config_value, created_at
               FROM performance_test_configs
               WHERE test_id = $1
               ORDER BY created_at"#,
        )
        .bind(test_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(TestConfigEntry::from).collect())
    }

    pub async fn delete_test_config(&self, id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(r#"DELETE FROM performance_test_configs WHERE id = $1"#)
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn record_test_result(
        &self,
        test_id: Uuid,
        req: RecordTestResultRequest,
    ) -> Result<TestResultMetric, sqlx::Error> {
        let id = Uuid::new_v4();
        let now = Utc::now();

        sqlx::query(
            r#"INSERT INTO performance_test_results (id, test_id, metric_name, metric_value, percentile, recorded_at)
               VALUES ($1, $2, $3, $4, $5, $6)"#,
        )
        .bind(id)
        .bind(test_id)
        .bind(&req.metric_name)
        .bind(req.metric_value)
        .bind(req.percentile)
        .bind(now)
        .execute(&self.pool)
        .await?;

        Ok(TestResultMetric {
            id,
            test_id,
            metric_name: req.metric_name,
            metric_value: req.metric_value,
            percentile: req.percentile,
            recorded_at: now,
        })
    }

    pub async fn get_test_results(
        &self,
        test_id: Uuid,
    ) -> Result<Vec<TestResultMetric>, sqlx::Error> {
        let rows = sqlx::query_as::<_, TestResultRow>(
            r#"SELECT id, test_id, metric_name, metric_value, percentile, recorded_at
               FROM performance_test_results
               WHERE test_id = $1
               ORDER BY metric_name, percentile"#,
        )
        .bind(test_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(TestResultMetric::from).collect())
    }

    pub async fn get_percentile_analysis(
        &self,
        test_id: Uuid,
        metric_name: &str,
    ) -> Result<PercentileAnalysis, sqlx::Error> {
        let row = sqlx::query_as::<_, PercentileRow>(
            r#"SELECT
                $2 as metric_name,
                (SELECT metric_value FROM performance_test_results WHERE test_id = $1 AND metric_name = $2 AND percentile = 50.0 LIMIT 1) as p50,
                (SELECT metric_value FROM performance_test_results WHERE test_id = $1 AND metric_name = $2 AND percentile = 90.0 LIMIT 1) as p90,
                (SELECT metric_value FROM performance_test_results WHERE test_id = $1 AND metric_name = $2 AND percentile = 95.0 LIMIT 1) as p95,
                (SELECT metric_value FROM performance_test_results WHERE test_id = $1 AND metric_name = $2 AND percentile = 99.0 LIMIT 1) as p99,
                COALESCE(AVG(metric_value), 0) as avg,
                COALESCE(MIN(metric_value), 0) as min_val,
                COALESCE(MAX(metric_value), 0) as max_val
             FROM performance_test_results
             WHERE test_id = $1 AND metric_name = $2"#,
        )
        .bind(test_id)
        .bind(metric_name)
        .fetch_one(&self.pool)
        .await?;

        Ok(PercentileAnalysis {
            metric_name: row.metric_name,
            p50: row.p50,
            p90: row.p90,
            p95: row.p95,
            p99: row.p99,
            avg: row.avg,
            min: row.min_val,
            max: row.max_val,
        })
    }

    pub async fn compare_tests(
        &self,
        test_id_1: Uuid,
        test_id_2: Uuid,
    ) -> Result<PerformanceComparison, sqlx::Error> {
        let test_1 = sqlx::query_scalar::<_, String>(
            r#"SELECT name FROM performance_tests WHERE id = $1"#,
        )
        .bind(test_id_1)
        .fetch_one(&self.pool)
        .await?;

        let test_2 = sqlx::query_scalar::<_, String>(
            r#"SELECT name FROM performance_tests WHERE id = $1"#,
        )
        .bind(test_id_2)
        .fetch_one(&self.pool)
        .await?;

        let rows = sqlx::query_as::<_, MetricCompareRow>(
            r#"SELECT
                COALESCE(r1.metric_name, r2.metric_name) as metric_name,
                COALESCE(r1.metric_value, 0) as value_1,
                COALESCE(r2.metric_value, 0) as value_2
             FROM performance_test_results r1
             FULL OUTER JOIN performance_test_results r2
                ON r1.metric_name = r2.metric_name AND r1.percentile IS NULL AND r2.percentile IS NULL
             WHERE (r1.test_id = $1 OR r2.test_id = $2)
               AND r1.percentile IS NULL AND r2.percentile IS NULL
             GROUP BY r1.metric_name, r2.metric_name, r1.metric_value, r2.metric_value
             ORDER BY COALESCE(r1.metric_name, r2.metric_name)"#,
        )
        .bind(test_id_1)
        .bind(test_id_2)
        .fetch_all(&self.pool)
        .await?;

        let metrics = rows
            .into_iter()
            .map(|r| {
                let change = if r.value_1 != 0.0 {
                    ((r.value_2 - r.value_1) / r.value_1) * 100.0
                } else {
                    0.0
                };
                MetricComparison {
                    metric_name: r.metric_name,
                    value_1: r.value_1,
                    value_2: r.value_2,
                    change_percent: change,
                    improved: r.value_2 < r.value_1,
                }
            })
            .collect();

        Ok(PerformanceComparison {
            test_id_1,
            test_id_2,
            test_name_1: test_1,
            test_name_2: test_2,
            metrics,
        })
    }

    pub async fn create_test(
        &self,
        repo_id: Uuid,
        req: CreatePerformanceTestRequest,
    ) -> Result<PerformanceTestRecord, sqlx::Error> {
        let id = Uuid::new_v4();
        let now = Utc::now();
        let config = serde_json::to_value(&req.config.unwrap_or_default())
            .unwrap_or(serde_json::json!({}));

        sqlx::query(
            r#"INSERT INTO performance_tests (id, repo_id, name, test_type, endpoint, config, status, started_at)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8)"#,
        )
        .bind(id)
        .bind(repo_id)
        .bind(&req.name)
        .bind(req.test_type.to_string())
        .bind(&req.endpoint)
        .bind(&config)
        .bind(TestStatus::Pending.to_string())
        .bind(now)
        .execute(&self.pool)
        .await?;

        Ok(PerformanceTestRecord {
            id,
            repo_id,
            name: req.name,
            test_type: req.test_type,
            endpoint: req.endpoint,
            config,
            status: TestStatus::Pending,
            results: serde_json::json!({}),
            started_at: now,
            completed_at: None,
        })
    }

    pub async fn get_test(&self, id: Uuid) -> Result<Option<PerformanceTestRecord>, sqlx::Error> {
        let row = sqlx::query_as::<_, TestRow>(
            r#"SELECT id, repo_id, name, test_type, endpoint, config, status, results, started_at, completed_at
               FROM performance_tests WHERE id = $1"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(PerformanceTestRecord::from))
    }

    pub async fn list_tests(
        &self,
        repo_id: Uuid,
        test_type: Option<&str>,
        status: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<PerformanceTestRecord>, sqlx::Error> {
        let rows = sqlx::query_as::<_, TestRow>(
            r#"SELECT id, repo_id, name, test_type, endpoint, config, status, results, started_at, completed_at
               FROM performance_tests
               WHERE repo_id = $1
                 AND ($2::varchar IS NULL OR test_type = $2)
                 AND ($3::varchar IS NULL OR status = $3)
               ORDER BY started_at DESC
               LIMIT $4 OFFSET $5"#,
        )
        .bind(repo_id)
        .bind(test_type)
        .bind(status)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(PerformanceTestRecord::from).collect())
    }

    pub async fn start_test(&self, id: Uuid) -> Result<PerformanceTestRecord, sqlx::Error> {
        sqlx::query(
            r#"UPDATE performance_tests SET status = $1, started_at = NOW() WHERE id = $2"#,
        )
        .bind(TestStatus::Running.to_string())
        .bind(id)
        .execute(&self.pool)
        .await?;

        self.get_test(id).await?.ok_or_else(|| sqlx::Error::RowNotFound)
    }

    pub async fn complete_test(
        &self,
        id: Uuid,
        status: TestStatus,
        results: serde_json::Value,
    ) -> Result<PerformanceTestRecord, sqlx::Error> {
        sqlx::query(
            r#"UPDATE performance_tests SET status = $1, results = $2, completed_at = NOW() WHERE id = $3"#,
        )
        .bind(status.to_string())
        .bind(&results)
        .bind(id)
        .execute(&self.pool)
        .await?;

        self.get_test(id).await?.ok_or_else(|| sqlx::Error::RowNotFound)
    }

    pub async fn get_summary(
        &self,
        repo_id: Uuid,
    ) -> Result<PerformanceTestSummary, sqlx::Error> {
        let row = sqlx::query_as::<_, SummaryRow>(
            r#"SELECT
                COUNT(*) as total_tests,
                COUNT(*) FILTER (WHERE status = 'completed') as completed_tests,
                COUNT(*) FILTER (WHERE status = 'failed') as failed_tests,
                COUNT(*) FILTER (WHERE status = 'running') as running_tests,
                COUNT(*) FILTER (WHERE status = 'pending') as pending_tests
             FROM performance_tests
             WHERE repo_id = $1"#,
        )
        .bind(repo_id)
        .fetch_one(&self.pool)
        .await?;

        let type_rows = sqlx::query_as::<_, TypeCountRow>(
            r#"SELECT test_type, COUNT(*) as count
               FROM performance_tests
               WHERE repo_id = $1
               GROUP BY test_type"#,
        )
        .bind(repo_id)
        .fetch_all(&self.pool)
        .await?;

        let mut by_type = serde_json::json!({});
        for tr in type_rows {
            by_type[tr.test_type] = serde_json::json!(tr.count);
        }

        let latest_results: Option<PerformanceTestResults> = sqlx::query_scalar::<_, serde_json::Value>(
            r#"SELECT results FROM performance_tests
               WHERE repo_id = $1 AND status = 'completed'
               ORDER BY completed_at DESC LIMIT 1"#,
        )
        .bind(repo_id)
        .fetch_optional(&self.pool)
        .await?
        .and_then(|v| serde_json::from_value(v).ok());

        Ok(PerformanceTestSummary {
            total_tests: row.total_tests,
            completed_tests: row.completed_tests,
            failed_tests: row.failed_tests,
            running_tests: row.running_tests,
            pending_tests: row.pending_tests,
            by_type,
            latest_results,
        })
    }

    pub async fn delete_test(&self, id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(r#"DELETE FROM performance_tests WHERE id = $1"#)
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }
}

#[derive(sqlx::FromRow)]
struct TestRow {
    id: Uuid,
    repo_id: Uuid,
    name: String,
    test_type: String,
    endpoint: Option<String>,
    config: serde_json::Value,
    status: String,
    results: serde_json::Value,
    started_at: chrono::DateTime<Utc>,
    completed_at: Option<chrono::DateTime<Utc>>,
}

impl From<TestRow> for PerformanceTestRecord {
    fn from(row: TestRow) -> Self {
        Self {
            id: row.id,
            repo_id: row.repo_id,
            name: row.name,
            test_type: row.test_type.parse().unwrap_or(TestType::Load),
            endpoint: row.endpoint,
            config: row.config,
            status: row.status.parse().unwrap_or(TestStatus::Pending),
            results: row.results,
            started_at: row.started_at,
            completed_at: row.completed_at,
        }
    }
}

#[derive(sqlx::FromRow)]
struct SummaryRow {
    total_tests: i64,
    completed_tests: i64,
    failed_tests: i64,
    running_tests: i64,
    pending_tests: i64,
}

#[derive(sqlx::FromRow)]
struct TypeCountRow {
    test_type: String,
    count: i64,
}

#[derive(sqlx::FromRow)]
struct TestConfigRow {
    id: Uuid,
    test_id: Uuid,
    config_key: String,
    config_value: serde_json::Value,
    created_at: chrono::DateTime<Utc>,
}

impl From<TestConfigRow> for TestConfigEntry {
    fn from(row: TestConfigRow) -> Self {
        Self {
            id: row.id,
            test_id: row.test_id,
            config_key: row.config_key,
            config_value: row.config_value,
            created_at: row.created_at,
        }
    }
}

#[derive(sqlx::FromRow)]
struct TestResultRow {
    id: Uuid,
    test_id: Uuid,
    metric_name: String,
    metric_value: f64,
    percentile: Option<f64>,
    recorded_at: chrono::DateTime<Utc>,
}

impl From<TestResultRow> for TestResultMetric {
    fn from(row: TestResultRow) -> Self {
        Self {
            id: row.id,
            test_id: row.test_id,
            metric_name: row.metric_name,
            metric_value: row.metric_value,
            percentile: row.percentile,
            recorded_at: row.recorded_at,
        }
    }
}

#[derive(sqlx::FromRow)]
struct PercentileRow {
    metric_name: String,
    p50: Option<f64>,
    p90: Option<f64>,
    p95: Option<f64>,
    p99: Option<f64>,
    avg: f64,
    min_val: f64,
    max_val: f64,
}

#[derive(sqlx::FromRow)]
struct MetricCompareRow {
    metric_name: String,
    value_1: f64,
    value_2: f64,
}
