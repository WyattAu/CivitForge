#![forbid(unsafe_code)]

use crate::api::AppState;
use axum::{
    Router,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, patch, post},
    Json,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ResidencyAudit {
    pub id: Uuid,
    pub rule_id: Uuid,
    pub audit_type: String,
    pub findings: serde_json::Value,
    pub score: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ResidencyMigration {
    pub id: Uuid,
    pub violation_id: Uuid,
    pub target_region: String,
    pub status: String,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateAuditRequest {
    pub rule_id: Uuid,
    pub audit_type: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateMigrationRequest {
    pub violation_id: Uuid,
    pub target_region: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ResidencyAuditResponse {
    pub id: Uuid,
    pub rule_id: Uuid,
    pub audit_type: String,
    pub findings: serde_json::Value,
    pub score: i32,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ResidencyMigrationResponse {
    pub id: Uuid,
    pub violation_id: Uuid,
    pub target_region: String,
    pub status: String,
    pub started_at: String,
    pub completed_at: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ResidencyComplianceResponse {
    pub total_rules: i64,
    pub enabled_rules: i64,
    pub total_violations: i64,
    pub resolved_violations: i64,
    pub average_score: f64,
    pub compliance_percentage: f64,
}

impl From<ResidencyAudit> for ResidencyAuditResponse {
    fn from(audit: ResidencyAudit) -> Self {
        Self {
            id: audit.id,
            rule_id: audit.rule_id,
            audit_type: audit.audit_type,
            findings: audit.findings,
            score: audit.score,
            created_at: audit.created_at.to_rfc3339(),
        }
    }
}

impl From<ResidencyMigration> for ResidencyMigrationResponse {
    fn from(migration: ResidencyMigration) -> Self {
        Self {
            id: migration.id,
            violation_id: migration.violation_id,
            target_region: migration.target_region,
            status: migration.status,
            started_at: migration.started_at.to_rfc3339(),
            completed_at: migration.completed_at.map(|c| c.to_rfc3339()),
        }
    }
}

async fn create_residency_audit(
    State(state): State<AppState>,
    Json(input): Json<CreateAuditRequest>,
) -> Result<(StatusCode, Json<ResidencyAuditResponse>), Response> {
    let audit = sqlx::query_as::<_, ResidencyAudit>(
        r#"INSERT INTO data_residency_audits (rule_id, audit_type)
         VALUES ($1, $2)
         RETURNING id, rule_id, audit_type, findings, score, created_at"#,
    )
    .bind(input.rule_id)
    .bind(&input.audit_type)
    .fetch_one(state.db.pool())
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response())?;

    Ok((StatusCode::CREATED, Json(audit.into())))
}

async fn list_residency_audits(
    State(state): State<AppState>,
) -> Result<Json<Vec<ResidencyAuditResponse>>, Response> {
    let audits = sqlx::query_as::<_, ResidencyAudit>(
        r#"SELECT id, rule_id, audit_type, findings, score, created_at
         FROM data_residency_audits ORDER BY created_at DESC LIMIT 100"#,
    )
    .fetch_all(state.db.pool())
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response())?;

    Ok(Json(audits.into_iter().map(|a| a.into()).collect()))
}

async fn update_residency_audit_findings(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(input): Json<serde_json::Value>,
) -> Result<Json<ResidencyAuditResponse>, Response> {
    let findings = input.get("findings").cloned().unwrap_or(serde_json::json!([]));
    let score = input.get("score").and_then(|v| v.as_i64()).unwrap_or(0) as i32;

    let audit = sqlx::query_as::<_, ResidencyAudit>(
        r#"UPDATE data_residency_audits SET findings = $2, score = $3 WHERE id = $1
         RETURNING id, rule_id, audit_type, findings, score, created_at"#,
    )
    .bind(id)
    .bind(findings)
    .bind(score)
    .fetch_optional(state.db.pool())
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response())?
    .ok_or_else(|| (StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "audit not found"}))).into_response())?;

    Ok(Json(audit.into()))
}

async fn create_residency_migration(
    State(state): State<AppState>,
    Json(input): Json<CreateMigrationRequest>,
) -> Result<(StatusCode, Json<ResidencyMigrationResponse>), Response> {
    let migration = sqlx::query_as::<_, ResidencyMigration>(
        r#"INSERT INTO data_residency_migrations (violation_id, target_region)
         VALUES ($1, $2)
         RETURNING id, violation_id, target_region, status, started_at, completed_at"#,
    )
    .bind(input.violation_id)
    .bind(&input.target_region)
    .fetch_one(state.db.pool())
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response())?;

    Ok((StatusCode::CREATED, Json(migration.into())))
}

async fn list_residency_migrations(
    State(state): State<AppState>,
) -> Result<Json<Vec<ResidencyMigrationResponse>>, Response> {
    let migrations = sqlx::query_as::<_, ResidencyMigration>(
        r#"SELECT id, violation_id, target_region, status, started_at, completed_at
         FROM data_residency_migrations ORDER BY started_at DESC LIMIT 100"#,
    )
    .fetch_all(state.db.pool())
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response())?;

    Ok(Json(migrations.into_iter().map(|m| m.into()).collect()))
}

async fn complete_residency_migration(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<ResidencyMigrationResponse>, Response> {
    let migration = sqlx::query_as::<_, ResidencyMigration>(
        r#"UPDATE data_residency_migrations SET status = 'completed', completed_at = NOW()
         WHERE id = $1
         RETURNING id, violation_id, target_region, status, started_at, completed_at"#,
    )
    .bind(id)
    .fetch_optional(state.db.pool())
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response())?
    .ok_or_else(|| (StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "migration not found"}))).into_response())?;

    Ok(Json(migration.into()))
}

async fn get_residency_compliance(
    State(state): State<AppState>,
) -> Result<Json<ResidencyComplianceResponse>, Response> {
    let total_rules = sqlx::query_scalar::<_, i64>(
        r#"SELECT COUNT(*) FROM data_residency_rules"#,
    )
    .fetch_one(state.db.pool())
    .await
    .unwrap_or(0);

    let enabled_rules = sqlx::query_scalar::<_, i64>(
        r#"SELECT COUNT(*) FROM data_residency_rules WHERE enabled = true"#,
    )
    .fetch_one(state.db.pool())
    .await
    .unwrap_or(0);

    let total_violations = sqlx::query_scalar::<_, i64>(
        r#"SELECT COUNT(*) FROM data_residency_violations"#,
    )
    .fetch_one(state.db.pool())
    .await
    .unwrap_or(0);

    let resolved_violations = sqlx::query_scalar::<_, i64>(
        r#"SELECT COUNT(*) FROM data_residency_migrations WHERE status = 'completed'"#,
    )
    .fetch_one(state.db.pool())
    .await
    .unwrap_or(0);

    let average_score = sqlx::query_scalar::<_, f64>(
        r#"SELECT COALESCE(AVG(score), 0.0) FROM data_residency_audits"#,
    )
    .fetch_one(state.db.pool())
    .await
    .unwrap_or(0.0);

    let compliance_percentage = if total_rules > 0 {
        ((enabled_rules as f64 / total_rules as f64) * 100.0).min(100.0)
    } else {
        100.0
    };

    Ok(Json(ResidencyComplianceResponse {
        total_rules,
        enabled_rules,
        total_violations,
        resolved_violations,
        average_score,
        compliance_percentage,
    }))
}

pub fn data_residency_v2_routes() -> Router<AppState> {
    Router::new()
        .route(
            "/api/v1/data-residency/audits",
            post(create_residency_audit).get(list_residency_audits),
        )
        .route(
            "/api/v1/data-residency/audits/{id}",
            patch(update_residency_audit_findings),
        )
        .route(
            "/api/v1/data-residency/migrations",
            post(create_residency_migration).get(list_residency_migrations),
        )
        .route(
            "/api/v1/data-residency/migrations/{id}/complete",
            post(complete_residency_migration),
        )
        .route(
            "/api/v1/data-residency/compliance",
            get(get_residency_compliance),
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_residency_audit_response_conversion() {
        let audit = ResidencyAudit {
            id: Uuid::nil(),
            rule_id: Uuid::nil(),
            audit_type: "compliance_check".to_string(),
            findings: serde_json::json!([]),
            score: 85,
            created_at: Utc::now(),
        };
        let response: ResidencyAuditResponse = audit.into();
        assert_eq!(response.score, 85);
        assert_eq!(response.audit_type, "compliance_check");
    }

    #[test]
    fn test_residency_migration_response_conversion() {
        let migration = ResidencyMigration {
            id: Uuid::nil(),
            violation_id: Uuid::nil(),
            target_region: "eu-west-1".to_string(),
            status: "pending".to_string(),
            started_at: Utc::now(),
            completed_at: None,
        };
        let response: ResidencyMigrationResponse = migration.into();
        assert_eq!(response.target_region, "eu-west-1");
        assert!(response.completed_at.is_none());
    }
}
