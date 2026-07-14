#![forbid(unsafe_code)]

use crate::api::AppState;
use axum::{
    Router,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct KeyRotation {
    pub id: Uuid,
    pub key_id: Uuid,
    pub old_key_id: Option<Uuid>,
    pub rotated_at: DateTime<Utc>,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct EncryptionAuditLog {
    pub id: Uuid,
    pub key_id: Uuid,
    pub action: String,
    pub user_id: Option<Uuid>,
    pub details: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RotateKeyRequest {
    pub key_id: Uuid,
    pub old_key_id: Option<Uuid>,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LogAuditRequest {
    pub key_id: Uuid,
    pub action: String,
    pub user_id: Option<Uuid>,
    pub details: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ComplianceReportRequest {
    pub key_id: Option<Uuid>,
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct KeyRotationResponse {
    pub id: Uuid,
    pub key_id: Uuid,
    pub old_key_id: Option<Uuid>,
    pub rotated_at: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct EncryptionAuditLogResponse {
    pub id: Uuid,
    pub key_id: Uuid,
    pub action: String,
    pub user_id: Option<Uuid>,
    pub details: serde_json::Value,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct KeyLifecycleResponse {
    pub key_id: Uuid,
    pub key_name: String,
    pub algorithm: String,
    pub enabled: bool,
    pub created_at: String,
    pub rotation_date: Option<String>,
    pub days_since_creation: i64,
    pub days_since_rotation: Option<i64>,
    pub needs_rotation: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct ComplianceReportResponse {
    pub total_keys: i64,
    pub active_keys: i64,
    pub rotated_last_30_days: i64,
    pub audit_events_last_30_days: i64,
    pub keys_needing_rotation: i64,
    pub compliance_score: f64,
}

impl From<KeyRotation> for KeyRotationResponse {
    fn from(r: KeyRotation) -> Self {
        Self {
            id: r.id,
            key_id: r.key_id,
            old_key_id: r.old_key_id,
            rotated_at: r.rotated_at.to_rfc3339(),
            reason: r.reason,
        }
    }
}

impl From<EncryptionAuditLog> for EncryptionAuditLogResponse {
    fn from(log: EncryptionAuditLog) -> Self {
        Self {
            id: log.id,
            key_id: log.key_id,
            action: log.action,
            user_id: log.user_id,
            details: log.details,
            created_at: log.created_at.to_rfc3339(),
        }
    }
}

async fn rotate_key(
    State(state): State<AppState>,
    Json(input): Json<RotateKeyRequest>,
) -> Result<(StatusCode, Json<KeyRotationResponse>), Response> {
    let rotation = sqlx::query_as::<_, KeyRotation>(
        r#"INSERT INTO encryption_key_rotations (key_id, old_key_id, reason)
         VALUES ($1, $2, $3)
         RETURNING id, key_id, old_key_id, rotated_at, reason"#,
    )
    .bind(input.key_id)
    .bind(input.old_key_id)
    .bind(input.reason.unwrap_or_else(|| "scheduled".to_string()))
    .fetch_one(state.db.pool())
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response())?;

    Ok((StatusCode::CREATED, Json(rotation.into())))
}

async fn list_key_rotations(
    State(state): State<AppState>,
) -> Result<Json<Vec<KeyRotationResponse>>, Response> {
    let rotations = sqlx::query_as::<_, KeyRotation>(
        r#"SELECT id, key_id, old_key_id, rotated_at, reason
         FROM encryption_key_rotations ORDER BY rotated_at DESC LIMIT 100"#,
    )
    .fetch_all(state.db.pool())
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response())?;

    Ok(Json(rotations.into_iter().map(|r| r.into()).collect()))
}

async fn log_audit_event(
    State(state): State<AppState>,
    Json(input): Json<LogAuditRequest>,
) -> Result<(StatusCode, Json<EncryptionAuditLogResponse>), Response> {
    let log = sqlx::query_as::<_, EncryptionAuditLog>(
        r#"INSERT INTO encryption_audit_logs (key_id, action, user_id, details)
         VALUES ($1, $2, $3, $4)
         RETURNING id, key_id, action, user_id, details, created_at"#,
    )
    .bind(input.key_id)
    .bind(&input.action)
    .bind(input.user_id)
    .bind(input.details.unwrap_or(serde_json::json!({})))
    .fetch_one(state.db.pool())
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response())?;

    Ok((StatusCode::CREATED, Json(log.into())))
}

async fn list_audit_logs(
    State(state): State<AppState>,
) -> Result<Json<Vec<EncryptionAuditLogResponse>>, Response> {
    let logs = sqlx::query_as::<_, EncryptionAuditLog>(
        r#"SELECT id, key_id, action, user_id, details, created_at
         FROM encryption_audit_logs ORDER BY created_at DESC LIMIT 100"#,
    )
    .fetch_all(state.db.pool())
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response())?;

    Ok(Json(logs.into_iter().map(|l| l.into()).collect()))
}

async fn get_key_lifecycle(
    State(state): State<AppState>,
    Path(key_id): Path<Uuid>,
) -> Result<Json<KeyLifecycleResponse>, Response> {
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

    let now = Utc::now();
    let days_since_creation = (now - key.created_at).num_days();
    let days_since_rotation = key.rotation_date.map(|r| (now - r).num_days());
    let needs_rotation = days_since_rotation.unwrap_or(days_since_creation) > 90;

    Ok(Json(KeyLifecycleResponse {
        key_id: key.id,
        key_name: key.name,
        algorithm: key.algorithm,
        enabled: key.enabled,
        created_at: key.created_at.to_rfc3339(),
        rotation_date: key.rotation_date.map(|r| r.to_rfc3339()),
        days_since_creation,
        days_since_rotation,
        needs_rotation,
    }))
}

async fn get_compliance_report(
    State(state): State<AppState>,
    Json(input): Json<ComplianceReportRequest>,
) -> Result<Json<ComplianceReportResponse>, Response> {
    let key_filter = input.key_id
        .map(|_| "WHERE key_id = $1")
        .unwrap_or("");

    let total_keys = sqlx::query_scalar::<_, i64>(
        r#"SELECT COUNT(*) FROM encryption_keys"#,
    )
    .fetch_one(state.db.pool())
    .await
    .unwrap_or(0);

    let active_keys = sqlx::query_scalar::<_, i64>(
        r#"SELECT COUNT(*) FROM encryption_keys WHERE enabled = true"#,
    )
    .fetch_one(state.db.pool())
    .await
    .unwrap_or(0);

    let rotated_last_30 = sqlx::query_scalar::<_, i64>(
        r#"SELECT COUNT(*) FROM encryption_key_rotations
         WHERE rotated_at > NOW() - INTERVAL '30 days'"#,
    )
    .fetch_one(state.db.pool())
    .await
    .unwrap_or(0);

    let audit_last_30 = sqlx::query_scalar::<_, i64>(
        r#"SELECT COUNT(*) FROM encryption_audit_logs
         WHERE created_at > NOW() - INTERVAL '30 days'"#,
    )
    .fetch_one(state.db.pool())
    .await
    .unwrap_or(0);

    let keys_needing_rotation = sqlx::query_scalar::<_, i64>(
        r#"SELECT COUNT(*) FROM encryption_keys
         WHERE enabled = true AND (
             rotation_date IS NULL AND created_at < NOW() - INTERVAL '90 days'
             OR rotation_date < NOW() - INTERVAL '90 days'
         )"#,
    )
    .fetch_one(state.db.pool())
    .await
    .unwrap_or(0);

    let compliance_score = if total_keys > 0 {
        let active_ratio = active_keys as f64 / total_keys as f64;
        let rotation_ratio = 1.0 - (keys_needing_rotation as f64 / total_keys as f64);
        ((active_ratio * 50.0) + (rotation_ratio * 50.0)).min(100.0)
    } else {
        100.0
    };

    Ok(Json(ComplianceReportResponse {
        total_keys,
        active_keys,
        rotated_last_30_days: rotated_last_30,
        audit_events_last_30_days: audit_last_30,
        keys_needing_rotation,
        compliance_score,
    }))
}

pub fn encryption_v3_routes() -> Router<AppState> {
    Router::new()
        .route(
            "/api/v1/encryption/rotate",
            post(rotate_key),
        )
        .route(
            "/api/v1/encryption/rotations",
            get(list_key_rotations),
        )
        .route(
            "/api/v1/encryption/audit-log",
            post(log_audit_event).get(list_audit_logs),
        )
        .route(
            "/api/v1/encryption/key-lifecycle/{key_id}",
            get(get_key_lifecycle),
        )
        .route(
            "/api/v1/encryption/compliance-report",
            post(get_compliance_report),
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_rotation_response_conversion() {
        let rotation = KeyRotation {
            id: Uuid::nil(),
            key_id: Uuid::nil(),
            old_key_id: None,
            rotated_at: Utc::now(),
            reason: "scheduled".to_string(),
        };
        let response: KeyRotationResponse = rotation.into();
        assert_eq!(response.reason, "scheduled");
    }

    #[test]
    fn test_audit_log_response_conversion() {
        let log = EncryptionAuditLog {
            id: Uuid::nil(),
            key_id: Uuid::nil(),
            action: "encrypt".to_string(),
            user_id: None,
            details: serde_json::json!({"test": true}),
            created_at: Utc::now(),
        };
        let response: EncryptionAuditLogResponse = log.into();
        assert_eq!(response.action, "encrypt");
    }
}
