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

    pub async fn create_baseline(
        &self,
        repo_id: Uuid,
        req: CreatePerformanceBaselineRequest,
    ) -> Result<PerformanceBaseline, sqlx::Error> {
        let id = Uuid::new_v4();
        let now = Utc::now();
        let threshold_percent = req.threshold_percent.unwrap_or(10.0);

        sqlx::query(
            r#"INSERT INTO performance_baselines (id, repo_id, metric_name, baseline_value, threshold_percent, created_at)
               VALUES ($1, $2, $3, $4, $5, $6)"#,
        )
        .bind(id)
        .bind(repo_id)
        .bind(&req.metric_name)
        .bind(req.baseline_value)
        .bind(threshold_percent)
        .bind(now)
        .execute(&self.pool)
        .await?;

        Ok(PerformanceBaseline {
            id,
            repo_id,
            metric_name: req.metric_name,
            baseline_value: req.baseline_value,
            threshold_percent,
            created_at: now,
        })
    }

    pub async fn get_baseline(&self, id: Uuid) -> Result<Option<PerformanceBaseline>, sqlx::Error> {
        let row = sqlx::query_as::<_, BaselineRow>(
            r#"SELECT id, repo_id, metric_name, baseline_value, threshold_percent, created_at
               FROM performance_baselines WHERE id = $1"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(PerformanceBaseline::from))
    }

    pub async fn list_baselines(
        &self,
        repo_id: Uuid,
    ) -> Result<Vec<PerformanceBaseline>, sqlx::Error> {
        let rows = sqlx::query_as::<_, BaselineRow>(
            r#"SELECT id, repo_id, metric_name, baseline_value, threshold_percent, created_at
               FROM performance_baselines
               WHERE repo_id = $1
               ORDER BY metric_name"#,
        )
        .bind(repo_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(PerformanceBaseline::from).collect())
    }

    pub async fn update_baseline(
        &self,
        id: Uuid,
        req: UpdatePerformanceBaselineRequest,
    ) -> Result<PerformanceBaseline, sqlx::Error> {
        if let Some(value) = req.baseline_value {
            sqlx::query(r#"UPDATE performance_baselines SET baseline_value = $1 WHERE id = $2"#)
                .bind(value)
                .bind(id)
                .execute(&self.pool)
                .await?;
        }
        if let Some(threshold) = req.threshold_percent {
            sqlx::query(r#"UPDATE performance_baselines SET threshold_percent = $1 WHERE id = $2"#)
                .bind(threshold)
                .bind(id)
                .execute(&self.pool)
                .await?;
        }

        self.get_baseline(id).await?.ok_or_else(|| sqlx::Error::RowNotFound)
    }

    pub async fn delete_baseline(&self, id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(r#"DELETE FROM performance_baselines WHERE id = $1"#)
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn record_trend_data(
        &self,
        repo_id: Uuid,
        req: RecordTrendDataRequest,
    ) -> Result<PerformanceTrendData, sqlx::Error> {
        let id = Uuid::new_v4();
        let now = Utc::now();

        sqlx::query(
            r#"INSERT INTO performance_trend_data (id, repo_id, metric_name, metric_value, recorded_at)
               VALUES ($1, $2, $3, $4, $5)"#,
        )
        .bind(id)
        .bind(repo_id)
        .bind(&req.metric_name)
        .bind(req.metric_value)
        .bind(now)
        .execute(&self.pool)
        .await?;

        Ok(PerformanceTrendData {
            id,
            repo_id,
            metric_name: req.metric_name,
            metric_value: req.metric_value,
            recorded_at: now,
        })
    }

    pub async fn get_trend_analysis(
        &self,
        repo_id: Uuid,
        metric_name: &str,
        days: i64,
    ) -> Result<PerformanceTrendAnalysis, sqlx::Error> {
        let rows = sqlx::query_as::<_, TrendDataRow>(
            r#"SELECT id, repo_id, metric_name, metric_value, recorded_at
               FROM performance_trend_data
               WHERE repo_id = $1 AND metric_name = $2
                 AND recorded_at >= NOW() - ($3 || ' days')::INTERVAL
               ORDER BY recorded_at"#,
        )
        .bind(repo_id)
        .bind(metric_name)
        .bind(days)
        .fetch_all(&self.pool)
        .await?;

        let data_points: Vec<PerformanceTrendData> = rows
            .into_iter()
            .map(PerformanceTrendData::from)
            .collect();

        let avg_value = if data_points.is_empty() {
            0.0
        } else {
            data_points.iter().map(|d| d.metric_value).sum::<f64>() / data_points.len() as f64
        };

        let min_value = data_points.iter().map(|d| d.metric_value).fold(f64::INFINITY, f64::min);
        let max_value = data_points.iter().map(|d| d.metric_value).fold(f64::NEG_INFINITY, f64::max);

        let (trend_direction, change_percent) = if data_points.len() >= 2 {
            let first = data_points.first().unwrap().metric_value;
            let last = data_points.last().unwrap().metric_value;
            let change = if first != 0.0 {
                ((last - first) / first) * 100.0
            } else {
                0.0
            };
            let direction = if change > 5.0 {
                "increasing"
            } else if change < -5.0 {
                "decreasing"
            } else {
                "stable"
            };
            (direction.to_string(), change)
        } else {
            ("unknown".to_string(), 0.0)
        };

        Ok(PerformanceTrendAnalysis {
            metric_name: metric_name.to_string(),
            data_points,
            avg_value,
            min_value,
            max_value,
            trend_direction,
            change_percent,
        })
    }

    pub async fn detect_regressions(
        &self,
        test_id: Uuid,
        repo_id: Uuid,
    ) -> Result<Vec<PerformanceAlert>, sqlx::Error> {
        let baselines = self.list_baselines(repo_id).await?;
        let mut alerts = Vec::new();

        for baseline in baselines {
            if let Some(result) = sqlx::query_as::<_, TestResultRow>(
                r#"SELECT id, test_id, metric_name, metric_value, percentile, recorded_at
                   FROM performance_test_results
                   WHERE test_id = $1 AND metric_name = $2 AND percentile IS NULL
                   LIMIT 1"#,
            )
            .bind(test_id)
            .bind(&baseline.metric_name)
            .fetch_optional(&self.pool)
            .await?
            {
                let current_value = result.metric_value;
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

                    sqlx::query(
                        r#"INSERT INTO performance_regressions (baseline_id, test_id, regression_percent, status, created_at)
                           VALUES ($1, $2, $3, $4, $5)"#,
                    )
                    .bind(baseline.id)
                    .bind(test_id)
                    .bind(regression_percent)
                    .bind("open")
                    .bind(Utc::now())
                    .execute(&self.pool)
                    .await?;

                    alerts.push(PerformanceAlert {
                        baseline_id: baseline.id,
                        metric_name: baseline.metric_name,
                        baseline_value: baseline.baseline_value,
                        current_value,
                        regression_percent,
                        threshold_percent: baseline.threshold_percent,
                        severity: severity.to_string(),
                    });
                }
            }
        }

        Ok(alerts)
    }

    pub async fn list_regressions(
        &self,
        repo_id: Uuid,
        status_filter: Option<&str>,
    ) -> Result<Vec<PerformanceRegression>, sqlx::Error> {
        let query = if let Some(status) = status_filter {
            sqlx::query_as::<_, RegressionRow>(
                r#"SELECT pr.id, pr.baseline_id, pr.test_id, pr.regression_percent, pr.status, pr.created_at
                   FROM performance_regressions pr
                   JOIN performance_baselines pb ON pr.baseline_id = pb.id
                   WHERE pb.repo_id = $1 AND pr.status = $2
                   ORDER BY pr.created_at DESC"#,
            )
            .bind(repo_id)
            .bind(status)
        } else {
            sqlx::query_as::<_, RegressionRow>(
                r#"SELECT pr.id, pr.baseline_id, pr.test_id, pr.regression_percent, pr.status, pr.created_at
                   FROM performance_regressions pr
                   JOIN performance_baselines pb ON pr.baseline_id = pb.id
                   WHERE pb.repo_id = $1
                   ORDER BY pr.created_at DESC"#,
            )
            .bind(repo_id)
        };

        let rows = query.fetch_all(&self.pool).await?;
        Ok(rows.into_iter().map(PerformanceRegression::from).collect())
    }

    pub async fn update_regression_status(
        &self,
        id: Uuid,
        req: RegressionStatusUpdate,
    ) -> Result<PerformanceRegression, sqlx::Error> {
        sqlx::query(r#"UPDATE performance_regressions SET status = $1 WHERE id = $2"#)
            .bind(&req.status)
            .bind(id)
            .execute(&self.pool)
            .await?;

        let row = sqlx::query_as::<_, RegressionRow>(
            r#"SELECT id, baseline_id, test_id, regression_percent, status, created_at
               FROM performance_regressions WHERE id = $1"#,
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await?;

        Ok(PerformanceRegression::from(row))
    }

    pub async fn get_baseline_summary(
        &self,
        repo_id: Uuid,
    ) -> Result<PerformanceBaselineSummary, sqlx::Error> {
        let baselines = self.list_baselines(repo_id).await?;
        let total_baselines = baselines.len() as i64;

        let stats = sqlx::query_as::<_, RegressionStatsRow>(
            r#"SELECT
                COUNT(*) FILTER (WHERE status = 'open') as active_regressions,
                COUNT(*) FILTER (WHERE status = 'resolved') as resolved_regressions
               FROM performance_regressions pr
               JOIN performance_baselines pb ON pr.baseline_id = pb.id
               WHERE pb.repo_id = $1"#,
        )
        .bind(repo_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(PerformanceBaselineSummary {
            total_baselines,
            active_regressions: stats.active_regressions,
            resolved_regressions: stats.resolved_regressions,
            baselines,
        })
    }

    pub async fn create_alert_config(
        &self,
        baseline_id: Uuid,
        req: CreateAlertConfigRequest,
    ) -> Result<PerformanceTestAlertConfig, sqlx::Error> {
        let id = Uuid::new_v4();
        let now = Utc::now();
        let enabled = req.enabled.unwrap_or(true);

        sqlx::query(
            r#"INSERT INTO performance_test_alerts (id, baseline_id, alert_type, threshold, enabled, created_at)
               VALUES ($1, $2, $3, $4, $5, $6)"#,
        )
        .bind(id)
        .bind(baseline_id)
        .bind(&req.alert_type)
        .bind(req.threshold)
        .bind(enabled)
        .bind(now)
        .execute(&self.pool)
        .await?;

        Ok(PerformanceTestAlertConfig {
            id,
            baseline_id,
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
    ) -> Result<Option<PerformanceTestAlertConfig>, sqlx::Error> {
        let row = sqlx::query_as::<_, AlertConfigRow>(
            r#"SELECT id, baseline_id, alert_type, threshold, enabled, last_triggered_at, created_at
               FROM performance_test_alerts WHERE id = $1"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(PerformanceTestAlertConfig::from))
    }

    pub async fn list_alert_configs(
        &self,
        baseline_id: Uuid,
    ) -> Result<Vec<PerformanceTestAlertConfig>, sqlx::Error> {
        let rows = sqlx::query_as::<_, AlertConfigRow>(
            r#"SELECT id, baseline_id, alert_type, threshold, enabled, last_triggered_at, created_at
               FROM performance_test_alerts
               WHERE baseline_id = $1
               ORDER BY created_at"#,
        )
        .bind(baseline_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(PerformanceTestAlertConfig::from).collect())
    }

    pub async fn list_alert_configs_for_repo(
        &self,
        repo_id: Uuid,
    ) -> Result<Vec<PerformanceTestAlertConfig>, sqlx::Error> {
        let rows = sqlx::query_as::<_, AlertConfigRow>(
            r#"SELECT pta.id, pta.baseline_id, pta.alert_type, pta.threshold, pta.enabled, pta.last_triggered_at, pta.created_at
               FROM performance_test_alerts pta
               JOIN performance_baselines pb ON pta.baseline_id = pb.id
               WHERE pb.repo_id = $1
               ORDER BY pta.created_at"#,
        )
        .bind(repo_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(PerformanceTestAlertConfig::from).collect())
    }

    pub async fn update_alert_config(
        &self,
        id: Uuid,
        req: UpdateAlertConfigRequest,
    ) -> Result<PerformanceTestAlertConfig, sqlx::Error> {
        if let Some(ref alert_type) = req.alert_type {
            sqlx::query(r#"UPDATE performance_test_alerts SET alert_type = $1 WHERE id = $2"#)
                .bind(alert_type)
                .bind(id)
                .execute(&self.pool)
                .await?;
        }
        if let Some(threshold) = req.threshold {
            sqlx::query(r#"UPDATE performance_test_alerts SET threshold = $1 WHERE id = $2"#)
                .bind(threshold)
                .bind(id)
                .execute(&self.pool)
                .await?;
        }
        if let Some(enabled) = req.enabled {
            sqlx::query(r#"UPDATE performance_test_alerts SET enabled = $1 WHERE id = $2"#)
                .bind(enabled)
                .bind(id)
                .execute(&self.pool)
                .await?;
        }

        self.get_alert_config(id).await?.ok_or_else(|| sqlx::Error::RowNotFound)
    }

    pub async fn delete_alert_config(&self, id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(r#"DELETE FROM performance_test_alerts WHERE id = $1"#)
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
    ) -> Result<PerformanceAlertHistory, sqlx::Error> {
        let id = Uuid::new_v4();
        let now = Utc::now();

        sqlx::query(
            r#"INSERT INTO performance_test_alert_history (id, alert_id, metric_name, metric_value, threshold, created_at)
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

        sqlx::query(r#"UPDATE performance_test_alerts SET last_triggered_at = $1 WHERE id = $2"#)
            .bind(now)
            .bind(alert_id)
            .execute(&self.pool)
            .await?;

        Ok(PerformanceAlertHistory {
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
    ) -> Result<Vec<PerformanceAlertHistory>, sqlx::Error> {
        let rows = sqlx::query_as::<_, AlertHistoryRow>(
            r#"SELECT id, alert_id, metric_name, metric_value, threshold, created_at
               FROM performance_test_alert_history
               WHERE alert_id = $1
               ORDER BY created_at DESC
               LIMIT $2"#,
        )
        .bind(alert_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(PerformanceAlertHistory::from).collect())
    }

    pub async fn get_alert_notifications(
        &self,
        baseline_id: Uuid,
    ) -> Result<Vec<AlertNotification>, sqlx::Error> {
        let alerts = sqlx::query_as::<_, AlertConfigRow>(
            r#"SELECT pta.id, pta.baseline_id, pta.alert_type, pta.threshold, pta.enabled, pta.last_triggered_at, pta.created_at
               FROM performance_test_alerts pta
               WHERE pta.baseline_id = $1 AND pta.enabled = true"#,
        )
        .bind(baseline_id)
        .fetch_all(&self.pool)
        .await?;

        let mut notifications = Vec::new();
        for alert in alerts {
            let baseline = self.get_baseline(alert.baseline_id).await?;
            if let Some(baseline) = baseline {
                notifications.push(AlertNotification {
                    alert_id: alert.id,
                    metric_name: baseline.metric_name,
                    current_value: baseline.baseline_value,
                    threshold: alert.threshold,
                    severity: if alert.threshold > 50.0 {
                        "critical".to_string()
                    } else if alert.threshold > 25.0 {
                        "high".to_string()
                    } else {
                        "medium".to_string()
                    },
                    message: format!(
                        "Alert configured: {} exceeds {}",
                        alert.alert_type, alert.threshold
                    ),
                });
            }
        }

        Ok(notifications)
    }

    pub async fn get_alert_analytics(
        &self,
        repo_id: Uuid,
    ) -> Result<AlertAnalytics, sqlx::Error> {
        let alerts = self.list_alert_configs_for_repo(repo_id).await?;
        let total_alerts = alerts.len() as i64;
        let active_alerts = alerts.iter().filter(|a| a.enabled).count() as i64;

        let total_triggers = sqlx::query_scalar::<_, i64>(
            r#"SELECT COUNT(*)
               FROM performance_test_alert_history pah
               JOIN performance_test_alerts pta ON pah.alert_id = pta.id
               JOIN performance_baselines pb ON pta.baseline_id = pb.id
               WHERE pb.repo_id = $1"#,
        )
        .bind(repo_id)
        .fetch_one(&self.pool)
        .await?;

        let triggers_by_type = sqlx::query_as::<_, AlertTypeCountRow>(
            r#"SELECT pta.alert_type, COUNT(*) as count
               FROM performance_test_alert_history pah
               JOIN performance_test_alerts pta ON pah.alert_id = pta.id
               JOIN performance_baselines pb ON pta.baseline_id = pb.id
               WHERE pb.repo_id = $1
               GROUP BY pta.alert_type"#,
        )
        .bind(repo_id)
        .fetch_all(&self.pool)
        .await?;

        let mut triggers_by_type_map = serde_json::json!({});
        for row in triggers_by_type {
            triggers_by_type_map[row.alert_type] = serde_json::json!(row.count);
        }

        let last_triggered = alerts.iter().filter_map(|a| a.last_triggered_at).max();

        let trigger_rows = sqlx::query_as::<_, AlertTriggerTrendRow>(
            r#"SELECT
                DATE(pah.created_at) as date,
                COUNT(*) as trigger_count
               FROM performance_test_alert_history pah
               JOIN performance_test_alerts pta ON pah.alert_id = pta.id
               JOIN performance_baselines pb ON pta.baseline_id = pb.id
               WHERE pb.repo_id = $1
                 AND pah.created_at >= NOW() - INTERVAL '30 days'
               GROUP BY DATE(pah.created_at)
               ORDER BY DATE(pah.created_at) DESC"#,
        )
        .bind(repo_id)
        .fetch_all(&self.pool)
        .await?;

        let trigger_trend = trigger_rows.into_iter().map(AlertTriggerTrend::from).collect();

        Ok(AlertAnalytics {
            total_alerts,
            active_alerts,
            total_triggers,
            triggers_by_type: triggers_by_type_map,
            avg_time_between_triggers_ms: 0.0,
            last_triggered_at: last_triggered,
            trigger_trend,
        })
    }

    pub async fn check_and_trigger_alerts(
        &self,
        test_id: Uuid,
        repo_id: Uuid,
    ) -> Result<Vec<AlertNotification>, sqlx::Error> {
        let baselines = self.list_baselines(repo_id).await?;
        let mut notifications = Vec::new();

        for baseline in baselines {
            if let Some(result) = sqlx::query_as::<_, TestResultRow>(
                r#"SELECT id, test_id, metric_name, metric_value, percentile, recorded_at
                   FROM performance_test_results
                   WHERE test_id = $1 AND metric_name = $2 AND percentile IS NULL
                   LIMIT 1"#,
            )
            .bind(test_id)
            .bind(&baseline.metric_name)
            .fetch_optional(&self.pool)
            .await?
            {
                let current_value = result.metric_value;
                let alerts = sqlx::query_as::<_, AlertConfigRow>(
                    r#"SELECT id, baseline_id, alert_type, threshold, enabled, last_triggered_at, created_at
                       FROM performance_test_alerts
                       WHERE baseline_id = $1 AND enabled = true"#,
                )
                .bind(baseline.id)
                .fetch_all(&self.pool)
                .await?;

                for alert in alerts {
                    let triggered = match alert.alert_type.as_str() {
                        "regression" => current_value > baseline.baseline_value * (1.0 + alert.threshold / 100.0),
                        "improvement" => current_value < baseline.baseline_value * (1.0 - alert.threshold / 100.0),
                        "absolute" => current_value > alert.threshold,
                        _ => false,
                    };

                    if triggered {
                        self.trigger_alert(
                            alert.id,
                            &baseline.metric_name,
                            current_value,
                            alert.threshold,
                        )
                        .await?;

                        notifications.push(AlertNotification {
                            alert_id: alert.id,
                            metric_name: baseline.metric_name.clone(),
                            current_value,
                            threshold: alert.threshold,
                            severity: if alert.threshold > 50.0 {
                                "critical".to_string()
                            } else if alert.threshold > 25.0 {
                                "high".to_string()
                            } else {
                                "medium".to_string()
                            },
                            message: format!(
                                "Performance alert triggered: {} {} (current: {}, threshold: {})",
                                baseline.metric_name, alert.alert_type, current_value, alert.threshold
                            ),
                        });
                    }
                }
            }
        }

        Ok(notifications)
    }

    pub async fn create_alert_config_v3(
        &self,
        baseline_id: Uuid,
        req: CreateAlertConfigV3Request,
    ) -> Result<PerformanceTestAlertConfigV3, sqlx::Error> {
        let id = Uuid::new_v4();
        let now = Utc::now();
        let enabled = req.enabled.unwrap_or(true);

        sqlx::query(
            r#"INSERT INTO performance_test_alerts_v3 (id, baseline_id, alert_type, threshold, enabled, created_at)
               VALUES ($1, $2, $3, $4, $5, $6)"#,
        )
        .bind(id)
        .bind(baseline_id)
        .bind(&req.alert_type)
        .bind(req.threshold)
        .bind(enabled)
        .bind(now)
        .execute(&self.pool)
        .await?;

        Ok(PerformanceTestAlertConfigV3 {
            id,
            baseline_id,
            alert_type: req.alert_type,
            threshold: req.threshold,
            enabled,
            last_triggered_at: None,
            created_at: now,
        })
    }

    pub async fn get_alert_config_v3(
        &self,
        id: Uuid,
    ) -> Result<Option<PerformanceTestAlertConfigV3>, sqlx::Error> {
        let row = sqlx::query_as::<_, AlertConfigV3Row>(
            r#"SELECT id, baseline_id, alert_type, threshold, enabled, last_triggered_at, created_at
               FROM performance_test_alerts_v3 WHERE id = $1"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(PerformanceTestAlertConfigV3::from))
    }

    pub async fn list_alert_configs_v3(
        &self,
        baseline_id: Uuid,
    ) -> Result<Vec<PerformanceTestAlertConfigV3>, sqlx::Error> {
        let rows = sqlx::query_as::<_, AlertConfigV3Row>(
            r#"SELECT id, baseline_id, alert_type, threshold, enabled, last_triggered_at, created_at
               FROM performance_test_alerts_v3
               WHERE baseline_id = $1
               ORDER BY created_at"#,
        )
        .bind(baseline_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(PerformanceTestAlertConfigV3::from).collect())
    }

    pub async fn list_alert_configs_v3_for_repo(
        &self,
        repo_id: Uuid,
    ) -> Result<Vec<PerformanceTestAlertConfigV3>, sqlx::Error> {
        let rows = sqlx::query_as::<_, AlertConfigV3Row>(
            r#"SELECT pta.id, pta.baseline_id, pta.alert_type, pta.threshold, pta.enabled, pta.last_triggered_at, pta.created_at
               FROM performance_test_alerts_v3 pta
               JOIN performance_baselines pb ON pta.baseline_id = pb.id
               WHERE pb.repo_id = $1
               ORDER BY pta.created_at"#,
        )
        .bind(repo_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(PerformanceTestAlertConfigV3::from).collect())
    }

    pub async fn update_alert_config_v3(
        &self,
        id: Uuid,
        req: UpdateAlertConfigV3Request,
    ) -> Result<PerformanceTestAlertConfigV3, sqlx::Error> {
        if let Some(ref alert_type) = req.alert_type {
            sqlx::query(r#"UPDATE performance_test_alerts_v3 SET alert_type = $1 WHERE id = $2"#)
                .bind(alert_type)
                .bind(id)
                .execute(&self.pool)
                .await?;
        }
        if let Some(threshold) = req.threshold {
            sqlx::query(r#"UPDATE performance_test_alerts_v3 SET threshold = $1 WHERE id = $2"#)
                .bind(threshold)
                .bind(id)
                .execute(&self.pool)
                .await?;
        }
        if let Some(enabled) = req.enabled {
            sqlx::query(r#"UPDATE performance_test_alerts_v3 SET enabled = $1 WHERE id = $2"#)
                .bind(enabled)
                .bind(id)
                .execute(&self.pool)
                .await?;
        }

        self.get_alert_config_v3(id).await?.ok_or_else(|| sqlx::Error::RowNotFound)
    }

    pub async fn delete_alert_config_v3(&self, id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(r#"DELETE FROM performance_test_alerts_v3 WHERE id = $1"#)
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn trigger_alert_v3(
        &self,
        alert_id: Uuid,
        metric_name: &str,
        metric_value: f64,
        threshold: f64,
    ) -> Result<PerformanceAlertHistoryV3, sqlx::Error> {
        let id = Uuid::new_v4();
        let now = Utc::now();

        sqlx::query(
            r#"INSERT INTO performance_test_alert_history_v3 (id, alert_id, metric_name, metric_value, threshold, created_at)
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

        sqlx::query(r#"UPDATE performance_test_alerts_v3 SET last_triggered_at = $1 WHERE id = $2"#)
            .bind(now)
            .bind(alert_id)
            .execute(&self.pool)
            .await?;

        Ok(PerformanceAlertHistoryV3 {
            id,
            alert_id,
            metric_name: metric_name.to_string(),
            metric_value,
            threshold,
            created_at: now,
        })
    }

    pub async fn get_alert_history_v3(
        &self,
        alert_id: Uuid,
        limit: i64,
    ) -> Result<Vec<PerformanceAlertHistoryV3>, sqlx::Error> {
        let rows = sqlx::query_as::<_, AlertHistoryV3Row>(
            r#"SELECT id, alert_id, metric_name, metric_value, threshold, created_at
               FROM performance_test_alert_history_v3
               WHERE alert_id = $1
               ORDER BY created_at DESC
               LIMIT $2"#,
        )
        .bind(alert_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(PerformanceAlertHistoryV3::from).collect())
    }

    pub async fn get_alert_notifications_v3(
        &self,
        baseline_id: Uuid,
    ) -> Result<Vec<AlertNotificationV3>, sqlx::Error> {
        let alerts = sqlx::query_as::<_, AlertConfigV3Row>(
            r#"SELECT pta.id, pta.baseline_id, pta.alert_type, pta.threshold, pta.enabled, pta.last_triggered_at, pta.created_at
               FROM performance_test_alerts_v3 pta
               WHERE pta.baseline_id = $1 AND pta.enabled = true"#,
        )
        .bind(baseline_id)
        .fetch_all(&self.pool)
        .await?;

        let mut notifications = Vec::new();
        for alert in alerts {
            let baseline = self.get_baseline(alert.baseline_id).await?;
            if let Some(baseline) = baseline {
                notifications.push(AlertNotificationV3 {
                    alert_id: alert.id,
                    metric_name: baseline.metric_name,
                    current_value: baseline.baseline_value,
                    threshold: alert.threshold,
                    severity: if alert.threshold > 50.0 {
                        "critical".to_string()
                    } else if alert.threshold > 25.0 {
                        "high".to_string()
                    } else {
                        "medium".to_string()
                    },
                    message: format!(
                        "Alert configured: {} exceeds {}",
                        alert.alert_type, alert.threshold
                    ),
                });
            }
        }

        Ok(notifications)
    }

    pub async fn get_alert_analytics_v3(
        &self,
        repo_id: Uuid,
    ) -> Result<AlertAnalyticsV3, sqlx::Error> {
        let alerts = self.list_alert_configs_v3_for_repo(repo_id).await?;
        let total_alerts = alerts.len() as i64;
        let active_alerts = alerts.iter().filter(|a| a.enabled).count() as i64;

        let total_triggers = sqlx::query_scalar::<_, i64>(
            r#"SELECT COUNT(*)
               FROM performance_test_alert_history_v3 pah
               JOIN performance_test_alerts_v3 pta ON pah.alert_id = pta.id
               JOIN performance_baselines pb ON pta.baseline_id = pb.id
               WHERE pb.repo_id = $1"#,
        )
        .bind(repo_id)
        .fetch_one(&self.pool)
        .await?;

        let triggers_by_type = sqlx::query_as::<_, AlertTypeCountV3Row>(
            r#"SELECT pta.alert_type, COUNT(*) as count
               FROM performance_test_alert_history_v3 pah
               JOIN performance_test_alerts_v3 pta ON pah.alert_id = pta.id
               JOIN performance_baselines pb ON pta.baseline_id = pb.id
               WHERE pb.repo_id = $1
               GROUP BY pta.alert_type"#,
        )
        .bind(repo_id)
        .fetch_all(&self.pool)
        .await?;

        let mut triggers_by_type_map = serde_json::json!({});
        for row in triggers_by_type {
            triggers_by_type_map[row.alert_type] = serde_json::json!(row.count);
        }

        let last_triggered = alerts.iter().filter_map(|a| a.last_triggered_at).max();

        let trigger_rows = sqlx::query_as::<_, AlertTriggerTrendV3Row>(
            r#"SELECT
                DATE(pah.created_at) as date,
                COUNT(*) as trigger_count
               FROM performance_test_alert_history_v3 pah
               JOIN performance_test_alerts_v3 pta ON pah.alert_id = pta.id
               JOIN performance_baselines pb ON pta.baseline_id = pb.id
               WHERE pb.repo_id = $1
                 AND pah.created_at >= NOW() - INTERVAL '30 days'
               GROUP BY DATE(pah.created_at)
               ORDER BY DATE(pah.created_at) DESC"#,
        )
        .bind(repo_id)
        .fetch_all(&self.pool)
        .await?;

        let trigger_trend = trigger_rows.into_iter().map(AlertTriggerTrendV3::from).collect();

        Ok(AlertAnalyticsV3 {
            total_alerts,
            active_alerts,
            total_triggers,
            triggers_by_type: triggers_by_type_map,
            avg_time_between_triggers_ms: 0.0,
            last_triggered_at: last_triggered,
            trigger_trend,
        })
    }

    pub async fn check_and_trigger_alerts_v3(
        &self,
        test_id: Uuid,
        repo_id: Uuid,
    ) -> Result<Vec<AlertNotificationV3>, sqlx::Error> {
        let baselines = self.list_baselines(repo_id).await?;
        let mut notifications = Vec::new();

        for baseline in baselines {
            if let Some(result) = sqlx::query_as::<_, TestResultRow>(
                r#"SELECT id, test_id, metric_name, metric_value, percentile, recorded_at
                   FROM performance_test_results
                   WHERE test_id = $1 AND metric_name = $2 AND percentile IS NULL
                   LIMIT 1"#,
            )
            .bind(test_id)
            .bind(&baseline.metric_name)
            .fetch_optional(&self.pool)
            .await?
            {
                let current_value = result.metric_value;
                let alerts = sqlx::query_as::<_, AlertConfigV3Row>(
                    r#"SELECT id, baseline_id, alert_type, threshold, enabled, last_triggered_at, created_at
                       FROM performance_test_alerts_v3
                       WHERE baseline_id = $1 AND enabled = true"#,
                )
                .bind(baseline.id)
                .fetch_all(&self.pool)
                .await?;

                for alert in alerts {
                    let triggered = match alert.alert_type.as_str() {
                        "regression" => current_value > baseline.baseline_value * (1.0 + alert.threshold / 100.0),
                        "improvement" => current_value < baseline.baseline_value * (1.0 - alert.threshold / 100.0),
                        "absolute" => current_value > alert.threshold,
                        _ => false,
                    };

                    if triggered {
                        self.trigger_alert_v3(
                            alert.id,
                            &baseline.metric_name,
                            current_value,
                            alert.threshold,
                        )
                        .await?;

                        notifications.push(AlertNotificationV3 {
                            alert_id: alert.id,
                            metric_name: baseline.metric_name.clone(),
                            current_value,
                            threshold: alert.threshold,
                            severity: if alert.threshold > 50.0 {
                                "critical".to_string()
                            } else if alert.threshold > 25.0 {
                                "high".to_string()
                            } else {
                                "medium".to_string()
                            },
                            message: format!(
                                "Performance alert triggered: {} {} (current: {}, threshold: {})",
                                baseline.metric_name, alert.alert_type, current_value, alert.threshold
                            ),
                        });
                    }
                }
            }
        }

        Ok(notifications)
    }

    pub async fn create_alert_config_v5(
        &self,
        baseline_id: Uuid,
        req: CreateAlertConfigV5Request,
    ) -> Result<PerformanceTestAlertConfigV5, sqlx::Error> {
        let id = Uuid::new_v4();
        let now = Utc::now();
        let enabled = req.enabled.unwrap_or(true);

        sqlx::query(
            r#"INSERT INTO performance_test_alerts_v5 (id, baseline_id, alert_type, threshold, enabled, created_at)
               VALUES ($1, $2, $3, $4, $5, $6)"#,
        )
        .bind(id)
        .bind(baseline_id)
        .bind(&req.alert_type)
        .bind(req.threshold)
        .bind(enabled)
        .bind(now)
        .execute(&self.pool)
        .await?;

        Ok(PerformanceTestAlertConfigV5 {
            id,
            baseline_id,
            alert_type: req.alert_type,
            threshold: req.threshold,
            enabled,
            last_triggered_at: None,
            created_at: now,
        })
    }

    pub async fn get_alert_config_v5(
        &self,
        id: Uuid,
    ) -> Result<Option<PerformanceTestAlertConfigV5>, sqlx::Error> {
        let row = sqlx::query_as::<_, AlertConfigV5Row>(
            r#"SELECT id, baseline_id, alert_type, threshold, enabled, last_triggered_at, created_at
               FROM performance_test_alerts_v5 WHERE id = $1"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(PerformanceTestAlertConfigV5::from))
    }

    pub async fn list_alert_configs_v5(
        &self,
        baseline_id: Uuid,
    ) -> Result<Vec<PerformanceTestAlertConfigV5>, sqlx::Error> {
        let rows = sqlx::query_as::<_, AlertConfigV5Row>(
            r#"SELECT id, baseline_id, alert_type, threshold, enabled, last_triggered_at, created_at
               FROM performance_test_alerts_v5
               WHERE baseline_id = $1
               ORDER BY created_at"#,
        )
        .bind(baseline_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(PerformanceTestAlertConfigV5::from).collect())
    }

    pub async fn list_alert_configs_v5_for_repo(
        &self,
        repo_id: Uuid,
    ) -> Result<Vec<PerformanceTestAlertConfigV5>, sqlx::Error> {
        let rows = sqlx::query_as::<_, AlertConfigV5Row>(
            r#"SELECT pta.id, pta.baseline_id, pta.alert_type, pta.threshold, pta.enabled, pta.last_triggered_at, pta.created_at
               FROM performance_test_alerts_v5 pta
               JOIN performance_baselines pb ON pta.baseline_id = pb.id
               WHERE pb.repo_id = $1
               ORDER BY pta.created_at"#,
        )
        .bind(repo_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(PerformanceTestAlertConfigV5::from).collect())
    }

    pub async fn update_alert_config_v5(
        &self,
        id: Uuid,
        req: UpdateAlertConfigV5Request,
    ) -> Result<PerformanceTestAlertConfigV5, sqlx::Error> {
        if let Some(ref alert_type) = req.alert_type {
            sqlx::query(r#"UPDATE performance_test_alerts_v5 SET alert_type = $1 WHERE id = $2"#)
                .bind(alert_type)
                .bind(id)
                .execute(&self.pool)
                .await?;
        }
        if let Some(threshold) = req.threshold {
            sqlx::query(r#"UPDATE performance_test_alerts_v5 SET threshold = $1 WHERE id = $2"#)
                .bind(threshold)
                .bind(id)
                .execute(&self.pool)
                .await?;
        }
        if let Some(enabled) = req.enabled {
            sqlx::query(r#"UPDATE performance_test_alerts_v5 SET enabled = $1 WHERE id = $2"#)
                .bind(enabled)
                .bind(id)
                .execute(&self.pool)
                .await?;
        }

        self.get_alert_config_v5(id).await?.ok_or_else(|| sqlx::Error::RowNotFound)
    }

    pub async fn delete_alert_config_v5(&self, id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(r#"DELETE FROM performance_test_alerts_v5 WHERE id = $1"#)
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn trigger_alert_v5(
        &self,
        alert_id: Uuid,
        metric_name: &str,
        metric_value: f64,
        threshold: f64,
    ) -> Result<PerformanceAlertHistoryV5, sqlx::Error> {
        let id = Uuid::new_v4();
        let now = Utc::now();

        sqlx::query(
            r#"INSERT INTO performance_test_alert_history_v5 (id, alert_id, metric_name, metric_value, threshold, created_at)
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

        sqlx::query(r#"UPDATE performance_test_alerts_v5 SET last_triggered_at = $1 WHERE id = $2"#)
            .bind(now)
            .bind(alert_id)
            .execute(&self.pool)
            .await?;

        Ok(PerformanceAlertHistoryV5 {
            id,
            alert_id,
            metric_name: metric_name.to_string(),
            metric_value,
            threshold,
            created_at: now,
        })
    }

    pub async fn get_alert_history_v5(
        &self,
        alert_id: Uuid,
        limit: i64,
    ) -> Result<Vec<PerformanceAlertHistoryV5>, sqlx::Error> {
        let rows = sqlx::query_as::<_, AlertHistoryV5Row>(
            r#"SELECT id, alert_id, metric_name, metric_value, threshold, created_at
               FROM performance_test_alert_history_v5
               WHERE alert_id = $1
               ORDER BY created_at DESC
               LIMIT $2"#,
        )
        .bind(alert_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(PerformanceAlertHistoryV5::from).collect())
    }

    pub async fn get_alert_notifications_v5(
        &self,
        baseline_id: Uuid,
    ) -> Result<Vec<AlertNotificationV5>, sqlx::Error> {
        let alerts = sqlx::query_as::<_, AlertConfigV5Row>(
            r#"SELECT pta.id, pta.baseline_id, pta.alert_type, pta.threshold, pta.enabled, pta.last_triggered_at, pta.created_at
               FROM performance_test_alerts_v5 pta
               WHERE pta.baseline_id = $1 AND pta.enabled = true"#,
        )
        .bind(baseline_id)
        .fetch_all(&self.pool)
        .await?;

        let mut notifications = Vec::new();
        for alert in alerts {
            let baseline = self.get_baseline(alert.baseline_id).await?;
            if let Some(baseline) = baseline {
                notifications.push(AlertNotificationV5 {
                    alert_id: alert.id,
                    metric_name: baseline.metric_name,
                    current_value: baseline.baseline_value,
                    threshold: alert.threshold,
                    severity: if alert.threshold > 50.0 {
                        "critical".to_string()
                    } else if alert.threshold > 25.0 {
                        "high".to_string()
                    } else {
                        "medium".to_string()
                    },
                    message: format!(
                        "Alert configured: {} exceeds {}",
                        alert.alert_type, alert.threshold
                    ),
                });
            }
        }

        Ok(notifications)
    }

    pub async fn get_alert_analytics_v5(
        &self,
        repo_id: Uuid,
    ) -> Result<AlertAnalyticsV5, sqlx::Error> {
        let alerts = self.list_alert_configs_v5_for_repo(repo_id).await?;
        let total_alerts = alerts.len() as i64;
        let active_alerts = alerts.iter().filter(|a| a.enabled).count() as i64;

        let total_triggers = sqlx::query_scalar::<_, i64>(
            r#"SELECT COUNT(*)
               FROM performance_test_alert_history_v5 pah
               JOIN performance_test_alerts_v5 pta ON pah.alert_id = pta.id
               JOIN performance_baselines pb ON pta.baseline_id = pb.id
               WHERE pb.repo_id = $1"#,
        )
        .bind(repo_id)
        .fetch_one(&self.pool)
        .await?;

        let triggers_by_type = sqlx::query_as::<_, AlertTypeCountV5Row>(
            r#"SELECT pta.alert_type, COUNT(*) as count
               FROM performance_test_alert_history_v5 pah
               JOIN performance_test_alerts_v5 pta ON pah.alert_id = pta.id
               JOIN performance_baselines pb ON pta.baseline_id = pb.id
               WHERE pb.repo_id = $1
               GROUP BY pta.alert_type"#,
        )
        .bind(repo_id)
        .fetch_all(&self.pool)
        .await?;

        let mut triggers_by_type_map = serde_json::json!({});
        for row in triggers_by_type {
            triggers_by_type_map[row.alert_type] = serde_json::json!(row.count);
        }

        let last_triggered = alerts.iter().filter_map(|a| a.last_triggered_at).max();

        let trigger_rows = sqlx::query_as::<_, AlertTriggerTrendV5Row>(
            r#"SELECT
                DATE(pah.created_at) as date,
                COUNT(*) as trigger_count
               FROM performance_test_alert_history_v5 pah
               JOIN performance_test_alerts_v5 pta ON pah.alert_id = pta.id
               JOIN performance_baselines pb ON pta.baseline_id = pb.id
               WHERE pb.repo_id = $1
                 AND pah.created_at >= NOW() - INTERVAL '30 days'
               GROUP BY DATE(pah.created_at)
               ORDER BY DATE(pah.created_at) DESC"#,
        )
        .bind(repo_id)
        .fetch_all(&self.pool)
        .await?;

        let trigger_trend = trigger_rows.into_iter().map(AlertTriggerTrendV5::from).collect();

        Ok(AlertAnalyticsV5 {
            total_alerts,
            active_alerts,
            total_triggers,
            triggers_by_type: triggers_by_type_map,
            avg_time_between_triggers_ms: 0.0,
            last_triggered_at: last_triggered,
            trigger_trend,
        })
    }

    pub async fn check_and_trigger_alerts_v5(
        &self,
        test_id: Uuid,
        repo_id: Uuid,
    ) -> Result<Vec<AlertNotificationV5>, sqlx::Error> {
        let baselines = self.list_baselines(repo_id).await?;
        let mut notifications = Vec::new();

        for baseline in baselines {
            if let Some(result) = sqlx::query_as::<_, TestResultRow>(
                r#"SELECT id, test_id, metric_name, metric_value, percentile, recorded_at
                   FROM performance_test_results
                   WHERE test_id = $1 AND metric_name = $2 AND percentile IS NULL
                   LIMIT 1"#,
            )
            .bind(test_id)
            .bind(&baseline.metric_name)
            .fetch_optional(&self.pool)
            .await?
            {
                let current_value = result.metric_value;
                let alerts = sqlx::query_as::<_, AlertConfigV5Row>(
                    r#"SELECT id, baseline_id, alert_type, threshold, enabled, last_triggered_at, created_at
                       FROM performance_test_alerts_v5
                       WHERE baseline_id = $1 AND enabled = true"#,
                )
                .bind(baseline.id)
                .fetch_all(&self.pool)
                .await?;

                for alert in alerts {
                    let triggered = match alert.alert_type.as_str() {
                        "regression" => current_value > baseline.baseline_value * (1.0 + alert.threshold / 100.0),
                        "improvement" => current_value < baseline.baseline_value * (1.0 - alert.threshold / 100.0),
                        "absolute" => current_value > alert.threshold,
                        _ => false,
                    };

                    if triggered {
                        self.trigger_alert_v5(
                            alert.id,
                            &baseline.metric_name,
                            current_value,
                            alert.threshold,
                        )
                        .await?;

                        notifications.push(AlertNotificationV5 {
                            alert_id: alert.id,
                            metric_name: baseline.metric_name.clone(),
                            current_value,
                            threshold: alert.threshold,
                            severity: if alert.threshold > 50.0 {
                                "critical".to_string()
                            } else if alert.threshold > 25.0 {
                                "high".to_string()
                            } else {
                                "medium".to_string()
                            },
                            message: format!(
                                "Performance alert triggered: {} {} (current: {}, threshold: {})",
                                baseline.metric_name, alert.alert_type, current_value, alert.threshold
                            ),
                        });
                    }
                }
            }
        }

        Ok(notifications)
    }

    pub async fn create_alert_config_v9(
        &self,
        baseline_id: Uuid,
        req: CreateAlertConfigV9Request,
    ) -> Result<PerformanceTestAlertConfigV9, sqlx::Error> {
        let id = Uuid::new_v4();
        let now = Utc::now();
        let enabled = req.enabled.unwrap_or(true);

        sqlx::query(
            r#"INSERT INTO performance_test_alerts_v9 (id, baseline_id, alert_type, threshold, enabled, created_at)
               VALUES ($1, $2, $3, $4, $5, $6)"#,
        )
        .bind(id)
        .bind(baseline_id)
        .bind(&req.alert_type)
        .bind(req.threshold)
        .bind(enabled)
        .bind(now)
        .execute(&self.pool)
        .await?;

        Ok(PerformanceTestAlertConfigV9 {
            id,
            baseline_id,
            alert_type: req.alert_type,
            threshold: req.threshold,
            enabled,
            last_triggered_at: None,
            created_at: now,
        })
    }

    pub async fn get_alert_config_v9(
        &self,
        id: Uuid,
    ) -> Result<Option<PerformanceTestAlertConfigV9>, sqlx::Error> {
        let row = sqlx::query_as::<_, AlertConfigV9Row>(
            r#"SELECT id, baseline_id, alert_type, threshold, enabled, last_triggered_at, created_at
               FROM performance_test_alerts_v9 WHERE id = $1"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(PerformanceTestAlertConfigV9::from))
    }

    pub async fn list_alert_configs_v9(
        &self,
        baseline_id: Uuid,
    ) -> Result<Vec<PerformanceTestAlertConfigV9>, sqlx::Error> {
        let rows = sqlx::query_as::<_, AlertConfigV9Row>(
            r#"SELECT id, baseline_id, alert_type, threshold, enabled, last_triggered_at, created_at
               FROM performance_test_alerts_v9
               WHERE baseline_id = $1
               ORDER BY created_at"#,
        )
        .bind(baseline_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(PerformanceTestAlertConfigV9::from).collect())
    }

    pub async fn list_alert_configs_v9_for_repo(
        &self,
        repo_id: Uuid,
    ) -> Result<Vec<PerformanceTestAlertConfigV9>, sqlx::Error> {
        let rows = sqlx::query_as::<_, AlertConfigV9Row>(
            r#"SELECT pta.id, pta.baseline_id, pta.alert_type, pta.threshold, pta.enabled, pta.last_triggered_at, pta.created_at
               FROM performance_test_alerts_v9 pta
               JOIN performance_baselines pb ON pta.baseline_id = pb.id
               WHERE pb.repo_id = $1
               ORDER BY pta.created_at"#,
        )
        .bind(repo_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(PerformanceTestAlertConfigV9::from).collect())
    }

    pub async fn update_alert_config_v9(
        &self,
        id: Uuid,
        req: UpdateAlertConfigV9Request,
    ) -> Result<PerformanceTestAlertConfigV9, sqlx::Error> {
        if let Some(ref alert_type) = req.alert_type {
            sqlx::query(r#"UPDATE performance_test_alerts_v9 SET alert_type = $1 WHERE id = $2"#)
                .bind(alert_type)
                .bind(id)
                .execute(&self.pool)
                .await?;
        }
        if let Some(threshold) = req.threshold {
            sqlx::query(r#"UPDATE performance_test_alerts_v9 SET threshold = $1 WHERE id = $2"#)
                .bind(threshold)
                .bind(id)
                .execute(&self.pool)
                .await?;
        }
        if let Some(enabled) = req.enabled {
            sqlx::query(r#"UPDATE performance_test_alerts_v9 SET enabled = $1 WHERE id = $2"#)
                .bind(enabled)
                .bind(id)
                .execute(&self.pool)
                .await?;
        }

        self.get_alert_config_v9(id).await?.ok_or_else(|| sqlx::Error::RowNotFound)
    }

    pub async fn delete_alert_config_v9(&self, id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(r#"DELETE FROM performance_test_alerts_v9 WHERE id = $1"#)
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn trigger_alert_v9(
        &self,
        alert_id: Uuid,
        metric_name: &str,
        metric_value: f64,
        threshold: f64,
    ) -> Result<PerformanceAlertHistoryV9, sqlx::Error> {
        let id = Uuid::new_v4();
        let now = Utc::now();

        sqlx::query(
            r#"INSERT INTO performance_test_alert_history_v9 (id, alert_id, metric_name, metric_value, threshold, created_at)
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

        sqlx::query(r#"UPDATE performance_test_alerts_v9 SET last_triggered_at = $1 WHERE id = $2"#)
            .bind(now)
            .bind(alert_id)
            .execute(&self.pool)
            .await?;

        Ok(PerformanceAlertHistoryV9 {
            id,
            alert_id,
            metric_name: metric_name.to_string(),
            metric_value,
            threshold,
            created_at: now,
        })
    }

    pub async fn get_alert_history_v9(
        &self,
        alert_id: Uuid,
        limit: i64,
    ) -> Result<Vec<PerformanceAlertHistoryV9>, sqlx::Error> {
        let rows = sqlx::query_as::<_, AlertHistoryV9Row>(
            r#"SELECT id, alert_id, metric_name, metric_value, threshold, created_at
               FROM performance_test_alert_history_v9
               WHERE alert_id = $1
               ORDER BY created_at DESC
               LIMIT $2"#,
        )
        .bind(alert_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(PerformanceAlertHistoryV9::from).collect())
    }

    pub async fn get_alert_notifications_v9(
        &self,
        baseline_id: Uuid,
    ) -> Result<Vec<AlertNotificationV9>, sqlx::Error> {
        let alerts = sqlx::query_as::<_, AlertConfigV9Row>(
            r#"SELECT pta.id, pta.baseline_id, pta.alert_type, pta.threshold, pta.enabled, pta.last_triggered_at, pta.created_at
               FROM performance_test_alerts_v9 pta
               WHERE pta.baseline_id = $1 AND pta.enabled = true"#,
        )
        .bind(baseline_id)
        .fetch_all(&self.pool)
        .await?;

        let mut notifications = Vec::new();
        for alert in alerts {
            let baseline = self.get_baseline(alert.baseline_id).await?;
            if let Some(baseline) = baseline {
                notifications.push(AlertNotificationV9 {
                    alert_id: alert.id,
                    metric_name: baseline.metric_name,
                    current_value: baseline.baseline_value,
                    threshold: alert.threshold,
                    severity: if alert.threshold > 50.0 {
                        "critical".to_string()
                    } else if alert.threshold > 25.0 {
                        "high".to_string()
                    } else {
                        "medium".to_string()
                    },
                    message: format!(
                        "Alert configured: {} exceeds {}",
                        alert.alert_type, alert.threshold
                    ),
                });
            }
        }

        Ok(notifications)
    }

    pub async fn get_alert_analytics_v9(
        &self,
        repo_id: Uuid,
    ) -> Result<AlertAnalyticsV9, sqlx::Error> {
        let alerts = self.list_alert_configs_v9_for_repo(repo_id).await?;
        let total_alerts = alerts.len() as i64;
        let active_alerts = alerts.iter().filter(|a| a.enabled).count() as i64;

        let total_triggers = sqlx::query_scalar::<_, i64>(
            r#"SELECT COUNT(*)
               FROM performance_test_alert_history_v9 pah
               JOIN performance_test_alerts_v9 pta ON pah.alert_id = pta.id
               JOIN performance_baselines pb ON pta.baseline_id = pb.id
               WHERE pb.repo_id = $1"#,
        )
        .bind(repo_id)
        .fetch_one(&self.pool)
        .await?;

        let triggers_by_type = sqlx::query_as::<_, AlertTypeCountV9Row>(
            r#"SELECT pta.alert_type, COUNT(*) as count
               FROM performance_test_alert_history_v9 pah
               JOIN performance_test_alerts_v9 pta ON pah.alert_id = pta.id
               JOIN performance_baselines pb ON pta.baseline_id = pb.id
               WHERE pb.repo_id = $1
               GROUP BY pta.alert_type"#,
        )
        .bind(repo_id)
        .fetch_all(&self.pool)
        .await?;

        let mut triggers_by_type_map = serde_json::json!({});
        for row in triggers_by_type {
            triggers_by_type_map[row.alert_type] = serde_json::json!(row.count);
        }

        let last_triggered = alerts.iter().filter_map(|a| a.last_triggered_at).max();

        let trigger_rows = sqlx::query_as::<_, AlertTriggerTrendV9Row>(
            r#"SELECT
                DATE(pah.created_at) as date,
                COUNT(*) as trigger_count
               FROM performance_test_alert_history_v9 pah
               JOIN performance_test_alerts_v9 pta ON pah.alert_id = pta.id
               JOIN performance_baselines pb ON pta.baseline_id = pb.id
               WHERE pb.repo_id = $1
                 AND pah.created_at >= NOW() - INTERVAL '30 days'
               GROUP BY DATE(pah.created_at)
               ORDER BY DATE(pah.created_at) DESC"#,
        )
        .bind(repo_id)
        .fetch_all(&self.pool)
        .await?;

        let trigger_trend = trigger_rows.into_iter().map(AlertTriggerTrendV9::from).collect();

        Ok(AlertAnalyticsV9 {
            total_alerts,
            active_alerts,
            total_triggers,
            triggers_by_type: triggers_by_type_map,
            avg_time_between_triggers_ms: 0.0,
            last_triggered_at: last_triggered,
            trigger_trend,
        })
    }

    pub async fn check_and_trigger_alerts_v9(
        &self,
        test_id: Uuid,
        repo_id: Uuid,
    ) -> Result<Vec<AlertNotificationV9>, sqlx::Error> {
        let baselines = self.list_baselines(repo_id).await?;
        let mut notifications = Vec::new();

        for baseline in baselines {
            if let Some(result) = sqlx::query_as::<_, TestResultRow>(
                r#"SELECT id, test_id, metric_name, metric_value, percentile, recorded_at
                   FROM performance_test_results
                   WHERE test_id = $1 AND metric_name = $2 AND percentile IS NULL
                   LIMIT 1"#,
            )
            .bind(test_id)
            .bind(&baseline.metric_name)
            .fetch_optional(&self.pool)
            .await?
            {
                let current_value = result.metric_value;
                let alerts = sqlx::query_as::<_, AlertConfigV9Row>(
                    r#"SELECT id, baseline_id, alert_type, threshold, enabled, last_triggered_at, created_at
                       FROM performance_test_alerts_v9
                       WHERE baseline_id = $1 AND enabled = true"#,
                )
                .bind(baseline.id)
                .fetch_all(&self.pool)
                .await?;

                for alert in alerts {
                    let triggered = match alert.alert_type.as_str() {
                        "regression" => current_value > baseline.baseline_value * (1.0 + alert.threshold / 100.0),
                        "improvement" => current_value < baseline.baseline_value * (1.0 - alert.threshold / 100.0),
                        "absolute" => current_value > alert.threshold,
                        _ => false,
                    };

                    if triggered {
                        self.trigger_alert_v9(
                            alert.id,
                            &baseline.metric_name,
                            current_value,
                            alert.threshold,
                        )
                        .await?;

                        notifications.push(AlertNotificationV9 {
                            alert_id: alert.id,
                            metric_name: baseline.metric_name.clone(),
                            current_value,
                            threshold: alert.threshold,
                            severity: if alert.threshold > 50.0 {
                                "critical".to_string()
                            } else if alert.threshold > 25.0 {
                                "high".to_string()
                            } else {
                                "medium".to_string()
                            },
                            message: format!(
                                "Performance alert triggered: {} {} (current: {}, threshold: {})",
                                baseline.metric_name, alert.alert_type, current_value, alert.threshold
                            ),
                        });
                    }
                }
            }
        }

        Ok(notifications)
    }

    pub async fn create_alert_config_v16(
        &self,
        baseline_id: Uuid,
        req: CreateAlertConfigV16Request,
    ) -> Result<PerformanceTestAlertConfigV16, sqlx::Error> {
        let id = Uuid::new_v4();
        let now = Utc::now();
        let enabled = req.enabled.unwrap_or(true);

        sqlx::query(
            r#"INSERT INTO performance_test_alerts_v16 (id, baseline_id, alert_type, threshold, enabled, created_at)
               VALUES ($1, $2, $3, $4, $5, $6)"#,
        )
        .bind(id)
        .bind(baseline_id)
        .bind(&req.alert_type)
        .bind(req.threshold)
        .bind(enabled)
        .bind(now)
        .execute(&self.pool)
        .await?;

        Ok(PerformanceTestAlertConfigV16 {
            id,
            baseline_id,
            alert_type: req.alert_type,
            threshold: req.threshold,
            enabled,
            last_triggered_at: None,
            created_at: now,
        })
    }

    pub async fn get_alert_config_v16(
        &self,
        id: Uuid,
    ) -> Result<Option<PerformanceTestAlertConfigV16>, sqlx::Error> {
        let row = sqlx::query_as::<_, AlertConfigV16Row>(
            r#"SELECT id, baseline_id, alert_type, threshold, enabled, last_triggered_at, created_at
               FROM performance_test_alerts_v16 WHERE id = $1"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(PerformanceTestAlertConfigV16::from))
    }

    pub async fn list_alert_configs_v16(
        &self,
        baseline_id: Uuid,
    ) -> Result<Vec<PerformanceTestAlertConfigV16>, sqlx::Error> {
        let rows = sqlx::query_as::<_, AlertConfigV16Row>(
            r#"SELECT id, baseline_id, alert_type, threshold, enabled, last_triggered_at, created_at
               FROM performance_test_alerts_v16
               WHERE baseline_id = $1
               ORDER BY created_at"#,
        )
        .bind(baseline_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(PerformanceTestAlertConfigV16::from).collect())
    }

    pub async fn list_alert_configs_v16_for_repo(
        &self,
        repo_id: Uuid,
    ) -> Result<Vec<PerformanceTestAlertConfigV16>, sqlx::Error> {
        let rows = sqlx::query_as::<_, AlertConfigV16Row>(
            r#"SELECT pta.id, pta.baseline_id, pta.alert_type, pta.threshold, pta.enabled, pta.last_triggered_at, pta.created_at
               FROM performance_test_alerts_v16 pta
               JOIN performance_baselines pb ON pta.baseline_id = pb.id
               WHERE pb.repo_id = $1
               ORDER BY pta.created_at"#,
        )
        .bind(repo_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(PerformanceTestAlertConfigV16::from).collect())
    }

    pub async fn update_alert_config_v16(
        &self,
        id: Uuid,
        req: UpdateAlertConfigV16Request,
    ) -> Result<PerformanceTestAlertConfigV16, sqlx::Error> {
        if let Some(ref alert_type) = req.alert_type {
            sqlx::query(r#"UPDATE performance_test_alerts_v16 SET alert_type = $1 WHERE id = $2"#)
                .bind(alert_type)
                .bind(id)
                .execute(&self.pool)
                .await?;
        }
        if let Some(threshold) = req.threshold {
            sqlx::query(r#"UPDATE performance_test_alerts_v16 SET threshold = $1 WHERE id = $2"#)
                .bind(threshold)
                .bind(id)
                .execute(&self.pool)
                .await?;
        }
        if let Some(enabled) = req.enabled {
            sqlx::query(r#"UPDATE performance_test_alerts_v16 SET enabled = $1 WHERE id = $2"#)
                .bind(enabled)
                .bind(id)
                .execute(&self.pool)
                .await?;
        }

        self.get_alert_config_v16(id).await?.ok_or_else(|| sqlx::Error::RowNotFound)
    }

    pub async fn delete_alert_config_v16(&self, id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(r#"DELETE FROM performance_test_alerts_v16 WHERE id = $1"#)
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn trigger_alert_v16(
        &self,
        alert_id: Uuid,
        metric_name: &str,
        metric_value: f64,
        threshold: f64,
    ) -> Result<PerformanceAlertHistoryV16, sqlx::Error> {
        let id = Uuid::new_v4();
        let now = Utc::now();

        sqlx::query(
            r#"INSERT INTO performance_test_alert_history_v16 (id, alert_id, metric_name, metric_value, threshold, created_at)
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

        sqlx::query(r#"UPDATE performance_test_alerts_v16 SET last_triggered_at = $1 WHERE id = $2"#)
            .bind(now)
            .bind(alert_id)
            .execute(&self.pool)
            .await?;

        Ok(PerformanceAlertHistoryV16 {
            id,
            alert_id,
            metric_name: metric_name.to_string(),
            metric_value,
            threshold,
            created_at: now,
        })
    }

    pub async fn get_alert_history_v16(
        &self,
        alert_id: Uuid,
        limit: i64,
    ) -> Result<Vec<PerformanceAlertHistoryV16>, sqlx::Error> {
        let rows = sqlx::query_as::<_, AlertHistoryV16Row>(
            r#"SELECT id, alert_id, metric_name, metric_value, threshold, created_at
               FROM performance_test_alert_history_v16
               WHERE alert_id = $1
               ORDER BY created_at DESC
               LIMIT $2"#,
        )
        .bind(alert_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(PerformanceAlertHistoryV16::from).collect())
    }

    pub async fn get_alert_notifications_v16(
        &self,
        baseline_id: Uuid,
    ) -> Result<Vec<AlertNotificationV16>, sqlx::Error> {
        let alerts = sqlx::query_as::<_, AlertConfigV16Row>(
            r#"SELECT pta.id, pta.baseline_id, pta.alert_type, pta.threshold, pta.enabled, pta.last_triggered_at, pta.created_at
               FROM performance_test_alerts_v16 pta
               WHERE pta.baseline_id = $1 AND pta.enabled = true"#,
        )
        .bind(baseline_id)
        .fetch_all(&self.pool)
        .await?;

        let mut notifications = Vec::new();
        for alert in alerts {
            let baseline = self.get_baseline(alert.baseline_id).await?;
            if let Some(baseline) = baseline {
                notifications.push(AlertNotificationV16 {
                    alert_id: alert.id,
                    metric_name: baseline.metric_name,
                    current_value: baseline.baseline_value,
                    threshold: alert.threshold,
                    severity: if alert.threshold > 50.0 {
                        "critical".to_string()
                    } else if alert.threshold > 25.0 {
                        "high".to_string()
                    } else {
                        "medium".to_string()
                    },
                    message: format!(
                        "Alert configured: {} exceeds {}",
                        alert.alert_type, alert.threshold
                    ),
                });
            }
        }

        Ok(notifications)
    }

    pub async fn get_alert_analytics_v16(
        &self,
        repo_id: Uuid,
    ) -> Result<AlertAnalyticsV16, sqlx::Error> {
        let alerts = self.list_alert_configs_v16_for_repo(repo_id).await?;
        let total_alerts = alerts.len() as i64;
        let active_alerts = alerts.iter().filter(|a| a.enabled).count() as i64;

        let total_triggers = sqlx::query_scalar::<_, i64>(
            r#"SELECT COUNT(*)
               FROM performance_test_alert_history_v16 pah
               JOIN performance_test_alerts_v16 pta ON pah.alert_id = pta.id
               JOIN performance_baselines pb ON pta.baseline_id = pb.id
               WHERE pb.repo_id = $1"#,
        )
        .bind(repo_id)
        .fetch_one(&self.pool)
        .await?;

        let triggers_by_type = sqlx::query_as::<_, AlertTypeCountV16Row>(
            r#"SELECT pta.alert_type, COUNT(*) as count
               FROM performance_test_alert_history_v16 pah
               JOIN performance_test_alerts_v16 pta ON pah.alert_id = pta.id
               JOIN performance_baselines pb ON pta.baseline_id = pb.id
               WHERE pb.repo_id = $1
               GROUP BY pta.alert_type"#,
        )
        .bind(repo_id)
        .fetch_all(&self.pool)
        .await?;

        let mut triggers_by_type_map = serde_json::json!({});
        for row in triggers_by_type {
            triggers_by_type_map[row.alert_type] = serde_json::json!(row.count);
        }

        let last_triggered = alerts.iter().filter_map(|a| a.last_triggered_at).max();

        let trigger_rows = sqlx::query_as::<_, AlertTriggerTrendV16Row>(
            r#"SELECT
                DATE(pah.created_at) as date,
                COUNT(*) as trigger_count
               FROM performance_test_alert_history_v16 pah
               JOIN performance_test_alerts_v16 pta ON pah.alert_id = pta.id
               JOIN performance_baselines pb ON pta.baseline_id = pb.id
               WHERE pb.repo_id = $1
                 AND pah.created_at >= NOW() - INTERVAL '30 days'
               GROUP BY DATE(pah.created_at)
               ORDER BY DATE(pah.created_at) DESC"#,
        )
        .bind(repo_id)
        .fetch_all(&self.pool)
        .await?;

        let trigger_trend = trigger_rows.into_iter().map(AlertTriggerTrendV16::from).collect();

        Ok(AlertAnalyticsV16 {
            total_alerts,
            active_alerts,
            total_triggers,
            triggers_by_type: triggers_by_type_map,
            avg_time_between_triggers_ms: 0.0,
            last_triggered_at: last_triggered,
            trigger_trend,
        })
    }

    pub async fn check_and_trigger_alerts_v16(
        &self,
        test_id: Uuid,
        repo_id: Uuid,
    ) -> Result<Vec<AlertNotificationV16>, sqlx::Error> {
        let baselines = self.list_baselines(repo_id).await?;
        let mut notifications = Vec::new();

        for baseline in baselines {
            if let Some(result) = sqlx::query_as::<_, TestResultRow>(
                r#"SELECT id, test_id, metric_name, metric_value, percentile, recorded_at
                   FROM performance_test_results
                   WHERE test_id = $1 AND metric_name = $2 AND percentile IS NULL
                   LIMIT 1"#,
            )
            .bind(test_id)
            .bind(&baseline.metric_name)
            .fetch_optional(&self.pool)
            .await?
            {
                let current_value = result.metric_value;
                let alerts = sqlx::query_as::<_, AlertConfigV16Row>(
                    r#"SELECT id, baseline_id, alert_type, threshold, enabled, last_triggered_at, created_at
                       FROM performance_test_alerts_v16
                       WHERE baseline_id = $1 AND enabled = true"#,
                )
                .bind(baseline.id)
                .fetch_all(&self.pool)
                .await?;

                for alert in alerts {
                    let triggered = match alert.alert_type.as_str() {
                        "regression" => current_value > baseline.baseline_value * (1.0 + alert.threshold / 100.0),
                        "improvement" => current_value < baseline.baseline_value * (1.0 - alert.threshold / 100.0),
                        "absolute" => current_value > alert.threshold,
                        _ => false,
                    };

                    if triggered {
                        self.trigger_alert_v16(
                            alert.id,
                            &baseline.metric_name,
                            current_value,
                            alert.threshold,
                        )
                        .await?;

                        notifications.push(AlertNotificationV16 {
                            alert_id: alert.id,
                            metric_name: baseline.metric_name.clone(),
                            current_value,
                            threshold: alert.threshold,
                            severity: if alert.threshold > 50.0 {
                                "critical".to_string()
                            } else if alert.threshold > 25.0 {
                                "high".to_string()
                            } else {
                                "medium".to_string()
                            },
                            message: format!(
                                "Performance alert triggered: {} {} (current: {}, threshold: {})",
                                baseline.metric_name, alert.alert_type, current_value, alert.threshold
                            ),
                        });
                    }
                }
            }
        }

        Ok(notifications)
    }

    pub async fn create_alert_config_v18(
        &self,
        baseline_id: Uuid,
        req: CreateAlertConfigV18Request,
    ) -> Result<PerformanceTestAlertConfigV18, sqlx::Error> {
        let id = Uuid::new_v4();
        let now = Utc::now();
        let enabled = req.enabled.unwrap_or(true);

        sqlx::query(
            r#"INSERT INTO performance_test_alerts_v19 (id, baseline_id, alert_type, threshold, enabled, created_at)
               VALUES ($1, $2, $3, $4, $5, $6)"#,
        )
        .bind(id)
        .bind(baseline_id)
        .bind(&req.alert_type)
        .bind(req.threshold)
        .bind(enabled)
        .bind(now)
        .execute(&self.pool)
        .await?;

        Ok(PerformanceTestAlertConfigV18 {
            id,
            baseline_id,
            alert_type: req.alert_type,
            threshold: req.threshold,
            enabled,
            last_triggered_at: None,
            created_at: now,
        })
    }

    pub async fn get_alert_config_v18(
        &self,
        id: Uuid,
    ) -> Result<Option<PerformanceTestAlertConfigV18>, sqlx::Error> {
        let row = sqlx::query_as::<_, AlertConfigV18Row>(
            r#"SELECT id, baseline_id, alert_type, threshold, enabled, last_triggered_at, created_at
               FROM performance_test_alerts_v19 WHERE id = $1"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(PerformanceTestAlertConfigV18::from))
    }

    pub async fn list_alert_configs_v18(
        &self,
        baseline_id: Uuid,
    ) -> Result<Vec<PerformanceTestAlertConfigV18>, sqlx::Error> {
        let rows = sqlx::query_as::<_, AlertConfigV18Row>(
            r#"SELECT id, baseline_id, alert_type, threshold, enabled, last_triggered_at, created_at
               FROM performance_test_alerts_v19
               WHERE baseline_id = $1
               ORDER BY created_at"#,
        )
        .bind(baseline_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(PerformanceTestAlertConfigV18::from).collect())
    }

    pub async fn list_alert_configs_v18_for_repo(
        &self,
        repo_id: Uuid,
    ) -> Result<Vec<PerformanceTestAlertConfigV18>, sqlx::Error> {
        let rows = sqlx::query_as::<_, AlertConfigV18Row>(
            r#"SELECT pta.id, pta.baseline_id, pta.alert_type, pta.threshold, pta.enabled, pta.last_triggered_at, pta.created_at
               FROM performance_test_alerts_v19 pta
               JOIN performance_baselines pb ON pta.baseline_id = pb.id
               WHERE pb.repo_id = $1
               ORDER BY pta.created_at"#,
        )
        .bind(repo_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(PerformanceTestAlertConfigV18::from).collect())
    }

    pub async fn update_alert_config_v18(
        &self,
        id: Uuid,
        req: UpdateAlertConfigV18Request,
    ) -> Result<PerformanceTestAlertConfigV18, sqlx::Error> {
        if let Some(ref alert_type) = req.alert_type {
            sqlx::query(r#"UPDATE performance_test_alerts_v19 SET alert_type = $1 WHERE id = $2"#)
                .bind(alert_type)
                .bind(id)
                .execute(&self.pool)
                .await?;
        }
        if let Some(threshold) = req.threshold {
            sqlx::query(r#"UPDATE performance_test_alerts_v19 SET threshold = $1 WHERE id = $2"#)
                .bind(threshold)
                .bind(id)
                .execute(&self.pool)
                .await?;
        }
        if let Some(enabled) = req.enabled {
            sqlx::query(r#"UPDATE performance_test_alerts_v19 SET enabled = $1 WHERE id = $2"#)
                .bind(enabled)
                .bind(id)
                .execute(&self.pool)
                .await?;
        }

        self.get_alert_config_v18(id).await?.ok_or_else(|| sqlx::Error::RowNotFound)
    }

    pub async fn delete_alert_config_v18(&self, id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(r#"DELETE FROM performance_test_alerts_v19 WHERE id = $1"#)
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn trigger_alert_v18(
        &self,
        alert_id: Uuid,
        metric_name: &str,
        metric_value: f64,
        threshold: f64,
    ) -> Result<PerformanceAlertHistoryV18, sqlx::Error> {
        let id = Uuid::new_v4();
        let now = Utc::now();

        sqlx::query(
            r#"INSERT INTO performance_test_alert_history_v19 (id, alert_id, metric_name, metric_value, threshold, created_at)
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

        sqlx::query(r#"UPDATE performance_test_alerts_v19 SET last_triggered_at = $1 WHERE id = $2"#)
            .bind(now)
            .bind(alert_id)
            .execute(&self.pool)
            .await?;

        Ok(PerformanceAlertHistoryV18 {
            id,
            alert_id,
            metric_name: metric_name.to_string(),
            metric_value,
            threshold,
            created_at: now,
        })
    }

    pub async fn get_alert_history_v18(
        &self,
        alert_id: Uuid,
        limit: i64,
    ) -> Result<Vec<PerformanceAlertHistoryV18>, sqlx::Error> {
        let rows = sqlx::query_as::<_, AlertHistoryV18Row>(
            r#"SELECT id, alert_id, metric_name, metric_value, threshold, created_at
               FROM performance_test_alert_history_v19
               WHERE alert_id = $1
               ORDER BY created_at DESC
               LIMIT $2"#,
        )
        .bind(alert_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(PerformanceAlertHistoryV18::from).collect())
    }

    pub async fn get_alert_notifications_v18(
        &self,
        baseline_id: Uuid,
    ) -> Result<Vec<AlertNotificationV18>, sqlx::Error> {
        let alerts = sqlx::query_as::<_, AlertConfigV18Row>(
            r#"SELECT pta.id, pta.baseline_id, pta.alert_type, pta.threshold, pta.enabled, pta.last_triggered_at, pta.created_at
               FROM performance_test_alerts_v19 pta
               WHERE pta.baseline_id = $1 AND pta.enabled = true"#,
        )
        .bind(baseline_id)
        .fetch_all(&self.pool)
        .await?;

        let mut notifications = Vec::new();
        for alert in alerts {
            let baseline = self.get_baseline(alert.baseline_id).await?;
            if let Some(baseline) = baseline {
                notifications.push(AlertNotificationV18 {
                    alert_id: alert.id,
                    metric_name: baseline.metric_name,
                    current_value: baseline.baseline_value,
                    threshold: alert.threshold,
                    severity: if alert.threshold > 50.0 {
                        "critical".to_string()
                    } else if alert.threshold > 25.0 {
                        "high".to_string()
                    } else {
                        "medium".to_string()
                    },
                    message: format!(
                        "Alert configured: {} exceeds {}",
                        alert.alert_type, alert.threshold
                    ),
                });
            }
        }

        Ok(notifications)
    }

    pub async fn get_alert_analytics_v18(
        &self,
        repo_id: Uuid,
    ) -> Result<AlertAnalyticsV18, sqlx::Error> {
        let alerts = self.list_alert_configs_v18_for_repo(repo_id).await?;
        let total_alerts = alerts.len() as i64;
        let active_alerts = alerts.iter().filter(|a| a.enabled).count() as i64;

        let total_triggers = sqlx::query_scalar::<_, i64>(
            r#"SELECT COUNT(*)
               FROM performance_test_alert_history_v19 pah
               JOIN performance_test_alerts_v19 pta ON pah.alert_id = pta.id
               JOIN performance_baselines pb ON pta.baseline_id = pb.id
               WHERE pb.repo_id = $1"#,
        )
        .bind(repo_id)
        .fetch_one(&self.pool)
        .await?;

        let triggers_by_type = sqlx::query_as::<_, AlertTypeCountV18Row>(
            r#"SELECT pta.alert_type, COUNT(*) as count
               FROM performance_test_alert_history_v19 pah
               JOIN performance_test_alerts_v19 pta ON pah.alert_id = pta.id
               JOIN performance_baselines pb ON pta.baseline_id = pb.id
               WHERE pb.repo_id = $1
               GROUP BY pta.alert_type"#,
        )
        .bind(repo_id)
        .fetch_all(&self.pool)
        .await?;

        let mut triggers_by_type_map = serde_json::json!({});
        for row in triggers_by_type {
            triggers_by_type_map[row.alert_type] = serde_json::json!(row.count);
        }

        let last_triggered = alerts.iter().filter_map(|a| a.last_triggered_at).max();

        let trigger_rows = sqlx::query_as::<_, AlertTriggerTrendV18Row>(
            r#"SELECT
                DATE(pah.created_at) as date,
                COUNT(*) as trigger_count
               FROM performance_test_alert_history_v19 pah
               JOIN performance_test_alerts_v19 pta ON pah.alert_id = pta.id
               JOIN performance_baselines pb ON pta.baseline_id = pb.id
               WHERE pb.repo_id = $1
                 AND pah.created_at >= NOW() - INTERVAL '30 days'
               GROUP BY DATE(pah.created_at)
               ORDER BY DATE(pah.created_at) DESC"#,
        )
        .bind(repo_id)
        .fetch_all(&self.pool)
        .await?;

        let trigger_trend = trigger_rows.into_iter().map(AlertTriggerTrendV18::from).collect();

        Ok(AlertAnalyticsV18 {
            total_alerts,
            active_alerts,
            total_triggers,
            triggers_by_type: triggers_by_type_map,
            avg_time_between_triggers_ms: 0.0,
            last_triggered_at: last_triggered,
            trigger_trend,
        })
    }

    pub async fn check_and_trigger_alerts_v18(
        &self,
        test_id: Uuid,
        repo_id: Uuid,
    ) -> Result<Vec<AlertNotificationV18>, sqlx::Error> {
        let baselines = self.list_baselines(repo_id).await?;
        let mut notifications = Vec::new();

        for baseline in baselines {
            if let Some(result) = sqlx::query_as::<_, TestResultRow>(
                r#"SELECT id, test_id, metric_name, metric_value, percentile, recorded_at
                   FROM performance_test_results
                   WHERE test_id = $1 AND metric_name = $2 AND percentile IS NULL
                   LIMIT 1"#,
            )
            .bind(test_id)
            .bind(&baseline.metric_name)
            .fetch_optional(&self.pool)
            .await?
            {
                let current_value = result.metric_value;
                let alerts = sqlx::query_as::<_, AlertConfigV18Row>(
                    r#"SELECT id, baseline_id, alert_type, threshold, enabled, last_triggered_at, created_at
                       FROM performance_test_alerts_v19
                       WHERE baseline_id = $1 AND enabled = true"#,
                )
                .bind(baseline.id)
                .fetch_all(&self.pool)
                .await?;

                for alert in alerts {
                    let triggered = match alert.alert_type.as_str() {
                        "regression" => current_value > baseline.baseline_value * (1.0 + alert.threshold / 100.0),
                        "improvement" => current_value < baseline.baseline_value * (1.0 - alert.threshold / 100.0),
                        "absolute" => current_value > alert.threshold,
                        _ => false,
                    };

                    if triggered {
                        self.trigger_alert_v18(
                            alert.id,
                            &baseline.metric_name,
                            current_value,
                            alert.threshold,
                        )
                        .await?;

                        notifications.push(AlertNotificationV18 {
                            alert_id: alert.id,
                            metric_name: baseline.metric_name.clone(),
                            current_value,
                            threshold: alert.threshold,
                            severity: if alert.threshold > 50.0 {
                                "critical".to_string()
                            } else if alert.threshold > 25.0 {
                                "high".to_string()
                            } else {
                                "medium".to_string()
                            },
                            message: format!(
                                "Performance alert triggered: {} {} (current: {}, threshold: {})",
                                baseline.metric_name, alert.alert_type, current_value, alert.threshold
                            ),
                        });
                    }
                }
            }
        }

        Ok(notifications)
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

#[derive(sqlx::FromRow)]
struct BaselineRow {
    id: Uuid,
    repo_id: Uuid,
    metric_name: String,
    baseline_value: f64,
    threshold_percent: f64,
    created_at: chrono::DateTime<Utc>,
}

impl From<BaselineRow> for PerformanceBaseline {
    fn from(row: BaselineRow) -> Self {
        Self {
            id: row.id,
            repo_id: row.repo_id,
            metric_name: row.metric_name,
            baseline_value: row.baseline_value,
            threshold_percent: row.threshold_percent,
            created_at: row.created_at,
        }
    }
}

#[derive(sqlx::FromRow)]
struct RegressionRow {
    id: Uuid,
    baseline_id: Uuid,
    test_id: Uuid,
    regression_percent: f64,
    status: String,
    created_at: chrono::DateTime<Utc>,
}

impl From<RegressionRow> for PerformanceRegression {
    fn from(row: RegressionRow) -> Self {
        Self {
            id: row.id,
            baseline_id: row.baseline_id,
            test_id: row.test_id,
            regression_percent: row.regression_percent,
            status: row.status,
            created_at: row.created_at,
        }
    }
}

#[derive(sqlx::FromRow)]
struct TrendDataRow {
    id: Uuid,
    repo_id: Uuid,
    metric_name: String,
    metric_value: f64,
    recorded_at: chrono::DateTime<Utc>,
}

impl From<TrendDataRow> for PerformanceTrendData {
    fn from(row: TrendDataRow) -> Self {
        Self {
            id: row.id,
            repo_id: row.repo_id,
            metric_name: row.metric_name,
            metric_value: row.metric_value,
            recorded_at: row.recorded_at,
        }
    }
}

#[derive(sqlx::FromRow)]
struct RegressionStatsRow {
    active_regressions: i64,
    resolved_regressions: i64,
}

#[derive(sqlx::FromRow)]
struct AlertConfigRow {
    id: Uuid,
    baseline_id: Uuid,
    alert_type: String,
    threshold: f64,
    enabled: bool,
    last_triggered_at: Option<chrono::DateTime<Utc>>,
    created_at: chrono::DateTime<Utc>,
}

impl From<AlertConfigRow> for PerformanceTestAlertConfig {
    fn from(row: AlertConfigRow) -> Self {
        Self {
            id: row.id,
            baseline_id: row.baseline_id,
            alert_type: row.alert_type,
            threshold: row.threshold,
            enabled: row.enabled,
            last_triggered_at: row.last_triggered_at,
            created_at: row.created_at,
        }
    }
}

#[derive(sqlx::FromRow)]
struct AlertHistoryRow {
    id: Uuid,
    alert_id: Uuid,
    metric_name: String,
    metric_value: f64,
    threshold: f64,
    created_at: chrono::DateTime<Utc>,
}

impl From<AlertHistoryRow> for PerformanceAlertHistory {
    fn from(row: AlertHistoryRow) -> Self {
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
struct AlertTypeCountRow {
    alert_type: String,
    count: i64,
}

#[derive(sqlx::FromRow)]
struct AlertTriggerTrendRow {
    date: chrono::NaiveDate,
    trigger_count: i64,
}

impl From<AlertTriggerTrendRow> for AlertTriggerTrend {
    fn from(row: AlertTriggerTrendRow) -> Self {
        Self {
            date: row.date,
            trigger_count: row.trigger_count,
            alert_types: Vec::new(),
        }
    }
}

#[derive(sqlx::FromRow)]
struct AlertConfigV3Row {
    id: Uuid,
    baseline_id: Uuid,
    alert_type: String,
    threshold: f64,
    enabled: bool,
    last_triggered_at: Option<chrono::DateTime<Utc>>,
    created_at: chrono::DateTime<Utc>,
}

impl From<AlertConfigV3Row> for PerformanceTestAlertConfigV3 {
    fn from(row: AlertConfigV3Row) -> Self {
        Self {
            id: row.id,
            baseline_id: row.baseline_id,
            alert_type: row.alert_type,
            threshold: row.threshold,
            enabled: row.enabled,
            last_triggered_at: row.last_triggered_at,
            created_at: row.created_at,
        }
    }
}

#[derive(sqlx::FromRow)]
struct AlertHistoryV3Row {
    id: Uuid,
    alert_id: Uuid,
    metric_name: String,
    metric_value: f64,
    threshold: f64,
    created_at: chrono::DateTime<Utc>,
}

impl From<AlertHistoryV3Row> for PerformanceAlertHistoryV3 {
    fn from(row: AlertHistoryV3Row) -> Self {
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
struct AlertTypeCountV3Row {
    alert_type: String,
    count: i64,
}

#[derive(sqlx::FromRow)]
struct AlertTriggerTrendV3Row {
    date: chrono::NaiveDate,
    trigger_count: i64,
}

impl From<AlertTriggerTrendV3Row> for AlertTriggerTrendV3 {
    fn from(row: AlertTriggerTrendV3Row) -> Self {
        Self {
            date: row.date,
            trigger_count: row.trigger_count,
            alert_types: Vec::new(),
        }
    }
}

#[derive(sqlx::FromRow)]
struct AlertConfigV5Row {
    id: Uuid,
    baseline_id: Uuid,
    alert_type: String,
    threshold: f64,
    enabled: bool,
    last_triggered_at: Option<chrono::DateTime<Utc>>,
    created_at: chrono::DateTime<Utc>,
}

impl From<AlertConfigV5Row> for PerformanceTestAlertConfigV5 {
    fn from(row: AlertConfigV5Row) -> Self {
        Self {
            id: row.id,
            baseline_id: row.baseline_id,
            alert_type: row.alert_type,
            threshold: row.threshold,
            enabled: row.enabled,
            last_triggered_at: row.last_triggered_at,
            created_at: row.created_at,
        }
    }
}

#[derive(sqlx::FromRow)]
struct AlertHistoryV5Row {
    id: Uuid,
    alert_id: Uuid,
    metric_name: String,
    metric_value: f64,
    threshold: f64,
    created_at: chrono::DateTime<Utc>,
}

impl From<AlertHistoryV5Row> for PerformanceAlertHistoryV5 {
    fn from(row: AlertHistoryV5Row) -> Self {
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
struct AlertTypeCountV5Row {
    alert_type: String,
    count: i64,
}

#[derive(sqlx::FromRow)]
struct AlertTriggerTrendV5Row {
    date: chrono::NaiveDate,
    trigger_count: i64,
}

impl From<AlertTriggerTrendV5Row> for AlertTriggerTrendV5 {
    fn from(row: AlertTriggerTrendV5Row) -> Self {
        Self {
            date: row.date,
            trigger_count: row.trigger_count,
            alert_types: Vec::new(),
        }
    }
}

#[derive(sqlx::FromRow)]
struct AlertConfigV9Row {
    id: Uuid,
    baseline_id: Uuid,
    alert_type: String,
    threshold: f64,
    enabled: bool,
    last_triggered_at: Option<chrono::DateTime<Utc>>,
    created_at: chrono::DateTime<Utc>,
}

impl From<AlertConfigV9Row> for PerformanceTestAlertConfigV9 {
    fn from(row: AlertConfigV9Row) -> Self {
        Self {
            id: row.id,
            baseline_id: row.baseline_id,
            alert_type: row.alert_type,
            threshold: row.threshold,
            enabled: row.enabled,
            last_triggered_at: row.last_triggered_at,
            created_at: row.created_at,
        }
    }
}

#[derive(sqlx::FromRow)]
struct AlertHistoryV9Row {
    id: Uuid,
    alert_id: Uuid,
    metric_name: String,
    metric_value: f64,
    threshold: f64,
    created_at: chrono::DateTime<Utc>,
}

impl From<AlertHistoryV9Row> for PerformanceAlertHistoryV9 {
    fn from(row: AlertHistoryV9Row) -> Self {
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
struct AlertTypeCountV9Row {
    alert_type: String,
    count: i64,
}

#[derive(sqlx::FromRow)]
struct AlertTriggerTrendV9Row {
    date: chrono::NaiveDate,
    trigger_count: i64,
}

impl From<AlertTriggerTrendV9Row> for AlertTriggerTrendV9 {
    fn from(row: AlertTriggerTrendV9Row) -> Self {
        Self {
            date: row.date,
            trigger_count: row.trigger_count,
            alert_types: Vec::new(),
        }
    }
}

#[derive(sqlx::FromRow)]
struct AlertConfigV16Row {
    id: Uuid,
    baseline_id: Uuid,
    alert_type: String,
    threshold: f64,
    enabled: bool,
    last_triggered_at: Option<chrono::DateTime<Utc>>,
    created_at: chrono::DateTime<Utc>,
}

impl From<AlertConfigV16Row> for PerformanceTestAlertConfigV16 {
    fn from(row: AlertConfigV16Row) -> Self {
        Self {
            id: row.id,
            baseline_id: row.baseline_id,
            alert_type: row.alert_type,
            threshold: row.threshold,
            enabled: row.enabled,
            last_triggered_at: row.last_triggered_at,
            created_at: row.created_at,
        }
    }
}

#[derive(sqlx::FromRow)]
struct AlertHistoryV16Row {
    id: Uuid,
    alert_id: Uuid,
    metric_name: String,
    metric_value: f64,
    threshold: f64,
    created_at: chrono::DateTime<Utc>,
}

impl From<AlertHistoryV16Row> for PerformanceAlertHistoryV16 {
    fn from(row: AlertHistoryV16Row) -> Self {
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
struct AlertTypeCountV16Row {
    alert_type: String,
    count: i64,
}

#[derive(sqlx::FromRow)]
struct AlertTriggerTrendV16Row {
    date: chrono::NaiveDate,
    trigger_count: i64,
}

impl From<AlertTriggerTrendV16Row> for AlertTriggerTrendV16 {
    fn from(row: AlertTriggerTrendV16Row) -> Self {
        Self {
            date: row.date,
            trigger_count: row.trigger_count,
            alert_types: Vec::new(),
        }
    }
}

#[derive(sqlx::FromRow)]
struct AlertConfigV18Row {
    id: Uuid,
    baseline_id: Uuid,
    alert_type: String,
    threshold: f64,
    enabled: bool,
    last_triggered_at: Option<chrono::DateTime<Utc>>,
    created_at: chrono::DateTime<Utc>,
}

impl From<AlertConfigV18Row> for PerformanceTestAlertConfigV18 {
    fn from(row: AlertConfigV18Row) -> Self {
        Self {
            id: row.id,
            baseline_id: row.baseline_id,
            alert_type: row.alert_type,
            threshold: row.threshold,
            enabled: row.enabled,
            last_triggered_at: row.last_triggered_at,
            created_at: row.created_at,
        }
    }
}

#[derive(sqlx::FromRow)]
struct AlertHistoryV18Row {
    id: Uuid,
    alert_id: Uuid,
    metric_name: String,
    metric_value: f64,
    threshold: f64,
    created_at: chrono::DateTime<Utc>,
}

impl From<AlertHistoryV18Row> for PerformanceAlertHistoryV18 {
    fn from(row: AlertHistoryV18Row) -> Self {
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
struct AlertTypeCountV18Row {
    alert_type: String,
    count: i64,
}

#[derive(sqlx::FromRow)]
struct AlertTriggerTrendV18Row {
    date: chrono::NaiveDate,
    trigger_count: i64,
}

impl From<AlertTriggerTrendV18Row> for AlertTriggerTrendV18 {
    fn from(row: AlertTriggerTrendV18Row) -> Self {
        Self {
            date: row.date,
            trigger_count: row.trigger_count,
            alert_types: Vec::new(),
        }
    }
}
