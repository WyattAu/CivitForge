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
