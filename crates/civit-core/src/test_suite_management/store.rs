use super::types::*;
use chrono::Utc;
use sqlx::{PgPool, QueryBuilder};
use uuid::Uuid;
use std::future::Future;
use std::pin::Pin;

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

    pub async fn create_config(
        &self,
        suite_id: Uuid,
        req: CreateTestSuiteConfigRequest,
    ) -> Result<TestSuiteConfiguration, sqlx::Error> {
        let id = Uuid::new_v4();
        let now = Utc::now();

        sqlx::query(
            r#"INSERT INTO test_suite_configurations (id, suite_id, config_key, config_value, created_at)
               VALUES ($1, $2, $3, $4, $5)
               ON CONFLICT (suite_id, config_key) DO UPDATE SET config_value = $4"#,
        )
        .bind(id)
        .bind(suite_id)
        .bind(&req.config_key)
        .bind(&req.config_value)
        .bind(now)
        .execute(&self.pool)
        .await?;

        Ok(TestSuiteConfiguration {
            id,
            suite_id,
            config_key: req.config_key,
            config_value: req.config_value,
            created_at: now,
        })
    }

    pub async fn get_configs(
        &self,
        suite_id: Uuid,
    ) -> Result<Vec<TestSuiteConfiguration>, sqlx::Error> {
        let rows = sqlx::query_as::<_, ConfigRow>(
            r#"SELECT id, suite_id, config_key, config_value, created_at
               FROM test_suite_configurations
               WHERE suite_id = $1
               ORDER BY config_key"#,
        )
        .bind(suite_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(TestSuiteConfiguration::from).collect())
    }

    pub async fn update_config(
        &self,
        suite_id: Uuid,
        config_key: &str,
        req: UpdateTestSuiteConfigRequest,
    ) -> Result<TestSuiteConfiguration, sqlx::Error> {
        let row = sqlx::query_as::<_, ConfigRow>(
            r#"UPDATE test_suite_configurations
               SET config_value = $3
               WHERE suite_id = $1 AND config_key = $2
               RETURNING id, suite_id, config_key, config_value, created_at"#,
        )
        .bind(suite_id)
        .bind(config_key)
        .bind(&req.config_value)
        .fetch_one(&self.pool)
        .await?;

        Ok(TestSuiteConfiguration::from(row))
    }

    pub async fn delete_config(
        &self,
        suite_id: Uuid,
        config_key: &str,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(
            r#"DELETE FROM test_suite_configurations WHERE suite_id = $1 AND config_key = $2"#,
        )
        .bind(suite_id)
        .bind(config_key)
        .execute(&self.pool)
        .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn create_notification(
        &self,
        suite_id: Uuid,
        req: CreateTestSuiteNotificationRequest,
    ) -> Result<TestSuiteNotification, sqlx::Error> {
        let id = Uuid::new_v4();
        let now = Utc::now();
        let config = req.config.unwrap_or(serde_json::json!({}));
        let enabled = req.enabled.unwrap_or(true);

        sqlx::query(
            r#"INSERT INTO test_suite_notifications (id, suite_id, notification_type, config, enabled, created_at)
               VALUES ($1, $2, $3, $4, $5, $6)"#,
        )
        .bind(id)
        .bind(suite_id)
        .bind(&req.notification_type)
        .bind(&config)
        .bind(enabled)
        .bind(now)
        .execute(&self.pool)
        .await?;

        Ok(TestSuiteNotification {
            id,
            suite_id,
            notification_type: req.notification_type,
            config,
            enabled,
            created_at: now,
        })
    }

    pub async fn list_notifications(
        &self,
        suite_id: Uuid,
    ) -> Result<Vec<TestSuiteNotification>, sqlx::Error> {
        let rows = sqlx::query_as::<_, NotificationRow>(
            r#"SELECT id, suite_id, notification_type, config, enabled, created_at
               FROM test_suite_notifications
               WHERE suite_id = $1
               ORDER BY created_at DESC"#,
        )
        .bind(suite_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(TestSuiteNotification::from).collect())
    }

    pub async fn update_notification(
        &self,
        id: Uuid,
        req: UpdateTestSuiteNotificationRequest,
    ) -> Result<TestSuiteNotification, sqlx::Error> {
        let row = sqlx::query_as::<_, NotificationRow>(
            r#"UPDATE test_suite_notifications SET
               notification_type = COALESCE($2, notification_type),
               config = COALESCE($3, config),
               enabled = COALESCE($4, enabled)
               WHERE id = $1
               RETURNING id, suite_id, notification_type, config, enabled, created_at"#,
        )
        .bind(id)
        .bind(&req.notification_type)
        .bind(&req.config)
        .bind(req.enabled)
        .fetch_one(&self.pool)
        .await?;

        Ok(TestSuiteNotification::from(row))
    }

    pub async fn delete_notification(&self, id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM test_suite_notifications WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn create_tag(
        &self,
        suite_id: Uuid,
        req: CreateTestSuiteTagRequest,
    ) -> Result<TestSuiteTag, sqlx::Error> {
        let id = Uuid::new_v4();
        let now = Utc::now();

        sqlx::query(
            r#"INSERT INTO test_suite_tags (id, suite_id, tag, created_at)
               VALUES ($1, $2, $3, $4)
               ON CONFLICT (suite_id, tag) DO NOTHING"#,
        )
        .bind(id)
        .bind(suite_id)
        .bind(&req.tag)
        .bind(now)
        .execute(&self.pool)
        .await?;

        Ok(TestSuiteTag {
            id,
            suite_id,
            tag: req.tag,
            created_at: now,
        })
    }

    pub async fn list_tags(&self, suite_id: Uuid) -> Result<Vec<TestSuiteTag>, sqlx::Error> {
        let rows = sqlx::query_as::<_, TagRow>(
            r#"SELECT id, suite_id, tag, created_at
               FROM test_suite_tags
               WHERE suite_id = $1
               ORDER BY tag"#,
        )
        .bind(suite_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(TestSuiteTag::from).collect())
    }

    pub async fn delete_tag(&self, id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM test_suite_tags WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn list_suites_by_tag(
        &self,
        repo_id: Uuid,
        tag: &str,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<TestSuite>, sqlx::Error> {
        let rows = sqlx::query_as::<_, SuiteRow>(
            r#"SELECT ts.id, ts.repo_id, ts.name, ts.description, ts.test_type, ts.config, ts.enabled, ts.created_at
               FROM test_suites ts
               JOIN test_suite_tags tst ON ts.id = tst.suite_id
               WHERE ts.repo_id = $1 AND tst.tag = $2
               ORDER BY ts.created_at DESC
               LIMIT $3 OFFSET $4"#,
        )
        .bind(repo_id)
        .bind(tag)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(TestSuite::from).collect())
    }

    pub async fn list_suites_by_tags(
        &self,
        repo_id: Uuid,
        tags: &[String],
        limit: i64,
        offset: i64,
    ) -> Result<Vec<TestSuite>, sqlx::Error> {
        let rows = sqlx::query_as::<_, SuiteRow>(
            r#"SELECT DISTINCT ts.id, ts.repo_id, ts.name, ts.description, ts.test_type, ts.config, ts.enabled, ts.created_at
               FROM test_suites ts
               JOIN test_suite_tags tst ON ts.id = tst.suite_id
               WHERE ts.repo_id = $1 AND tst.tag = ANY($2)
               ORDER BY ts.created_at DESC
               LIMIT $3 OFFSET $4"#,
        )
        .bind(repo_id)
        .bind(tags)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(TestSuite::from).collect())
    }

    pub async fn create_dependency(
        &self,
        suite_id: Uuid,
        req: CreateTestSuiteDependencyRequest,
    ) -> Result<TestSuiteDependency, sqlx::Error> {
        let id = Uuid::new_v4();
        let now = Utc::now();
        let dependency_type = req.dependency_type.unwrap_or_else(|| "blocks".to_string());

        sqlx::query(
            r#"INSERT INTO test_suite_dependencies (id, suite_id, depends_on_suite_id, dependency_type, created_at)
               VALUES ($1, $2, $3, $4, $5)
               ON CONFLICT (suite_id, depends_on_suite_id) DO NOTHING"#,
        )
        .bind(id)
        .bind(suite_id)
        .bind(req.depends_on_suite_id)
        .bind(&dependency_type)
        .bind(now)
        .execute(&self.pool)
        .await?;

        Ok(TestSuiteDependency {
            id,
            suite_id,
            depends_on_suite_id: req.depends_on_suite_id,
            dependency_type,
            created_at: now,
        })
    }

    pub async fn list_dependencies(
        &self,
        suite_id: Uuid,
    ) -> Result<Vec<TestSuiteDependency>, sqlx::Error> {
        let rows = sqlx::query_as::<_, DependencyRow>(
            r#"SELECT id, suite_id, depends_on_suite_id, dependency_type, created_at
               FROM test_suite_dependencies
               WHERE suite_id = $1
               ORDER BY created_at"#,
        )
        .bind(suite_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(TestSuiteDependency::from).collect())
    }

    pub async fn list_reverse_dependencies(
        &self,
        suite_id: Uuid,
    ) -> Result<Vec<TestSuiteDependency>, sqlx::Error> {
        let rows = sqlx::query_as::<_, DependencyRow>(
            r#"SELECT id, suite_id, depends_on_suite_id, dependency_type, created_at
               FROM test_suite_dependencies
               WHERE depends_on_suite_id = $1
               ORDER BY created_at"#,
        )
        .bind(suite_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(TestSuiteDependency::from).collect())
    }

    pub async fn delete_dependency(&self, id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM test_suite_dependencies WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn get_dependency_summary(
        &self,
        repo_id: Uuid,
    ) -> Result<TestSuiteDependencySummary, sqlx::Error> {
        let total_row = sqlx::query_as::<_, DependencySummaryRow>(
            r#"SELECT
                COUNT(*) as total_dependencies,
                0 as circular_dependencies_detected,
                COUNT(DISTINCT suite_id) as suites_with_dependencies,
                (SELECT COUNT(*) FROM test_suites WHERE repo_id = $1) - COUNT(DISTINCT suite_id) as suites_without_dependencies
               FROM test_suite_dependencies tsd
               JOIN test_suites ts ON tsd.suite_id = ts.id
               WHERE ts.repo_id = $1"#,
        )
        .bind(repo_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(TestSuiteDependencySummary {
            total_dependencies: total_row.total_dependencies,
            circular_dependencies_detected: total_row.circular_dependencies_detected,
            suites_with_dependencies: total_row.suites_with_dependencies,
            suites_without_dependencies: total_row.suites_without_dependencies,
        })
    }

    pub async fn get_execution_order(
        &self,
        repo_id: Uuid,
    ) -> Result<ExecutionPlan, sqlx::Error> {
        let suites = self.list_suites(repo_id, None, 1000, 0).await?;
        let mut execution_orders = Vec::new();
        let mut visited = std::collections::HashSet::new();
        let mut in_stack = std::collections::HashSet::new();

        for suite in &suites {
            if !visited.contains(&suite.id) {
                Box::pin(self.topological_sort(
                    suite.id,
                    &suites,
                    &mut visited,
                    &mut in_stack,
                    &mut execution_orders,
                ))
                .await?;
            }
        }

        let mut groups = Vec::new();
        let mut current_group = Vec::new();
        let mut last_deps = Vec::new();

        for order in execution_orders {
            if order.dependencies != last_deps && !current_group.is_empty() {
                groups.push(current_group);
                current_group = Vec::new();
            }
            last_deps = order.dependencies.clone();
            current_group.push(order);
        }
        if !current_group.is_empty() {
            groups.push(current_group);
        }

        Ok(ExecutionPlan {
            repo_id,
            execution_groups: groups,
            total_suites: suites.len() as i32,
            estimated_duration_ms: 0,
        })
    }

    fn topological_sort<'a>(
        &'a self,
        suite_id: Uuid,
        all_suites: &'a [TestSuite],
        visited: &'a mut std::collections::HashSet<Uuid>,
        in_stack: &'a mut std::collections::HashSet<Uuid>,
        result: &'a mut Vec<TestExecutionOrder>,
    ) -> Pin<Box<dyn Future<Output = Result<(), sqlx::Error>> + Send + 'a>> {
        Box::pin(async move {
            if in_stack.contains(&suite_id) {
                return Ok(());
            }
            if visited.contains(&suite_id) {
                return Ok(());
            }

            visited.insert(suite_id);
            in_stack.insert(suite_id);

            let deps = self.list_dependencies(suite_id).await?;
            let mut dep_ids = Vec::new();

            for dep in &deps {
                dep_ids.push(dep.depends_on_suite_id);
                Box::pin(self.topological_sort(
                    dep.depends_on_suite_id,
                    all_suites,
                    visited,
                    in_stack,
                    result,
                ))
                .await?;
            }

            in_stack.remove(&suite_id);

            if let Some(suite) = all_suites.iter().find(|s| s.id == suite_id) {
                let can_run_parallel = deps.is_empty();
                result.push(TestExecutionOrder {
                    suite_id,
                    suite_name: suite.name.clone(),
                    order: result.len() as i32,
                    dependencies: dep_ids,
                    can_run_parallel,
                });
            }

            Ok(())
        })
    }

    pub async fn get_analytics(
        &self,
        repo_id: Uuid,
    ) -> Result<TestSuiteAnalytics, sqlx::Error> {
        let summary = self.get_suite_summary(repo_id).await?;

        let activity_rows = sqlx::query_as::<_, ActivityRow>(
            r#"SELECT ts.id as suite_id, ts.name as suite_name,
                COUNT(tr.id) as run_count,
                MAX(tr.started_at) as last_run_at
               FROM test_suites ts
               LEFT JOIN test_runs tr ON ts.id = tr.suite_id
               WHERE ts.repo_id = $1
               GROUP BY ts.id, ts.name
               ORDER BY run_count DESC
               LIMIT 10"#,
        )
        .bind(repo_id)
        .fetch_all(&self.pool)
        .await?;

        let most_active_suites = activity_rows
            .into_iter()
            .map(SuiteActivity::from)
            .collect();

        let failure_rows = sqlx::query_as::<_, FailureTrendRow>(
            r#"SELECT
                DATE(tr.started_at) as date,
                COUNT(*) FILTER (WHERE tr.status = 'failed') as failure_count,
                CASE WHEN COUNT(*) > 0 THEN
                    COUNT(*) FILTER (WHERE tr.status = 'failed')::double precision / COUNT(*)::double precision * 100.0
                ELSE 0.0 END as failure_rate
               FROM test_runs tr
               JOIN test_suites ts ON tr.suite_id = ts.id
               WHERE ts.repo_id = $1
                 AND tr.started_at >= NOW() - INTERVAL '30 days'
               GROUP BY DATE(tr.started_at)
               ORDER BY DATE(tr.started_at) DESC"#,
        )
        .bind(repo_id)
        .fetch_all(&self.pool)
        .await?;

        let failure_trends = failure_rows
            .into_iter()
            .map(FailureTrend::from)
            .collect();

        let avg_pass_rate = if summary.total_runs > 0 {
            summary.passed_runs as f64 / summary.total_runs as f64 * 100.0
        } else {
            0.0
        };

        Ok(TestSuiteAnalytics {
            total_suites: summary.total_suites,
            total_runs: summary.total_runs,
            avg_pass_rate,
            avg_duration_ms: 0.0,
            most_active_suites,
            failure_trends,
        })
    }

    pub async fn create_metric(
        &self,
        suite_id: Uuid,
        req: CreateTestSuiteMetricRequest,
    ) -> Result<TestSuiteMetric, sqlx::Error> {
        let id = Uuid::new_v4();
        let now = Utc::now();

        sqlx::query(
            r#"INSERT INTO test_suite_metrics (id, suite_id, metric_name, metric_value, measured_at)
               VALUES ($1, $2, $3, $4, $5)"#,
        )
        .bind(id)
        .bind(suite_id)
        .bind(&req.metric_name)
        .bind(req.metric_value)
        .bind(now)
        .execute(&self.pool)
        .await?;

        Ok(TestSuiteMetric {
            id,
            suite_id,
            metric_name: req.metric_name,
            metric_value: req.metric_value,
            measured_at: now,
        })
    }

    pub async fn list_metrics(
        &self,
        suite_id: Uuid,
        metric_name: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<TestSuiteMetric>, sqlx::Error> {
        let rows = sqlx::query_as::<_, MetricRow>(
            r#"SELECT id, suite_id, metric_name, metric_value, measured_at
               FROM test_suite_metrics
               WHERE suite_id = $1
                 AND ($2::varchar IS NULL OR metric_name = $2)
               ORDER BY measured_at DESC
               LIMIT $3 OFFSET $4"#,
        )
        .bind(suite_id)
        .bind(metric_name)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(TestSuiteMetric::from).collect())
    }

    pub async fn get_metric_summary(
        &self,
        suite_id: Uuid,
        metric_name: &str,
    ) -> Result<Option<MetricSummary>, sqlx::Error> {
        let row = sqlx::query_as::<_, MetricSummaryRow>(
            r#"SELECT
                metric_name,
                (SELECT metric_value FROM test_suite_metrics WHERE suite_id = $1 AND metric_name = $2 ORDER BY measured_at DESC LIMIT 1) as latest_value,
                COALESCE(AVG(metric_value), 0) as avg_value,
                COALESCE(MIN(metric_value), 0) as min_value,
                COALESCE(MAX(metric_value), 0) as max_value,
                COUNT(*) as measurement_count,
                COUNT(DISTINCT suite_id) as suites_affected
               FROM test_suite_metrics
               WHERE suite_id = $1 AND metric_name = $2"#,
        )
        .bind(suite_id)
        .bind(metric_name)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(MetricSummary::from))
    }

    pub async fn create_baseline(
        &self,
        suite_id: Uuid,
        req: CreateTestSuiteBaselineRequest,
    ) -> Result<TestSuiteBaseline, sqlx::Error> {
        let id = Uuid::new_v4();
        let now = Utc::now();
        let threshold_percent = req.threshold_percent.unwrap_or(10.0);

        sqlx::query(
            r#"INSERT INTO test_suite_baselines (id, suite_id, metric_name, baseline_value, threshold_percent, created_at)
               VALUES ($1, $2, $3, $4, $5, $6)
               ON CONFLICT (suite_id, metric_name) DO UPDATE SET baseline_value = $4, threshold_percent = $5"#,
        )
        .bind(id)
        .bind(suite_id)
        .bind(&req.metric_name)
        .bind(req.baseline_value)
        .bind(threshold_percent)
        .bind(now)
        .execute(&self.pool)
        .await?;

        Ok(TestSuiteBaseline {
            id,
            suite_id,
            metric_name: req.metric_name,
            baseline_value: req.baseline_value,
            threshold_percent,
            created_at: now,
        })
    }

    pub async fn list_baselines(
        &self,
        suite_id: Uuid,
    ) -> Result<Vec<TestSuiteBaseline>, sqlx::Error> {
        let rows = sqlx::query_as::<_, BaselineRow>(
            r#"SELECT id, suite_id, metric_name, baseline_value, threshold_percent, created_at
               FROM test_suite_baselines
               WHERE suite_id = $1
               ORDER BY metric_name"#,
        )
        .bind(suite_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(TestSuiteBaseline::from).collect())
    }

    pub async fn update_baseline(
        &self,
        id: Uuid,
        req: UpdateTestSuiteBaselineRequest,
    ) -> Result<TestSuiteBaseline, sqlx::Error> {
        if let Some(value) = req.baseline_value {
            sqlx::query(r#"UPDATE test_suite_baselines SET baseline_value = $1 WHERE id = $2"#)
                .bind(value)
                .bind(id)
                .execute(&self.pool)
                .await?;
        }
        if let Some(threshold) = req.threshold_percent {
            sqlx::query(r#"UPDATE test_suite_baselines SET threshold_percent = $1 WHERE id = $2"#)
                .bind(threshold)
                .bind(id)
                .execute(&self.pool)
                .await?;
        }

        let row = sqlx::query_as::<_, BaselineRow>(
            r#"SELECT id, suite_id, metric_name, baseline_value, threshold_percent, created_at
               FROM test_suite_baselines WHERE id = $1"#,
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await?;

        Ok(TestSuiteBaseline::from(row))
    }

    pub async fn delete_baseline(&self, id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(r#"DELETE FROM test_suite_baselines WHERE id = $1"#)
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn detect_regressions(
        &self,
        suite_id: Uuid,
    ) -> Result<Vec<TestSuiteRegression>, sqlx::Error> {
        let baselines = self.list_baselines(suite_id).await?;
        let mut regressions = Vec::new();

        for baseline in baselines {
            if let Some(latest_metric) = sqlx::query_as::<_, MetricRow>(
                r#"SELECT id, suite_id, metric_name, metric_value, measured_at
                   FROM test_suite_metrics
                   WHERE suite_id = $1 AND metric_name = $2
                   ORDER BY measured_at DESC
                   LIMIT 1"#,
            )
            .bind(suite_id)
            .bind(&baseline.metric_name)
            .fetch_optional(&self.pool)
            .await?
            {
                let current_value = latest_metric.metric_value;
                let regression_percent = if baseline.baseline_value != 0.0 {
                    ((current_value - baseline.baseline_value) / baseline.baseline_value) * 100.0
                } else {
                    0.0
                };

                if regression_percent > baseline.threshold_percent {
                    let severity = if regression_percent > 50.0 {
                        "critical"
                    } else if regression_percent > 25.0 {
                        "high"
                    } else if regression_percent > 10.0 {
                        "medium"
                    } else {
                        "low"
                    };

                    regressions.push(TestSuiteRegression {
                        id: Uuid::new_v4(),
                        baseline_id: baseline.id,
                        metric_name: baseline.metric_name,
                        baseline_value: baseline.baseline_value,
                        current_value,
                        regression_percent,
                        threshold_percent: baseline.threshold_percent,
                        status: "open".to_string(),
                        detected_at: Utc::now(),
                    });
                }
            }
        }

        Ok(regressions)
    }

    pub async fn get_metrics_summary(
        &self,
        suite_id: Uuid,
    ) -> Result<TestSuiteMetricsSummary, sqlx::Error> {
        let metrics = self.list_metrics(suite_id, None, 1000, 0).await?;
        let baselines = self.list_baselines(suite_id).await?;
        let regressions = self.detect_regressions(suite_id).await?;

        let active_regressions = regressions.iter().filter(|r| r.status == "open").count() as i64;
        let resolved_regressions = regressions.iter().filter(|r| r.status == "resolved").count() as i64;

        Ok(TestSuiteMetricsSummary {
            suite_id,
            total_metrics: metrics.len() as i64,
            total_baselines: baselines.len() as i64,
            active_regressions,
            resolved_regressions,
            metrics,
            baselines,
        })
    }

    pub async fn get_performance_report(
        &self,
        suite_id: Uuid,
    ) -> Result<TestSuitePerformanceReport, sqlx::Error> {
        let suite = self.get_suite(suite_id).await?.ok_or_else(|| sqlx::Error::RowNotFound)?;
        let metrics_summary = self.get_metrics_summary(suite_id).await?;
        let regressions = self.detect_regressions(suite_id).await?;

        let mut alerts = Vec::new();
        for regression in &regressions {
            alerts.push(TestSuitePerformanceAlert {
                id: Uuid::new_v4(),
                suite_id,
                metric_name: regression.metric_name.clone(),
                baseline_value: regression.baseline_value,
                current_value: regression.current_value,
                regression_percent: regression.regression_percent,
                threshold_percent: regression.threshold_percent,
                severity: if regression.regression_percent > 50.0 {
                    "critical".to_string()
                } else if regression.regression_percent > 25.0 {
                    "high".to_string()
                } else if regression.regression_percent > 10.0 {
                    "medium".to_string()
                } else {
                    "low".to_string()
                },
                message: format!(
                    "Performance regression detected: {} regressed by {:.1}% (threshold: {:.1}%)",
                    regression.metric_name, regression.regression_percent, regression.threshold_percent
                ),
                created_at: Utc::now(),
            });
        }

        let overall_score = if metrics_summary.total_baselines > 0 {
            let passing_baselines = metrics_summary.total_baselines - metrics_summary.active_regressions;
            (passing_baselines as f64 / metrics_summary.total_baselines as f64) * 100.0
        } else {
            100.0
        };

        let last_measured_at = metrics_summary.metrics.iter().map(|m| m.measured_at).max();

        Ok(TestSuitePerformanceReport {
            suite_id,
            suite_name: suite.name,
            metrics_summary,
            regressions,
            alerts,
            overall_score,
            last_measured_at,
        })
    }

    pub async fn create_metric_v2(
        &self,
        suite_id: Uuid,
        req: CreateTestSuiteMetricV2Request,
    ) -> Result<TestSuiteMetricV2, sqlx::Error> {
        let id = Uuid::new_v4();
        let now = Utc::now();

        sqlx::query(
            r#"INSERT INTO test_suite_metrics_v2 (id, suite_id, metric_name, metric_value, measured_at)
               VALUES ($1, $2, $3, $4, $5)"#,
        )
        .bind(id)
        .bind(suite_id)
        .bind(&req.metric_name)
        .bind(req.metric_value)
        .bind(now)
        .execute(&self.pool)
        .await?;

        Ok(TestSuiteMetricV2 {
            id,
            suite_id,
            metric_name: req.metric_name,
            metric_value: req.metric_value,
            measured_at: now,
        })
    }

    pub async fn list_metrics_v2(
        &self,
        suite_id: Uuid,
        metric_name: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<TestSuiteMetricV2>, sqlx::Error> {
        let rows = sqlx::query_as::<_, MetricV2Row>(
            r#"SELECT id, suite_id, metric_name, metric_value, measured_at
               FROM test_suite_metrics_v2
               WHERE suite_id = $1
                 AND ($2::varchar IS NULL OR metric_name = $2)
               ORDER BY measured_at DESC
               LIMIT $3 OFFSET $4"#,
        )
        .bind(suite_id)
        .bind(metric_name)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(TestSuiteMetricV2::from).collect())
    }

    pub async fn get_metric_summary_v2(
        &self,
        suite_id: Uuid,
        metric_name: &str,
    ) -> Result<Option<MetricSummaryV2>, sqlx::Error> {
        let row = sqlx::query_as::<_, MetricSummaryV2Row>(
            r#"SELECT
                metric_name,
                (SELECT metric_value FROM test_suite_metrics_v2 WHERE suite_id = $1 AND metric_name = $2 ORDER BY measured_at DESC LIMIT 1) as latest_value,
                COALESCE(AVG(metric_value), 0) as avg_value,
                COALESCE(MIN(metric_value), 0) as min_value,
                COALESCE(MAX(metric_value), 0) as max_value,
                COUNT(*) as measurement_count,
                COUNT(DISTINCT suite_id) as suites_affected
               FROM test_suite_metrics_v2
               WHERE suite_id = $1 AND metric_name = $2"#,
        )
        .bind(suite_id)
        .bind(metric_name)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(MetricSummaryV2::from))
    }

    pub async fn create_baseline_v2(
        &self,
        suite_id: Uuid,
        req: CreateTestSuiteBaselineV2Request,
    ) -> Result<TestSuiteBaselineV2, sqlx::Error> {
        let id = Uuid::new_v4();
        let now = Utc::now();
        let threshold_percent = req.threshold_percent.unwrap_or(10.0);

        sqlx::query(
            r#"INSERT INTO test_suite_baselines_v2 (id, suite_id, metric_name, baseline_value, threshold_percent, created_at)
               VALUES ($1, $2, $3, $4, $5, $6)
               ON CONFLICT (suite_id, metric_name) DO UPDATE SET baseline_value = $4, threshold_percent = $5"#,
        )
        .bind(id)
        .bind(suite_id)
        .bind(&req.metric_name)
        .bind(req.baseline_value)
        .bind(threshold_percent)
        .bind(now)
        .execute(&self.pool)
        .await?;

        Ok(TestSuiteBaselineV2 {
            id,
            suite_id,
            metric_name: req.metric_name,
            baseline_value: req.baseline_value,
            threshold_percent,
            created_at: now,
        })
    }

    pub async fn list_baselines_v2(
        &self,
        suite_id: Uuid,
    ) -> Result<Vec<TestSuiteBaselineV2>, sqlx::Error> {
        let rows = sqlx::query_as::<_, BaselineV2Row>(
            r#"SELECT id, suite_id, metric_name, baseline_value, threshold_percent, created_at
               FROM test_suite_baselines_v2
               WHERE suite_id = $1
               ORDER BY metric_name"#,
        )
        .bind(suite_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(TestSuiteBaselineV2::from).collect())
    }

    pub async fn update_baseline_v2(
        &self,
        id: Uuid,
        req: UpdateTestSuiteBaselineV2Request,
    ) -> Result<TestSuiteBaselineV2, sqlx::Error> {
        if let Some(value) = req.baseline_value {
            sqlx::query(r#"UPDATE test_suite_baselines_v2 SET baseline_value = $1 WHERE id = $2"#)
                .bind(value)
                .bind(id)
                .execute(&self.pool)
                .await?;
        }
        if let Some(threshold) = req.threshold_percent {
            sqlx::query(r#"UPDATE test_suite_baselines_v2 SET threshold_percent = $1 WHERE id = $2"#)
                .bind(threshold)
                .bind(id)
                .execute(&self.pool)
                .await?;
        }

        let row = sqlx::query_as::<_, BaselineV2Row>(
            r#"SELECT id, suite_id, metric_name, baseline_value, threshold_percent, created_at
               FROM test_suite_baselines_v2 WHERE id = $1"#,
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await?;

        Ok(TestSuiteBaselineV2::from(row))
    }

    pub async fn delete_baseline_v2(&self, id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(r#"DELETE FROM test_suite_baselines_v2 WHERE id = $1"#)
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn detect_regressions_v2(
        &self,
        suite_id: Uuid,
    ) -> Result<Vec<TestSuiteRegressionV2>, sqlx::Error> {
        let baselines = self.list_baselines_v2(suite_id).await?;
        let mut regressions = Vec::new();

        for baseline in baselines {
            if let Some(latest_metric) = sqlx::query_as::<_, MetricV2Row>(
                r#"SELECT id, suite_id, metric_name, metric_value, measured_at
                   FROM test_suite_metrics_v2
                   WHERE suite_id = $1 AND metric_name = $2
                   ORDER BY measured_at DESC
                   LIMIT 1"#,
            )
            .bind(suite_id)
            .bind(&baseline.metric_name)
            .fetch_optional(&self.pool)
            .await?
            {
                let current_value = latest_metric.metric_value;
                let regression_percent = if baseline.baseline_value != 0.0 {
                    ((current_value - baseline.baseline_value) / baseline.baseline_value) * 100.0
                } else {
                    0.0
                };

                if regression_percent > baseline.threshold_percent {
                    let severity = if regression_percent > 50.0 {
                        "critical"
                    } else if regression_percent > 25.0 {
                        "high"
                    } else if regression_percent > 10.0 {
                        "medium"
                    } else {
                        "low"
                    };

                    regressions.push(TestSuiteRegressionV2 {
                        id: Uuid::new_v4(),
                        baseline_id: baseline.id,
                        metric_name: baseline.metric_name,
                        baseline_value: baseline.baseline_value,
                        current_value,
                        regression_percent,
                        threshold_percent: baseline.threshold_percent,
                        status: "open".to_string(),
                        detected_at: Utc::now(),
                    });
                }
            }
        }

        Ok(regressions)
    }

    pub async fn get_metrics_summary_v2(
        &self,
        suite_id: Uuid,
    ) -> Result<TestSuiteMetricsSummaryV2, sqlx::Error> {
        let metrics = self.list_metrics_v2(suite_id, None, 1000, 0).await?;
        let baselines = self.list_baselines_v2(suite_id).await?;
        let regressions = self.detect_regressions_v2(suite_id).await?;

        let active_regressions = regressions.iter().filter(|r| r.status == "open").count() as i64;
        let resolved_regressions = regressions.iter().filter(|r| r.status == "resolved").count() as i64;

        Ok(TestSuiteMetricsSummaryV2 {
            suite_id,
            total_metrics: metrics.len() as i64,
            total_baselines: baselines.len() as i64,
            active_regressions,
            resolved_regressions,
            metrics,
            baselines,
        })
    }

    pub async fn get_performance_report_v2(
        &self,
        suite_id: Uuid,
    ) -> Result<TestSuitePerformanceReportV2, sqlx::Error> {
        let suite = self.get_suite(suite_id).await?.ok_or_else(|| sqlx::Error::RowNotFound)?;
        let metrics_summary = self.get_metrics_summary_v2(suite_id).await?;
        let regressions = self.detect_regressions_v2(suite_id).await?;

        let mut alerts = Vec::new();
        for regression in &regressions {
            alerts.push(TestSuitePerformanceAlertV2 {
                id: Uuid::new_v4(),
                suite_id,
                metric_name: regression.metric_name.clone(),
                baseline_value: regression.baseline_value,
                current_value: regression.current_value,
                regression_percent: regression.regression_percent,
                threshold_percent: regression.threshold_percent,
                severity: if regression.regression_percent > 50.0 {
                    "critical".to_string()
                } else if regression.regression_percent > 25.0 {
                    "high".to_string()
                } else if regression.regression_percent > 10.0 {
                    "medium".to_string()
                } else {
                    "low".to_string()
                },
                message: format!(
                    "Performance regression detected: {} regressed by {:.1}% (threshold: {:.1}%)",
                    regression.metric_name, regression.regression_percent, regression.threshold_percent
                ),
                created_at: Utc::now(),
            });
        }

        let overall_score = if metrics_summary.total_baselines > 0 {
            let passing_baselines = metrics_summary.total_baselines - metrics_summary.active_regressions;
            (passing_baselines as f64 / metrics_summary.total_baselines as f64) * 100.0
        } else {
            100.0
        };

        let last_measured_at = metrics_summary.metrics.iter().map(|m| m.measured_at).max();

        Ok(TestSuitePerformanceReportV2 {
            suite_id,
            suite_name: suite.name,
            metrics_summary,
            regressions,
            alerts,
            overall_score,
            last_measured_at,
        })
    }

    pub async fn create_alert_config(
        &self,
        suite_id: Uuid,
        req: CreateTestSuiteAlertConfigRequest,
    ) -> Result<TestSuitePerformanceAlertConfig, sqlx::Error> {
        let id = Uuid::new_v4();
        let now = Utc::now();
        let enabled = req.enabled.unwrap_or(true);

        sqlx::query(
            r#"INSERT INTO test_suite_alert_configs (id, suite_id, alert_type, threshold, enabled, created_at)
               VALUES ($1, $2, $3, $4, $5, $6)"#,
        )
        .bind(id)
        .bind(suite_id)
        .bind(&req.alert_type)
        .bind(req.threshold)
        .bind(enabled)
        .bind(now)
        .execute(&self.pool)
        .await?;

        Ok(TestSuitePerformanceAlertConfig {
            id,
            suite_id,
            alert_type: req.alert_type,
            threshold: req.threshold,
            enabled,
            last_triggered_at: None,
            created_at: now,
        })
    }

    pub async fn get_alert_config(
        &self,
        id: Uuid,
    ) -> Result<Option<TestSuitePerformanceAlertConfig>, sqlx::Error> {
        let row = sqlx::query_as::<_, AlertConfigV2Row>(
            r#"SELECT id, suite_id, alert_type, threshold, enabled, last_triggered_at, created_at
               FROM test_suite_alert_configs WHERE id = $1"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(TestSuitePerformanceAlertConfig::from))
    }

    pub async fn list_alert_configs(
        &self,
        suite_id: Uuid,
    ) -> Result<Vec<TestSuitePerformanceAlertConfig>, sqlx::Error> {
        let rows = sqlx::query_as::<_, AlertConfigV2Row>(
            r#"SELECT id, suite_id, alert_type, threshold, enabled, last_triggered_at, created_at
               FROM test_suite_alert_configs
               WHERE suite_id = $1
               ORDER BY created_at"#,
        )
        .bind(suite_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(TestSuitePerformanceAlertConfig::from).collect())
    }

    pub async fn update_alert_config(
        &self,
        id: Uuid,
        req: UpdateTestSuiteAlertConfigRequest,
    ) -> Result<TestSuitePerformanceAlertConfig, sqlx::Error> {
        if let Some(ref alert_type) = req.alert_type {
            sqlx::query(r#"UPDATE test_suite_alert_configs SET alert_type = $1 WHERE id = $2"#)
                .bind(alert_type)
                .bind(id)
                .execute(&self.pool)
                .await?;
        }
        if let Some(threshold) = req.threshold {
            sqlx::query(r#"UPDATE test_suite_alert_configs SET threshold = $1 WHERE id = $2"#)
                .bind(threshold)
                .bind(id)
                .execute(&self.pool)
                .await?;
        }
        if let Some(enabled) = req.enabled {
            sqlx::query(r#"UPDATE test_suite_alert_configs SET enabled = $1 WHERE id = $2"#)
                .bind(enabled)
                .bind(id)
                .execute(&self.pool)
                .await?;
        }

        self.get_alert_config(id).await?.ok_or_else(|| sqlx::Error::RowNotFound)
    }

    pub async fn delete_alert_config(&self, id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(r#"DELETE FROM test_suite_alert_configs WHERE id = $1"#)
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn trigger_alert(
        &self,
        alert_id: Uuid,
        metric_name: &str,
        metric_value: f64,
        threshold: f64,
    ) -> Result<TestSuiteAlertHistory, sqlx::Error> {
        let id = Uuid::new_v4();
        let now = Utc::now();

        sqlx::query(
            r#"INSERT INTO test_suite_alert_history (id, alert_id, metric_name, metric_value, threshold, created_at)
               VALUES ($1, $2, $3, $4, $5, $6)"#,
        )
        .bind(id)
        .bind(alert_id)
        .bind(metric_name)
        .bind(metric_value)
        .bind(threshold)
        .bind(now)
        .execute(&self.pool)
        .await?;

        sqlx::query(r#"UPDATE test_suite_alert_configs SET last_triggered_at = $1 WHERE id = $2"#)
            .bind(now)
            .bind(alert_id)
            .execute(&self.pool)
            .await?;

        Ok(TestSuiteAlertHistory {
            id,
            alert_id,
            metric_name: metric_name.to_string(),
            metric_value,
            threshold,
            created_at: now,
        })
    }

    pub async fn get_alert_history(
        &self,
        alert_id: Uuid,
        limit: i64,
    ) -> Result<Vec<TestSuiteAlertHistory>, sqlx::Error> {
        let rows = sqlx::query_as::<_, AlertHistoryV2Row>(
            r#"SELECT id, alert_id, metric_name, metric_value, threshold, created_at
               FROM test_suite_alert_history
               WHERE alert_id = $1
               ORDER BY created_at DESC
               LIMIT $2"#,
        )
        .bind(alert_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(TestSuiteAlertHistory::from).collect())
    }

    pub async fn create_metric_v4(
        &self,
        suite_id: Uuid,
        req: CreateTestSuiteMetricV4Request,
    ) -> Result<TestSuiteMetricV4, sqlx::Error> {
        let id = Uuid::new_v4();
        let now = Utc::now();

        sqlx::query(
            r#"INSERT INTO test_suite_metrics_v4 (id, suite_id, metric_name, metric_value, measured_at)
               VALUES ($1, $2, $3, $4, $5)"#,
        )
        .bind(id)
        .bind(suite_id)
        .bind(&req.metric_name)
        .bind(req.metric_value)
        .bind(now)
        .execute(&self.pool)
        .await?;

        Ok(TestSuiteMetricV4 {
            id,
            suite_id,
            metric_name: req.metric_name,
            metric_value: req.metric_value,
            measured_at: now,
        })
    }

    pub async fn list_metrics_v4(
        &self,
        suite_id: Uuid,
        metric_name: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<TestSuiteMetricV4>, sqlx::Error> {
        let rows = sqlx::query_as::<_, MetricV4Row>(
            r#"SELECT id, suite_id, metric_name, metric_value, measured_at
               FROM test_suite_metrics_v4
               WHERE suite_id = $1
                 AND ($2::varchar IS NULL OR metric_name = $2)
               ORDER BY measured_at DESC
               LIMIT $3 OFFSET $4"#,
        )
        .bind(suite_id)
        .bind(metric_name)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(TestSuiteMetricV4::from).collect())
    }

    pub async fn create_baseline_v4(
        &self,
        suite_id: Uuid,
        req: CreateTestSuiteBaselineV4Request,
    ) -> Result<TestSuiteBaselineV4, sqlx::Error> {
        let id = Uuid::new_v4();
        let now = Utc::now();
        let threshold_percent = req.threshold_percent.unwrap_or(10.0);

        sqlx::query(
            r#"INSERT INTO test_suite_baselines_v4 (id, suite_id, metric_name, baseline_value, threshold_percent, created_at)
               VALUES ($1, $2, $3, $4, $5, $6)
               ON CONFLICT (suite_id, metric_name) DO UPDATE SET baseline_value = $4, threshold_percent = $5"#,
        )
        .bind(id)
        .bind(suite_id)
        .bind(&req.metric_name)
        .bind(req.baseline_value)
        .bind(threshold_percent)
        .bind(now)
        .execute(&self.pool)
        .await?;

        Ok(TestSuiteBaselineV4 {
            id,
            suite_id,
            metric_name: req.metric_name,
            baseline_value: req.baseline_value,
            threshold_percent,
            created_at: now,
        })
    }

    pub async fn list_baselines_v4(
        &self,
        suite_id: Uuid,
    ) -> Result<Vec<TestSuiteBaselineV4>, sqlx::Error> {
        let rows = sqlx::query_as::<_, BaselineV4Row>(
            r#"SELECT id, suite_id, metric_name, baseline_value, threshold_percent, created_at
               FROM test_suite_baselines_v4
               WHERE suite_id = $1
               ORDER BY metric_name"#,
        )
        .bind(suite_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(TestSuiteBaselineV4::from).collect())
    }

    pub async fn update_baseline_v4(
        &self,
        id: Uuid,
        req: UpdateTestSuiteBaselineV4Request,
    ) -> Result<TestSuiteBaselineV4, sqlx::Error> {
        if let Some(value) = req.baseline_value {
            sqlx::query(r#"UPDATE test_suite_baselines_v4 SET baseline_value = $1 WHERE id = $2"#)
                .bind(value)
                .bind(id)
                .execute(&self.pool)
                .await?;
        }
        if let Some(threshold) = req.threshold_percent {
            sqlx::query(r#"UPDATE test_suite_baselines_v4 SET threshold_percent = $1 WHERE id = $2"#)
                .bind(threshold)
                .bind(id)
                .execute(&self.pool)
                .await?;
        }

        let row = sqlx::query_as::<_, BaselineV4Row>(
            r#"SELECT id, suite_id, metric_name, baseline_value, threshold_percent, created_at
               FROM test_suite_baselines_v4 WHERE id = $1"#,
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await?;

        Ok(TestSuiteBaselineV4::from(row))
    }

    pub async fn delete_baseline_v4(&self, id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(r#"DELETE FROM test_suite_baselines_v4 WHERE id = $1"#)
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn detect_regressions_v4(
        &self,
        suite_id: Uuid,
    ) -> Result<Vec<TestSuiteRegressionV4>, sqlx::Error> {
        let baselines = self.list_baselines_v4(suite_id).await?;
        let mut regressions = Vec::new();

        for baseline in baselines {
            if let Some(latest_metric) = sqlx::query_as::<_, MetricV4Row>(
                r#"SELECT id, suite_id, metric_name, metric_value, measured_at
                   FROM test_suite_metrics_v4
                   WHERE suite_id = $1 AND metric_name = $2
                   ORDER BY measured_at DESC
                   LIMIT 1"#,
            )
            .bind(suite_id)
            .bind(&baseline.metric_name)
            .fetch_optional(&self.pool)
            .await?
            {
                let current_value = latest_metric.metric_value;
                let regression_percent = if baseline.baseline_value != 0.0 {
                    ((current_value - baseline.baseline_value) / baseline.baseline_value) * 100.0
                } else {
                    0.0
                };

                if regression_percent > baseline.threshold_percent {
                    regressions.push(TestSuiteRegressionV4 {
                        id: Uuid::new_v4(),
                        baseline_id: baseline.id,
                        metric_name: baseline.metric_name,
                        baseline_value: baseline.baseline_value,
                        current_value,
                        regression_percent,
                        threshold_percent: baseline.threshold_percent,
                        status: "open".to_string(),
                        detected_at: Utc::now(),
                    });
                }
            }
        }

        Ok(regressions)
    }

    pub async fn get_metrics_summary_v4(
        &self,
        suite_id: Uuid,
    ) -> Result<TestSuiteMetricsSummaryV4, sqlx::Error> {
        let metrics = self.list_metrics_v4(suite_id, None, 1000, 0).await?;
        let baselines = self.list_baselines_v4(suite_id).await?;
        let regressions = self.detect_regressions_v4(suite_id).await?;

        let active_regressions = regressions.iter().filter(|r| r.status == "open").count() as i64;
        let resolved_regressions = regressions.iter().filter(|r| r.status == "resolved").count() as i64;

        Ok(TestSuiteMetricsSummaryV4 {
            suite_id,
            total_metrics: metrics.len() as i64,
            total_baselines: baselines.len() as i64,
            active_regressions,
            resolved_regressions,
            metrics,
            baselines,
        })
    }

    pub async fn get_performance_report_v4(
        &self,
        suite_id: Uuid,
    ) -> Result<TestSuitePerformanceReportV4, sqlx::Error> {
        let suite = self.get_suite(suite_id).await?.ok_or_else(|| sqlx::Error::RowNotFound)?;
        let metrics_summary = self.get_metrics_summary_v4(suite_id).await?;
        let regressions = self.detect_regressions_v4(suite_id).await?;

        let mut alerts = Vec::new();
        for regression in &regressions {
            let severity = if regression.regression_percent > 50.0 {
                "critical"
            } else if regression.regression_percent > 25.0 {
                "high"
            } else if regression.regression_percent > 10.0 {
                "medium"
            } else {
                "low"
            };

            alerts.push(TestSuitePerformanceAlertV4 {
                id: Uuid::new_v4(),
                suite_id,
                metric_name: regression.metric_name.clone(),
                baseline_value: regression.baseline_value,
                current_value: regression.current_value,
                regression_percent: regression.regression_percent,
                threshold_percent: regression.threshold_percent,
                severity: severity.to_string(),
                message: format!(
                    "Performance regression detected: {} regressed by {:.1}% (threshold: {:.1}%)",
                    regression.metric_name, regression.regression_percent, regression.threshold_percent
                ),
                created_at: Utc::now(),
            });
        }

        let overall_score = if metrics_summary.total_baselines > 0 {
            let passing_baselines = metrics_summary.total_baselines - metrics_summary.active_regressions;
            (passing_baselines as f64 / metrics_summary.total_baselines as f64) * 100.0
        } else {
            100.0
        };

        let last_measured_at = metrics_summary.metrics.iter().map(|m| m.measured_at).max();

        Ok(TestSuitePerformanceReportV4 {
            suite_id,
            suite_name: suite.name,
            metrics_summary,
            regressions,
            alerts,
            overall_score,
            last_measured_at,
        })
    }

    pub async fn create_metric_v8(
        &self,
        suite_id: Uuid,
        req: CreateTestSuiteMetricV8Request,
    ) -> Result<TestSuiteMetricV8, sqlx::Error> {
        let id = Uuid::new_v4();
        let now = Utc::now();

        sqlx::query(
            r#"INSERT INTO test_suite_metrics_v8 (id, suite_id, metric_name, metric_value, measured_at)
               VALUES ($1, $2, $3, $4, $5)"#,
        )
        .bind(id)
        .bind(suite_id)
        .bind(&req.metric_name)
        .bind(req.metric_value)
        .bind(now)
        .execute(&self.pool)
        .await?;

        Ok(TestSuiteMetricV8 {
            id,
            suite_id,
            metric_name: req.metric_name,
            metric_value: req.metric_value,
            measured_at: now,
        })
    }

    pub async fn list_metrics_v8(
        &self,
        suite_id: Uuid,
        metric_name: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<TestSuiteMetricV8>, sqlx::Error> {
        let rows = sqlx::query_as::<_, MetricV8Row>(
            r#"SELECT id, suite_id, metric_name, metric_value, measured_at
               FROM test_suite_metrics_v8
               WHERE suite_id = $1
                 AND ($2::varchar IS NULL OR metric_name = $2)
               ORDER BY measured_at DESC
               LIMIT $3 OFFSET $4"#,
        )
        .bind(suite_id)
        .bind(metric_name)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(TestSuiteMetricV8::from).collect())
    }

    pub async fn create_baseline_v8(
        &self,
        suite_id: Uuid,
        req: CreateTestSuiteBaselineV8Request,
    ) -> Result<TestSuiteBaselineV8, sqlx::Error> {
        let id = Uuid::new_v4();
        let now = Utc::now();
        let threshold_percent = req.threshold_percent.unwrap_or(10.0);

        sqlx::query(
            r#"INSERT INTO test_suite_baselines_v8 (id, suite_id, metric_name, baseline_value, threshold_percent, created_at)
               VALUES ($1, $2, $3, $4, $5, $6)
               ON CONFLICT (suite_id, metric_name) DO UPDATE SET baseline_value = $4, threshold_percent = $5"#,
        )
        .bind(id)
        .bind(suite_id)
        .bind(&req.metric_name)
        .bind(req.baseline_value)
        .bind(threshold_percent)
        .bind(now)
        .execute(&self.pool)
        .await?;

        Ok(TestSuiteBaselineV8 {
            id,
            suite_id,
            metric_name: req.metric_name,
            baseline_value: req.baseline_value,
            threshold_percent,
            created_at: now,
        })
    }

    pub async fn list_baselines_v8(
        &self,
        suite_id: Uuid,
    ) -> Result<Vec<TestSuiteBaselineV8>, sqlx::Error> {
        let rows = sqlx::query_as::<_, BaselineV8Row>(
            r#"SELECT id, suite_id, metric_name, baseline_value, threshold_percent, created_at
               FROM test_suite_baselines_v8
               WHERE suite_id = $1
               ORDER BY metric_name"#,
        )
        .bind(suite_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(TestSuiteBaselineV8::from).collect())
    }

    pub async fn update_baseline_v8(
        &self,
        id: Uuid,
        req: UpdateTestSuiteBaselineV8Request,
    ) -> Result<TestSuiteBaselineV8, sqlx::Error> {
        if let Some(value) = req.baseline_value {
            sqlx::query(r#"UPDATE test_suite_baselines_v8 SET baseline_value = $1 WHERE id = $2"#)
                .bind(value)
                .bind(id)
                .execute(&self.pool)
                .await?;
        }
        if let Some(threshold) = req.threshold_percent {
            sqlx::query(r#"UPDATE test_suite_baselines_v8 SET threshold_percent = $1 WHERE id = $2"#)
                .bind(threshold)
                .bind(id)
                .execute(&self.pool)
                .await?;
        }

        let row = sqlx::query_as::<_, BaselineV8Row>(
            r#"SELECT id, suite_id, metric_name, baseline_value, threshold_percent, created_at
               FROM test_suite_baselines_v8 WHERE id = $1"#,
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await?;

        Ok(TestSuiteBaselineV8::from(row))
    }

    pub async fn delete_baseline_v8(&self, id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(r#"DELETE FROM test_suite_baselines_v8 WHERE id = $1"#)
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn detect_regressions_v8(
        &self,
        suite_id: Uuid,
    ) -> Result<Vec<TestSuiteRegressionV8>, sqlx::Error> {
        let baselines = self.list_baselines_v8(suite_id).await?;
        let mut regressions = Vec::new();

        for baseline in baselines {
            if let Some(latest_metric) = sqlx::query_as::<_, MetricV8Row>(
                r#"SELECT id, suite_id, metric_name, metric_value, measured_at
                   FROM test_suite_metrics_v8
                   WHERE suite_id = $1 AND metric_name = $2
                   ORDER BY measured_at DESC
                   LIMIT 1"#,
            )
            .bind(suite_id)
            .bind(&baseline.metric_name)
            .fetch_optional(&self.pool)
            .await?
            {
                let current_value = latest_metric.metric_value;
                let regression_percent = if baseline.baseline_value != 0.0 {
                    ((current_value - baseline.baseline_value) / baseline.baseline_value) * 100.0
                } else {
                    0.0
                };

                if regression_percent > baseline.threshold_percent {
                    regressions.push(TestSuiteRegressionV8 {
                        id: Uuid::new_v4(),
                        baseline_id: baseline.id,
                        metric_name: baseline.metric_name,
                        baseline_value: baseline.baseline_value,
                        current_value,
                        regression_percent,
                        threshold_percent: baseline.threshold_percent,
                        status: "open".to_string(),
                        detected_at: Utc::now(),
                    });
                }
            }
        }

        Ok(regressions)
    }

    pub async fn get_metrics_summary_v8(
        &self,
        suite_id: Uuid,
    ) -> Result<TestSuiteMetricsSummaryV8, sqlx::Error> {
        let metrics = self.list_metrics_v8(suite_id, None, 1000, 0).await?;
        let baselines = self.list_baselines_v8(suite_id).await?;
        let regressions = self.detect_regressions_v8(suite_id).await?;

        let active_regressions = regressions.iter().filter(|r| r.status == "open").count() as i64;
        let resolved_regressions = regressions.iter().filter(|r| r.status == "resolved").count() as i64;

        Ok(TestSuiteMetricsSummaryV8 {
            suite_id,
            total_metrics: metrics.len() as i64,
            total_baselines: baselines.len() as i64,
            active_regressions,
            resolved_regressions,
            metrics,
            baselines,
        })
    }

    pub async fn get_performance_report_v8(
        &self,
        suite_id: Uuid,
    ) -> Result<TestSuitePerformanceReportV8, sqlx::Error> {
        let suite = self.get_suite(suite_id).await?.ok_or_else(|| sqlx::Error::RowNotFound)?;
        let metrics_summary = self.get_metrics_summary_v8(suite_id).await?;
        let regressions = self.detect_regressions_v8(suite_id).await?;

        let mut alerts = Vec::new();
        for regression in &regressions {
            let severity = if regression.regression_percent > 50.0 {
                "critical"
            } else if regression.regression_percent > 25.0 {
                "high"
            } else if regression.regression_percent > 10.0 {
                "medium"
            } else {
                "low"
            };

            alerts.push(TestSuitePerformanceAlertV8 {
                id: Uuid::new_v4(),
                suite_id,
                metric_name: regression.metric_name.clone(),
                baseline_value: regression.baseline_value,
                current_value: regression.current_value,
                regression_percent: regression.regression_percent,
                threshold_percent: regression.threshold_percent,
                severity: severity.to_string(),
                message: format!(
                    "Performance regression detected: {} regressed by {:.1}% (threshold: {:.1}%)",
                    regression.metric_name, regression.regression_percent, regression.threshold_percent
                ),
                created_at: Utc::now(),
            });
        }

        let overall_score = if metrics_summary.total_baselines > 0 {
            let passing_baselines = metrics_summary.total_baselines - metrics_summary.active_regressions;
            (passing_baselines as f64 / metrics_summary.total_baselines as f64) * 100.0
        } else {
            100.0
        };

        let last_measured_at = metrics_summary.metrics.iter().map(|m| m.measured_at).max();

        Ok(TestSuitePerformanceReportV8 {
            suite_id,
            suite_name: suite.name,
            metrics_summary,
            regressions,
            alerts,
            overall_score,
            last_measured_at,
        })
    }

    pub async fn create_metric_v15(
        &self,
        suite_id: Uuid,
        req: CreateTestSuiteMetricV15Request,
    ) -> Result<TestSuiteMetricV15, sqlx::Error> {
        let id = Uuid::new_v4();
        let now = Utc::now();

        sqlx::query(
            r#"INSERT INTO test_suite_metrics_v15 (id, suite_id, metric_name, metric_value, measured_at)
               VALUES ($1, $2, $3, $4, $5)"#,
        )
        .bind(id)
        .bind(suite_id)
        .bind(&req.metric_name)
        .bind(req.metric_value)
        .bind(now)
        .execute(&self.pool)
        .await?;

        Ok(TestSuiteMetricV15 {
            id,
            suite_id,
            metric_name: req.metric_name,
            metric_value: req.metric_value,
            measured_at: now,
        })
    }

    pub async fn list_metrics_v15(
        &self,
        suite_id: Uuid,
        metric_name: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<TestSuiteMetricV15>, sqlx::Error> {
        let rows = sqlx::query_as::<_, MetricV15Row>(
            r#"SELECT id, suite_id, metric_name, metric_value, measured_at
               FROM test_suite_metrics_v15
               WHERE suite_id = $1
                 AND ($2::varchar IS NULL OR metric_name = $2)
               ORDER BY measured_at DESC
               LIMIT $3 OFFSET $4"#,
        )
        .bind(suite_id)
        .bind(metric_name)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(TestSuiteMetricV15::from).collect())
    }

    pub async fn create_baseline_v15(
        &self,
        suite_id: Uuid,
        req: CreateTestSuiteBaselineV15Request,
    ) -> Result<TestSuiteBaselineV15, sqlx::Error> {
        let id = Uuid::new_v4();
        let now = Utc::now();
        let threshold_percent = req.threshold_percent.unwrap_or(10.0);

        sqlx::query(
            r#"INSERT INTO test_suite_baselines_v15 (id, suite_id, metric_name, baseline_value, threshold_percent, created_at)
               VALUES ($1, $2, $3, $4, $5, $6)
               ON CONFLICT (suite_id, metric_name) DO UPDATE SET baseline_value = $4, threshold_percent = $5"#,
        )
        .bind(id)
        .bind(suite_id)
        .bind(&req.metric_name)
        .bind(req.baseline_value)
        .bind(threshold_percent)
        .bind(now)
        .execute(&self.pool)
        .await?;

        Ok(TestSuiteBaselineV15 {
            id,
            suite_id,
            metric_name: req.metric_name,
            baseline_value: req.baseline_value,
            threshold_percent,
            created_at: now,
        })
    }

    pub async fn list_baselines_v15(
        &self,
        suite_id: Uuid,
    ) -> Result<Vec<TestSuiteBaselineV15>, sqlx::Error> {
        let rows = sqlx::query_as::<_, BaselineV15Row>(
            r#"SELECT id, suite_id, metric_name, baseline_value, threshold_percent, created_at
               FROM test_suite_baselines_v15
               WHERE suite_id = $1
               ORDER BY metric_name"#,
        )
        .bind(suite_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(TestSuiteBaselineV15::from).collect())
    }

    pub async fn update_baseline_v15(
        &self,
        id: Uuid,
        req: UpdateTestSuiteBaselineV15Request,
    ) -> Result<TestSuiteBaselineV15, sqlx::Error> {
        if let Some(value) = req.baseline_value {
            sqlx::query(r#"UPDATE test_suite_baselines_v15 SET baseline_value = $1 WHERE id = $2"#)
                .bind(value)
                .bind(id)
                .execute(&self.pool)
                .await?;
        }
        if let Some(threshold) = req.threshold_percent {
            sqlx::query(r#"UPDATE test_suite_baselines_v15 SET threshold_percent = $1 WHERE id = $2"#)
                .bind(threshold)
                .bind(id)
                .execute(&self.pool)
                .await?;
        }

        let row = sqlx::query_as::<_, BaselineV15Row>(
            r#"SELECT id, suite_id, metric_name, baseline_value, threshold_percent, created_at
               FROM test_suite_baselines_v15 WHERE id = $1"#,
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await?;

        Ok(TestSuiteBaselineV15::from(row))
    }

    pub async fn delete_baseline_v15(&self, id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(r#"DELETE FROM test_suite_baselines_v15 WHERE id = $1"#)
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn detect_regressions_v15(
        &self,
        suite_id: Uuid,
    ) -> Result<Vec<TestSuiteRegressionV15>, sqlx::Error> {
        let baselines = self.list_baselines_v15(suite_id).await?;
        let mut regressions = Vec::new();

        for baseline in baselines {
            if let Some(latest_metric) = sqlx::query_as::<_, MetricV15Row>(
                r#"SELECT id, suite_id, metric_name, metric_value, measured_at
                   FROM test_suite_metrics_v15
                   WHERE suite_id = $1 AND metric_name = $2
                   ORDER BY measured_at DESC
                   LIMIT 1"#,
            )
            .bind(suite_id)
            .bind(&baseline.metric_name)
            .fetch_optional(&self.pool)
            .await?
            {
                let current_value = latest_metric.metric_value;
                let regression_percent = if baseline.baseline_value != 0.0 {
                    ((current_value - baseline.baseline_value) / baseline.baseline_value) * 100.0
                } else {
                    0.0
                };

                if regression_percent > baseline.threshold_percent {
                    regressions.push(TestSuiteRegressionV15 {
                        id: Uuid::new_v4(),
                        baseline_id: baseline.id,
                        metric_name: baseline.metric_name,
                        baseline_value: baseline.baseline_value,
                        current_value,
                        regression_percent,
                        threshold_percent: baseline.threshold_percent,
                        status: "open".to_string(),
                        detected_at: Utc::now(),
                    });
                }
            }
        }

        Ok(regressions)
    }

    pub async fn get_metrics_summary_v15(
        &self,
        suite_id: Uuid,
    ) -> Result<TestSuiteMetricsSummaryV15, sqlx::Error> {
        let metrics = self.list_metrics_v15(suite_id, None, 1000, 0).await?;
        let baselines = self.list_baselines_v15(suite_id).await?;
        let regressions = self.detect_regressions_v15(suite_id).await?;

        let active_regressions = regressions.iter().filter(|r| r.status == "open").count() as i64;
        let resolved_regressions = regressions.iter().filter(|r| r.status == "resolved").count() as i64;

        Ok(TestSuiteMetricsSummaryV15 {
            suite_id,
            total_metrics: metrics.len() as i64,
            total_baselines: baselines.len() as i64,
            active_regressions,
            resolved_regressions,
            metrics,
            baselines,
        })
    }

    pub async fn get_performance_report_v15(
        &self,
        suite_id: Uuid,
    ) -> Result<TestSuitePerformanceReportV15, sqlx::Error> {
        let suite = self.get_suite(suite_id).await?.ok_or_else(|| sqlx::Error::RowNotFound)?;
        let metrics_summary = self.get_metrics_summary_v15(suite_id).await?;
        let regressions = self.detect_regressions_v15(suite_id).await?;

        let mut alerts = Vec::new();
        for regression in &regressions {
            let severity = if regression.regression_percent > 50.0 {
                "critical"
            } else if regression.regression_percent > 25.0 {
                "high"
            } else if regression.regression_percent > 10.0 {
                "medium"
            } else {
                "low"
            };

            alerts.push(TestSuitePerformanceAlertV15 {
                id: Uuid::new_v4(),
                suite_id,
                metric_name: regression.metric_name.clone(),
                baseline_value: regression.baseline_value,
                current_value: regression.current_value,
                regression_percent: regression.regression_percent,
                threshold_percent: regression.threshold_percent,
                severity: severity.to_string(),
                message: format!(
                    "Performance regression detected: {} regressed by {:.1}% (threshold: {:.1}%)",
                    regression.metric_name, regression.regression_percent, regression.threshold_percent
                ),
                created_at: Utc::now(),
            });
        }

        let overall_score = if metrics_summary.total_baselines > 0 {
            let passing_baselines = metrics_summary.total_baselines - metrics_summary.active_regressions;
            (passing_baselines as f64 / metrics_summary.total_baselines as f64) * 100.0
        } else {
            100.0
        };

        let last_measured_at = metrics_summary.metrics.iter().map(|m| m.measured_at).max();

        Ok(TestSuitePerformanceReportV15 {
            suite_id,
            suite_name: suite.name,
            metrics_summary,
            regressions,
            alerts,
            overall_score,
            last_measured_at,
        })
    }

    pub async fn create_flaky_detection(
        &self,
        req: CreateFlakyTestDetectionV19Request,
    ) -> Result<FlakyTestDetectionV19, sqlx::Error> {
        let id = Uuid::new_v4();
        let now = Utc::now();
        let flaky_score = req.flaky_score.unwrap_or(0.0);
        let total_runs = req.total_runs.unwrap_or(0);
        let failure_count = req.failure_count.unwrap_or(0);

        sqlx::query(
            r#"INSERT INTO test_suite_flaky_detection_v19 (id, test_name, suite_id, flaky_score, total_runs, failure_count, detected_at)
               VALUES ($1, $2, $3, $4, $5, $6, $7)
               ON CONFLICT (test_name, suite_id) DO UPDATE SET
               flaky_score = $4, total_runs = $5, failure_count = $6, last_flaky_at = $7"#,
        )
        .bind(id)
        .bind(&req.test_name)
        .bind(req.suite_id)
        .bind(flaky_score)
        .bind(total_runs)
        .bind(failure_count)
        .bind(now)
        .execute(&self.pool)
        .await?;

        Ok(FlakyTestDetectionV19 {
            id,
            test_name: req.test_name,
            suite_id: req.suite_id,
            flaky_score,
            total_runs,
            failure_count,
            last_flaky_at: None,
            detected_at: now,
        })
    }

    pub async fn list_flaky_tests(
        &self,
        suite_id: Uuid,
        min_score: Option<f64>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<FlakyTestDetectionV19>, sqlx::Error> {
        let rows = sqlx::query_as::<_, FlakyDetectionV19Row>(
            r#"SELECT id, test_name, suite_id, flaky_score, total_runs, failure_count, last_flaky_at, detected_at
               FROM test_suite_flaky_detection_v19
               WHERE suite_id = $1
                 AND ($2::double precision IS NULL OR flaky_score >= $2)
               ORDER BY flaky_score DESC
               LIMIT $3 OFFSET $4"#,
        )
        .bind(suite_id)
        .bind(min_score)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(FlakyTestDetectionV19::from).collect())
    }

    pub async fn update_flaky_detection(
        &self,
        id: Uuid,
        req: UpdateFlakyTestDetectionV19Request,
    ) -> Result<FlakyTestDetectionV19, sqlx::Error> {
        if let Some(score) = req.flaky_score {
            sqlx::query(r#"UPDATE test_suite_flaky_detection_v19 SET flaky_score = $1 WHERE id = $2"#)
                .bind(score)
                .bind(id)
                .execute(&self.pool)
                .await?;
        }
        if let Some(runs) = req.total_runs {
            sqlx::query(r#"UPDATE test_suite_flaky_detection_v19 SET total_runs = $1 WHERE id = $2"#)
                .bind(runs)
                .bind(id)
                .execute(&self.pool)
                .await?;
        }
        if let Some(failures) = req.failure_count {
            sqlx::query(r#"UPDATE test_suite_flaky_detection_v19 SET failure_count = $1 WHERE id = $2"#)
                .bind(failures)
                .bind(id)
                .execute(&self.pool)
                .await?;
        }
        if let Some(at) = req.last_flaky_at {
            sqlx::query(r#"UPDATE test_suite_flaky_detection_v19 SET last_flaky_at = $1 WHERE id = $2"#)
                .bind(at)
                .bind(id)
                .execute(&self.pool)
                .await?;
        }

        let row = sqlx::query_as::<_, FlakyDetectionV19Row>(
            r#"SELECT id, test_name, suite_id, flaky_score, total_runs, failure_count, last_flaky_at, detected_at
               FROM test_suite_flaky_detection_v19 WHERE id = $1"#,
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await?;

        Ok(FlakyTestDetectionV19::from(row))
    }

    pub async fn delete_flaky_detection(&self, id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(r#"DELETE FROM test_suite_flaky_detection_v19 WHERE id = $1"#)
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn get_flaky_summary(
        &self,
        suite_id: Uuid,
    ) -> Result<FlakyTestSummaryV19, sqlx::Error> {
        let stats = sqlx::query_as::<_, FlakySummaryRow>(
            r#"SELECT
                COUNT(*) as total_flaky_tests,
                COUNT(*) FILTER (WHERE flaky_score >= 70.0) as high_flaky_count,
                COUNT(*) FILTER (WHERE flaky_score >= 40.0 AND flaky_score < 70.0) as medium_flaky_count,
                COUNT(*) FILTER (WHERE flaky_score < 40.0) as low_flaky_count,
                COALESCE(AVG(flaky_score), 0) as avg_flaky_score
               FROM test_suite_flaky_detection_v19
               WHERE suite_id = $1"#,
        )
        .bind(suite_id)
        .fetch_one(&self.pool)
        .await?;

        let most_flaky = sqlx::query_as::<_, FlakyDetectionV19Row>(
            r#"SELECT id, test_name, suite_id, flaky_score, total_runs, failure_count, last_flaky_at, detected_at
               FROM test_suite_flaky_detection_v19
               WHERE suite_id = $1
               ORDER BY flaky_score DESC
               LIMIT 10"#,
        )
        .bind(suite_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(FlakyTestSummaryV19 {
            total_flaky_tests: stats.total_flaky_tests,
            high_flaky_count: stats.high_flaky_count,
            medium_flaky_count: stats.medium_flaky_count,
            low_flaky_count: stats.low_flaky_count,
            avg_flaky_score: stats.avg_flaky_score,
            most_flaky_tests: most_flaky.into_iter().map(FlakyTestDetectionV19::from).collect(),
        })
    }

    pub async fn create_trend(
        &self,
        suite_id: Uuid,
        req: CreateTestSuiteTrendV19Request,
    ) -> Result<TestSuiteTrendV19, sqlx::Error> {
        let id = Uuid::new_v4();
        let now = Utc::now();

        sqlx::query(
            r#"INSERT INTO test_suite_trends_v19 (id, suite_id, metric_name, metric_value, period_start, period_end, created_at)
               VALUES ($1, $2, $3, $4, $5, $6, $7)"#,
        )
        .bind(id)
        .bind(suite_id)
        .bind(&req.metric_name)
        .bind(req.metric_value)
        .bind(req.period_start)
        .bind(req.period_end)
        .bind(now)
        .execute(&self.pool)
        .await?;

        Ok(TestSuiteTrendV19 {
            id,
            suite_id,
            metric_name: req.metric_name,
            metric_value: req.metric_value,
            period_start: req.period_start,
            period_end: req.period_end,
            created_at: now,
        })
    }

    pub async fn list_trends(
        &self,
        suite_id: Uuid,
        metric_name: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<TestSuiteTrendV19>, sqlx::Error> {
        let rows = sqlx::query_as::<_, TrendV19Row>(
            r#"SELECT id, suite_id, metric_name, metric_value, period_start, period_end, created_at
               FROM test_suite_trends_v19
               WHERE suite_id = $1
                 AND ($2::varchar IS NULL OR metric_name = $2)
               ORDER BY period_start DESC
               LIMIT $3 OFFSET $4"#,
        )
        .bind(suite_id)
        .bind(metric_name)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(TestSuiteTrendV19::from).collect())
    }

    pub async fn get_trend_analysis(
        &self,
        suite_id: Uuid,
        metric_name: &str,
    ) -> Result<TestSuiteTrendAnalysisV19, sqlx::Error> {
        let trends = self.list_trends(suite_id, Some(metric_name), 100, 0).await?;

        let avg_value = if trends.is_empty() {
            0.0
        } else {
            trends.iter().map(|t| t.metric_value).sum::<f64>() / trends.len() as f64
        };

        let min_value = trends.iter().map(|t| t.metric_value).fold(f64::INFINITY, f64::min);
        let min_value = if min_value == f64::INFINITY { 0.0 } else { min_value };

        let max_value = trends.iter().map(|t| t.metric_value).fold(f64::NEG_INFINITY, f64::max);
        let max_value = if max_value == f64::NEG_INFINITY { 0.0 } else { max_value };

        let trend_direction = if trends.len() < 2 {
            "stable".to_string()
        } else {
            let first_half: f64 = trends.iter().take(trends.len() / 2).map(|t| t.metric_value).sum::<f64>() / (trends.len() / 2) as f64;
            let second_half: f64 = trends.iter().skip(trends.len() / 2).map(|t| t.metric_value).sum::<f64>() / (trends.len() - trends.len() / 2) as f64;
            if second_half > first_half * 1.1 {
                "increasing".to_string()
            } else if second_half < first_half * 0.9 {
                "decreasing".to_string()
            } else {
                "stable".to_string()
            }
        };

        let change_percent = if trends.len() >= 2 {
            let first = trends.last().unwrap().metric_value;
            let last = trends.first().unwrap().metric_value;
            if first != 0.0 {
                ((last - first) / first) * 100.0
            } else {
                0.0
            }
        } else {
            0.0
        };

        Ok(TestSuiteTrendAnalysisV19 {
            suite_id,
            metric_name: metric_name.to_string(),
            trends,
            avg_value,
            min_value,
            max_value,
            trend_direction,
            change_percent,
        })
    }

    pub async fn generate_optimization_suggestions(
        &self,
        suite_id: Uuid,
    ) -> Result<Vec<TestOptimizationSuggestionV22>, sqlx::Error> {
        let mut suggestions = Vec::new();

        let runs = self.list_runs(suite_id, None, 100, 0).await?;
        let avg_duration = if runs.is_empty() {
            0.0
        } else {
            runs.iter().map(|r| r.duration_ms as f64).sum::<f64>() / runs.len() as f64
        };

        if avg_duration > 300000.0 {
            suggestions.push(TestOptimizationSuggestionV22 {
                suite_id,
                suggestion_type: "slow_tests".to_string(),
                description: format!("Average test duration is {:.0}ms. Consider splitting into parallel suites.", avg_duration),
                impact_score: 80.0,
                estimated_time_savings_ms: (avg_duration * 0.3) as i64,
            });
        }

        let fail_rate = if runs.is_empty() {
            0.0
        } else {
            runs.iter().filter(|r| r.status == TestRunStatus::Failed).count() as f64 / runs.len() as f64 * 100.0
        };

        if fail_rate > 10.0 {
            suggestions.push(TestOptimizationSuggestionV22 {
                suite_id,
                suggestion_type: "flaky_tests".to_string(),
                description: format!("Failure rate is {:.1}%. Investigate flaky tests.", fail_rate),
                impact_score: 70.0,
                estimated_time_savings_ms: 0,
            });
        }

        Ok(suggestions)
    }

    pub async fn analyze_coverage_gaps(
        &self,
        repo_id: Uuid,
    ) -> Result<CoverageGapAnalysisV22, sqlx::Error> {
        let total_files = sqlx::query_scalar::<_, i64>(
            r#"SELECT COUNT(DISTINCT file_path) FROM code_quality_metrics WHERE repo_id = $1"#,
        )
        .bind(repo_id)
        .fetch_one(&self.pool)
        .await?;

        let covered_files = sqlx::query_scalar::<_, i64>(
            r#"SELECT COUNT(DISTINCT file_path) FROM code_quality_metrics WHERE repo_id = $1 AND metric_name LIKE '%test%'"#,
        )
        .bind(repo_id)
        .fetch_one(&self.pool)
        .await?;

        let coverage_percent = if total_files > 0 {
            (covered_files as f64 / total_files as f64) * 100.0
        } else {
            0.0
        };

        let gap_severity = if coverage_percent < 30.0 {
            "critical".to_string()
        } else if coverage_percent < 60.0 {
            "high".to_string()
        } else if coverage_percent < 80.0 {
            "medium".to_string()
        } else {
            "low".to_string()
        };

        Ok(CoverageGapAnalysisV22 {
            repo_id,
            total_files,
            covered_files,
            coverage_percent,
            uncovered_files: Vec::new(),
            gap_severity,
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

#[derive(sqlx::FromRow)]
struct ConfigRow {
    id: Uuid,
    suite_id: Uuid,
    config_key: String,
    config_value: serde_json::Value,
    created_at: chrono::DateTime<Utc>,
}

impl From<ConfigRow> for TestSuiteConfiguration {
    fn from(row: ConfigRow) -> Self {
        Self {
            id: row.id,
            suite_id: row.suite_id,
            config_key: row.config_key,
            config_value: row.config_value,
            created_at: row.created_at,
        }
    }
}

#[derive(sqlx::FromRow)]
struct NotificationRow {
    id: Uuid,
    suite_id: Uuid,
    notification_type: String,
    config: serde_json::Value,
    enabled: bool,
    created_at: chrono::DateTime<Utc>,
}

impl From<NotificationRow> for TestSuiteNotification {
    fn from(row: NotificationRow) -> Self {
        Self {
            id: row.id,
            suite_id: row.suite_id,
            notification_type: row.notification_type,
            config: row.config,
            enabled: row.enabled,
            created_at: row.created_at,
        }
    }
}

#[derive(sqlx::FromRow)]
struct ActivityRow {
    suite_id: Uuid,
    suite_name: String,
    run_count: i64,
    last_run_at: Option<chrono::DateTime<Utc>>,
}

impl From<ActivityRow> for SuiteActivity {
    fn from(row: ActivityRow) -> Self {
        Self {
            suite_id: row.suite_id,
            suite_name: row.suite_name,
            run_count: row.run_count,
            last_run_at: row.last_run_at,
        }
    }
}

#[derive(sqlx::FromRow)]
struct FailureTrendRow {
    date: chrono::NaiveDate,
    failure_count: i64,
    failure_rate: f64,
}

impl From<FailureTrendRow> for FailureTrend {
    fn from(row: FailureTrendRow) -> Self {
        Self {
            date: row.date,
            failure_count: row.failure_count,
            failure_rate: row.failure_rate,
        }
    }
}

#[derive(sqlx::FromRow)]
struct TagRow {
    id: Uuid,
    suite_id: Uuid,
    tag: String,
    created_at: chrono::DateTime<Utc>,
}

impl From<TagRow> for TestSuiteTag {
    fn from(row: TagRow) -> Self {
        Self {
            id: row.id,
            suite_id: row.suite_id,
            tag: row.tag,
            created_at: row.created_at,
        }
    }
}

#[derive(sqlx::FromRow)]
struct DependencyRow {
    id: Uuid,
    suite_id: Uuid,
    depends_on_suite_id: Uuid,
    dependency_type: String,
    created_at: chrono::DateTime<Utc>,
}

impl From<DependencyRow> for TestSuiteDependency {
    fn from(row: DependencyRow) -> Self {
        Self {
            id: row.id,
            suite_id: row.suite_id,
            depends_on_suite_id: row.depends_on_suite_id,
            dependency_type: row.dependency_type,
            created_at: row.created_at,
        }
    }
}

#[derive(sqlx::FromRow)]
struct DependencySummaryRow {
    total_dependencies: i64,
    circular_dependencies_detected: i64,
    suites_with_dependencies: i64,
    suites_without_dependencies: i64,
}

#[derive(sqlx::FromRow)]
struct MetricRow {
    id: Uuid,
    suite_id: Uuid,
    metric_name: String,
    metric_value: f64,
    measured_at: chrono::DateTime<Utc>,
}

impl From<MetricRow> for TestSuiteMetric {
    fn from(row: MetricRow) -> Self {
        Self {
            id: row.id,
            suite_id: row.suite_id,
            metric_name: row.metric_name,
            metric_value: row.metric_value,
            measured_at: row.measured_at,
        }
    }
}

#[derive(sqlx::FromRow)]
struct MetricSummaryRow {
    metric_name: String,
    latest_value: f64,
    avg_value: f64,
    min_value: f64,
    max_value: f64,
    measurement_count: i64,
    suites_affected: i64,
}

struct MetricSummary {
    metric_name: String,
    latest_value: f64,
    avg_value: f64,
    min_value: f64,
    max_value: f64,
    measurement_count: i64,
    suites_affected: i64,
}

impl From<MetricSummaryRow> for MetricSummary {
    fn from(row: MetricSummaryRow) -> Self {
        Self {
            metric_name: row.metric_name,
            latest_value: row.latest_value,
            avg_value: row.avg_value,
            min_value: row.min_value,
            max_value: row.max_value,
            measurement_count: row.measurement_count,
            suites_affected: row.suites_affected,
        }
    }
}

#[derive(sqlx::FromRow)]
struct BaselineRow {
    id: Uuid,
    suite_id: Uuid,
    metric_name: String,
    baseline_value: f64,
    threshold_percent: f64,
    created_at: chrono::DateTime<Utc>,
}

impl From<BaselineRow> for TestSuiteBaseline {
    fn from(row: BaselineRow) -> Self {
        Self {
            id: row.id,
            suite_id: row.suite_id,
            metric_name: row.metric_name,
            baseline_value: row.baseline_value,
            threshold_percent: row.threshold_percent,
            created_at: row.created_at,
        }
    }
}

#[derive(sqlx::FromRow)]
struct MetricV2Row {
    id: Uuid,
    suite_id: Uuid,
    metric_name: String,
    metric_value: f64,
    measured_at: chrono::DateTime<Utc>,
}

impl From<MetricV2Row> for TestSuiteMetricV2 {
    fn from(row: MetricV2Row) -> Self {
        Self {
            id: row.id,
            suite_id: row.suite_id,
            metric_name: row.metric_name,
            metric_value: row.metric_value,
            measured_at: row.measured_at,
        }
    }
}

#[derive(sqlx::FromRow)]
struct MetricSummaryV2Row {
    metric_name: String,
    latest_value: f64,
    avg_value: f64,
    min_value: f64,
    max_value: f64,
    measurement_count: i64,
    suites_affected: i64,
}

struct MetricSummaryV2 {
    metric_name: String,
    latest_value: f64,
    avg_value: f64,
    min_value: f64,
    max_value: f64,
    measurement_count: i64,
    suites_affected: i64,
}

impl From<MetricSummaryV2Row> for MetricSummaryV2 {
    fn from(row: MetricSummaryV2Row) -> Self {
        Self {
            metric_name: row.metric_name,
            latest_value: row.latest_value,
            avg_value: row.avg_value,
            min_value: row.min_value,
            max_value: row.max_value,
            measurement_count: row.measurement_count,
            suites_affected: row.suites_affected,
        }
    }
}

#[derive(sqlx::FromRow)]
struct BaselineV2Row {
    id: Uuid,
    suite_id: Uuid,
    metric_name: String,
    baseline_value: f64,
    threshold_percent: f64,
    created_at: chrono::DateTime<Utc>,
}

impl From<BaselineV2Row> for TestSuiteBaselineV2 {
    fn from(row: BaselineV2Row) -> Self {
        Self {
            id: row.id,
            suite_id: row.suite_id,
            metric_name: row.metric_name,
            baseline_value: row.baseline_value,
            threshold_percent: row.threshold_percent,
            created_at: row.created_at,
        }
    }
}

#[derive(sqlx::FromRow)]
struct AlertConfigV2Row {
    id: Uuid,
    suite_id: Uuid,
    alert_type: String,
    threshold: f64,
    enabled: bool,
    last_triggered_at: Option<chrono::DateTime<Utc>>,
    created_at: chrono::DateTime<Utc>,
}

impl From<AlertConfigV2Row> for TestSuitePerformanceAlertConfig {
    fn from(row: AlertConfigV2Row) -> Self {
        Self {
            id: row.id,
            suite_id: row.suite_id,
            alert_type: row.alert_type,
            threshold: row.threshold,
            enabled: row.enabled,
            last_triggered_at: row.last_triggered_at,
            created_at: row.created_at,
        }
    }
}

#[derive(sqlx::FromRow)]
struct AlertHistoryV2Row {
    id: Uuid,
    alert_id: Uuid,
    metric_name: String,
    metric_value: f64,
    threshold: f64,
    created_at: chrono::DateTime<Utc>,
}

impl From<AlertHistoryV2Row> for TestSuiteAlertHistory {
    fn from(row: AlertHistoryV2Row) -> Self {
        Self {
            id: row.id,
            alert_id: row.alert_id,
            metric_name: row.metric_name,
            metric_value: row.metric_value,
            threshold: row.threshold,
            created_at: row.created_at,
        }
    }
}

#[derive(sqlx::FromRow)]
struct MetricV4Row {
    id: Uuid,
    suite_id: Uuid,
    metric_name: String,
    metric_value: f64,
    measured_at: chrono::DateTime<Utc>,
}

impl From<MetricV4Row> for TestSuiteMetricV4 {
    fn from(row: MetricV4Row) -> Self {
        Self {
            id: row.id,
            suite_id: row.suite_id,
            metric_name: row.metric_name,
            metric_value: row.metric_value,
            measured_at: row.measured_at,
        }
    }
}

#[derive(sqlx::FromRow)]
struct BaselineV4Row {
    id: Uuid,
    suite_id: Uuid,
    metric_name: String,
    baseline_value: f64,
    threshold_percent: f64,
    created_at: chrono::DateTime<Utc>,
}

impl From<BaselineV4Row> for TestSuiteBaselineV4 {
    fn from(row: BaselineV4Row) -> Self {
        Self {
            id: row.id,
            suite_id: row.suite_id,
            metric_name: row.metric_name,
            baseline_value: row.baseline_value,
            threshold_percent: row.threshold_percent,
            created_at: row.created_at,
        }
    }
}

#[derive(sqlx::FromRow)]
struct MetricV8Row {
    id: Uuid,
    suite_id: Uuid,
    metric_name: String,
    metric_value: f64,
    measured_at: chrono::DateTime<Utc>,
}

impl From<MetricV8Row> for TestSuiteMetricV8 {
    fn from(row: MetricV8Row) -> Self {
        Self {
            id: row.id,
            suite_id: row.suite_id,
            metric_name: row.metric_name,
            metric_value: row.metric_value,
            measured_at: row.measured_at,
        }
    }
}

#[derive(sqlx::FromRow)]
struct BaselineV8Row {
    id: Uuid,
    suite_id: Uuid,
    metric_name: String,
    baseline_value: f64,
    threshold_percent: f64,
    created_at: chrono::DateTime<Utc>,
}

impl From<BaselineV8Row> for TestSuiteBaselineV8 {
    fn from(row: BaselineV8Row) -> Self {
        Self {
            id: row.id,
            suite_id: row.suite_id,
            metric_name: row.metric_name,
            baseline_value: row.baseline_value,
            threshold_percent: row.threshold_percent,
            created_at: row.created_at,
        }
    }
}

#[derive(sqlx::FromRow)]
struct MetricV15Row {
    id: Uuid,
    suite_id: Uuid,
    metric_name: String,
    metric_value: f64,
    measured_at: chrono::DateTime<Utc>,
}

impl From<MetricV15Row> for TestSuiteMetricV15 {
    fn from(row: MetricV15Row) -> Self {
        Self {
            id: row.id,
            suite_id: row.suite_id,
            metric_name: row.metric_name,
            metric_value: row.metric_value,
            measured_at: row.measured_at,
        }
    }
}

#[derive(sqlx::FromRow)]
struct BaselineV15Row {
    id: Uuid,
    suite_id: Uuid,
    metric_name: String,
    baseline_value: f64,
    threshold_percent: f64,
    created_at: chrono::DateTime<Utc>,
}

impl From<BaselineV15Row> for TestSuiteBaselineV15 {
    fn from(row: BaselineV15Row) -> Self {
        Self {
            id: row.id,
            suite_id: row.suite_id,
            metric_name: row.metric_name,
            baseline_value: row.baseline_value,
            threshold_percent: row.threshold_percent,
            created_at: row.created_at,
        }
    }
}

#[derive(sqlx::FromRow)]
struct FlakyDetectionV19Row {
    id: Uuid,
    test_name: String,
    suite_id: Uuid,
    flaky_score: f64,
    total_runs: i32,
    failure_count: i32,
    last_flaky_at: Option<chrono::DateTime<Utc>>,
    detected_at: chrono::DateTime<Utc>,
}

impl From<FlakyDetectionV19Row> for FlakyTestDetectionV19 {
    fn from(row: FlakyDetectionV19Row) -> Self {
        Self {
            id: row.id,
            test_name: row.test_name,
            suite_id: row.suite_id,
            flaky_score: row.flaky_score,
            total_runs: row.total_runs,
            failure_count: row.failure_count,
            last_flaky_at: row.last_flaky_at,
            detected_at: row.detected_at,
        }
    }
}

#[derive(sqlx::FromRow)]
struct TrendV19Row {
    id: Uuid,
    suite_id: Uuid,
    metric_name: String,
    metric_value: f64,
    period_start: chrono::DateTime<Utc>,
    period_end: chrono::DateTime<Utc>,
    created_at: chrono::DateTime<Utc>,
}

impl From<TrendV19Row> for TestSuiteTrendV19 {
    fn from(row: TrendV19Row) -> Self {
        Self {
            id: row.id,
            suite_id: row.suite_id,
            metric_name: row.metric_name,
            metric_value: row.metric_value,
            period_start: row.period_start,
            period_end: row.period_end,
            created_at: row.created_at,
        }
    }
}

#[derive(sqlx::FromRow)]
struct FlakySummaryRow {
    total_flaky_tests: i64,
    high_flaky_count: i64,
    medium_flaky_count: i64,
    low_flaky_count: i64,
    avg_flaky_score: f64,
}
