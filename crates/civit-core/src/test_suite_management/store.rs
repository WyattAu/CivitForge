use super::types::*;
use chrono::Utc;
use sqlx::PgPool;
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
