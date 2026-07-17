use super::types::*;
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;
use sqlx::QueryBuilder;

pub struct CodeQualityStore {
    pool: PgPool,
}

impl CodeQualityStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create_rule(
        &self,
        repo_id: Uuid,
        req: CreateQualityRuleRequest,
    ) -> Result<QualityRule, sqlx::Error> {
        let id = Uuid::new_v4();
        let now = Utc::now();
        let enabled = req.enabled.unwrap_or(true);

        sqlx::query(
            r#"INSERT INTO code_quality_rules (id, repo_id, name, description, rule_type, severity, pattern, enabled, created_at)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)"#,
        )
        .bind(id)
        .bind(repo_id)
        .bind(&req.name)
        .bind(&req.description)
        .bind(req.rule_type.to_string())
        .bind(req.severity.to_string())
        .bind(&req.pattern)
        .bind(enabled)
        .bind(now)
        .execute(&self.pool)
        .await?;

        Ok(QualityRule {
            id,
            repo_id,
            name: req.name,
            description: req.description,
            rule_type: req.rule_type,
            severity: req.severity,
            pattern: req.pattern,
            enabled,
            created_at: now,
        })
    }

    pub async fn get_rule(&self, id: Uuid) -> Result<Option<QualityRule>, sqlx::Error> {
        let row = sqlx::query_as::<_, RuleRow>(
            r#"SELECT id, repo_id, name, description, rule_type, severity, pattern, enabled, created_at
               FROM code_quality_rules WHERE id = $1"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(QualityRule::from))
    }

    pub async fn list_rules(
        &self,
        repo_id: Uuid,
        enabled_only: bool,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<QualityRule>, sqlx::Error> {
        let rows = if enabled_only {
            sqlx::query_as::<_, RuleRow>(
                r#"SELECT id, repo_id, name, description, rule_type, severity, pattern, enabled, created_at
                   FROM code_quality_rules
                   WHERE repo_id = $1 AND enabled = true
                   ORDER BY created_at DESC
                   LIMIT $2 OFFSET $3"#,
            )
            .bind(repo_id)
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query_as::<_, RuleRow>(
                r#"SELECT id, repo_id, name, description, rule_type, severity, pattern, enabled, created_at
                   FROM code_quality_rules
                   WHERE repo_id = $1
                   ORDER BY created_at DESC
                   LIMIT $2 OFFSET $3"#,
            )
            .bind(repo_id)
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool)
            .await?
        };

        Ok(rows.into_iter().map(QualityRule::from).collect())
    }

    pub async fn update_rule(
        &self,
        id: Uuid,
        req: UpdateQualityRuleRequest,
    ) -> Result<QualityRule, sqlx::Error> {
        if let Some(name) = &req.name {
            sqlx::query(r#"UPDATE code_quality_rules SET name = $1 WHERE id = $2"#)
                .bind(name)
                .bind(id)
                .execute(&self.pool)
                .await?;
        }
        if let Some(desc) = &req.description {
            sqlx::query(r#"UPDATE code_quality_rules SET description = $1 WHERE id = $2"#)
                .bind(desc)
                .bind(id)
                .execute(&self.pool)
                .await?;
        }
        if let Some(rule_type) = &req.rule_type {
            sqlx::query(r#"UPDATE code_quality_rules SET rule_type = $1 WHERE id = $2"#)
                .bind(rule_type.to_string())
                .bind(id)
                .execute(&self.pool)
                .await?;
        }
        if let Some(severity) = &req.severity {
            sqlx::query(r#"UPDATE code_quality_rules SET severity = $1 WHERE id = $2"#)
                .bind(severity.to_string())
                .bind(id)
                .execute(&self.pool)
                .await?;
        }
        if let Some(pattern) = &req.pattern {
            sqlx::query(r#"UPDATE code_quality_rules SET pattern = $1 WHERE id = $2"#)
                .bind(pattern)
                .bind(id)
                .execute(&self.pool)
                .await?;
        }
        if let Some(enabled) = req.enabled {
            sqlx::query(r#"UPDATE code_quality_rules SET enabled = $1 WHERE id = $2"#)
                .bind(enabled)
                .bind(id)
                .execute(&self.pool)
                .await?;
        }

        self.get_rule(id)
            .await?
            .ok_or_else(|| sqlx::Error::RowNotFound)
    }

    pub async fn delete_rule(&self, id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(r#"DELETE FROM code_quality_rules WHERE id = $1"#)
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn get_rules_summary(
        &self,
        repo_id: Uuid,
    ) -> Result<Vec<QualityRuleEnforcementResult>, sqlx::Error> {
        let rows = sqlx::query_as::<_, RuleEnforcementRow>(
            r#"SELECT
                cqr.id as rule_id,
                cqr.name as rule_name,
                COALESCE(cqm.metric_value, 0) as violations,
                COUNT(DISTINCT cqm.file_path) as files_checked,
                COUNT(DISTINCT cqm.file_path) FILTER (WHERE cqm.metric_value > 0) as files_violating
             FROM code_quality_rules cqr
             LEFT JOIN code_quality_metrics cqm ON cqr.repo_id = cqm.repo_id
                AND cqm.metric_name = cqr.name
                AND cqm.measured_at = (SELECT MAX(measured_at) FROM code_quality_metrics WHERE repo_id = cqr.repo_id AND metric_name = cqr.name)
             WHERE cqr.repo_id = $1 AND cqr.enabled = true
             GROUP BY cqr.id, cqr.name, cqm.metric_value
             ORDER BY cqr.name"#,
        )
        .bind(repo_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(QualityRuleEnforcementResult::from)
            .collect())
    }

    pub async fn record_metric(
        &self,
        repo_id: Uuid,
        req: RecordMetricRequest,
    ) -> Result<QualityMetricReport, sqlx::Error> {
        let now = Utc::now();

        sqlx::query(
            r#"INSERT INTO code_quality_metrics (repo_id, metric_name, metric_value, file_path, measured_at)
               VALUES ($1, $2, $3, $4, $5)"#,
        )
        .bind(repo_id)
        .bind(&req.metric_name)
        .bind(req.metric_value)
        .bind(&req.file_path)
        .bind(now)
        .execute(&self.pool)
        .await?;

        Ok(QualityMetricReport {
            repo_id,
            metric_name: req.metric_name,
            metric_value: req.metric_value,
            file_path: req.file_path,
        })
    }

    pub async fn get_metrics(
        &self,
        repo_id: Uuid,
        metric_name: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<QualityMetricReport>, sqlx::Error> {
        let rows = if let Some(name) = metric_name {
            sqlx::query_as::<_, MetricRow>(
                r#"SELECT repo_id, metric_name, metric_value, file_path
                   FROM code_quality_metrics
                   WHERE repo_id = $1 AND metric_name = $2
                   ORDER BY measured_at DESC
                   LIMIT $3 OFFSET $4"#,
            )
            .bind(repo_id)
            .bind(name)
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query_as::<_, MetricRow>(
                r#"SELECT repo_id, metric_name, metric_value, file_path
                   FROM code_quality_metrics
                   WHERE repo_id = $1
                   ORDER BY measured_at DESC
                   LIMIT $2 OFFSET $3"#,
            )
            .bind(repo_id)
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool)
            .await?
        };

        Ok(rows.into_iter().map(QualityMetricReport::from).collect())
    }

    pub async fn get_summary(
        &self,
        repo_id: Uuid,
    ) -> Result<Vec<QualityMetricSummary>, sqlx::Error> {
        let rows = sqlx::query_as::<_, SummaryRow>(
            r#"SELECT
                metric_name,
                (SELECT metric_value FROM code_quality_metrics WHERE repo_id = $1 AND metric_name = cqm.metric_name ORDER BY measured_at DESC LIMIT 1) as latest_value,
                AVG(metric_value) as avg_value,
                MIN(metric_value) as min_value,
                MAX(metric_value) as max_value,
                COUNT(*) as measurement_count,
                COUNT(DISTINCT file_path) as files_affected
             FROM code_quality_metrics cqm
             WHERE repo_id = $1
             GROUP BY metric_name
             ORDER BY metric_name"#,
        )
        .bind(repo_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(QualityMetricSummary::from).collect())
    }

    pub async fn get_trends(
        &self,
        repo_id: Uuid,
        metric_name: &str,
        days: i64,
    ) -> Result<Vec<QualityTrend>, sqlx::Error> {
        let rows = sqlx::query_as::<_, TrendRow>(
            r#"SELECT
                DATE(measured_at) as date,
                AVG(metric_value) as avg_value,
                MIN(metric_value) as min_value,
                MAX(metric_value) as max_value,
                COUNT(*) as measurement_count
             FROM code_quality_metrics
             WHERE repo_id = $1 AND metric_name = $2
               AND measured_at >= NOW() - ($3 || ' days')::INTERVAL
             GROUP BY DATE(measured_at)
             ORDER BY DATE(measured_at) DESC"#,
        )
        .bind(repo_id)
        .bind(metric_name)
        .bind(days)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(QualityTrend::from).collect())
    }

    pub async fn get_complexity_analysis(
        &self,
        repo_id: Uuid,
    ) -> Result<ComplexityAnalysis, sqlx::Error> {
        let row = sqlx::query_as::<_, ComplexityRow>(
            r#"SELECT
                COALESCE(AVG(metric_value) FILTER (WHERE metric_name = 'cyclomatic_complexity'), 0) as avg_complexity,
                COALESCE(MAX(metric_value) FILTER (WHERE metric_name = 'cyclomatic_complexity'), 0) as max_complexity,
                COALESCE(AVG(metric_value) FILTER (WHERE metric_name = 'cognitive_complexity'), 0) as avg_cognitive_complexity,
                COUNT(DISTINCT file_path) FILTER (WHERE metric_name = 'cyclomatic_complexity' AND metric_value > 15) as high_complexity_files,
                COUNT(*) as total_measurements
             FROM code_quality_metrics
             WHERE repo_id = $1
               AND metric_name IN ('cyclomatic_complexity', 'cognitive_complexity')
               AND measured_at = (SELECT MAX(measured_at) FROM code_quality_metrics WHERE repo_id = $1 AND metric_name IN ('cyclomatic_complexity', 'cognitive_complexity'))"#,
        )
        .bind(repo_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(ComplexityAnalysis {
            avg_complexity: row.avg_complexity,
            max_complexity: row.max_complexity,
            avg_cognitive_complexity: row.avg_cognitive_complexity,
            high_complexity_files: row.high_complexity_files,
            total_measurements: row.total_measurements,
        })
    }

    pub async fn get_duplication_report(
        &self,
        repo_id: Uuid,
    ) -> Result<DuplicationReport, sqlx::Error> {
        let row = sqlx::query_as::<_, DuplicationRow>(
            r#"SELECT
                COALESCE(AVG(metric_value) FILTER (WHERE metric_name = 'duplication_ratio'), 0) as duplication_ratio,
                COALESCE(SUM(metric_value) FILTER (WHERE metric_name = 'duplicated_lines'), 0) as total_duplicated_lines,
                COUNT(DISTINCT file_path) FILTER (WHERE metric_name = 'duplication_ratio' AND metric_value > 0) as files_with_duplication
             FROM code_quality_metrics
             WHERE repo_id = $1
               AND metric_name IN ('duplication_ratio', 'duplicated_lines')
               AND measured_at = (SELECT MAX(measured_at) FROM code_quality_metrics WHERE repo_id = $1 AND metric_name IN ('duplication_ratio', 'duplicated_lines'))"#,
        )
        .bind(repo_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(DuplicationReport {
            duplication_ratio: row.duplication_ratio,
            total_duplicated_lines: row.total_duplicated_lines,
            files_with_duplication: row.files_with_duplication,
        })
    }

    pub async fn get_code_smells_report(
        &self,
        repo_id: Uuid,
    ) -> Result<CodeSmellsReport, sqlx::Error> {
        let row = sqlx::query_as::<_, CodeSmellsRow>(
            r#"SELECT
                COALESCE(SUM(metric_value) FILTER (WHERE metric_name = 'code_smells'), 0) as total_smells,
                COALESCE(AVG(metric_value) FILTER (WHERE metric_name = 'smell_density'), 0) as smell_density,
                COUNT(DISTINCT file_path) FILTER (WHERE metric_name = 'code_smells' AND metric_value > 0) as files_with_smells,
                COALESCE(SUM(metric_value) FILTER (WHERE metric_name = 'critical_smells'), 0) as critical_smells,
                COALESCE(SUM(metric_value) FILTER (WHERE metric_name = 'major_smells'), 0) as major_smells,
                COALESCE(SUM(metric_value) FILTER (WHERE metric_name = 'minor_smells'), 0) as minor_smells
             FROM code_quality_metrics
             WHERE repo_id = $1
               AND metric_name IN ('code_smells', 'smell_density', 'critical_smells', 'major_smells', 'minor_smells')
               AND measured_at = (SELECT MAX(measured_at) FROM code_quality_metrics WHERE repo_id = $1 AND metric_name IN ('code_smells', 'smell_density', 'critical_smells', 'major_smells', 'minor_smells'))"#,
        )
        .bind(repo_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(CodeSmellsReport {
            total_smells: row.total_smells,
            smell_density: row.smell_density,
            files_with_smells: row.files_with_smells,
            critical_smells: row.critical_smells,
            major_smells: row.major_smells,
            minor_smells: row.minor_smells,
        })
    }

    pub async fn get_technical_debt_report(
        &self,
        repo_id: Uuid,
    ) -> Result<TechnicalDebtReport, sqlx::Error> {
        let row = sqlx::query_as::<_, TechDebtRow>(
            r#"SELECT
                COALESCE(SUM(metric_value) FILTER (WHERE metric_name = 'technical_debt_hours'), 0) as total_debt_hours,
                COALESCE(AVG(metric_value) FILTER (WHERE metric_name = 'debt_ratio'), 0) as debt_ratio,
                COALESCE(AVG(metric_value) FILTER (WHERE metric_name = 'debt_per_file'), 0) as debt_per_file,
                COALESCE(AVG(metric_value) FILTER (WHERE metric_name = 'remediation_time_priority'), 0) as remediation_time_priority,
                COUNT(DISTINCT file_path) FILTER (WHERE metric_name = 'technical_debt_hours' AND metric_value > 0) as files_with_debt
             FROM code_quality_metrics
             WHERE repo_id = $1
               AND metric_name IN ('technical_debt_hours', 'debt_ratio', 'debt_per_file', 'remediation_time_priority')
               AND measured_at = (SELECT MAX(measured_at) FROM code_quality_metrics WHERE repo_id = $1 AND metric_name IN ('technical_debt_hours', 'debt_ratio', 'debt_per_file', 'remediation_time_priority'))"#,
        )
        .bind(repo_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(TechnicalDebtReport {
            total_debt_hours: row.total_debt_hours,
            debt_ratio: row.debt_ratio,
            debt_per_file: row.debt_per_file,
            remediation_time_priority: row.remediation_time_priority,
            files_with_debt: row.files_with_debt,
        })
    }

    pub async fn create_rule_v2(
        &self,
        repo_id: Uuid,
        req: CreateQualityRuleV2Request,
    ) -> Result<QualityRuleV2, sqlx::Error> {
        let id = Uuid::new_v4();
        let now = Utc::now();
        let enabled = req.enabled.unwrap_or(true);
        let auto_fix = req.auto_fix.unwrap_or(false);
        let fix_config = req.fix_config.unwrap_or(serde_json::json!({}));

        sqlx::query(
            r#"INSERT INTO code_quality_rules_v2 (id, repo_id, name, description, rule_type, severity, pattern, auto_fix, fix_config, enabled, created_at)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)"#,
        )
        .bind(id)
        .bind(repo_id)
        .bind(&req.name)
        .bind(&req.description)
        .bind(req.rule_type.to_string())
        .bind(req.severity.to_string())
        .bind(&req.pattern)
        .bind(auto_fix)
        .bind(&fix_config)
        .bind(enabled)
        .bind(now)
        .execute(&self.pool)
        .await?;

        let config_snapshot = serde_json::json!({
            "name": req.name,
            "description": req.description,
            "rule_type": req.rule_type.to_string(),
            "severity": req.severity.to_string(),
            "pattern": req.pattern,
            "auto_fix": auto_fix,
            "fix_config": fix_config,
        });

        sqlx::query(
            r#"INSERT INTO code_quality_rule_versions (rule_id, version, config_snapshot, change_description, created_at)
               VALUES ($1, 1, $2, $3, $4)"#,
        )
        .bind(id)
        .bind(&config_snapshot)
        .bind("Initial version")
        .bind(now)
        .execute(&self.pool)
        .await?;

        Ok(QualityRuleV2 {
            id,
            repo_id,
            name: req.name,
            description: req.description,
            rule_type: req.rule_type,
            severity: req.severity,
            pattern: req.pattern,
            auto_fix,
            fix_config,
            enabled,
            created_at: now,
        })
    }

    pub async fn get_rule_v2(&self, id: Uuid) -> Result<Option<QualityRuleV2>, sqlx::Error> {
        let row = sqlx::query_as::<_, RuleV2Row>(
            r#"SELECT id, repo_id, name, description, rule_type, severity, pattern, auto_fix, fix_config, enabled, created_at
               FROM code_quality_rules_v2 WHERE id = $1"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(QualityRuleV2::from))
    }

    pub async fn list_rules_v2(
        &self,
        repo_id: Uuid,
        enabled_only: bool,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<QualityRuleV2>, sqlx::Error> {
        let query = if enabled_only {
            r#"SELECT id, repo_id, name, description, rule_type, severity, pattern, auto_fix, fix_config, enabled, created_at
               FROM code_quality_rules_v2
               WHERE repo_id = $1 AND enabled = true
               ORDER BY created_at DESC
               LIMIT $2 OFFSET $3"#
        } else {
            r#"SELECT id, repo_id, name, description, rule_type, severity, pattern, auto_fix, fix_config, enabled, created_at
               FROM code_quality_rules_v2
               WHERE repo_id = $1
               ORDER BY created_at DESC
               LIMIT $2 OFFSET $3"#
        };

        let rows = sqlx::query_as::<_, RuleV2Row>(query)
            .bind(repo_id)
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool)
            .await?;

        Ok(rows.into_iter().map(QualityRuleV2::from).collect())
    }

    pub async fn update_rule_v2(
        &self,
        id: Uuid,
        req: UpdateQualityRuleV2Request,
    ) -> Result<QualityRuleV2, sqlx::Error> {
        if let Some(name) = &req.name {
            sqlx::query(r#"UPDATE code_quality_rules_v2 SET name = $1 WHERE id = $2"#)
                .bind(name)
                .bind(id)
                .execute(&self.pool)
                .await?;
        }
        if let Some(desc) = &req.description {
            sqlx::query(r#"UPDATE code_quality_rules_v2 SET description = $1 WHERE id = $2"#)
                .bind(desc)
                .bind(id)
                .execute(&self.pool)
                .await?;
        }
        if let Some(rule_type) = &req.rule_type {
            sqlx::query(r#"UPDATE code_quality_rules_v2 SET rule_type = $1 WHERE id = $2"#)
                .bind(rule_type.to_string())
                .bind(id)
                .execute(&self.pool)
                .await?;
        }
        if let Some(severity) = &req.severity {
            sqlx::query(r#"UPDATE code_quality_rules_v2 SET severity = $1 WHERE id = $2"#)
                .bind(severity.to_string())
                .bind(id)
                .execute(&self.pool)
                .await?;
        }
        if let Some(pattern) = &req.pattern {
            sqlx::query(r#"UPDATE code_quality_rules_v2 SET pattern = $1 WHERE id = $2"#)
                .bind(pattern)
                .bind(id)
                .execute(&self.pool)
                .await?;
        }
        if let Some(auto_fix) = req.auto_fix {
            sqlx::query(r#"UPDATE code_quality_rules_v2 SET auto_fix = $1 WHERE id = $2"#)
                .bind(auto_fix)
                .bind(id)
                .execute(&self.pool)
                .await?;
        }
        if let Some(fix_config) = &req.fix_config {
            sqlx::query(r#"UPDATE code_quality_rules_v2 SET fix_config = $1 WHERE id = $2"#)
                .bind(fix_config)
                .bind(id)
                .execute(&self.pool)
                .await?;
        }
        if let Some(enabled) = req.enabled {
            sqlx::query(r#"UPDATE code_quality_rules_v2 SET enabled = $1 WHERE id = $2"#)
                .bind(enabled)
                .bind(id)
                .execute(&self.pool)
                .await?;
        }

        let rule = self.get_rule_v2(id).await?.ok_or_else(|| sqlx::Error::RowNotFound)?;

        let config_snapshot = serde_json::json!({
            "name": rule.name,
            "description": rule.description,
            "rule_type": rule.rule_type.to_string(),
            "severity": rule.severity.to_string(),
            "pattern": rule.pattern,
            "auto_fix": rule.auto_fix,
            "fix_config": rule.fix_config,
        });

        let version_row = sqlx::query_scalar::<_, i32>(
            r#"SELECT COALESCE(MAX(version), 0) FROM code_quality_rule_versions WHERE rule_id = $1"#,
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await?;

        let new_version = version_row + 1;

        sqlx::query(
            r#"INSERT INTO code_quality_rule_versions (rule_id, version, config_snapshot, change_description, created_at)
               VALUES ($1, $2, $3, $4, $5)"#,
        )
        .bind(id)
        .bind(new_version)
        .bind(&config_snapshot)
        .bind("Rule updated")
        .bind(Utc::now())
        .execute(&self.pool)
        .await?;

        Ok(rule)
    }

    pub async fn delete_rule_v2(&self, id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(r#"DELETE FROM code_quality_rules_v2 WHERE id = $1"#)
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn get_rule_versions(
        &self,
        rule_id: Uuid,
    ) -> Result<Vec<QualityRuleVersion>, sqlx::Error> {
        let rows = sqlx::query_as::<_, RuleVersionRow>(
            r#"SELECT id, rule_id, version, config_snapshot, change_description, created_at
               FROM code_quality_rule_versions
               WHERE rule_id = $1
               ORDER BY version DESC"#,
        )
        .bind(rule_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(QualityRuleVersion::from).collect())
    }

    pub async fn test_rule(
        &self,
        rule_id: Uuid,
        req: RuleTestRequest,
    ) -> Result<QualityRuleTestResult, sqlx::Error> {
        let id = Uuid::new_v4();
        let now = Utc::now();

        let rule = self.get_rule_v2(rule_id).await?.ok_or_else(|| sqlx::Error::RowNotFound)?;

        let actual_violations = if let Some(_pattern) = rule.pattern {
            match rule.rule_type {
                RuleType::Regex => {
                    sqlx::query_scalar::<_, i64>(
                        r#"SELECT COUNT(*) FROM code_quality_metrics
                           WHERE repo_id = $1 AND metric_name = $2
                           AND file_path = $3"#,
                    )
                    .bind(rule.repo_id)
                    .bind(&rule.name)
                    .bind(&req.test_file)
                    .fetch_one(&self.pool)
                    .await?
                }
                _ => 0,
            }
        } else {
            0
        };

        let passed = actual_violations == req.expected_violations as i64;

        sqlx::query(
            r#"INSERT INTO code_quality_rule_test_results (id, rule_id, test_file, expected_violations, actual_violations, passed, tested_at)
               VALUES ($1, $2, $3, $4, $5, $6, $7)"#,
        )
        .bind(id)
        .bind(rule_id)
        .bind(&req.test_file)
        .bind(req.expected_violations)
        .bind(actual_violations as i32)
        .bind(passed)
        .bind(now)
        .execute(&self.pool)
        .await?;

        Ok(QualityRuleTestResult {
            id,
            rule_id,
            test_file: req.test_file,
            expected_violations: req.expected_violations,
            actual_violations: actual_violations as i32,
            passed,
            tested_at: now,
        })
    }

    pub async fn get_rule_analytics(
        &self,
        rule_id: Uuid,
    ) -> Result<RuleAnalytics, sqlx::Error> {
        let stats_row = sqlx::query_as::<_, RuleStatsRow>(
            r#"SELECT
                COUNT(DISTINCT cqr.id) as total_enforcements,
                COALESCE(SUM(cqm.metric_value), 0) as total_violations,
                CASE WHEN COUNT(DISTINCT cqr.id) > 0 THEN
                    COALESCE(SUM(cqm.metric_value), 0) / COUNT(DISTINCT cqr.id)::double precision
                ELSE 0.0 END as avg_violations_per_run,
                MAX(cqm.measured_at) as last_enforced_at
             FROM code_quality_rules_v2 cqr
             LEFT JOIN code_quality_metrics cqm ON cqr.repo_id = cqm.repo_id AND cqm.metric_name = cqr.name
             WHERE cqr.id = $1"#,
        )
        .bind(rule_id)
        .fetch_one(&self.pool)
        .await?;

        let trend_rows = sqlx::query_as::<_, RuleTrendRow>(
            r#"SELECT
                DATE(cqm.measured_at) as date,
                COUNT(*) as enforcement_count,
                COALESCE(SUM(cqm.metric_value), 0) as violation_count
             FROM code_quality_metrics cqm
             JOIN code_quality_rules_v2 cqr ON cqm.repo_id = cqr.repo_id AND cqm.metric_name = cqr.name
             WHERE cqr.id = $1
               AND cqm.measured_at >= NOW() - INTERVAL '30 days'
             GROUP BY DATE(cqm.measured_at)
             ORDER BY DATE(cqm.measured_at) DESC"#,
        )
        .bind(rule_id)
        .fetch_all(&self.pool)
        .await?;

        let trend = trend_rows
            .into_iter()
            .map(RuleEnforcementTrend::from)
            .collect();

        Ok(RuleAnalytics {
            rule_id,
            total_enforcements: stats_row.total_enforcements,
            total_violations: stats_row.total_violations,
            avg_violations_per_run: stats_row.avg_violations_per_run,
            last_enforced_at: stats_row.last_enforced_at,
            trend,
        })
    }

    pub async fn create_rule_v3(
        &self,
        repo_id: Uuid,
        req: CreateQualityRuleV3Request,
    ) -> Result<QualityRuleV3, sqlx::Error> {
        let id = Uuid::new_v4();
        let now = Utc::now();
        let enabled = req.enabled.unwrap_or(true);
        let auto_fix = req.auto_fix.unwrap_or(false);
        let fix_config = req.fix_config.unwrap_or(serde_json::json!({}));

        sqlx::query(
            r#"INSERT INTO code_quality_rules_v3 (id, repo_id, name, description, rule_type, severity, pattern, auto_fix, fix_config, enabled, version, created_at)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, 1, $11)"#,
        )
        .bind(id)
        .bind(repo_id)
        .bind(&req.name)
        .bind(&req.description)
        .bind(req.rule_type.to_string())
        .bind(req.severity.to_string())
        .bind(&req.pattern)
        .bind(auto_fix)
        .bind(&fix_config)
        .bind(enabled)
        .bind(now)
        .execute(&self.pool)
        .await?;

        Ok(QualityRuleV3 {
            id,
            repo_id,
            name: req.name,
            description: req.description,
            rule_type: req.rule_type,
            severity: req.severity,
            pattern: req.pattern,
            auto_fix,
            fix_config,
            enabled,
            version: 1,
            created_at: now,
        })
    }

    pub async fn get_rule_v3(&self, id: Uuid) -> Result<Option<QualityRuleV3>, sqlx::Error> {
        let row = sqlx::query_as::<_, RuleV3Row>(
            r#"SELECT id, repo_id, name, description, rule_type, severity, pattern, auto_fix, fix_config, enabled, version, created_at
               FROM code_quality_rules_v3 WHERE id = $1"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(QualityRuleV3::from))
    }

    pub async fn list_rules_v3(
        &self,
        repo_id: Uuid,
        enabled_only: bool,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<QualityRuleV3>, sqlx::Error> {
        let query = if enabled_only {
            r#"SELECT id, repo_id, name, description, rule_type, severity, pattern, auto_fix, fix_config, enabled, version, created_at
               FROM code_quality_rules_v3
               WHERE repo_id = $1 AND enabled = true
               ORDER BY created_at DESC
               LIMIT $2 OFFSET $3"#
        } else {
            r#"SELECT id, repo_id, name, description, rule_type, severity, pattern, auto_fix, fix_config, enabled, version, created_at
               FROM code_quality_rules_v3
               WHERE repo_id = $1
               ORDER BY created_at DESC
               LIMIT $2 OFFSET $3"#
        };

        let rows = sqlx::query_as::<_, RuleV3Row>(query)
            .bind(repo_id)
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool)
            .await?;

        Ok(rows.into_iter().map(QualityRuleV3::from).collect())
    }

    pub async fn update_rule_v3(
        &self,
        id: Uuid,
        req: UpdateQualityRuleV3Request,
    ) -> Result<QualityRuleV3, sqlx::Error> {
        let has_updates = req.name.is_some()
            || req.description.is_some()
            || req.rule_type.is_some()
            || req.severity.is_some()
            || req.pattern.is_some()
            || req.auto_fix.is_some()
            || req.fix_config.is_some()
            || req.enabled.is_some();

        if !has_updates {
            return self.get_rule_v3(id).await?.ok_or_else(|| sqlx::Error::RowNotFound);
        }

        let mut builder: QueryBuilder<sqlx::Postgres> =
            QueryBuilder::new("UPDATE code_quality_rules_v3 SET ");

        let mut first = true;

        if let Some(ref v) = req.name {
            if !first { builder.push(", "); }
            builder.push("name = ").push_bind(v.as_str());
            first = false;
        }
        if let Some(ref v) = req.description {
            if !first { builder.push(", "); }
            builder.push("description = ").push_bind(v.as_str());
            first = false;
        }
        if let Some(ref v) = req.rule_type {
            if !first { builder.push(", "); }
            builder.push("rule_type = ").push_bind(v.to_string());
            first = false;
        }
        if let Some(ref v) = req.severity {
            if !first { builder.push(", "); }
            builder.push("severity = ").push_bind(v.to_string());
            first = false;
        }
        if let Some(ref v) = req.pattern {
            if !first { builder.push(", "); }
            builder.push("pattern = ").push_bind(v.as_str());
            first = false;
        }
        if let Some(v) = req.auto_fix {
            if !first { builder.push(", "); }
            builder.push("auto_fix = ").push_bind(v);
            first = false;
        }
        if let Some(ref v) = req.fix_config {
            if !first { builder.push(", "); }
            builder.push("fix_config = ").push_bind(v);
            first = false;
        }
        if let Some(v) = req.enabled {
            if !first { builder.push(", "); }
            builder.push("enabled = ").push_bind(v);
        }

        builder.push(", version = version + 1 WHERE id = ").push_bind(id);
        builder.push(" RETURNING id, repo_id, name, description, rule_type, severity, pattern, auto_fix, fix_config, enabled, version, created_at");

        let row = builder
            .build_query_as::<RuleV3Row>()
            .fetch_one(&self.pool)
            .await?;

        Ok(QualityRuleV3::from(row))
    }

    pub async fn delete_rule_v3(&self, id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(r#"DELETE FROM code_quality_rules_v3 WHERE id = $1"#)
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn create_enforcement(
        &self,
        rule_id: Uuid,
        req: CreateEnforcementRequest,
    ) -> Result<QualityRuleEnforcement, sqlx::Error> {
        let id = Uuid::new_v4();
        let now = Utc::now();
        let enforcement_type = req.enforcement_type.unwrap_or_else(|| "warn".to_string());
        let threshold = req.threshold.unwrap_or(0);
        let enabled = req.enabled.unwrap_or(true);

        sqlx::query(
            r#"INSERT INTO code_quality_rule_enforcement (id, rule_id, enforcement_type, threshold, enabled, created_at)
               VALUES ($1, $2, $3, $4, $5, $6)"#,
        )
        .bind(id)
        .bind(rule_id)
        .bind(&enforcement_type)
        .bind(threshold)
        .bind(enabled)
        .bind(now)
        .execute(&self.pool)
        .await?;

        Ok(QualityRuleEnforcement {
            id,
            rule_id,
            enforcement_type: enforcement_type.parse().unwrap_or(EnforcementType::Warn),
            threshold,
            enabled,
            created_at: now,
        })
    }

    pub async fn get_enforcement(
        &self,
        rule_id: Uuid,
    ) -> Result<Option<QualityRuleEnforcement>, sqlx::Error> {
        let row = sqlx::query_as::<_, EnforcementRow>(
            r#"SELECT id, rule_id, enforcement_type, threshold, enabled, created_at
               FROM code_quality_rule_enforcement
               WHERE rule_id = $1
               LIMIT 1"#,
        )
        .bind(rule_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(QualityRuleEnforcement::from))
    }

    pub async fn update_enforcement(
        &self,
        rule_id: Uuid,
        req: UpdateEnforcementRequest,
    ) -> Result<QualityRuleEnforcement, sqlx::Error> {
        if let Some(ref et) = req.enforcement_type {
            sqlx::query(r#"UPDATE code_quality_rule_enforcement SET enforcement_type = $1 WHERE rule_id = $2"#)
                .bind(et)
                .bind(rule_id)
                .execute(&self.pool)
                .await?;
        }
        if let Some(threshold) = req.threshold {
            sqlx::query(r#"UPDATE code_quality_rule_enforcement SET threshold = $1 WHERE rule_id = $2"#)
                .bind(threshold)
                .bind(rule_id)
                .execute(&self.pool)
                .await?;
        }
        if let Some(enabled) = req.enabled {
            sqlx::query(r#"UPDATE code_quality_rule_enforcement SET enabled = $1 WHERE rule_id = $2"#)
                .bind(enabled)
                .bind(rule_id)
                .execute(&self.pool)
                .await?;
        }

        self.get_enforcement(rule_id)
            .await?
            .ok_or_else(|| sqlx::Error::RowNotFound)
    }

    pub async fn check_threshold(
        &self,
        rule_id: Uuid,
        current_violations: i64,
    ) -> Result<EnforcementThresholdResult, sqlx::Error> {
        let rule = self.get_rule_v3(rule_id).await?.ok_or_else(|| sqlx::Error::RowNotFound)?;
        let enforcement = self.get_enforcement(rule_id).await?;

        let (enforcement_type, threshold) = match enforcement {
            Some(e) => (e.enforcement_type.to_string(), e.threshold),
            None => ("warn".to_string(), 0),
        };

        let would_block = enforcement_type == "block" && current_violations > threshold as i64;

        Ok(EnforcementThresholdResult {
            rule_id,
            rule_name: rule.name,
            enforcement_type,
            threshold,
            current_violations,
            would_block,
        })
    }

    pub async fn get_enforcement_analytics(
        &self,
        rule_id: Uuid,
    ) -> Result<EnforcementAnalytics, sqlx::Error> {
        let stats = sqlx::query_as::<_, EnforcementStatsRow>(
            r#"SELECT
                COUNT(*) as total_enforcements,
                COUNT(*) FILTER (WHERE enforcement_type = 'block') as blocked_count,
                COUNT(*) FILTER (WHERE enforcement_type = 'warn') as warned_count,
                COUNT(*) FILTER (WHERE enforcement_type = 'audit') as audited_count
             FROM code_quality_rule_enforcement
             WHERE rule_id = $1"#,
        )
        .bind(rule_id)
        .fetch_one(&self.pool)
        .await?;

        let avg_violations = sqlx::query_scalar::<_, f64>(
            r#"SELECT COALESCE(AVG(metric_value), 0)
               FROM code_quality_metrics cqm
               JOIN code_quality_rules_v3 cqr ON cqm.repo_id = cqr.repo_id AND cqm.metric_name = cqr.name
               WHERE cqr.id = $1"#,
        )
        .bind(rule_id)
        .fetch_one(&self.pool)
        .await?;

        let trend_rows = sqlx::query_as::<_, EnforcementTrendRow>(
            r#"SELECT
                DATE(cqm.measured_at) as date,
                COUNT(*) as enforcement_count,
                COUNT(*) FILTER (WHERE EXISTS (
                    SELECT 1 FROM code_quality_rule_enforcement eqr
                    WHERE eqr.rule_id = $1 AND eqr.enforcement_type = 'block'
                )) as blocked_count,
                COUNT(*) FILTER (WHERE EXISTS (
                    SELECT 1 FROM code_quality_rule_enforcement eqr
                    WHERE eqr.rule_id = $1 AND eqr.enforcement_type = 'warn'
                )) as warned_count,
                COUNT(*) FILTER (WHERE EXISTS (
                    SELECT 1 FROM code_quality_rule_enforcement eqr
                    WHERE eqr.rule_id = $1 AND eqr.enforcement_type = 'audit'
                )) as audited_count
             FROM code_quality_metrics cqm
             JOIN code_quality_rules_v3 cqr ON cqm.repo_id = cqr.repo_id AND cqm.metric_name = cqr.name
             WHERE cqr.id = $1
               AND cqm.measured_at >= NOW() - INTERVAL '30 days'
             GROUP BY DATE(cqm.measured_at)
             ORDER BY DATE(cqm.measured_at) DESC"#,
        )
        .bind(rule_id)
        .fetch_all(&self.pool)
        .await?;

        let trend = trend_rows.into_iter().map(EnforcementTrend::from).collect();

        Ok(EnforcementAnalytics {
            rule_id,
            total_enforcements: stats.total_enforcements,
            blocked_count: stats.blocked_count,
            warned_count: stats.warned_count,
            audited_count: stats.audited_count,
            avg_violations_per_run: avg_violations,
            last_enforced_at: None,
            trend,
        })
    }

    pub async fn record_metric_v3(
        &self,
        repo_id: Uuid,
        req: RecordMetricV3Request,
    ) -> Result<CodeQualityMetricV3, sqlx::Error> {
        let id = Uuid::new_v4();
        let now = Utc::now();

        sqlx::query(
            r#"INSERT INTO code_quality_metrics_v3 (id, repo_id, file_path, metric_name, metric_value, measured_at)
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

        Ok(CodeQualityMetricV3 {
            id,
            repo_id,
            file_path: req.file_path,
            metric_name: req.metric_name,
            metric_value: req.metric_value,
            measured_at: now,
        })
    }

    pub async fn list_metrics_v3(
        &self,
        repo_id: Uuid,
        metric_name: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<CodeQualityMetricV3>, sqlx::Error> {
        let rows = sqlx::query_as::<_, MetricV3Row>(
            r#"SELECT id, repo_id, file_path, metric_name, metric_value, measured_at
               FROM code_quality_metrics_v3
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

        Ok(rows.into_iter().map(CodeQualityMetricV3::from).collect())
    }

    pub async fn get_metric_summary_v3(
        &self,
        repo_id: Uuid,
        metric_name: &str,
    ) -> Result<Option<CodeQualityMetricSummaryV2>, sqlx::Error> {
        let threshold = sqlx::query_as::<_, ThresholdV2Row>(
            r#"SELECT id, repo_id, metric_name, threshold_value, enabled, created_at
               FROM code_quality_thresholds_v2
               WHERE repo_id = $1 AND metric_name = $2 AND enabled = true
               LIMIT 1"#,
        )
        .bind(repo_id)
        .bind(metric_name)
        .fetch_optional(&self.pool)
        .await?;

        let row = sqlx::query_as::<_, MetricSummaryV3Row>(
            r#"SELECT
                metric_name,
                (SELECT metric_value FROM code_quality_metrics_v3 WHERE repo_id = $1 AND metric_name = $2 ORDER BY measured_at DESC LIMIT 1) as latest_value,
                COALESCE(AVG(metric_value), 0) as avg_value,
                COALESCE(MIN(metric_value), 0) as min_value,
                COALESCE(MAX(metric_value), 0) as max_value,
                COUNT(*) as measurement_count,
                COUNT(DISTINCT file_path) as files_affected
               FROM code_quality_metrics_v3
               WHERE repo_id = $1 AND metric_name = $2"#,
        )
        .bind(repo_id)
        .bind(metric_name)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| {
            let threshold_value = threshold.as_ref().map(|t| t.threshold_value);
            let threshold_exceeded = threshold_value.is_some_and(|tv| r.latest_value > tv);
            CodeQualityMetricSummaryV2 {
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

    pub async fn create_threshold_v2(
        &self,
        repo_id: Uuid,
        req: CreateCodeQualityThresholdV2Request,
    ) -> Result<CodeQualityThresholdV2, sqlx::Error> {
        let id = Uuid::new_v4();
        let now = Utc::now();
        let enabled = req.enabled.unwrap_or(true);

        sqlx::query(
            r#"INSERT INTO code_quality_thresholds_v2 (id, repo_id, metric_name, threshold_value, enabled, created_at)
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

        Ok(CodeQualityThresholdV2 {
            id,
            repo_id,
            metric_name: req.metric_name,
            threshold_value: req.threshold_value,
            enabled,
            created_at: now,
        })
    }

    pub async fn list_thresholds_v2(
        &self,
        repo_id: Uuid,
    ) -> Result<Vec<CodeQualityThresholdV2>, sqlx::Error> {
        let rows = sqlx::query_as::<_, ThresholdV2Row>(
            r#"SELECT id, repo_id, metric_name, threshold_value, enabled, created_at
               FROM code_quality_thresholds_v2
               WHERE repo_id = $1
               ORDER BY metric_name"#,
        )
        .bind(repo_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(CodeQualityThresholdV2::from).collect())
    }

    pub async fn update_threshold_v2(
        &self,
        id: Uuid,
        req: UpdateCodeQualityThresholdV2Request,
    ) -> Result<CodeQualityThresholdV2, sqlx::Error> {
        if let Some(value) = req.threshold_value {
            sqlx::query(r#"UPDATE code_quality_thresholds_v2 SET threshold_value = $1 WHERE id = $2"#)
                .bind(value)
                .bind(id)
                .execute(&self.pool)
                .await?;
        }
        if let Some(enabled) = req.enabled {
            sqlx::query(r#"UPDATE code_quality_thresholds_v2 SET enabled = $1 WHERE id = $2"#)
                .bind(enabled)
                .bind(id)
                .execute(&self.pool)
                .await?;
        }

        let row = sqlx::query_as::<_, ThresholdV2Row>(
            r#"SELECT id, repo_id, metric_name, threshold_value, enabled, created_at
               FROM code_quality_thresholds_v2 WHERE id = $1"#,
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await?;

        Ok(CodeQualityThresholdV2::from(row))
    }

    pub async fn delete_threshold_v2(&self, id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(r#"DELETE FROM code_quality_thresholds_v2 WHERE id = $1"#)
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn detect_violations_v2(
        &self,
        repo_id: Uuid,
    ) -> Result<Vec<CodeQualityViolation>, sqlx::Error> {
        let thresholds = self.list_thresholds_v2(repo_id).await?;
        let mut violations = Vec::new();

        for threshold in thresholds {
            if !threshold.enabled {
                continue;
            }

            let metrics = sqlx::query_as::<_, MetricV3Row>(
                r#"SELECT id, repo_id, file_path, metric_name, metric_value, measured_at
                   FROM code_quality_metrics_v3
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

                    violations.push(CodeQualityViolation {
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

    pub async fn get_enforcement_report_v2(
        &self,
        repo_id: Uuid,
    ) -> Result<CodeQualityEnforcementReportV2, sqlx::Error> {
        let thresholds = self.list_thresholds_v2(repo_id).await?;
        let violations = self.detect_violations_v2(repo_id).await?;

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

        Ok(CodeQualityEnforcementReportV2 {
            repo_id,
            total_thresholds,
            active_thresholds,
            total_violations,
            violations_by_severity,
            violations_by_metric,
            violations,
        })
    }

    pub async fn calculate_quality_score_v2(
        &self,
        repo_id: Uuid,
    ) -> Result<CodeQualityScoreV2, sqlx::Error> {
        let thresholds = self.list_thresholds_v2(repo_id).await?;
        let violations = self.detect_violations_v2(repo_id).await?;

        let _total_thresholds = thresholds.len() as i64;
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

        Ok(CodeQualityScoreV2 {
            repo_id,
            overall_score,
            metrics_evaluated: active_thresholds,
            thresholds_passed,
            thresholds_failed,
            score_breakdown,
        })
    }

    pub async fn record_metric_v5(
        &self,
        repo_id: Uuid,
        req: RecordMetricV5Request,
    ) -> Result<CodeQualityMetricV5, sqlx::Error> {
        let id = Uuid::new_v4();
        let now = Utc::now();

        sqlx::query(
            r#"INSERT INTO code_quality_metrics_v5 (id, repo_id, file_path, metric_name, metric_value, measured_at)
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

        Ok(CodeQualityMetricV5 {
            id,
            repo_id,
            file_path: req.file_path,
            metric_name: req.metric_name,
            metric_value: req.metric_value,
            measured_at: now,
        })
    }

    pub async fn list_metrics_v5(
        &self,
        repo_id: Uuid,
        metric_name: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<CodeQualityMetricV5>, sqlx::Error> {
        let rows = sqlx::query_as::<_, MetricV5Row>(
            r#"SELECT id, repo_id, file_path, metric_name, metric_value, measured_at
               FROM code_quality_metrics_v5
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

        Ok(rows.into_iter().map(CodeQualityMetricV5::from).collect())
    }

    pub async fn get_metric_summary_v5(
        &self,
        repo_id: Uuid,
        metric_name: &str,
    ) -> Result<Option<CodeQualityMetricSummaryV3>, sqlx::Error> {
        let threshold = sqlx::query_as::<_, ThresholdV4Row>(
            r#"SELECT id, repo_id, metric_name, threshold_value, enabled, created_at
               FROM code_quality_thresholds_v4
               WHERE repo_id = $1 AND metric_name = $2 AND enabled = true
               LIMIT 1"#,
        )
        .bind(repo_id)
        .bind(metric_name)
        .fetch_optional(&self.pool)
        .await?;

        let row = sqlx::query_as::<_, MetricSummaryV5Row>(
            r#"SELECT
                metric_name,
                (SELECT metric_value FROM code_quality_metrics_v5 WHERE repo_id = $1 AND metric_name = $2 ORDER BY measured_at DESC LIMIT 1) as latest_value,
                COALESCE(AVG(metric_value), 0) as avg_value,
                COALESCE(MIN(metric_value), 0) as min_value,
                COALESCE(MAX(metric_value), 0) as max_value,
                COUNT(*) as measurement_count,
                COUNT(DISTINCT file_path) as files_affected
               FROM code_quality_metrics_v5
               WHERE repo_id = $1 AND metric_name = $2"#,
        )
        .bind(repo_id)
        .bind(metric_name)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| {
            let threshold_value = threshold.as_ref().map(|t| t.threshold_value);
            let threshold_exceeded = threshold_value.is_some_and(|tv| r.latest_value > tv);
            CodeQualityMetricSummaryV3 {
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

    pub async fn create_threshold_v4(
        &self,
        repo_id: Uuid,
        req: CreateCodeQualityThresholdV4Request,
    ) -> Result<CodeQualityThresholdV4, sqlx::Error> {
        let id = Uuid::new_v4();
        let now = Utc::now();
        let enabled = req.enabled.unwrap_or(true);

        sqlx::query(
            r#"INSERT INTO code_quality_thresholds_v4 (id, repo_id, metric_name, threshold_value, enabled, created_at)
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

        Ok(CodeQualityThresholdV4 {
            id,
            repo_id,
            metric_name: req.metric_name,
            threshold_value: req.threshold_value,
            enabled,
            created_at: now,
        })
    }

    pub async fn list_thresholds_v4(
        &self,
        repo_id: Uuid,
    ) -> Result<Vec<CodeQualityThresholdV4>, sqlx::Error> {
        let rows = sqlx::query_as::<_, ThresholdV4Row>(
            r#"SELECT id, repo_id, metric_name, threshold_value, enabled, created_at
               FROM code_quality_thresholds_v4
               WHERE repo_id = $1
               ORDER BY metric_name"#,
        )
        .bind(repo_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(CodeQualityThresholdV4::from).collect())
    }

    pub async fn update_threshold_v4(
        &self,
        id: Uuid,
        req: UpdateCodeQualityThresholdV4Request,
    ) -> Result<CodeQualityThresholdV4, sqlx::Error> {
        if let Some(value) = req.threshold_value {
            sqlx::query(r#"UPDATE code_quality_thresholds_v4 SET threshold_value = $1 WHERE id = $2"#)
                .bind(value)
                .bind(id)
                .execute(&self.pool)
                .await?;
        }
        if let Some(enabled) = req.enabled {
            sqlx::query(r#"UPDATE code_quality_thresholds_v4 SET enabled = $1 WHERE id = $2"#)
                .bind(enabled)
                .bind(id)
                .execute(&self.pool)
                .await?;
        }

        let row = sqlx::query_as::<_, ThresholdV4Row>(
            r#"SELECT id, repo_id, metric_name, threshold_value, enabled, created_at
               FROM code_quality_thresholds_v4 WHERE id = $1"#,
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await?;

        Ok(CodeQualityThresholdV4::from(row))
    }

    pub async fn delete_threshold_v4(&self, id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(r#"DELETE FROM code_quality_thresholds_v4 WHERE id = $1"#)
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn detect_violations_v3(
        &self,
        repo_id: Uuid,
    ) -> Result<Vec<CodeQualityViolationV2>, sqlx::Error> {
        let thresholds = self.list_thresholds_v4(repo_id).await?;
        let mut violations = Vec::new();

        for threshold in thresholds {
            if !threshold.enabled {
                continue;
            }

            let metrics = sqlx::query_as::<_, MetricV5Row>(
                r#"SELECT id, repo_id, file_path, metric_name, metric_value, measured_at
                   FROM code_quality_metrics_v5
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

                    violations.push(CodeQualityViolationV2 {
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

    pub async fn get_enforcement_report_v3(
        &self,
        repo_id: Uuid,
    ) -> Result<CodeQualityEnforcementReportV3, sqlx::Error> {
        let thresholds = self.list_thresholds_v4(repo_id).await?;
        let violations = self.detect_violations_v3(repo_id).await?;

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

        Ok(CodeQualityEnforcementReportV3 {
            repo_id,
            total_thresholds,
            active_thresholds,
            total_violations,
            violations_by_severity,
            violations_by_metric,
            violations,
        })
    }

    pub async fn calculate_quality_score_v3(
        &self,
        repo_id: Uuid,
    ) -> Result<CodeQualityScoreV3, sqlx::Error> {
        let thresholds = self.list_thresholds_v4(repo_id).await?;
        let violations = self.detect_violations_v3(repo_id).await?;

        let _total_thresholds = thresholds.len() as i64;
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

        Ok(CodeQualityScoreV3 {
            repo_id,
            overall_score,
            metrics_evaluated: active_thresholds,
            thresholds_passed,
            thresholds_failed,
            score_breakdown,
        })
    }

    pub async fn record_metric_v9(
        &self,
        repo_id: Uuid,
        req: RecordMetricV9Request,
    ) -> Result<CodeQualityMetricV9, sqlx::Error> {
        let id = Uuid::new_v4();
        let now = Utc::now();

        sqlx::query(
            r#"INSERT INTO code_quality_metrics_v9 (id, repo_id, file_path, metric_name, metric_value, measured_at)
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

        Ok(CodeQualityMetricV9 {
            id,
            repo_id,
            file_path: req.file_path,
            metric_name: req.metric_name,
            metric_value: req.metric_value,
            measured_at: now,
        })
    }

    pub async fn list_metrics_v9(
        &self,
        repo_id: Uuid,
        metric_name: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<CodeQualityMetricV9>, sqlx::Error> {
        let rows = sqlx::query_as::<_, MetricV9Row>(
            r#"SELECT id, repo_id, file_path, metric_name, metric_value, measured_at
               FROM code_quality_metrics_v9
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

        Ok(rows.into_iter().map(CodeQualityMetricV9::from).collect())
    }

    pub async fn create_threshold_v8(
        &self,
        repo_id: Uuid,
        req: CreateCodeQualityThresholdV8Request,
    ) -> Result<CodeQualityThresholdV8, sqlx::Error> {
        let id = Uuid::new_v4();
        let now = Utc::now();
        let enabled = req.enabled.unwrap_or(true);

        sqlx::query(
            r#"INSERT INTO code_quality_thresholds_v8 (id, repo_id, metric_name, threshold_value, enabled, created_at)
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

        Ok(CodeQualityThresholdV8 {
            id,
            repo_id,
            metric_name: req.metric_name,
            threshold_value: req.threshold_value,
            enabled,
            created_at: now,
        })
    }

    pub async fn list_thresholds_v8(
        &self,
        repo_id: Uuid,
    ) -> Result<Vec<CodeQualityThresholdV8>, sqlx::Error> {
        let rows = sqlx::query_as::<_, ThresholdV8Row>(
            r#"SELECT id, repo_id, metric_name, threshold_value, enabled, created_at
               FROM code_quality_thresholds_v8
               WHERE repo_id = $1
               ORDER BY metric_name"#,
        )
        .bind(repo_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(CodeQualityThresholdV8::from).collect())
    }

    pub async fn update_threshold_v8(
        &self,
        id: Uuid,
        req: UpdateCodeQualityThresholdV8Request,
    ) -> Result<CodeQualityThresholdV8, sqlx::Error> {
        if let Some(value) = req.threshold_value {
            sqlx::query(r#"UPDATE code_quality_thresholds_v8 SET threshold_value = $1 WHERE id = $2"#)
                .bind(value)
                .bind(id)
                .execute(&self.pool)
                .await?;
        }
        if let Some(enabled) = req.enabled {
            sqlx::query(r#"UPDATE code_quality_thresholds_v8 SET enabled = $1 WHERE id = $2"#)
                .bind(enabled)
                .bind(id)
                .execute(&self.pool)
                .await?;
        }

        let row = sqlx::query_as::<_, ThresholdV8Row>(
            r#"SELECT id, repo_id, metric_name, threshold_value, enabled, created_at
               FROM code_quality_thresholds_v8 WHERE id = $1"#,
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await?;

        Ok(CodeQualityThresholdV8::from(row))
    }

    pub async fn delete_threshold_v8(&self, id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(r#"DELETE FROM code_quality_thresholds_v8 WHERE id = $1"#)
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn detect_violations_v4(
        &self,
        repo_id: Uuid,
    ) -> Result<Vec<CodeQualityViolationV3>, sqlx::Error> {
        let thresholds = self.list_thresholds_v8(repo_id).await?;
        let mut violations = Vec::new();

        for threshold in thresholds {
            if !threshold.enabled {
                continue;
            }

            let metrics = sqlx::query_as::<_, MetricV9Row>(
                r#"SELECT id, repo_id, file_path, metric_name, metric_value, measured_at
                   FROM code_quality_metrics_v9
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

                    violations.push(CodeQualityViolationV3 {
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

    pub async fn get_enforcement_report_v4(
        &self,
        repo_id: Uuid,
    ) -> Result<CodeQualityEnforcementReportV4, sqlx::Error> {
        let thresholds = self.list_thresholds_v8(repo_id).await?;
        let violations = self.detect_violations_v4(repo_id).await?;

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

        Ok(CodeQualityEnforcementReportV4 {
            repo_id,
            total_thresholds,
            active_thresholds,
            total_violations,
            violations_by_severity,
            violations_by_metric,
            violations,
        })
    }

    pub async fn calculate_quality_score_v4(
        &self,
        repo_id: Uuid,
    ) -> Result<CodeQualityScoreV4, sqlx::Error> {
        let thresholds = self.list_thresholds_v8(repo_id).await?;
        let violations = self.detect_violations_v4(repo_id).await?;

        let _total_thresholds = thresholds.len() as i64;
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

        Ok(CodeQualityScoreV4 {
            repo_id,
            overall_score,
            metrics_evaluated: active_thresholds,
            thresholds_passed,
            thresholds_failed,
            score_breakdown,
        })
    }

    pub async fn record_metric_v16(
        &self,
        repo_id: Uuid,
        req: RecordMetricV16Request,
    ) -> Result<CodeQualityMetricV16, sqlx::Error> {
        let id = Uuid::new_v4();
        let now = Utc::now();

        sqlx::query(
            r#"INSERT INTO code_quality_metrics_v16 (id, repo_id, file_path, metric_name, metric_value, measured_at)
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

        Ok(CodeQualityMetricV16 {
            id,
            repo_id,
            file_path: req.file_path,
            metric_name: req.metric_name,
            metric_value: req.metric_value,
            measured_at: now,
        })
    }

    pub async fn list_metrics_v16(
        &self,
        repo_id: Uuid,
        metric_name: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<CodeQualityMetricV16>, sqlx::Error> {
        let rows = sqlx::query_as::<_, MetricV16Row>(
            r#"SELECT id, repo_id, file_path, metric_name, metric_value, measured_at
               FROM code_quality_metrics_v16
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

        Ok(rows.into_iter().map(CodeQualityMetricV16::from).collect())
    }

    pub async fn create_threshold_v15(
        &self,
        repo_id: Uuid,
        req: CreateCodeQualityThresholdV15Request,
    ) -> Result<CodeQualityThresholdV15, sqlx::Error> {
        let id = Uuid::new_v4();
        let now = Utc::now();
        let enabled = req.enabled.unwrap_or(true);

        sqlx::query(
            r#"INSERT INTO code_quality_thresholds_v15 (id, repo_id, metric_name, threshold_value, enabled, created_at)
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

        Ok(CodeQualityThresholdV15 {
            id,
            repo_id,
            metric_name: req.metric_name,
            threshold_value: req.threshold_value,
            enabled,
            created_at: now,
        })
    }

    pub async fn list_thresholds_v15(
        &self,
        repo_id: Uuid,
    ) -> Result<Vec<CodeQualityThresholdV15>, sqlx::Error> {
        let rows = sqlx::query_as::<_, ThresholdV15Row>(
            r#"SELECT id, repo_id, metric_name, threshold_value, enabled, created_at
               FROM code_quality_thresholds_v15
               WHERE repo_id = $1
               ORDER BY metric_name"#,
        )
        .bind(repo_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(CodeQualityThresholdV15::from).collect())
    }

    pub async fn update_threshold_v15(
        &self,
        id: Uuid,
        req: UpdateCodeQualityThresholdV15Request,
    ) -> Result<CodeQualityThresholdV15, sqlx::Error> {
        if let Some(value) = req.threshold_value {
            sqlx::query(r#"UPDATE code_quality_thresholds_v15 SET threshold_value = $1 WHERE id = $2"#)
                .bind(value)
                .bind(id)
                .execute(&self.pool)
                .await?;
        }
        if let Some(enabled) = req.enabled {
            sqlx::query(r#"UPDATE code_quality_thresholds_v15 SET enabled = $1 WHERE id = $2"#)
                .bind(enabled)
                .bind(id)
                .execute(&self.pool)
                .await?;
        }

        let row = sqlx::query_as::<_, ThresholdV15Row>(
            r#"SELECT id, repo_id, metric_name, threshold_value, enabled, created_at
               FROM code_quality_thresholds_v15 WHERE id = $1"#,
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await?;

        Ok(CodeQualityThresholdV15::from(row))
    }

    pub async fn delete_threshold_v15(&self, id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(r#"DELETE FROM code_quality_thresholds_v15 WHERE id = $1"#)
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn detect_violations_v5(
        &self,
        repo_id: Uuid,
    ) -> Result<Vec<CodeQualityViolationV16>, sqlx::Error> {
        let thresholds = self.list_thresholds_v15(repo_id).await?;
        let mut violations = Vec::new();

        for threshold in thresholds {
            if !threshold.enabled {
                continue;
            }

            let metrics = sqlx::query_as::<_, MetricV16Row>(
                r#"SELECT id, repo_id, file_path, metric_name, metric_value, measured_at
                   FROM code_quality_metrics_v16
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

                    violations.push(CodeQualityViolationV16 {
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
        let thresholds = self.list_thresholds_v15(repo_id).await?;
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
        let thresholds = self.list_thresholds_v15(repo_id).await?;
        let violations = self.detect_violations_v5(repo_id).await?;

        let _total_thresholds = thresholds.len() as i64;
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

    pub async fn get_metric_summary_v16(
        &self,
        repo_id: Uuid,
        metric_name: &str,
    ) -> Result<Option<CodeQualityMetricSummaryV5>, sqlx::Error> {
        let threshold = sqlx::query_as::<_, ThresholdV15Row>(
            r#"SELECT id, repo_id, metric_name, threshold_value, enabled, created_at
               FROM code_quality_thresholds_v15
               WHERE repo_id = $1 AND metric_name = $2 AND enabled = true
               LIMIT 1"#,
        )
        .bind(repo_id)
        .bind(metric_name)
        .fetch_optional(&self.pool)
        .await?;

        let row = sqlx::query_as::<_, MetricSummaryV16Row>(
            r#"SELECT
                metric_name,
                (SELECT metric_value FROM code_quality_metrics_v16 WHERE repo_id = $1 AND metric_name = $2 ORDER BY measured_at DESC LIMIT 1) as latest_value,
                COALESCE(AVG(metric_value), 0) as avg_value,
                COALESCE(MIN(metric_value), 0) as min_value,
                COALESCE(MAX(metric_value), 0) as max_value,
                COUNT(*) as measurement_count,
                COUNT(DISTINCT file_path) as files_affected
               FROM code_quality_metrics_v16
               WHERE repo_id = $1 AND metric_name = $2"#,
        )
        .bind(repo_id)
        .bind(metric_name)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| {
            let threshold_value = threshold.as_ref().map(|t| t.threshold_value);
            let threshold_exceeded = threshold_value.is_some_and(|tv| r.latest_value > tv);
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

    pub async fn record_metric_v18(
        &self,
        repo_id: Uuid,
        req: RecordMetricV18Request,
    ) -> Result<CodeQualityMetricV18, sqlx::Error> {
        let id = Uuid::new_v4();
        let now = Utc::now();

        sqlx::query(
            r#"INSERT INTO code_quality_metrics_v19 (id, repo_id, file_path, metric_name, metric_value, measured_at)
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

        Ok(CodeQualityMetricV18 {
            id,
            repo_id,
            file_path: req.file_path,
            metric_name: req.metric_name,
            metric_value: req.metric_value,
            measured_at: now,
        })
    }

    pub async fn list_metrics_v18(
        &self,
        repo_id: Uuid,
        metric_name: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<CodeQualityMetricV18>, sqlx::Error> {
        let rows = sqlx::query_as::<_, MetricV18Row>(
            r#"SELECT id, repo_id, file_path, metric_name, metric_value, measured_at
               FROM code_quality_metrics_v19
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

        Ok(rows.into_iter().map(CodeQualityMetricV18::from).collect())
    }

    pub async fn create_threshold_v18(
        &self,
        repo_id: Uuid,
        req: CreateCodeQualityThresholdV18Request,
    ) -> Result<CodeQualityThresholdV18, sqlx::Error> {
        let id = Uuid::new_v4();
        let now = Utc::now();
        let enabled = req.enabled.unwrap_or(true);

        sqlx::query(
            r#"INSERT INTO code_quality_thresholds_v18 (id, repo_id, metric_name, threshold_value, enabled, created_at)
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

        Ok(CodeQualityThresholdV18 {
            id,
            repo_id,
            metric_name: req.metric_name,
            threshold_value: req.threshold_value,
            enabled,
            created_at: now,
        })
    }

    pub async fn list_thresholds_v18(
        &self,
        repo_id: Uuid,
    ) -> Result<Vec<CodeQualityThresholdV18>, sqlx::Error> {
        let rows = sqlx::query_as::<_, ThresholdV18Row>(
            r#"SELECT id, repo_id, metric_name, threshold_value, enabled, created_at
               FROM code_quality_thresholds_v18
               WHERE repo_id = $1
               ORDER BY metric_name"#,
        )
        .bind(repo_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(CodeQualityThresholdV18::from).collect())
    }

    pub async fn update_threshold_v18(
        &self,
        id: Uuid,
        req: UpdateCodeQualityThresholdV18Request,
    ) -> Result<CodeQualityThresholdV18, sqlx::Error> {
        if let Some(value) = req.threshold_value {
            sqlx::query(r#"UPDATE code_quality_thresholds_v18 SET threshold_value = $1 WHERE id = $2"#)
                .bind(value)
                .bind(id)
                .execute(&self.pool)
                .await?;
        }
        if let Some(enabled) = req.enabled {
            sqlx::query(r#"UPDATE code_quality_thresholds_v18 SET enabled = $1 WHERE id = $2"#)
                .bind(enabled)
                .bind(id)
                .execute(&self.pool)
                .await?;
        }

        let row = sqlx::query_as::<_, ThresholdV18Row>(
            r#"SELECT id, repo_id, metric_name, threshold_value, enabled, created_at
               FROM code_quality_thresholds_v18 WHERE id = $1"#,
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await?;

        Ok(CodeQualityThresholdV18::from(row))
    }

    pub async fn delete_threshold_v18(&self, id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(r#"DELETE FROM code_quality_thresholds_v18 WHERE id = $1"#)
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn detect_violations_v6(
        &self,
        repo_id: Uuid,
    ) -> Result<Vec<CodeQualityViolationV18>, sqlx::Error> {
        let thresholds = self.list_thresholds_v18(repo_id).await?;
        let mut violations = Vec::new();

        for threshold in thresholds {
            if !threshold.enabled {
                continue;
            }

            let metrics = sqlx::query_as::<_, MetricV18Row>(
                r#"SELECT id, repo_id, file_path, metric_name, metric_value, measured_at
                   FROM code_quality_metrics_v19
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

                    violations.push(CodeQualityViolationV18 {
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

    pub async fn get_enforcement_report_v6(
        &self,
        repo_id: Uuid,
    ) -> Result<CodeQualityEnforcementReportV6, sqlx::Error> {
        let thresholds = self.list_thresholds_v18(repo_id).await?;
        let violations = self.detect_violations_v6(repo_id).await?;

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

        Ok(CodeQualityEnforcementReportV6 {
            repo_id,
            total_thresholds,
            active_thresholds,
            total_violations,
            violations_by_severity,
            violations_by_metric,
            violations,
        })
    }

    pub async fn calculate_quality_score_v6(
        &self,
        repo_id: Uuid,
    ) -> Result<CodeQualityScoreV6, sqlx::Error> {
        let thresholds = self.list_thresholds_v18(repo_id).await?;
        let violations = self.detect_violations_v6(repo_id).await?;

        let _total_thresholds = thresholds.len() as i64;
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

        Ok(CodeQualityScoreV6 {
            repo_id,
            overall_score,
            metrics_evaluated: active_thresholds,
            thresholds_passed,
            thresholds_failed,
            score_breakdown,
        })
    }

    pub async fn get_metric_summary_v18(
        &self,
        repo_id: Uuid,
        metric_name: &str,
    ) -> Result<Option<CodeQualityMetricSummaryV6>, sqlx::Error> {
        let threshold = sqlx::query_as::<_, ThresholdV18Row>(
            r#"SELECT id, repo_id, metric_name, threshold_value, enabled, created_at
               FROM code_quality_thresholds_v18
               WHERE repo_id = $1 AND metric_name = $2 AND enabled = true
               LIMIT 1"#,
        )
        .bind(repo_id)
        .bind(metric_name)
        .fetch_optional(&self.pool)
        .await?;

        let row = sqlx::query_as::<_, MetricSummaryV18Row>(
            r#"SELECT
                metric_name,
                (SELECT metric_value FROM code_quality_metrics_v19 WHERE repo_id = $1 AND metric_name = $2 ORDER BY measured_at DESC LIMIT 1) as latest_value,
                COALESCE(AVG(metric_value), 0) as avg_value,
                COALESCE(MIN(metric_value), 0) as min_value,
                COALESCE(MAX(metric_value), 0) as max_value,
                COUNT(*) as measurement_count,
                COUNT(DISTINCT file_path) as files_affected
               FROM code_quality_metrics_v19
               WHERE repo_id = $1 AND metric_name = $2"#,
        )
        .bind(repo_id)
        .bind(metric_name)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| {
            let threshold_value = threshold.as_ref().map(|t| t.threshold_value);
            let threshold_exceeded = threshold_value.is_some_and(|tv| r.latest_value > tv);
            CodeQualityMetricSummaryV6 {
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

    pub async fn get_metric_summary_v9(
        &self,
        repo_id: Uuid,
        metric_name: &str,
    ) -> Result<Option<CodeQualityMetricSummaryV4>, sqlx::Error> {
        let threshold = sqlx::query_as::<_, ThresholdV8Row>(
            r#"SELECT id, repo_id, metric_name, threshold_value, enabled, created_at
               FROM code_quality_thresholds_v8
               WHERE repo_id = $1 AND metric_name = $2 AND enabled = true
               LIMIT 1"#,
        )
        .bind(repo_id)
        .bind(metric_name)
        .fetch_optional(&self.pool)
        .await?;

        let row = sqlx::query_as::<_, MetricSummaryV9Row>(
            r#"SELECT
                metric_name,
                (SELECT metric_value FROM code_quality_metrics_v9 WHERE repo_id = $1 AND metric_name = $2 ORDER BY measured_at DESC LIMIT 1) as latest_value,
                COALESCE(AVG(metric_value), 0) as avg_value,
                COALESCE(MIN(metric_value), 0) as min_value,
                COALESCE(MAX(metric_value), 0) as max_value,
                COUNT(*) as measurement_count,
                COUNT(DISTINCT file_path) as files_affected
               FROM code_quality_metrics_v9
               WHERE repo_id = $1 AND metric_name = $2"#,
        )
        .bind(repo_id)
        .bind(metric_name)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| {
            let threshold_value = threshold.as_ref().map(|t| t.threshold_value);
            let threshold_exceeded = threshold_value.is_some_and(|tv| r.latest_value > tv);
            CodeQualityMetricSummaryV4 {
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

    pub async fn create_rule_v19(
        &self,
        req: CreateCodeQualityRuleV19Request,
    ) -> Result<CodeQualityRuleV19, sqlx::Error> {
        let id = Uuid::new_v4();
        let now = Utc::now();
        let description = req.description.unwrap_or_default();
        let severity = req.severity.unwrap_or_else(|| "warning".to_string());
        let enabled = req.enabled.unwrap_or(true);
        let rule_config = req.rule_config.unwrap_or(serde_json::json!({}));

        sqlx::query(
            r#"INSERT INTO code_quality_rules_v19 (id, name, description, rule_type, severity, enabled, rule_config, created_at)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8)"#,
        )
        .bind(id)
        .bind(&req.name)
        .bind(&description)
        .bind(&req.rule_type)
        .bind(&severity)
        .bind(enabled)
        .bind(&rule_config)
        .bind(now)
        .execute(&self.pool)
        .await?;

        Ok(CodeQualityRuleV19 {
            id,
            name: req.name,
            description,
            rule_type: req.rule_type,
            severity,
            enabled,
            rule_config,
            created_at: now,
        })
    }

    pub async fn get_rule_v19(&self, id: Uuid) -> Result<Option<CodeQualityRuleV19>, sqlx::Error> {
        let row = sqlx::query_as::<_, RuleV19Row>(
            r#"SELECT id, name, description, rule_type, severity, enabled, rule_config, created_at
               FROM code_quality_rules_v19 WHERE id = $1"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(CodeQualityRuleV19::from))
    }

    pub async fn list_rules_v19(
        &self,
        enabled_only: bool,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<CodeQualityRuleV19>, sqlx::Error> {
        let query = if enabled_only {
            r#"SELECT id, name, description, rule_type, severity, enabled, rule_config, created_at
               FROM code_quality_rules_v19
               WHERE enabled = true
               ORDER BY created_at DESC
               LIMIT $1 OFFSET $2"#
        } else {
            r#"SELECT id, name, description, rule_type, severity, enabled, rule_config, created_at
               FROM code_quality_rules_v19
               ORDER BY created_at DESC
               LIMIT $1 OFFSET $2"#
        };

        let rows = sqlx::query_as::<_, RuleV19Row>(query)
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool)
            .await?;

        Ok(rows.into_iter().map(CodeQualityRuleV19::from).collect())
    }

    pub async fn update_rule_v19(
        &self,
        id: Uuid,
        req: UpdateCodeQualityRuleV19Request,
    ) -> Result<CodeQualityRuleV19, sqlx::Error> {
        if let Some(name) = &req.name {
            sqlx::query(r#"UPDATE code_quality_rules_v19 SET name = $1 WHERE id = $2"#)
                .bind(name)
                .bind(id)
                .execute(&self.pool)
                .await?;
        }
        if let Some(desc) = &req.description {
            sqlx::query(r#"UPDATE code_quality_rules_v19 SET description = $1 WHERE id = $2"#)
                .bind(desc)
                .bind(id)
                .execute(&self.pool)
                .await?;
        }
        if let Some(rule_type) = &req.rule_type {
            sqlx::query(r#"UPDATE code_quality_rules_v19 SET rule_type = $1 WHERE id = $2"#)
                .bind(rule_type)
                .bind(id)
                .execute(&self.pool)
                .await?;
        }
        if let Some(severity) = &req.severity {
            sqlx::query(r#"UPDATE code_quality_rules_v19 SET severity = $1 WHERE id = $2"#)
                .bind(severity)
                .bind(id)
                .execute(&self.pool)
                .await?;
        }
        if let Some(enabled) = req.enabled {
            sqlx::query(r#"UPDATE code_quality_rules_v19 SET enabled = $1 WHERE id = $2"#)
                .bind(enabled)
                .bind(id)
                .execute(&self.pool)
                .await?;
        }
        if let Some(config) = &req.rule_config {
            sqlx::query(r#"UPDATE code_quality_rules_v19 SET rule_config = $1 WHERE id = $2"#)
                .bind(config)
                .bind(id)
                .execute(&self.pool)
                .await?;
        }

        self.get_rule_v19(id).await?.ok_or_else(|| sqlx::Error::RowNotFound)
    }

    pub async fn delete_rule_v19(&self, id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(r#"DELETE FROM code_quality_rules_v19 WHERE id = $1"#)
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn record_rule_usage(
        &self,
        req: RecordRuleUsageV19Request,
    ) -> Result<CodeQualityRuleUsageV19, sqlx::Error> {
        let id = Uuid::new_v4();
        let now = Utc::now();

        sqlx::query(
            r#"INSERT INTO code_quality_rule_usage_v19 (id, rule_id, repo_id, trigger_count, last_triggered_at)
               VALUES ($1, $2, $3, 1, $4)
               ON CONFLICT (rule_id, repo_id) DO UPDATE SET
               trigger_count = code_quality_rule_usage_v19.trigger_count + 1,
               last_triggered_at = $4"#,
        )
        .bind(id)
        .bind(req.rule_id)
        .bind(req.repo_id)
        .bind(now)
        .execute(&self.pool)
        .await?;

        let row = sqlx::query_as::<_, RuleUsageV19Row>(
            r#"SELECT id, rule_id, repo_id, trigger_count, last_triggered_at
               FROM code_quality_rule_usage_v19 WHERE rule_id = $1 AND repo_id = $2"#,
        )
        .bind(req.rule_id)
        .bind(req.repo_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(CodeQualityRuleUsageV19::from(row))
    }

    pub async fn get_rule_usage_summary(
        &self,
    ) -> Result<RuleUsageSummaryV19, sqlx::Error> {
        let stats = sqlx::query_as::<_, RuleUsageStatsRow>(
            r#"SELECT
                (SELECT COUNT(*) FROM code_quality_rules_v19) as total_rules,
                (SELECT COUNT(*) FROM code_quality_rules_v19 WHERE enabled = true) as active_rules,
                COALESCE(SUM(trigger_count), 0) as total_triggers
               FROM code_quality_rule_usage_v19"#,
        )
        .fetch_one(&self.pool)
        .await?;

        let most_used = sqlx::query_as::<_, RuleUsageV19Row>(
            r#"SELECT id, rule_id, repo_id, trigger_count, last_triggered_at
               FROM code_quality_rule_usage_v19
               ORDER BY trigger_count DESC
               LIMIT 10"#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(RuleUsageSummaryV19 {
            total_rules: stats.total_rules,
            active_rules: stats.active_rules,
            total_triggers: stats.total_triggers,
            most_used_rules: most_used.into_iter().map(CodeQualityRuleUsageV19::from).collect(),
        })
    }

    pub async fn create_custom_rule_v22(
        &self,
        req: CreateCustomRuleV22Request,
    ) -> Result<CustomRuleCreationV22, sqlx::Error> {
        let id = Uuid::new_v4();
        let now = Utc::now();
        let description = req.description.unwrap_or_default();
        let severity = req.severity.unwrap_or_else(|| "warning".to_string());

        sqlx::query(
            r#"INSERT INTO code_quality_rules_v19 (id, name, description, rule_type, severity, enabled, rule_config, created_at)
               VALUES ($1, $2, $3, $4, $5, true, $6, $7)"#,
        )
        .bind(id)
        .bind(&req.name)
        .bind(&description)
        .bind(&req.rule_type)
        .bind(&severity)
        .bind(&req.rule_config)
        .bind(now)
        .execute(&self.pool)
        .await?;

        let test_results = req.test_files.unwrap_or_default();

        Ok(CustomRuleCreationV22 {
            id,
            name: req.name,
            description,
            rule_type: req.rule_type,
            severity,
            rule_config: req.rule_config,
            test_results,
            created_at: now,
        })
    }

    pub async fn get_rule_effectiveness(
        &self,
        rule_id: Uuid,
    ) -> Result<RuleEffectivenessAnalysisV22, sqlx::Error> {
        let rule = self.get_rule_v19(rule_id).await?.ok_or_else(|| sqlx::Error::RowNotFound)?;

        let stats = sqlx::query_as::<_, RuleEffectivenessStatsRow>(
            r#"SELECT
                COALESCE(SUM(trigger_count), 0) as total_enforcements,
                COALESCE(SUM(trigger_count) FILTER (WHERE trigger_count > 0), 0) as total_violations
               FROM code_quality_rule_usage_v19
               WHERE rule_id = $1"#,
        )
        .bind(rule_id)
        .fetch_one(&self.pool)
        .await?;

        let effectiveness_score = if stats.total_enforcements > 0 {
            (stats.total_violations as f64 / stats.total_enforcements as f64) * 100.0
        } else {
            0.0
        };

        let false_positive_rate = 0.0;

        let trend = Vec::new();

        Ok(RuleEffectivenessAnalysisV22 {
            rule_id,
            rule_name: rule.name,
            total_enforcements: stats.total_enforcements,
            total_violations: stats.total_violations,
            effectiveness_score,
            false_positive_rate,
            trend,
        })
    }
}

#[derive(sqlx::FromRow)]
struct MetricRow {
    repo_id: Uuid,
    metric_name: String,
    metric_value: f64,
    file_path: Option<String>,
}

impl From<MetricRow> for QualityMetricReport {
    fn from(row: MetricRow) -> Self {
        Self {
            repo_id: row.repo_id,
            metric_name: row.metric_name,
            metric_value: row.metric_value,
            file_path: row.file_path,
        }
    }
}

#[derive(sqlx::FromRow)]
struct SummaryRow {
    metric_name: String,
    latest_value: f64,
    avg_value: f64,
    min_value: f64,
    max_value: f64,
    measurement_count: i64,
    files_affected: i64,
}

impl From<SummaryRow> for QualityMetricSummary {
    fn from(row: SummaryRow) -> Self {
        Self {
            metric_name: row.metric_name,
            latest_value: row.latest_value,
            avg_value: row.avg_value,
            min_value: row.min_value,
            max_value: row.max_value,
            measurement_count: row.measurement_count,
            files_affected: row.files_affected,
        }
    }
}

#[derive(sqlx::FromRow)]
struct TrendRow {
    date: chrono::NaiveDate,
    avg_value: f64,
    min_value: f64,
    max_value: f64,
    measurement_count: i64,
}

impl From<TrendRow> for QualityTrend {
    fn from(row: TrendRow) -> Self {
        Self {
            date: row.date,
            avg_value: row.avg_value,
            min_value: row.min_value,
            max_value: row.max_value,
            measurement_count: row.measurement_count,
        }
    }
}

#[derive(sqlx::FromRow)]
struct ComplexityRow {
    avg_complexity: f64,
    max_complexity: f64,
    avg_cognitive_complexity: f64,
    high_complexity_files: i64,
    total_measurements: i64,
}

#[derive(sqlx::FromRow)]
struct DuplicationRow {
    duplication_ratio: f64,
    total_duplicated_lines: f64,
    files_with_duplication: i64,
}

#[derive(sqlx::FromRow)]
struct CodeSmellsRow {
    total_smells: f64,
    smell_density: f64,
    files_with_smells: i64,
    critical_smells: f64,
    major_smells: f64,
    minor_smells: f64,
}

#[derive(sqlx::FromRow)]
struct TechDebtRow {
    total_debt_hours: f64,
    debt_ratio: f64,
    debt_per_file: f64,
    remediation_time_priority: f64,
    files_with_debt: i64,
}

#[derive(sqlx::FromRow)]
struct RuleRow {
    id: Uuid,
    repo_id: Uuid,
    name: String,
    description: String,
    rule_type: String,
    severity: String,
    pattern: Option<String>,
    enabled: bool,
    created_at: chrono::DateTime<Utc>,
}

impl From<RuleRow> for QualityRule {
    fn from(row: RuleRow) -> Self {
        Self {
            id: row.id,
            repo_id: row.repo_id,
            name: row.name,
            description: row.description,
            rule_type: row.rule_type.parse().unwrap_or(RuleType::Custom),
            severity: row.severity.parse().unwrap_or(Severity::Warning),
            pattern: row.pattern,
            enabled: row.enabled,
            created_at: row.created_at,
        }
    }
}

#[derive(sqlx::FromRow)]
struct RuleEnforcementRow {
    rule_id: Uuid,
    rule_name: String,
    violations: f64,
    files_checked: i64,
    files_violating: i64,
}

impl From<RuleEnforcementRow> for QualityRuleEnforcementResult {
    fn from(row: RuleEnforcementRow) -> Self {
        Self {
            rule_id: row.rule_id,
            rule_name: row.rule_name,
            violations: row.violations,
            files_checked: row.files_checked,
            files_violating: row.files_violating,
        }
    }
}

#[derive(sqlx::FromRow)]
struct RuleV2Row {
    id: Uuid,
    repo_id: Uuid,
    name: String,
    description: String,
    rule_type: String,
    severity: String,
    pattern: Option<String>,
    auto_fix: bool,
    fix_config: serde_json::Value,
    enabled: bool,
    created_at: chrono::DateTime<Utc>,
}

impl From<RuleV2Row> for QualityRuleV2 {
    fn from(row: RuleV2Row) -> Self {
        Self {
            id: row.id,
            repo_id: row.repo_id,
            name: row.name,
            description: row.description,
            rule_type: row.rule_type.parse().unwrap_or(RuleType::Custom),
            severity: row.severity.parse().unwrap_or(Severity::Warning),
            pattern: row.pattern,
            auto_fix: row.auto_fix,
            fix_config: row.fix_config,
            enabled: row.enabled,
            created_at: row.created_at,
        }
    }
}

#[derive(sqlx::FromRow)]
struct RuleVersionRow {
    id: Uuid,
    rule_id: Uuid,
    version: i32,
    config_snapshot: serde_json::Value,
    change_description: String,
    created_at: chrono::DateTime<Utc>,
}

impl From<RuleVersionRow> for QualityRuleVersion {
    fn from(row: RuleVersionRow) -> Self {
        Self {
            id: row.id,
            rule_id: row.rule_id,
            version: row.version,
            config_snapshot: row.config_snapshot,
            change_description: row.change_description,
            created_at: row.created_at,
        }
    }
}

#[derive(sqlx::FromRow)]
struct RuleStatsRow {
    total_enforcements: i64,
    total_violations: i64,
    avg_violations_per_run: f64,
    last_enforced_at: Option<chrono::DateTime<Utc>>,
}

#[derive(sqlx::FromRow)]
struct RuleTrendRow {
    date: chrono::NaiveDate,
    enforcement_count: i64,
    violation_count: i64,
}

impl From<RuleTrendRow> for RuleEnforcementTrend {
    fn from(row: RuleTrendRow) -> Self {
        Self {
            date: row.date,
            enforcement_count: row.enforcement_count,
            violation_count: row.violation_count,
        }
    }
}

#[derive(sqlx::FromRow)]
struct RuleV3Row {
    id: Uuid,
    repo_id: Uuid,
    name: String,
    description: String,
    rule_type: String,
    severity: String,
    pattern: Option<String>,
    auto_fix: bool,
    fix_config: serde_json::Value,
    enabled: bool,
    version: i32,
    created_at: chrono::DateTime<Utc>,
}

impl From<RuleV3Row> for QualityRuleV3 {
    fn from(row: RuleV3Row) -> Self {
        Self {
            id: row.id,
            repo_id: row.repo_id,
            name: row.name,
            description: row.description,
            rule_type: row.rule_type.parse().unwrap_or(RuleType::Custom),
            severity: row.severity.parse().unwrap_or(Severity::Warning),
            pattern: row.pattern,
            auto_fix: row.auto_fix,
            fix_config: row.fix_config,
            enabled: row.enabled,
            version: row.version,
            created_at: row.created_at,
        }
    }
}

#[derive(sqlx::FromRow)]
struct EnforcementRow {
    id: Uuid,
    rule_id: Uuid,
    enforcement_type: String,
    threshold: i32,
    enabled: bool,
    created_at: chrono::DateTime<Utc>,
}

impl From<EnforcementRow> for QualityRuleEnforcement {
    fn from(row: EnforcementRow) -> Self {
        Self {
            id: row.id,
            rule_id: row.rule_id,
            enforcement_type: row.enforcement_type.parse().unwrap_or(EnforcementType::Warn),
            threshold: row.threshold,
            enabled: row.enabled,
            created_at: row.created_at,
        }
    }
}

#[derive(sqlx::FromRow)]
struct EnforcementStatsRow {
    total_enforcements: i64,
    blocked_count: i64,
    warned_count: i64,
    audited_count: i64,
}

#[derive(sqlx::FromRow)]
struct EnforcementTrendRow {
    date: chrono::NaiveDate,
    enforcement_count: i64,
    blocked_count: i64,
    warned_count: i64,
    audited_count: i64,
}

impl From<EnforcementTrendRow> for EnforcementTrend {
    fn from(row: EnforcementTrendRow) -> Self {
        Self {
            date: row.date,
            enforcement_count: row.enforcement_count,
            blocked_count: row.blocked_count,
            warned_count: row.warned_count,
            audited_count: row.audited_count,
        }
    }
}

#[derive(sqlx::FromRow)]
struct MetricV3Row {
    id: Uuid,
    repo_id: Uuid,
    file_path: String,
    metric_name: String,
    metric_value: f64,
    measured_at: chrono::DateTime<Utc>,
}

impl From<MetricV3Row> for CodeQualityMetricV3 {
    fn from(row: MetricV3Row) -> Self {
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
struct MetricSummaryV3Row {
    metric_name: String,
    latest_value: f64,
    avg_value: f64,
    min_value: f64,
    max_value: f64,
    measurement_count: i64,
    files_affected: i64,
}

#[derive(sqlx::FromRow)]
struct ThresholdV2Row {
    id: Uuid,
    repo_id: Uuid,
    metric_name: String,
    threshold_value: f64,
    enabled: bool,
    created_at: chrono::DateTime<Utc>,
}

impl From<ThresholdV2Row> for CodeQualityThresholdV2 {
    fn from(row: ThresholdV2Row) -> Self {
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
struct MetricV5Row {
    id: Uuid,
    repo_id: Uuid,
    file_path: String,
    metric_name: String,
    metric_value: f64,
    measured_at: chrono::DateTime<Utc>,
}

impl From<MetricV5Row> for CodeQualityMetricV5 {
    fn from(row: MetricV5Row) -> Self {
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
struct MetricSummaryV5Row {
    metric_name: String,
    latest_value: f64,
    avg_value: f64,
    min_value: f64,
    max_value: f64,
    measurement_count: i64,
    files_affected: i64,
}

#[derive(sqlx::FromRow)]
struct ThresholdV4Row {
    id: Uuid,
    repo_id: Uuid,
    metric_name: String,
    threshold_value: f64,
    enabled: bool,
    created_at: chrono::DateTime<Utc>,
}

impl From<ThresholdV4Row> for CodeQualityThresholdV4 {
    fn from(row: ThresholdV4Row) -> Self {
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
struct MetricV9Row {
    id: Uuid,
    repo_id: Uuid,
    file_path: String,
    metric_name: String,
    metric_value: f64,
    measured_at: chrono::DateTime<Utc>,
}

impl From<MetricV9Row> for CodeQualityMetricV9 {
    fn from(row: MetricV9Row) -> Self {
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
struct ThresholdV8Row {
    id: Uuid,
    repo_id: Uuid,
    metric_name: String,
    threshold_value: f64,
    enabled: bool,
    created_at: chrono::DateTime<Utc>,
}

impl From<ThresholdV8Row> for CodeQualityThresholdV8 {
    fn from(row: ThresholdV8Row) -> Self {
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
struct MetricSummaryV9Row {
    metric_name: String,
    latest_value: f64,
    avg_value: f64,
    min_value: f64,
    max_value: f64,
    measurement_count: i64,
    files_affected: i64,
}

#[derive(sqlx::FromRow)]
struct MetricV16Row {
    id: Uuid,
    repo_id: Uuid,
    file_path: String,
    metric_name: String,
    metric_value: f64,
    measured_at: chrono::DateTime<Utc>,
}

impl From<MetricV16Row> for CodeQualityMetricV16 {
    fn from(row: MetricV16Row) -> Self {
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
struct ThresholdV15Row {
    id: Uuid,
    repo_id: Uuid,
    metric_name: String,
    threshold_value: f64,
    enabled: bool,
    created_at: chrono::DateTime<Utc>,
}

impl From<ThresholdV15Row> for CodeQualityThresholdV15 {
    fn from(row: ThresholdV15Row) -> Self {
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
struct MetricSummaryV16Row {
    metric_name: String,
    latest_value: f64,
    avg_value: f64,
    min_value: f64,
    max_value: f64,
    measurement_count: i64,
    files_affected: i64,
}

#[derive(sqlx::FromRow)]
struct MetricV18Row {
    id: Uuid,
    repo_id: Uuid,
    file_path: String,
    metric_name: String,
    metric_value: f64,
    measured_at: chrono::DateTime<Utc>,
}

impl From<MetricV18Row> for CodeQualityMetricV18 {
    fn from(row: MetricV18Row) -> Self {
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
struct ThresholdV18Row {
    id: Uuid,
    repo_id: Uuid,
    metric_name: String,
    threshold_value: f64,
    enabled: bool,
    created_at: chrono::DateTime<Utc>,
}

impl From<ThresholdV18Row> for CodeQualityThresholdV18 {
    fn from(row: ThresholdV18Row) -> Self {
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
struct MetricSummaryV18Row {
    metric_name: String,
    latest_value: f64,
    avg_value: f64,
    min_value: f64,
    max_value: f64,
    measurement_count: i64,
    files_affected: i64,
}

#[derive(sqlx::FromRow)]
struct RuleV19Row {
    id: Uuid,
    name: String,
    description: String,
    rule_type: String,
    severity: String,
    enabled: bool,
    rule_config: serde_json::Value,
    created_at: chrono::DateTime<Utc>,
}

impl From<RuleV19Row> for CodeQualityRuleV19 {
    fn from(row: RuleV19Row) -> Self {
        Self {
            id: row.id,
            name: row.name,
            description: row.description,
            rule_type: row.rule_type,
            severity: row.severity,
            enabled: row.enabled,
            rule_config: row.rule_config,
            created_at: row.created_at,
        }
    }
}

#[derive(sqlx::FromRow)]
struct RuleUsageV19Row {
    id: Uuid,
    rule_id: Uuid,
    repo_id: Uuid,
    trigger_count: i32,
    last_triggered_at: Option<chrono::DateTime<Utc>>,
}

impl From<RuleUsageV19Row> for CodeQualityRuleUsageV19 {
    fn from(row: RuleUsageV19Row) -> Self {
        Self {
            id: row.id,
            rule_id: row.rule_id,
            repo_id: row.repo_id,
            trigger_count: row.trigger_count,
            last_triggered_at: row.last_triggered_at,
        }
    }
}

#[derive(sqlx::FromRow)]
struct RuleUsageStatsRow {
    total_rules: i64,
    active_rules: i64,
    total_triggers: i64,
}

#[derive(sqlx::FromRow)]
struct RuleEffectivenessStatsRow {
    total_enforcements: i64,
    total_violations: i64,
}
