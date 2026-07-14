use super::types::*;
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

pub struct QualityGateStore {
    pool: PgPool,
}

impl QualityGateStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create_gate(
        &self,
        repo_id: Uuid,
        req: CreateQualityGateRequest,
    ) -> Result<QualityGate, sqlx::Error> {
        let id = Uuid::new_v4();
        let now = Utc::now();
        let conditions = serde_json::to_value(&req.conditions)
            .unwrap_or(serde_json::json!([]));
        let actions = serde_json::to_value(&req.actions)
            .unwrap_or(serde_json::json!([]));

        sqlx::query(
            r#"INSERT INTO quality_gates (id, repo_id, name, conditions, actions, enabled, created_at)
               VALUES ($1, $2, $3, $4, $5, $6, $7)"#,
        )
        .bind(id)
        .bind(repo_id)
        .bind(&req.name)
        .bind(&conditions)
        .bind(&actions)
        .bind(req.enabled.unwrap_or(true))
        .bind(now)
        .execute(&self.pool)
        .await?;

        Ok(QualityGate {
            id,
            repo_id,
            name: req.name,
            conditions: serde_json::from_value(conditions).unwrap_or_default(),
            actions: serde_json::from_value(actions).unwrap_or_default(),
            enabled: req.enabled.unwrap_or(true),
            created_at: now,
        })
    }

    pub async fn get_gate(&self, id: Uuid) -> Result<Option<QualityGate>, sqlx::Error> {
        let row = sqlx::query_as::<_, GateRow>(
            r#"SELECT id, repo_id, name, conditions, actions, enabled, created_at
               FROM quality_gates WHERE id = $1"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(QualityGate::from))
    }

    pub async fn list_gates(
        &self,
        repo_id: Uuid,
        enabled_only: bool,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<QualityGate>, sqlx::Error> {
        let rows = sqlx::query_as::<_, GateRow>(
            r#"SELECT id, repo_id, name, conditions, actions, enabled, created_at
               FROM quality_gates
               WHERE repo_id = $1
                 AND ($2 = false OR enabled = true)
               ORDER BY created_at DESC
               LIMIT $3 OFFSET $4"#,
        )
        .bind(repo_id)
        .bind(enabled_only)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(QualityGate::from).collect())
    }

    pub async fn update_gate(
        &self,
        id: Uuid,
        req: UpdateQualityGateRequest,
    ) -> Result<QualityGate, sqlx::Error> {
        let conditions_val = req
            .conditions
            .map(|c| serde_json::to_value(c).unwrap_or(serde_json::json!([])));
        let actions_val = req
            .actions
            .map(|a| serde_json::to_value(a).unwrap_or(serde_json::json!([])));

        let row = sqlx::query_as::<_, GateRow>(
            r#"UPDATE quality_gates SET
               name = COALESCE($2, name),
               conditions = COALESCE($3, conditions),
               actions = COALESCE($4, actions),
               enabled = COALESCE($5, enabled)
               WHERE id = $1
               RETURNING id, repo_id, name, conditions, actions, enabled, created_at"#,
        )
        .bind(id)
        .bind(&req.name)
        .bind(conditions_val)
        .bind(actions_val)
        .bind(req.enabled)
        .fetch_one(&self.pool)
        .await?;

        Ok(QualityGate::from(row))
    }

    pub async fn delete_gate(&self, id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM quality_gates WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn record_result(
        &self,
        gate_id: Uuid,
        pr_id: Option<Uuid>,
        status: &str,
        findings: Vec<QualityGateFinding>,
    ) -> Result<QualityGateResult, sqlx::Error> {
        let id = Uuid::new_v4();
        let now = Utc::now();
        let findings_json = serde_json::to_value(&findings)
            .unwrap_or(serde_json::json!([]));

        sqlx::query(
            r#"INSERT INTO quality_gate_results (id, gate_id, pr_id, status, findings, created_at)
               VALUES ($1, $2, $3, $4, $5, $6)"#,
        )
        .bind(id)
        .bind(gate_id)
        .bind(pr_id)
        .bind(status)
        .bind(&findings_json)
        .bind(now)
        .execute(&self.pool)
        .await?;

        Ok(QualityGateResult {
            id,
            gate_id,
            pr_id,
            status: status.to_string(),
            findings,
            created_at: now,
        })
    }

    pub async fn check_gate(
        &self,
        gate_id: Uuid,
        context: &serde_json::Value,
    ) -> Result<GateCheckResult, sqlx::Error> {
        let gate = self
            .get_gate(gate_id)
            .await?
            .ok_or_else(|| sqlx::Error::RowNotFound)?;

        let mut conditions_checked = 0;
        let mut conditions_passed = 0;
        let mut conditions_failed = 0;
        let findings = Vec::new();

        for condition in &gate.conditions {
            conditions_checked += 1;
            let passed = match condition {
                QualityGateCondition::MinTestPassRate { threshold } => {
                    context
                        .get("test_pass_rate")
                        .and_then(|v| v.as_f64())
                        .map(|rate| rate >= *threshold)
                        .unwrap_or(false)
                }
                QualityGateCondition::MaxCriticalFindings { threshold } => {
                    context
                        .get("critical_findings")
                        .and_then(|v| v.as_i64())
                        .map(|count| count <= *threshold as i64)
                        .unwrap_or(true)
                }
                QualityGateCondition::MaxHighFindings { threshold } => {
                    context
                        .get("high_findings")
                        .and_then(|v| v.as_i64())
                        .map(|count| count <= *threshold as i64)
                        .unwrap_or(true)
                }
                QualityGateCondition::MinCodeCoverage { threshold } => {
                    context
                        .get("code_coverage")
                        .and_then(|v| v.as_f64())
                        .map(|cov| cov >= *threshold)
                        .unwrap_or(false)
                }
                QualityGateCondition::NoFailingTests => {
                    context
                        .get("failing_tests")
                        .and_then(|v| v.as_i64())
                        .map(|count| count == 0)
                        .unwrap_or(true)
                }
                QualityGateCondition::LintClean => {
                    context
                        .get("lint_errors")
                        .and_then(|v| v.as_i64())
                        .map(|count| count == 0)
                        .unwrap_or(true)
                }
                QualityGateCondition::SecurityScanPass => {
                    context
                        .get("security_vulnerabilities")
                        .and_then(|v| v.as_i64())
                        .map(|count| count == 0)
                        .unwrap_or(true)
                }
            };

            if passed {
                conditions_passed += 1;
            } else {
                conditions_failed += 1;
            }
        }

        let passed = conditions_failed == 0;

        Ok(GateCheckResult {
            gate_id,
            gate_name: gate.name,
            passed,
            conditions_checked,
            conditions_passed,
            conditions_failed,
            findings,
        })
    }

    pub async fn enforce_gates_for_pr(
        &self,
        repo_id: Uuid,
        pr_id: Uuid,
        context: &serde_json::Value,
    ) -> Result<GateEnforcementResult, sqlx::Error> {
        let gates = self.list_gates(repo_id, true, 100, 0).await?;
        let total_gates = gates.len() as i64;
        let mut gates_passed: i64 = 0;
        let mut gates_failed: i64 = 0;
        let mut gate_results = Vec::new();

        for gate in gates {
            let result = self.check_gate(gate.id, context).await?;
            if result.passed {
                gates_passed += 1;
            } else {
                gates_failed += 1;
            }
            gate_results.push(result);
        }

        let can_merge = gates_failed == 0;

        Ok(GateEnforcementResult {
            pr_id,
            total_gates,
            gates_passed,
            gates_failed,
            can_merge,
            gate_results,
        })
    }

    pub async fn get_results_for_pr(
        &self,
        pr_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<QualityGateResult>, sqlx::Error> {
        let rows = sqlx::query_as::<_, ResultRow>(
            r#"SELECT id, gate_id, pr_id, status, findings, created_at
               FROM quality_gate_results
               WHERE pr_id = $1
               ORDER BY created_at DESC
               LIMIT $2 OFFSET $3"#,
        )
        .bind(pr_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(QualityGateResult::from).collect())
    }
}

#[derive(sqlx::FromRow)]
struct GateRow {
    id: Uuid,
    repo_id: Uuid,
    name: String,
    conditions: serde_json::Value,
    actions: serde_json::Value,
    enabled: bool,
    created_at: chrono::DateTime<Utc>,
}

impl From<GateRow> for QualityGate {
    fn from(row: GateRow) -> Self {
        Self {
            id: row.id,
            repo_id: row.repo_id,
            name: row.name,
            conditions: serde_json::from_value(row.conditions).unwrap_or_default(),
            actions: serde_json::from_value(row.actions).unwrap_or_default(),
            enabled: row.enabled,
            created_at: row.created_at,
        }
    }
}

#[derive(sqlx::FromRow)]
struct ResultRow {
    id: Uuid,
    gate_id: Uuid,
    pr_id: Option<Uuid>,
    status: String,
    findings: serde_json::Value,
    created_at: chrono::DateTime<Utc>,
}

impl From<ResultRow> for QualityGateResult {
    fn from(row: ResultRow) -> Self {
        Self {
            id: row.id,
            gate_id: row.gate_id,
            pr_id: row.pr_id,
            status: row.status,
            findings: serde_json::from_value(row.findings).unwrap_or_default(),
            created_at: row.created_at,
        }
    }
}
