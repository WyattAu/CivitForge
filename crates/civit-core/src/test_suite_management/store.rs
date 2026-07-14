use super::types::*;
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

pub struct TestSuiteStore {
    pool: PgPool,
}

impl TestSuiteStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create_suite(
        &self,
        repo_id: Uuid,
        req: CreateTestSuiteRequest,
    ) -> Result<TestSuite, sqlx::Error> {
        let id = Uuid::new_v4();
        let now = Utc::now();
        let config = serde_json::to_value(&req.config.unwrap_or_default())
            .unwrap_or(serde_json::json!({}));
        let config_parsed: TestSuiteConfig =
            serde_json::from_value(config.clone()).unwrap_or_default();
        let description = req.description.unwrap_or_default();
        let enabled = req.enabled.unwrap_or(true);

        sqlx::query(
            r#"INSERT INTO test_suites (id, repo_id, name, description, test_type, config, enabled, created_at)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8)"#,
        )
        .bind(id)
        .bind(repo_id)
        .bind(&req.name)
        .bind(&description)
        .bind(&req.test_type)
        .bind(&config)
        .bind(enabled)
        .bind(now)
        .execute(&self.pool)
        .await?;

        Ok(TestSuite {
            id,
            repo_id,
            name: req.name,
            description,
            test_type: req.test_type,
            config: config_parsed,
            enabled,
            created_at: now,
        })
    }

    pub async fn get_suite(&self, id: Uuid) -> Result<Option<TestSuite>, sqlx::Error> {
        let row = sqlx::query_as::<_, SuiteRow>(
            r#"SELECT id, repo_id, name, description, test_type, config, enabled, created_at
               FROM test_suites WHERE id = $1"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(TestSuite::from))
    }

    pub async fn list_suites(
        &self,
        repo_id: Uuid,
        test_type: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<TestSuite>, sqlx::Error> {
        let rows = sqlx::query_as::<_, SuiteRow>(
            r#"SELECT id, repo_id, name, description, test_type, config, enabled, created_at
               FROM test_suites
               WHERE repo_id = $1
                 AND ($2::varchar IS NULL OR test_type = $2)
               ORDER BY created_at DESC
               LIMIT $3 OFFSET $4"#,
        )
        .bind(repo_id)
        .bind(test_type)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(TestSuite::from).collect())
    }

    pub async fn update_suite(
        &self,
        id: Uuid,
        req: UpdateTestSuiteRequest,
    ) -> Result<TestSuite, sqlx::Error> {
        let config_val = req
            .config
            .map(|c| serde_json::to_value(c).unwrap_or(serde_json::json!({})));

        let row = sqlx::query_as::<_, SuiteRow>(
            r#"UPDATE test_suites SET
               name = COALESCE($2, name),
               description = COALESCE($3, description),
               test_type = COALESCE($4, test_type),
               config = COALESCE($5, config),
               enabled = COALESCE($6, enabled)
               WHERE id = $1
               RETURNING id, repo_id, name, description, test_type, config, enabled, created_at"#,
        )
        .bind(id)
        .bind(&req.name)
        .bind(&req.description)
        .bind(&req.test_type)
        .bind(config_val)
        .bind(req.enabled)
        .fetch_one(&self.pool)
        .await?;

        Ok(TestSuite::from(row))
    }

    pub async fn delete_suite(&self, id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM test_suites WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn create_run(
        &self,
        req: CreateTestRunRequest,
    ) -> Result<TestRun, sqlx::Error> {
        let id = Uuid::new_v4();
        let now = Utc::now();

        sqlx::query(
            r#"INSERT INTO test_runs (id, suite_id, status, started_at)
               VALUES ($1, $2, $3, $4)"#,
        )
        .bind(id)
        .bind(req.suite_id)
        .bind(TestRunStatus::Pending.to_string())
        .bind(now)
        .execute(&self.pool)
        .await?;

        Ok(TestRun {
            id,
            suite_id: req.suite_id,
            status: TestRunStatus::Pending,
            total_tests: 0,
            passed_tests: 0,
            failed_tests: 0,
            skipped_tests: 0,
            duration_ms: 0,
            started_at: now,
            completed_at: None,
        })
    }

    pub async fn get_run(&self, id: Uuid) -> Result<Option<TestRun>, sqlx::Error> {
        let row = sqlx::query_as::<_, RunRow>(
            r#"SELECT id, suite_id, status, total_tests, passed_tests, failed_tests,
                      skipped_tests, duration_ms, started_at, completed_at
               FROM test_runs WHERE id = $1"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(TestRun::from))
    }

    pub async fn list_runs(
        &self,
        suite_id: Uuid,
        status: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<TestRun>, sqlx::Error> {
        let rows = sqlx::query_as::<_, RunRow>(
            r#"SELECT id, suite_id, status, total_tests, passed_tests, failed_tests,
                      skipped_tests, duration_ms, started_at, completed_at
               FROM test_runs
               WHERE suite_id = $1
                 AND ($2::varchar IS NULL OR status = $2)
               ORDER BY started_at DESC
               LIMIT $3 OFFSET $4"#,
        )
        .bind(suite_id)
        .bind(status)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(TestRun::from).collect())
    }

    pub async fn update_run(
        &self,
        id: Uuid,
        status: TestRunStatus,
        total_tests: Option<i32>,
        passed_tests: Option<i32>,
        failed_tests: Option<i32>,
        skipped_tests: Option<i32>,
        duration_ms: Option<i32>,
    ) -> Result<TestRun, sqlx::Error> {
        let completed_at = match status {
            TestRunStatus::Passed | TestRunStatus::Failed | TestRunStatus::Error | TestRunStatus::Skipped => {
                Some(Utc::now())
            }
            _ => None,
        };

        let row = sqlx::query_as::<_, RunRow>(
            r#"UPDATE test_runs SET
               status = $2,
               total_tests = COALESCE($3, total_tests),
               passed_tests = COALESCE($4, passed_tests),
               failed_tests = COALESCE($5, failed_tests),
               skipped_tests = COALESCE($6, skipped_tests),
               duration_ms = COALESCE($7, duration_ms),
               completed_at = COALESCE($8, completed_at)
               WHERE id = $1
               RETURNING id, suite_id, status, total_tests, passed_tests, failed_tests,
                         skipped_tests, duration_ms, started_at, completed_at"#,
        )
        .bind(id)
        .bind(status.to_string())
        .bind(total_tests)
        .bind(passed_tests)
        .bind(failed_tests)
        .bind(skipped_tests)
        .bind(duration_ms)
        .bind(completed_at)
        .fetch_one(&self.pool)
        .await?;

        Ok(TestRun::from(row))
    }

    pub async fn get_suite_summary(
        &self,
        repo_id: Uuid,
    ) -> Result<TestSuiteSummary, sqlx::Error> {
        let suite_row = sqlx::query_as::<_, SuiteSummaryRow>(
            r#"SELECT
                COUNT(*) as total_suites,
                COUNT(*) FILTER (WHERE enabled = true) as enabled_suites
               FROM test_suites
               WHERE repo_id = $1"#,
        )
        .bind(repo_id)
        .fetch_one(&self.pool)
        .await?;

        let run_row = sqlx::query_as::<_, RunSummaryRow>(
            r#"SELECT
                COUNT(*) as total_runs,
                COUNT(*) FILTER (WHERE status = 'passed') as passed_runs,
                COUNT(*) FILTER (WHERE status = 'failed') as failed_runs
               FROM test_runs tr
               JOIN test_suites ts ON tr.suite_id = ts.id
               WHERE ts.repo_id = $1"#,
        )
        .bind(repo_id)
        .fetch_one(&self.pool)
        .await?;

        let type_rows = sqlx::query_as::<_, TypeCountRow>(
            r#"SELECT test_type, COUNT(*) as count
               FROM test_suites
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

        Ok(TestSuiteSummary {
            total_suites: suite_row.total_suites,
            enabled_suites: suite_row.enabled_suites,
            total_runs: run_row.total_runs,
            passed_runs: run_row.passed_runs,
            failed_runs: run_row.failed_runs,
            by_type,
        })
    }

    pub async fn get_run_history(
        &self,
        suite_id: Uuid,
        days: i64,
    ) -> Result<TestRunHistory, sqlx::Error> {
        let runs: Vec<TestRun> = self
            .list_runs(suite_id, None, 100, 0)
            .await?
            .into_iter()
            .filter(|r| {
                let age = Utc::now() - r.started_at;
                age.num_days() <= days
            })
            .collect();

        let total_runs = runs.len() as f64;
        let passed_runs = runs
            .iter()
            .filter(|r| r.status == TestRunStatus::Passed)
            .count() as f64;
        let pass_rate = if total_runs > 0.0 {
            passed_runs / total_runs * 100.0
        } else {
            0.0
        };
        let avg_duration_ms = if total_runs > 0.0 {
            runs.iter().map(|r| r.duration_ms as f64).sum::<f64>() / total_runs
        } else {
            0.0
        };

        let trend_rows = sqlx::query_as::<_, TrendRow>(
            r#"SELECT
                DATE(started_at) as date,
                COUNT(*) as total_runs,
                COUNT(*) FILTER (WHERE status = 'passed') as passed_runs,
                COUNT(*) FILTER (WHERE status = 'failed') as failed_runs,
                COALESCE(AVG(duration_ms), 0) as avg_duration_ms
               FROM test_runs
               WHERE suite_id = $1
                 AND started_at >= NOW() - ($2 || ' days')::INTERVAL
               GROUP BY DATE(started_at)
               ORDER BY DATE(started_at) DESC"#,
        )
        .bind(suite_id)
        .bind(days)
        .fetch_all(&self.pool)
        .await?;

        let trend = trend_rows.into_iter().map(TestRunTrend::from).collect();

        Ok(TestRunHistory {
            suite_id,
            runs,
            pass_rate,
            avg_duration_ms,
            trend,
        })
    }
}

#[derive(sqlx::FromRow)]
struct SuiteRow {
    id: Uuid,
    repo_id: Uuid,
    name: String,
    description: String,
    test_type: String,
    config: serde_json::Value,
    enabled: bool,
    created_at: chrono::DateTime<Utc>,
}

impl From<SuiteRow> for TestSuite {
    fn from(row: SuiteRow) -> Self {
        Self {
            id: row.id,
            repo_id: row.repo_id,
            name: row.name,
            description: row.description,
            test_type: row.test_type,
            config: serde_json::from_value(row.config).unwrap_or_default(),
            enabled: row.enabled,
            created_at: row.created_at,
        }
    }
}

#[derive(sqlx::FromRow)]
struct RunRow {
    id: Uuid,
    suite_id: Uuid,
    status: String,
    total_tests: i32,
    passed_tests: i32,
    failed_tests: i32,
    skipped_tests: i32,
    duration_ms: i32,
    started_at: chrono::DateTime<Utc>,
    completed_at: Option<chrono::DateTime<Utc>>,
}

impl From<RunRow> for TestRun {
    fn from(row: RunRow) -> Self {
        Self {
            id: row.id,
            suite_id: row.suite_id,
            status: row.status.parse().unwrap_or(TestRunStatus::Pending),
            total_tests: row.total_tests,
            passed_tests: row.passed_tests,
            failed_tests: row.failed_tests,
            skipped_tests: row.skipped_tests,
            duration_ms: row.duration_ms,
            started_at: row.started_at,
            completed_at: row.completed_at,
        }
    }
}

#[derive(sqlx::FromRow)]
struct SuiteSummaryRow {
    total_suites: i64,
    enabled_suites: i64,
}

#[derive(sqlx::FromRow)]
struct RunSummaryRow {
    total_runs: i64,
    passed_runs: i64,
    failed_runs: i64,
}

#[derive(sqlx::FromRow)]
struct TypeCountRow {
    test_type: String,
    count: i64,
}

#[derive(sqlx::FromRow)]
struct TrendRow {
    date: chrono::NaiveDate,
    total_runs: i64,
    passed_runs: i64,
    failed_runs: i64,
    avg_duration_ms: f64,
}

impl From<TrendRow> for TestRunTrend {
    fn from(row: TrendRow) -> Self {
        Self {
            date: row.date,
            total_runs: row.total_runs,
            passed_runs: row.passed_runs,
            failed_runs: row.failed_runs,
            avg_duration_ms: row.avg_duration_ms,
        }
    }
}
