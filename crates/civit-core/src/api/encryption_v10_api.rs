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
pub struct KeyVersionV5 {
    pub id: Uuid,
    pub key_id: Uuid,
    pub version: i32,
    pub key_material: Vec<u8>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ComplianceCheckV5 {
    pub id: Uuid,
    pub check_type: String,
    pub status: String,
    pub findings: serde_json::Value,
    pub score: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateKeyVersionRequest {
    pub key_id: Uuid,
    pub version: i32,
    pub key_material: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateComplianceCheckRequest {
    pub check_type: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateComplianceCheckRequest {
    pub status: String,
    pub findings: serde_json::Value,
    pub score: i32,
}

#[derive(Debug, Clone, Serialize)]
pub struct KeyVersionResponseV5 {
    pub id: Uuid,
    pub key_id: Uuid,
    pub version: i32,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ComplianceCheckResponseV5 {
    pub id: Uuid,
    pub check_type: String,
    pub status: String,
    pub findings: serde_json::Value,
    pub score: i32,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct KeyLifecycleResponseV5 {
    pub key_id: Uuid,
    pub key_name: String,
    pub algorithm: String,
    pub enabled: bool,
    pub created_at: String,
    pub rotation_date: Option<String>,
    pub days_since_creation: i64,
    pub days_since_rotation: Option<i64>,
    pub needs_rotation: bool,
    pub compliance_score: i32,
    pub latest_version: Option<i32>,
}

impl From<KeyVersionV5> for KeyVersionResponseV5 {
    fn from(v: KeyVersionV5) -> Self {
        Self {
            id: v.id,
            key_id: v.key_id,
            version: v.version,
            created_at: v.created_at.to_rfc3339(),
        }
    }
}

impl From<ComplianceCheckV5> for ComplianceCheckResponseV5 {
    fn from(c: ComplianceCheckV5) -> Self {
        Self {
            id: c.id,
            check_type: c.check_type,
            status: c.status,
            findings: c.findings,
            score: c.score,
            created_at: c.created_at.to_rfc3339(),
        }
    }
}

async fn create_key_version(
    State(state): State<AppState>,
    Json(input): Json<CreateKeyVersionRequest>,
) -> Result<(StatusCode, Json<KeyVersionResponseV5>), Response> {
    let key_material = input.key_material.as_bytes();
    let version = sqlx::query_as::<_, KeyVersionV5>(
        r#"INSERT INTO encryption_key_versions_v13 (key_id, version, key_material)
         VALUES ($1, $2, $3)
         RETURNING id, key_id, version, key_material, created_at"#,
    )
    .bind(input.key_id)
    .bind(input.version)
    .bind(key_material)
    .fetch_one(state.db.pool())
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response())?;

    Ok((StatusCode::CREATED, Json(version.into())))
}

async fn get_key_versions(
    State(state): State<AppState>,
    Path(key_id): Path<Uuid>,
) -> Result<Json<Vec<KeyVersionResponseV5>>, Response> {
    let versions = sqlx::query_as::<_, KeyVersionV5>(
        r#"SELECT id, key_id, version, key_material, created_at
         FROM encryption_key_versions_v13
         WHERE key_id = $1
         ORDER BY version DESC"#,
    )
    .bind(key_id)
    .fetch_all(state.db.pool())
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response())?;

    Ok(Json(versions.into_iter().map(|v| v.into()).collect()))
}

async fn create_compliance_check(
    State(state): State<AppState>,
    Json(input): Json<CreateComplianceCheckRequest>,
) -> Result<(StatusCode, Json<ComplianceCheckResponseV5>), Response> {
    let check = sqlx::query_as::<_, ComplianceCheckV5>(
        r#"INSERT INTO encryption_compliance_checks_v13 (check_type)
         VALUES ($1)
         RETURNING id, check_type, status, findings, score, created_at"#,
    )
    .bind(&input.check_type)
    .fetch_one(state.db.pool())
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response())?;

    Ok((StatusCode::CREATED, Json(check.into())))
}

async fn update_compliance_check(
    State(state): State<AppState>,
    Path(check_id): Path<Uuid>,
    Json(input): Json<UpdateComplianceCheckRequest>,
) -> Result<Json<ComplianceCheckResponseV5>, Response> {
    let check = sqlx::query_as::<_, ComplianceCheckV5>(
        r#"UPDATE encryption_compliance_checks_v13
         SET status = $2, findings = $3, score = $4
         WHERE id = $1
         RETURNING id, check_type, status, findings, score, created_at"#,
    )
    .bind(check_id)
    .bind(&input.status)
    .bind(&input.findings)
    .bind(input.score)
    .fetch_optional(state.db.pool())
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response())?
    .ok_or_else(|| (StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "check not found"}))).into_response())?;

    Ok(Json(check.into()))
}

async fn get_compliance_checks(
    State(state): State<AppState>,
) -> Result<Json<Vec<ComplianceCheckResponseV5>>, Response> {
    let checks = sqlx::query_as::<_, ComplianceCheckV5>(
        r#"SELECT id, check_type, status, findings, score, created_at
         FROM encryption_compliance_checks_v13
         ORDER BY created_at DESC"#,
    )
    .fetch_all(state.db.pool())
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response())?;

    Ok(Json(checks.into_iter().map(|c| c.into()).collect()))
}

async fn get_key_lifecycle(
    State(state): State<AppState>,
    Path(key_id): Path<Uuid>,
) -> Result<Json<KeyLifecycleResponseV5>, Response> {
    #[derive(sqlx::FromRow)]
    struct KeyInfo {
        id: Uuid,
        name: String,
        algorithm: String,
        enabled: bool,
        created_at: DateTime<Utc>,
        rotation_date: Option<DateTime<Utc>>,
    }

    let key = sqlx::query_as::<_, KeyInfo>(
        r#"SELECT id, name, algorithm, enabled, created_at, rotation_date
         FROM encryption_keys WHERE id = $1"#,
    )
    .bind(key_id)
    .fetch_optional(state.db.pool())
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response())?
    .ok_or_else(|| (StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "key not found"}))).into_response())?;

    let versions_count: (i64,) = sqlx::query_as(
        r#"SELECT COUNT(*) FROM encryption_key_versions_v13 WHERE key_id = $1"#,
    )
    .bind(key_id)
    .fetch_one(state.db.pool())
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response())?;

    let compliance_score: (i32,) = sqlx::query_as(
        r#"SELECT COALESCE(AVG(score), 0) FROM encryption_compliance_checks_v13"#,
    )
    .fetch_one(state.db.pool())
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response())?;

    let latest_version: Option<(Option<i32>,)> = sqlx::query_as(
        r#"SELECT MAX(version) FROM encryption_key_versions_v13 WHERE key_id = $1"#,
    )
    .bind(key_id)
    .fetch_optional(state.db.pool())
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response())?;

    let now = Utc::now();
    let days_since_creation = (now - key.created_at).num_days();
    let days_since_rotation = key.rotation_date.map(|r| (now - r).num_days());
    let needs_rotation = days_since_rotation.unwrap_or(days_since_creation) > 90;

    Ok(Json(KeyLifecycleResponseV5 {
        key_id: key.id,
        key_name: key.name,
        algorithm: key.algorithm,
        enabled: key.enabled,
        created_at: key.created_at.to_rfc3339(),
        rotation_date: key.rotation_date.map(|r| r.to_rfc3339()),
        days_since_creation,
        days_since_rotation,
        needs_rotation,
        compliance_score: compliance_score.0,
        latest_version: latest_version.and_then(|v| v.0),
    }))
}

async fn log_audit_event(
    State(state): State<AppState>,
    Json(input): Json<serde_json::Value>,
) -> Result<(StatusCode, Json<serde_json::Value>), Response> {
    let key_id: Uuid = input.get("key_id")
        .and_then(|v| v.as_str())
        .and_then(|s| s.parse().ok())
        .ok_or_else(|| (StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": "invalid key_id"}))).into_response())?;
    let action = input.get("action")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");
    let details = input.get("details").cloned().unwrap_or(serde_json::json!({}));

    let log = sqlx::query_as::<_, (Uuid, Uuid, String, serde_json::Value, DateTime<Utc>)>(
        r#"INSERT INTO encryption_audit_logs (key_id, action, details)
         VALUES ($1, $2, $3)
         RETURNING id, key_id, action, details, created_at"#,
    )
    .bind(key_id)
    .bind(action)
    .bind(&details)
    .fetch_one(state.db.pool())
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response())?;

    Ok((StatusCode::CREATED, Json(serde_json::json!({
        "id": log.0,
        "key_id": log.1,
        "action": log.2,
        "details": log.3,
        "created_at": log.4.to_rfc3339(),
    }))))
}

async fn list_audit_logs(
    State(state): State<AppState>,
) -> Result<Json<Vec<serde_json::Value>>, Response> {
    let logs = sqlx::query_as::<_, (Uuid, Uuid, String, serde_json::Value, DateTime<Utc>)>(
        r#"SELECT id, key_id, action, details, created_at
         FROM encryption_audit_logs ORDER BY created_at DESC LIMIT 100"#,
    )
    .fetch_all(state.db.pool())
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response())?;

    let result: Vec<serde_json::Value> = logs
        .into_iter()
        .map(|l| {
            serde_json::json!({
                "id": l.0,
                "key_id": l.1,
                "action": l.2,
                "details": l.3,
                "created_at": l.4.to_rfc3339(),
            })
        })
        .collect();

    Ok(Json(result))
}

pub fn encryption_v10_routes() -> Router<AppState> {
    Router::new()
        .route(
            "/api/v1/encryption-v10/key-versions",
            post(create_key_version),
        )
        .route(
            "/api/v1/encryption-v10/key-versions/{key_id}",
            get(get_key_versions),
        )
        .route(
            "/api/v1/encryption-v10/compliance-checks",
            post(create_compliance_check).get(get_compliance_checks),
        )
        .route(
            "/api/v1/encryption-v10/compliance-checks/{check_id}",
            patch(update_compliance_check),
        )
        .route(
            "/api/v1/encryption-v10/key-lifecycle/{key_id}",
            get(get_key_lifecycle),
        )
        .route(
            "/api/v1/encryption-v10/audit-log",
            post(log_audit_event).get(list_audit_logs),
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_version_response_conversion() {
        let version = KeyVersionV5 {
            id: Uuid::nil(),
            key_id: Uuid::nil(),
            version: 1,
            key_material: vec![1, 2, 3],
            created_at: Utc::now(),
        };
        let response: KeyVersionResponseV5 = version.into();
        assert_eq!(response.version, 1);
    }

    #[test]
    fn test_compliance_check_response_conversion() {
        let check = ComplianceCheckV5 {
            id: Uuid::nil(),
            check_type: "key_strength".to_string(),
            status: "completed".to_string(),
            findings: serde_json::json!({"result": "pass"}),
            score: 95,
            created_at: Utc::now(),
        };
        let response: ComplianceCheckResponseV5 = check.into();
        assert_eq!(response.score, 95);
        assert_eq!(response.status, "completed");
    }
}
