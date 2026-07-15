# Implementation Plan: Advanced Testing and Quality Assurance Features

## Overview

This plan adds three new versions of advanced testing and quality assurance features to CivitForge:
1. Test Suite Management v16 (with v13 database tables)
2. Code Quality Rules v16 (with v14 database tables)
3. Performance Testing v17 (with v14 database tables)

## Current Codebase Structure

The codebase follows a consistent pattern for versioned features:
- Each feature has `types.rs`, `store.rs`, and `mod.rs` files
- Versioned types are appended to the existing files (e.g., `TestSuiteMetricV8`, `TestSuiteMetricV13`)
- Versioned store methods are appended to the existing store files
- Database migrations are numbered sequentially in `crates/civit-db/src/migrations/`

## Implementation Strategy

Follow the existing pattern of appending new versioned types and store methods to the existing files, rather than creating separate files for each version.

---

## 1. Test Suite Management v16

### Files to Modify

#### 1.1 Database Migration (NEW)
**File**: `crates/civit-db/src/migrations/490_add_test_suite_management_v16.sql`

```sql
CREATE TABLE IF NOT EXISTS test_suite_metrics_v13 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    suite_id UUID NOT NULL REFERENCES test_suites(id) ON DELETE CASCADE,
    metric_name TEXT NOT NULL,
    metric_value DOUBLE PRECISION NOT NULL,
    measured_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS test_suite_baselines_v13 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    suite_id UUID NOT NULL REFERENCES test_suites(id),
    metric_name TEXT NOT NULL,
    baseline_value DOUBLE PRECISION NOT NULL,
    threshold_percent DOUBLE PRECISION NOT NULL DEFAULT 10.0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(suite_id, metric_name)
);
```

#### 1.2 Types (MODIFY)
**File**: `crates/civit-core/src/test_suite_management/types.rs`

Add new types at the end of the file:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestSuiteMetricV13 {
    pub id: Uuid,
    pub suite_id: Uuid,
    pub metric_name: String,
    pub metric_value: f64,
    pub measured_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTestSuiteMetricV13Request {
    pub metric_name: String,
    pub metric_value: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestSuiteBaselineV13 {
    pub id: Uuid,
    pub suite_id: Uuid,
    pub metric_name: String,
    pub baseline_value: f64,
    pub threshold_percent: f64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTestSuiteBaselineV13Request {
    pub metric_name: String,
    pub baseline_value: f64,
    pub threshold_percent: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateTestSuiteBaselineV13Request {
    pub baseline_value: Option<f64>,
    pub threshold_percent: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestSuiteRegressionV13 {
    pub id: Uuid,
    pub baseline_id: Uuid,
    pub metric_name: String,
    pub baseline_value: f64,
    pub current_value: f64,
    pub regression_percent: f64,
    pub threshold_percent: f64,
    pub status: String,
    pub detected_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestSuitePerformanceAlertV13 {
    pub id: Uuid,
    pub suite_id: Uuid,
    pub metric_name: String,
    pub baseline_value: f64,
    pub current_value: f64,
    pub regression_percent: f64,
    pub threshold_percent: f64,
    pub severity: String,
    pub message: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestSuiteMetricsSummaryV13 {
    pub suite_id: Uuid,
    pub total_metrics: i64,
    pub total_baselines: i64,
    pub active_regressions: i64,
    pub resolved_regressions: i64,
    pub metrics: Vec<TestSuiteMetricV13>,
    pub baselines: Vec<TestSuiteBaselineV13>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestSuitePerformanceReportV13 {
    pub suite_id: Uuid,
    pub suite_name: String,
    pub metrics_summary: TestSuiteMetricsSummaryV13,
    pub regressions: Vec<TestSuiteRegressionV13>,
    pub alerts: Vec<TestSuitePerformanceAlertV13>,
    pub overall_score: f64,
    pub last_measured_at: Option<DateTime<Utc>>,
}
```

#### 1.3 Store (MODIFY)
**File**: `crates/civit-core/src/test_suite_management/store.rs`

Add new store methods and row structs at the end of the file:

```rust
// Store methods for v13
pub async fn create_metric_v13(
    &self,
    suite_id: Uuid,
    req: CreateTestSuiteMetricV13Request,
) -> Result<TestSuiteMetricV13, sqlx::Error> {
    let id = Uuid::new_v4();
    let now = Utc::now();

    sqlx::query(
        r#"INSERT INTO test_suite_metrics_v13 (id, suite_id, metric_name, metric_value, measured_at)
           VALUES ($1, $2, $3, $4, $5)"#,
    )
    .bind(id)
    .bind(suite_id)
    .bind(&req.metric_name)
    .bind(req.metric_value)
    .bind(now)
    .execute(&self.pool)
    .await?;

    Ok(TestSuiteMetricV13 {
        id,
        suite_id,
        metric_name: req.metric_name,
        metric_value: req.metric_value,
        measured_at: now,
    })
}

pub async fn list_metrics_v13(
    &self,
    suite_id: Uuid,
    metric_name: Option<&str>,
    limit: i64,
    offset: i64,
) -> Result<Vec<TestSuiteMetricV13>, sqlx::Error> {
    let rows = sqlx::query_as::<_, MetricV13Row>(
        r#"SELECT id, suite_id, metric_name, metric_value, measured_at
           FROM test_suite_metrics_v13
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

    Ok(rows.into_iter().map(TestSuiteMetricV13::from).collect())
}

pub async fn create_baseline_v13(
    &self,
    suite_id: Uuid,
    req: CreateTestSuiteBaselineV13Request,
) -> Result<TestSuiteBaselineV13, sqlx::Error> {
    let id = Uuid::new_v4();
    let now = Utc::now();
    let threshold_percent = req.threshold_percent.unwrap_or(10.0);

    sqlx::query(
        r#"INSERT INTO test_suite_baselines_v13 (id, suite_id, metric_name, baseline_value, threshold_percent, created_at)
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

    Ok(TestSuiteBaselineV13 {
        id,
        suite_id,
        metric_name: req.metric_name,
        baseline_value: req.baseline_value,
        threshold_percent,
        created_at: now,
    })
}

pub async fn list_baselines_v13(
    &self,
    suite_id: Uuid,
) -> Result<Vec<TestSuiteBaselineV13>, sqlx::Error> {
    let rows = sqlx::query_as::<_, BaselineV13Row>(
        r#"SELECT id, suite_id, metric_name, baseline_value, threshold_percent, created_at
           FROM test_suite_baselines_v13
           WHERE suite_id = $1
           ORDER BY metric_name"#,
    )
    .bind(suite_id)
    .fetch_all(&self.pool)
    .await?;

    Ok(rows.into_iter().map(TestSuiteBaselineV13::from).collect())
}

pub async fn update_baseline_v13(
    &self,
    id: Uuid,
    req: UpdateTestSuiteBaselineV13Request,
) -> Result<TestSuiteBaselineV13, sqlx::Error> {
    if let Some(value) = req.baseline_value {
        sqlx::query(r#"UPDATE test_suite_baselines_v13 SET baseline_value = $1 WHERE id = $2"#)
            .bind(value)
            .bind(id)
            .execute(&self.pool)
            .await?;
    }
    if let Some(threshold) = req.threshold_percent {
        sqlx::query(r#"UPDATE test_suite_baselines_v13 SET threshold_percent = $1 WHERE id = $2"#)
            .bind(threshold)
            .bind(id)
            .execute(&self.pool)
            .await?;
    }

    let row = sqlx::query_as::<_, BaselineV13Row>(
        r#"SELECT id, suite_id, metric_name, baseline_value, threshold_percent, created_at
           FROM test_suite_baselines_v13 WHERE id = $1"#,
    )
    .bind(id)
    .fetch_one(&self.pool)
    .await?;

    Ok(TestSuiteBaselineV13::from(row))
}

pub async fn delete_baseline_v13(&self, id: Uuid) -> Result<bool, sqlx::Error> {
    let result = sqlx::query(r#"DELETE FROM test_suite_baselines_v13 WHERE id = $1"#)
        .bind(id)
        .execute(&self.pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

pub async fn detect_regressions_v13(
    &self,
    suite_id: Uuid,
) -> Result<Vec<TestSuiteRegressionV13>, sqlx::Error> {
    let baselines = self.list_baselines_v13(suite_id).await?;
    let mut regressions = Vec::new();

    for baseline in baselines {
        if let Some(latest_metric) = sqlx::query_as::<_, MetricV13Row>(
            r#"SELECT id, suite_id, metric_name, metric_value, measured_at
               FROM test_suite_metrics_v13
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
                regressions.push(TestSuiteRegressionV13 {
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

pub async fn get_metrics_summary_v13(
    &self,
    suite_id: Uuid,
) -> Result<TestSuiteMetricsSummaryV13, sqlx::Error> {
    let metrics = self.list_metrics_v13(suite_id, None, 1000, 0).await?;
    let baselines = self.list_baselines_v13(suite_id).await?;
    let regressions = self.detect_regressions_v13(suite_id).await?;

    let active_regressions = regressions.iter().filter(|r| r.status == "open").count() as i64;
    let resolved_regressions = regressions.iter().filter(|r| r.status == "resolved").count() as i64;

    Ok(TestSuiteMetricsSummaryV13 {
        suite_id,
        total_metrics: metrics.len() as i64,
        total_baselines: baselines.len() as i64,
        active_regressions,
        resolved_regressions,
        metrics,
        baselines,
    })
}

pub async fn get_performance_report_v13(
    &self,
    suite_id: Uuid,
) -> Result<TestSuitePerformanceReportV13, sqlx::Error> {
    let suite = self.get_suite(suite_id).await?.ok_or_else(|| sqlx::Error::RowNotFound)?;
    let metrics_summary = self.get_metrics_summary_v13(suite_id).await?;
    let regressions = self.detect_regressions_v13(suite_id).await?;

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

        alerts.push(TestSuitePerformanceAlertV13 {
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

    Ok(TestSuitePerformanceReportV13 {
        suite_id,
        suite_name: suite.name,
        metrics_summary,
        regressions,
        alerts,
        overall_score,
        last_measured_at,
    })
}

// Row structs for v13
#[derive(sqlx::FromRow)]
struct MetricV13Row {
    id: Uuid,
    suite_id: Uuid,
    metric_name: String,
    metric_value: f64,
    measured_at: chrono::DateTime<Utc>,
}

impl From<MetricV13Row> for TestSuiteMetricV13 {
    fn from(row: MetricV13Row) -> Self {
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
struct BaselineV13Row {
    id: Uuid,
    suite_id: Uuid,
    metric_name: String,
    baseline_value: f64,
    threshold_percent: f64,
    created_at: chrono::DateTime<Utc>,
}

impl From<BaselineV13Row> for TestSuiteBaselineV13 {
    fn from(row: BaselineV13Row) -> Self {
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
```

#### 1.4 Module Export (MODIFY)
**File**: `crates/civit-core/src/test_suite_management/mod.rs`

Add new exports to the `pub use types::` block:

```rust
TestSuiteMetricV13, CreateTestSuiteMetricV13Request,
TestSuiteBaselineV13, CreateTestSuiteBaselineV13Request, UpdateTestSuiteBaselineV13Request,
TestSuiteRegressionV13, TestSuitePerformanceAlertV13,
TestSuiteMetricsSummaryV13, TestSuitePerformanceReportV13,
```

---

## 2. Code Quality Rules v16

### Files to Modify

#### 2.1 Database Migration (NEW)
**File**: `crates/civit-db/src/migrations/491_add_code_quality_rules_v16.sql`

```sql
CREATE TABLE IF NOT EXISTS code_quality_metrics_v14 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    repo_id UUID NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    file_path TEXT NOT NULL,
    metric_name TEXT NOT NULL,
    metric_value DOUBLE PRECISION NOT NULL,
    measured_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS code_quality_thresholds_v13 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    repo_id UUID NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    metric_name TEXT NOT NULL,
    threshold_value DOUBLE PRECISION NOT NULL,
    enabled BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(repo_id, metric_name)
);
```

#### 2.2 Types (MODIFY)
**File**: `crates/civit-core/src/code_quality/types.rs`

Add new types at the end of the file:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeQualityMetricV14 {
    pub id: Uuid,
    pub repo_id: Uuid,
    pub file_path: String,
    pub metric_name: String,
    pub metric_value: f64,
    pub measured_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordMetricV14Request {
    pub file_path: String,
    pub metric_name: String,
    pub metric_value: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeQualityThresholdV13 {
    pub id: Uuid,
    pub repo_id: Uuid,
    pub metric_name: String,
    pub threshold_value: f64,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCodeQualityThresholdV13Request {
    pub metric_name: String,
    pub threshold_value: f64,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateCodeQualityThresholdV13Request {
    pub threshold_value: Option<f64>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeQualityViolationV4 {
    pub id: Uuid,
    pub repo_id: Uuid,
    pub file_path: String,
    pub metric_name: String,
    pub metric_value: f64,
    pub threshold_value: f64,
    pub severity: String,
    pub detected_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeQualityEnforcementReportV5 {
    pub repo_id: Uuid,
    pub total_thresholds: i64,
    pub active_thresholds: i64,
    pub total_violations: i64,
    pub violations_by_severity: serde_json::Value,
    pub violations_by_metric: serde_json::Value,
    pub violations: Vec<CodeQualityViolationV4>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeQualityScoreV5 {
    pub repo_id: Uuid,
    pub overall_score: f64,
    pub metrics_evaluated: i64,
    pub thresholds_passed: i64,
    pub thresholds_failed: i64,
    pub score_breakdown: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeQualityMetricSummaryV5 {
    pub metric_name: String,
    pub latest_value: f64,
    pub avg_value: f64,
    pub min_value: f64,
    pub max_value: f64,
    pub measurement_count: i64,
    pub files_affected: i64,
    pub threshold: Option<f64>,
    pub threshold_exceeded: bool,
}
```

#### 2.3 Store (MODIFY)
**File**: `crates/civit-core/src/code_quality/store.rs`

Add new store methods and row structs at the end of the file:

```rust
// Store methods for v14
pub async fn record_metric_v14(
    &self,
    repo_id: Uuid,
    req: RecordMetricV14Request,
) -> Result<CodeQualityMetricV14, sqlx::Error> {
    let id = Uuid::new_v4();
    let now = Utc::now();

    sqlx::query(
        r#"INSERT INTO code_quality_metrics_v14 (id, repo_id, file_path, metric_name, metric_value, measured_at)
           VALUES ($1, $2, $3, $4, $5, $6)"#,
    )
    .bind(id)
    .bind(repo_id)
    .bind(&req.file_path)
    .bind(&req.metric_name)
    .bind(req.metric_value)
    .bind(now)
    .execute(&self.pool)
    .await?;

    Ok(CodeQualityMetricV14 {
        id,
        repo_id,
        file_path: req.file_path,
        metric_name: req.metric_name,
        metric_value: req.metric_value,
        measured_at: now,
    })
}

pub async fn list_metrics_v14(
    &self,
    repo_id: Uuid,
    metric_name: Option<&str>,
    limit: i64,
    offset: i64,
) -> Result<Vec<CodeQualityMetricV14>, sqlx::Error> {
    let rows = sqlx::query_as::<_, MetricV14Row>(
        r#"SELECT id, repo_id, file_path, metric_name, metric_value, measured_at
           FROM code_quality_metrics_v14
           WHERE repo_id = $1
             AND ($2::varchar IS NULL OR metric_name = $2)
           ORDER BY measured_at DESC
           LIMIT $3 OFFSET $4"#,
    )
    .bind(repo_id)
    .bind(metric_name)
    .bind(limit)
    .bind(offset)
    .fetch_all(&self.pool)
    .await?;

    Ok(rows.into_iter().map(CodeQualityMetricV14::from).collect())
}

pub async fn create_threshold_v13(
    &self,
    repo_id: Uuid,
    req: CreateCodeQualityThresholdV13Request,
) -> Result<CodeQualityThresholdV13, sqlx::Error> {
    let id = Uuid::new_v4();
    let now = Utc::now();
    let enabled = req.enabled.unwrap_or(true);

    sqlx::query(
        r#"INSERT INTO code_quality_thresholds_v13 (id, repo_id, metric_name, threshold_value, enabled, created_at)
           VALUES ($1, $2, $3, $4, $5, $6)
           ON CONFLICT (repo_id, metric_name) DO UPDATE SET threshold_value = $4, enabled = $5"#,
    )
    .bind(id)
    .bind(repo_id)
    .bind(&req.metric_name)
    .bind(req.threshold_value)
    .bind(enabled)
    .bind(now)
    .execute(&self.pool)
    .await?;

    Ok(CodeQualityThresholdV13 {
        id,
        repo_id,
        metric_name: req.metric_name,
        threshold_value: req.threshold_value,
        enabled,
        created_at: now,
    })
}

pub async fn list_thresholds_v13(
    &self,
    repo_id: Uuid,
) -> Result<Vec<CodeQualityThresholdV13>, sqlx::Error> {
    let rows = sqlx::query_as::<_, ThresholdV13Row>(
        r#"SELECT id, repo_id, metric_name, threshold_value, enabled, created_at
           FROM code_quality_thresholds_v13
           WHERE repo_id = $1
           ORDER BY metric_name"#,
    )
    .bind(repo_id)
    .fetch_all(&self.pool)
    .await?;

    Ok(rows.into_iter().map(CodeQualityThresholdV13::from).collect())
}

pub async fn update_threshold_v13(
    &self,
    id: Uuid,
    req: UpdateCodeQualityThresholdV13Request,
) -> Result<CodeQualityThresholdV13, sqlx::Error> {
    if let Some(value) = req.threshold_value {
        sqlx::query(r#"UPDATE code_quality_thresholds_v13 SET threshold_value = $1 WHERE id = $2"#)
            .bind(value)
            .bind(id)
            .execute(&self.pool)
            .await?;
    }
    if let Some(enabled) = req.enabled {
        sqlx::query(r#"UPDATE code_quality_thresholds_v13 SET enabled = $1 WHERE id = $2"#)
            .bind(enabled)
            .bind(id)
            .execute(&self.pool)
            .await?;
    }

    let row = sqlx::query_as::<_, ThresholdV13Row>(
        r#"SELECT id, repo_id, metric_name, threshold_value, enabled, created_at
           FROM code_quality_thresholds_v13 WHERE id = $1"#,
    )
    .bind(id)
    .fetch_one(&self.pool)
    .await?;

    Ok(CodeQualityThresholdV13::from(row))
}

pub async fn delete_threshold_v13(&self, id: Uuid) -> Result<bool, sqlx::Error> {
    let result = sqlx::query(r#"DELETE FROM code_quality_thresholds_v13 WHERE id = $1"#)
        .bind(id)
        .execute(&self.pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

pub async fn detect_violations_v5(
    &self,
    repo_id: Uuid,
) -> Result<Vec<CodeQualityViolationV4>, sqlx::Error> {
    let thresholds = self.list_thresholds_v13(repo_id).await?;
    let mut violations = Vec::new();

    for threshold in thresholds {
        if !threshold.enabled {
            continue;
        }

        let metrics = sqlx::query_as::<_, MetricV14Row>(
            r#"SELECT id, repo_id, file_path, metric_name, metric_value, measured_at
               FROM code_quality_metrics_v14
               WHERE repo_id = $1 AND metric_name = $2
                 AND measured_at >= NOW() - INTERVAL '1 hour'
               ORDER BY measured_at DESC"#,
        )
        .bind(repo_id)
        .bind(&threshold.metric_name)
        .fetch_all(&self.pool)
        .await?;

        for metric in metrics {
            if metric.metric_value > threshold.threshold_value {
                let severity = if metric.metric_value > threshold.threshold_value * 2.0 {
                    "critical"
                } else if metric.metric_value > threshold.threshold_value * 1.5 {
                    "error"
                } else if metric.metric_value > threshold.threshold_value * 1.2 {
                    "warning"
                } else {
                    "info"
                };

                violations.push(CodeQualityViolationV4 {
                    id: Uuid::new_v4(),
                    repo_id,
                    file_path: metric.file_path,
                    metric_name: metric.metric_name,
                    metric_value: metric.metric_value,
                    threshold_value: threshold.threshold_value,
                    severity: severity.to_string(),
                    detected_at: metric.measured_at,
                });
            }
        }
    }

    Ok(violations)
}

pub async fn get_enforcement_report_v5(
    &self,
    repo_id: Uuid,
) -> Result<CodeQualityEnforcementReportV5, sqlx::Error> {
    let thresholds = self.list_thresholds_v13(repo_id).await?;
    let violations = self.detect_violations_v5(repo_id).await?;

    let total_thresholds = thresholds.len() as i64;
    let active_thresholds = thresholds.iter().filter(|t| t.enabled).count() as i64;
    let total_violations = violations.len() as i64;

    let mut violations_by_severity = serde_json::json!({});
    for v in &violations {
        let entry = violations_by_severity.get(&v.severity).and_then(|e| e.as_i64()).unwrap_or(0);
        violations_by_severity[&v.severity] = serde_json::json!(entry + 1);
    }

    let mut violations_by_metric = serde_json::json!({});
    for v in &violations {
        let entry = violations_by_metric.get(&v.metric_name).and_then(|e| e.as_i64()).unwrap_or(0);
        violations_by_metric[&v.metric_name] = serde_json::json!(entry + 1);
    }

    Ok(CodeQualityEnforcementReportV5 {
        repo_id,
        total_thresholds,
        active_thresholds,
        total_violations,
        violations_by_severity,
        violations_by_metric,
        violations,
    })
}

pub async fn calculate_quality_score_v5(
    &self,
    repo_id: Uuid,
) -> Result<CodeQualityScoreV5, sqlx::Error> {
    let thresholds = self.list_thresholds_v13(repo_id).await?;
    let violations = self.detect_violations_v5(repo_id).await?;

    let total_thresholds = thresholds.len() as i64;
    let active_thresholds = thresholds.iter().filter(|t| t.enabled).count() as i64;
    let violated_metrics: std::collections::HashSet<String> = violations.iter().map(|v| v.metric_name.clone()).collect();
    let thresholds_passed = active_thresholds - violated_metrics.len() as i64;
    let thresholds_failed = violated_metrics.len() as i64;

    let overall_score = if active_thresholds > 0 {
        (thresholds_passed as f64 / active_thresholds as f64) * 100.0
    } else {
        100.0
    };

    let mut score_breakdown = serde_json::json!({});
    for threshold in &thresholds {
        let metric_violations = violations.iter().filter(|v| v.metric_name == threshold.metric_name).count();
        let score = if metric_violations == 0 { 100.0 } else { 0.0 };
        score_breakdown[&threshold.metric_name] = serde_json::json!({
            "score": score,
            "violations": metric_violations,
            "threshold": threshold.threshold_value,
        });
    }

    Ok(CodeQualityScoreV5 {
        repo_id,
        overall_score,
        metrics_evaluated: active_thresholds,
        thresholds_passed,
        thresholds_failed,
        score_breakdown,
    })
}

pub async fn get_metric_summary_v14(
    &self,
    repo_id: Uuid,
    metric_name: &str,
) -> Result<Option<CodeQualityMetricSummaryV5>, sqlx::Error> {
    let threshold = sqlx::query_as::<_, ThresholdV13Row>(
        r#"SELECT id, repo_id, metric_name, threshold_value, enabled, created_at
           FROM code_quality_thresholds_v13
           WHERE repo_id = $1 AND metric_name = $2 AND enabled = true
           LIMIT 1"#,
    )
    .bind(repo_id)
    .bind(metric_name)
    .fetch_optional(&self.pool)
    .await?;

    let row = sqlx::query_as::<_, MetricSummaryV14Row>(
        r#"SELECT
            metric_name,
            (SELECT metric_value FROM code_quality_metrics_v14 WHERE repo_id = $1 AND metric_name = $2 ORDER BY measured_at DESC LIMIT 1) as latest_value,
            COALESCE(AVG(metric_value), 0) as avg_value,
            COALESCE(MIN(metric_value), 0) as min_value,
            COALESCE(MAX(metric_value), 0) as max_value,
            COUNT(*) as measurement_count,
            COUNT(DISTINCT file_path) as files_affected
           FROM code_quality_metrics_v14
           WHERE repo_id = $1 AND metric_name = $2"#,
    )
    .bind(repo_id)
    .bind(metric_name)
    .fetch_optional(&self.pool)
    .await?;

    Ok(row.map(|r| {
        let threshold_value = threshold.as_ref().map(|t| t.threshold_value);
        let threshold_exceeded = threshold_value.map_or(false, |tv| r.latest_value > tv);
        CodeQualityMetricSummaryV5 {
            metric_name: r.metric_name,
            latest_value: r.latest_value,
            avg_value: r.avg_value,
            min_value: r.min_value,
            max_value: r.max_value,
            measurement_count: r.measurement_count,
            files_affected: r.files_affected,
            threshold: threshold_value,
            threshold_exceeded,
        }
    }))
}

// Row structs for v14
#[derive(sqlx::FromRow)]
struct MetricV14Row {
    id: Uuid,
    repo_id: Uuid,
    file_path: String,
    metric_name: String,
    metric_value: f64,
    measured_at: chrono::DateTime<Utc>,
}

impl From<MetricV14Row> for CodeQualityMetricV14 {
    fn from(row: MetricV14Row) -> Self {
        Self {
            id: row.id,
            repo_id: row.repo_id,
            file_path: row.file_path,
            metric_name: row.metric_name,
            metric_value: row.metric_value,
            measured_at: row.measured_at,
        }
    }
}

#[derive(sqlx::FromRow)]
struct ThresholdV13Row {
    id: Uuid,
    repo_id: Uuid,
    metric_name: String,
    threshold_value: f64,
    enabled: bool,
    created_at: chrono::DateTime<Utc>,
}

impl From<ThresholdV13Row> for CodeQualityThresholdV13 {
    fn from(row: ThresholdV13Row) -> Self {
        Self {
            id: row.id,
            repo_id: row.repo_id,
            metric_name: row.metric_name,
            threshold_value: row.threshold_value,
            enabled: row.enabled,
            created_at: row.created_at,
        }
    }
}

#[derive(sqlx::FromRow)]
struct MetricSummaryV14Row {
    metric_name: String,
    latest_value: f64,
    avg_value: f64,
    min_value: f64,
    max_value: f64,
    measurement_count: i64,
    files_affected: i64,
}
```

#### 2.4 Module Export (MODIFY)
**File**: `crates/civit-core/src/code_quality/mod.rs`

Add new exports to the `pub use types::` block:

```rust
CodeQualityMetricV14, RecordMetricV14Request,
CodeQualityThresholdV13, CreateCodeQualityThresholdV13Request, UpdateCodeQualityThresholdV13Request,
CodeQualityViolationV4, CodeQualityEnforcementReportV5, CodeQualityScoreV5,
CodeQualityMetricSummaryV5,
```

---

## 3. Performance Testing v17

### Files to Modify

#### 3.1 Database Migration (NEW)
**File**: `crates/civit-db/src/migrations/492_add_performance_testing_v17.sql`

```sql
CREATE TABLE IF NOT EXISTS performance_test_alerts_v14 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    baseline_id UUID NOT NULL REFERENCES performance_baselines(id),
    alert_type TEXT NOT NULL,
    threshold DOUBLE PRECISION NOT NULL,
    enabled BOOLEAN NOT NULL DEFAULT true,
    last_triggered_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS performance_test_alert_history_v14 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    alert_id UUID NOT NULL REFERENCES performance_test_alerts_v14(id),
    metric_name TEXT NOT NULL,
    metric_value DOUBLE PRECISION NOT NULL,
    threshold DOUBLE PRECISION NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

#### 3.2 Types (MODIFY)
**File**: `crates/civit-core/src/performance_testing/types.rs`

Add new types at the end of the file:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceTestAlertConfigV14 {
    pub id: Uuid,
    pub baseline_id: Uuid,
    pub alert_type: String,
    pub threshold: f64,
    pub enabled: bool,
    pub last_triggered_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAlertConfigV14Request {
    pub alert_type: String,
    pub threshold: f64,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateAlertConfigV14Request {
    pub alert_type: Option<String>,
    pub threshold: Option<f64>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceAlertHistoryV14 {
    pub id: Uuid,
    pub alert_id: Uuid,
    pub metric_name: String,
    pub metric_value: f64,
    pub threshold: f64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertNotificationV14 {
    pub alert_id: Uuid,
    pub metric_name: String,
    pub current_value: f64,
    pub threshold: f64,
    pub severity: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertAnalyticsV14 {
    pub total_alerts: i64,
    pub active_alerts: i64,
    pub total_triggers: i64,
    pub triggers_by_type: serde_json::Value,
    pub avg_time_between_triggers_ms: f64,
    pub last_triggered_at: Option<DateTime<Utc>>,
    pub trigger_trend: Vec<AlertTriggerTrendV14>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertTriggerTrendV14 {
    pub date: chrono::NaiveDate,
    pub trigger_count: i64,
    pub alert_types: Vec<String>,
}
```

#### 3.3 Store (MODIFY)
**File**: `crates/civit-core/src/performance_testing/store.rs`

Add new store methods and row structs at the end of the file:

```rust
// Store methods for v14
pub async fn create_alert_config_v14(
    &self,
    baseline_id: Uuid,
    req: CreateAlertConfigV14Request,
) -> Result<PerformanceTestAlertConfigV14, sqlx::Error> {
    let id = Uuid::new_v4();
    let now = Utc::now();
    let enabled = req.enabled.unwrap_or(true);

    sqlx::query(
        r#"INSERT INTO performance_test_alerts_v14 (id, baseline_id, alert_type, threshold, enabled, created_at)
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

    Ok(PerformanceTestAlertConfigV14 {
        id,
        baseline_id,
        alert_type: req.alert_type,
        threshold: req.threshold,
        enabled,
        last_triggered_at: None,
        created_at: now,
    })
}

pub async fn get_alert_config_v14(
    &self,
    id: Uuid,
) -> Result<Option<PerformanceTestAlertConfigV14>, sqlx::Error> {
    let row = sqlx::query_as::<_, AlertConfigV14Row>(
        r#"SELECT id, baseline_id, alert_type, threshold, enabled, last_triggered_at, created_at
           FROM performance_test_alerts_v14 WHERE id = $1"#,
    )
    .bind(id)
    .fetch_optional(&self.pool)
    .await?;

    Ok(row.map(PerformanceTestAlertConfigV14::from))
}

pub async fn list_alert_configs_v14(
    &self,
    baseline_id: Uuid,
) -> Result<Vec<PerformanceTestAlertConfigV14>, sqlx::Error> {
    let rows = sqlx::query_as::<_, AlertConfigV14Row>(
        r#"SELECT id, baseline_id, alert_type, threshold, enabled, last_triggered_at, created_at
           FROM performance_test_alerts_v14
           WHERE baseline_id = $1
           ORDER BY created_at"#,
    )
    .bind(baseline_id)
    .fetch_all(&self.pool)
    .await?;

    Ok(rows.into_iter().map(PerformanceTestAlertConfigV14::from).collect())
}

pub async fn list_alert_configs_v14_for_repo(
    &self,
    repo_id: Uuid,
) -> Result<Vec<PerformanceTestAlertConfigV14>, sqlx::Error> {
    let rows = sqlx::query_as::<_, AlertConfigV14Row>(
        r#"SELECT pta.id, pta.baseline_id, pta.alert_type, pta.threshold, pta.enabled, pta.last_triggered_at, pta.created_at
           FROM performance_test_alerts_v14 pta
           JOIN performance_baselines pb ON pta.baseline_id = pb.id
           WHERE pb.repo_id = $1
           ORDER BY pta.created_at"#,
    )
    .bind(repo_id)
    .fetch_all(&self.pool)
    .await?;

    Ok(rows.into_iter().map(PerformanceTestAlertConfigV14::from).collect())
}

pub async fn update_alert_config_v14(
    &self,
    id: Uuid,
    req: UpdateAlertConfigV14Request,
) -> Result<PerformanceTestAlertConfigV14, sqlx::Error> {
    if let Some(ref alert_type) = req.alert_type {
        sqlx::query(r#"UPDATE performance_test_alerts_v14 SET alert_type = $1 WHERE id = $2"#)
            .bind(alert_type)
            .bind(id)
            .execute(&self.pool)
            .await?;
    }
    if let Some(threshold) = req.threshold {
        sqlx::query(r#"UPDATE performance_test_alerts_v14 SET threshold = $1 WHERE id = $2"#)
            .bind(threshold)
            .bind(id)
            .execute(&self.pool)
            .await?;
    }
    if let Some(enabled) = req.enabled {
        sqlx::query(r#"UPDATE performance_test_alerts_v14 SET enabled = $1 WHERE id = $2"#)
            .bind(enabled)
            .bind(id)
            .execute(&self.pool)
            .await?;
    }

    self.get_alert_config_v14(id).await?.ok_or_else(|| sqlx::Error::RowNotFound)
}

pub async fn delete_alert_config_v14(&self, id: Uuid) -> Result<bool, sqlx::Error> {
    let result = sqlx::query(r#"DELETE FROM performance_test_alerts_v14 WHERE id = $1"#)
        .bind(id)
        .execute(&self.pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

pub async fn trigger_alert_v14(
    &self,
    alert_id: Uuid,
    metric_name: &str,
    metric_value: f64,
    threshold: f64,
) -> Result<PerformanceAlertHistoryV14, sqlx::Error> {
    let id = Uuid::new_v4();
    let now = Utc::now();

    sqlx::query(
        r#"INSERT INTO performance_test_alert_history_v14 (id, alert_id, metric_name, metric_value, threshold, created_at)
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

    sqlx::query(r#"UPDATE performance_test_alerts_v14 SET last_triggered_at = $1 WHERE id = $2"#)
        .bind(now)
        .bind(alert_id)
        .execute(&self.pool)
        .await?;

    Ok(PerformanceAlertHistoryV14 {
        id,
        alert_id,
        metric_name: metric_name.to_string(),
        metric_value,
        threshold,
        created_at: now,
    })
}

pub async fn get_alert_history_v14(
    &self,
    alert_id: Uuid,
    limit: i64,
) -> Result<Vec<PerformanceAlertHistoryV14>, sqlx::Error> {
    let rows = sqlx::query_as::<_, AlertHistoryV14Row>(
        r#"SELECT id, alert_id, metric_name, metric_value, threshold, created_at
           FROM performance_test_alert_history_v14
           WHERE alert_id = $1
           ORDER BY created_at DESC
           LIMIT $2"#,
    )
    .bind(alert_id)
    .bind(limit)
    .fetch_all(&self.pool)
    .await?;

    Ok(rows.into_iter().map(PerformanceAlertHistoryV14::from).collect())
}

pub async fn get_alert_notifications_v14(
    &self,
    baseline_id: Uuid,
) -> Result<Vec<AlertNotificationV14>, sqlx::Error> {
    let alerts = sqlx::query_as::<_, AlertConfigV14Row>(
        r#"SELECT pta.id, pta.baseline_id, pta.alert_type, pta.threshold, pta.enabled, pta.last_triggered_at, pta.created_at
           FROM performance_test_alerts_v14 pta
           WHERE pta.baseline_id = $1 AND pta.enabled = true"#,
    )
    .bind(baseline_id)
    .fetch_all(&self.pool)
    .await?;

    let mut notifications = Vec::new();
    for alert in alerts {
        let baseline = self.get_baseline(alert.baseline_id).await?;
        if let Some(baseline) = baseline {
            notifications.push(AlertNotificationV14 {
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

pub async fn get_alert_analytics_v14(
    &self,
    repo_id: Uuid,
) -> Result<AlertAnalyticsV14, sqlx::Error> {
    let alerts = self.list_alert_configs_v14_for_repo(repo_id).await?;
    let total_alerts = alerts.len() as i64;
    let active_alerts = alerts.iter().filter(|a| a.enabled).count() as i64;

    let total_triggers = sqlx::query_scalar::<_, i64>(
        r#"SELECT COUNT(*)
           FROM performance_test_alert_history_v14 pah
           JOIN performance_test_alerts_v14 pta ON pah.alert_id = pta.id
           JOIN performance_baselines pb ON pta.baseline_id = pb.id
           WHERE pb.repo_id = $1"#,
    )
    .bind(repo_id)
    .fetch_one(&self.pool)
    .await?;

    let triggers_by_type = sqlx::query_as::<_, AlertTypeCountV14Row>(
        r#"SELECT pta.alert_type, COUNT(*) as count
           FROM performance_test_alert_history_v14 pah
           JOIN performance_test_alerts_v14 pta ON pah.alert_id = pta.id
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

    let trigger_rows = sqlx::query_as::<_, AlertTriggerTrendV14Row>(
        r#"SELECT
            DATE(pah.created_at) as date,
            COUNT(*) as trigger_count
           FROM performance_test_alert_history_v14 pah
           JOIN performance_test_alerts_v14 pta ON pah.alert_id = pta.id
           JOIN performance_baselines pb ON pta.baseline_id = pb.id
           WHERE pb.repo_id = $1
             AND pah.created_at >= NOW() - INTERVAL '30 days'
           GROUP BY DATE(pah.created_at)
           ORDER BY DATE(pah.created_at) DESC"#,
    )
    .bind(repo_id)
    .fetch_all(&self.pool)
    .await?;

    let trigger_trend = trigger_rows.into_iter().map(AlertTriggerTrendV14::from).collect();

    Ok(AlertAnalyticsV14 {
        total_alerts,
        active_alerts,
        total_triggers,
        triggers_by_type: triggers_by_type_map,
        avg_time_between_triggers_ms: 0.0,
        last_triggered_at: last_triggered,
        trigger_trend,
    })
}

pub async fn check_and_trigger_alerts_v14(
    &self,
    test_id: Uuid,
    repo_id: Uuid,
) -> Result<Vec<AlertNotificationV14>, sqlx::Error> {
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
            let alerts = sqlx::query_as::<_, AlertConfigV14Row>(
                r#"SELECT id, baseline_id, alert_type, threshold, enabled, last_triggered_at, created_at
                   FROM performance_test_alerts_v14
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
                    self.trigger_alert_v14(
                        alert.id,
                        &baseline.metric_name,
                        current_value,
                        alert.threshold,
                    )
                    .await?;

                    notifications.push(AlertNotificationV14 {
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

// Row structs for v14
#[derive(sqlx::FromRow)]
struct AlertConfigV14Row {
    id: Uuid,
    baseline_id: Uuid,
    alert_type: String,
    threshold: f64,
    enabled: bool,
    last_triggered_at: Option<chrono::DateTime<Utc>>,
    created_at: chrono::DateTime<Utc>,
}

impl From<AlertConfigV14Row> for PerformanceTestAlertConfigV14 {
    fn from(row: AlertConfigV14Row) -> Self {
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
struct AlertHistoryV14Row {
    id: Uuid,
    alert_id: Uuid,
    metric_name: String,
    metric_value: f64,
    threshold: f64,
    created_at: chrono::DateTime<Utc>,
}

impl From<AlertHistoryV14Row> for PerformanceAlertHistoryV14 {
    fn from(row: AlertHistoryV14Row) -> Self {
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
struct AlertTypeCountV14Row {
    alert_type: String,
    count: i64,
}

#[derive(sqlx::FromRow)]
struct AlertTriggerTrendV14Row {
    date: chrono::NaiveDate,
    trigger_count: i64,
}

impl From<AlertTriggerTrendV14Row> for AlertTriggerTrendV14 {
    fn from(row: AlertTriggerTrendV14Row) -> Self {
        Self {
            date: row.date,
            trigger_count: row.trigger_count,
            alert_types: Vec::new(),
        }
    }
}
```

#### 3.4 Module Export (MODIFY)
**File**: `crates/civit-core/src/performance_testing/mod.rs`

Add new exports to the `pub use types::` block:

```rust
PerformanceTestAlertConfigV14, CreateAlertConfigV14Request, UpdateAlertConfigV14Request,
PerformanceAlertHistoryV14, AlertNotificationV14, AlertAnalyticsV14, AlertTriggerTrendV14,
```

---

## Implementation Steps

### Phase 1: Database Migrations
1. Create migration file `490_add_test_suite_management_v16.sql`
2. Create migration file `491_add_code_quality_rules_v16.sql`
3. Create migration file `492_add_performance_testing_v17.sql`

### Phase 2: Test Suite Management v16
1. Add v13 types to `types.rs`
2. Add v13 store methods and row structs to `store.rs`
3. Update `mod.rs` exports

### Phase 3: Code Quality Rules v16
1. Add v14 types to `types.rs`
2. Add v14 store methods and row structs to `store.rs`
3. Update `mod.rs` exports

### Phase 4: Performance Testing v17
1. Add v14 types to `types.rs`
2. Add v14 store methods and row structs to `store.rs`
3. Update `mod.rs` exports

### Phase 5: Verification
1. Run `cargo check -p civit-core --locked 2>&1 | tail -5`
2. Verify no compilation errors

---

## Files Changed Summary

| File | Changes |
|------|---------|
| `crates/civit-db/src/migrations/490_add_test_suite_management_v16.sql` | NEW - Database tables |
| `crates/civit-db/src/migrations/491_add_code_quality_rules_v16.sql` | NEW - Database tables |
| `crates/civit-db/src/migrations/492_add_performance_testing_v17.sql` | NEW - Database tables |
| `crates/civit-core/src/test_suite_management/types.rs` | ADD - v13 types |
| `crates/civit-core/src/test_suite_management/store.rs` | ADD - v13 methods |
| `crates/civit-core/src/test_suite_management/mod.rs` | ADD - v13 exports |
| `crates/civit-core/src/code_quality/types.rs` | ADD - v14 types |
| `crates/civit-core/src/code_quality/store.rs` | ADD - v14 methods |
| `crates/civit-core/src/code_quality/mod.rs` | ADD - v14 exports |
| `crates/civit-core/src/performance_testing/types.rs` | ADD - v14 types |
| `crates/civit-core/src/performance_testing/store.rs` | ADD - v14 methods |
| `crates/civit-core/src/performance_testing/mod.rs` | ADD - v14 exports |
