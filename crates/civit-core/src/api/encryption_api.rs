#![forbid(unsafe_code)]

use crate::api::AppState;
use axum::{
    Router,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{delete, get, patch, post},
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
pub struct LogAuditRequestV3 {
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

async fn log_audit_event_v3(
    State(state): State<AppState>,
    Json(input): Json<LogAuditRequestV3>,
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

async fn list_audit_logs_v3(
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

async fn get_key_lifecycle_v3(
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
    let _key_filter = input.key_id
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

async fn get_key_lifecycle_v6(
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

    let _versions_count: (i64,) = sqlx::query_as(
        r#"SELECT COUNT(*) FROM encryption_key_versions_v13 WHERE key_id = $1"#,
    )
    .bind(key_id)
    .fetch_one(state.db.pool())
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response())?;

    let _compliance_score: (i32,) = sqlx::query_as(
        r#"SELECT COALESCE(AVG(score), 0) FROM encryption_compliance_checks_v13"#,
    )
    .fetch_one(state.db.pool())
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response())?;

    let _latest_version: Option<(Option<i32>,)> = sqlx::query_as(
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

async fn log_audit_event_v6(
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

async fn list_audit_logs_v6(
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

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct KeyAccessControlV21 {
    pub id: Uuid,
    pub key_id: Uuid,
    pub principal_type: String,
    pub principal_id: Uuid,
    pub permission: String,
    pub granted_by: Uuid,
    pub granted_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct EncryptionAuditLogV21 {
    pub id: Uuid,
    pub key_id: Uuid,
    pub operation: String,
    pub principal_id: Uuid,
    pub success: bool,
    pub ip_address: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GrantAccessRequest {
    pub key_id: Uuid,
    pub principal_type: String,
    pub principal_id: Uuid,
    pub granted_by: Uuid,
    pub permission: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LogAuditRequestV13 {
    pub key_id: Uuid,
    pub operation: String,
    pub principal_id: Uuid,
    pub success: bool,
    pub ip_address: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct KeyAccessControlResponseV21 {
    pub id: Uuid,
    pub key_id: Uuid,
    pub principal_type: String,
    pub principal_id: Uuid,
    pub permission: String,
    pub granted_by: Uuid,
    pub granted_at: String,
    pub expires_at: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct EncryptionAuditLogResponseV21 {
    pub id: Uuid,
    pub key_id: Uuid,
    pub operation: String,
    pub principal_id: Uuid,
    pub success: bool,
    pub ip_address: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct KeyUsageAnalyticsResponseV23 {
    pub key_id: Uuid,
    pub key_name: String,
    pub total_operations: i64,
    pub successful_operations: i64,
    pub failed_operations: i64,
    pub unique_principals: i64,
    pub operations_by_type: serde_json::Value,
    pub last_operation_at: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ComplianceReportResponseV23 {
    pub report_id: Uuid,
    pub report_type: String,
    pub total_keys: i64,
    pub keys_with_access_control: i64,
    pub keys_without_access_control: i64,
    pub expired_permissions: i64,
    pub total_audit_entries: i64,
    pub compliance_score: f64,
    pub generated_at: String,
    pub findings: serde_json::Value,
}

impl From<KeyAccessControlV21> for KeyAccessControlResponseV21 {
    fn from(a: KeyAccessControlV21) -> Self {
        Self {
            id: a.id,
            key_id: a.key_id,
            principal_type: a.principal_type,
            principal_id: a.principal_id,
            permission: a.permission,
            granted_by: a.granted_by,
            granted_at: a.granted_at.to_rfc3339(),
            expires_at: a.expires_at.map(|t| t.to_rfc3339()),
        }
    }
}

impl From<EncryptionAuditLogV21> for EncryptionAuditLogResponseV21 {
    fn from(a: EncryptionAuditLogV21) -> Self {
        Self {
            id: a.id,
            key_id: a.key_id,
            operation: a.operation,
            principal_id: a.principal_id,
            success: a.success,
            ip_address: a.ip_address,
            created_at: a.created_at.to_rfc3339(),
        }
    }
}

async fn grant_access(
    State(state): State<AppState>,
    Json(input): Json<GrantAccessRequest>,
) -> Result<(StatusCode, Json<KeyAccessControlResponseV21>), Response> {
    let permission = input.permission.unwrap_or_else(|| "use".to_string());

    let row = sqlx::query_as::<_, KeyAccessControlV21>(
        r#"INSERT INTO encryption_key_access_control_v21 (key_id, principal_type, principal_id, granted_by, permission, expires_at)
         VALUES ($1, $2, $3, $4, $5, $6)
         ON CONFLICT (key_id, principal_type, principal_id) DO UPDATE SET permission = $5, expires_at = $6
         RETURNING id, key_id, principal_type, principal_id, permission, granted_by, granted_at, expires_at"#,
    )
    .bind(input.key_id)
    .bind(&input.principal_type)
    .bind(input.principal_id)
    .bind(input.granted_by)
    .bind(&permission)
    .bind(input.expires_at)
    .fetch_one(state.db.pool())
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response())?;

    Ok((StatusCode::CREATED, Json(row.into())))
}

async fn revoke_access(
    State(state): State<AppState>,
    Path((key_id, principal_type, principal_id)): Path<(Uuid, String, Uuid)>,
) -> Result<StatusCode, Response> {
    let result = sqlx::query(
        r#"DELETE FROM encryption_key_access_control_v21
         WHERE key_id = $1 AND principal_type = $2 AND principal_id = $3"#,
    )
    .bind(key_id)
    .bind(&principal_type)
    .bind(principal_id)
    .execute(state.db.pool())
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response())?;

    if result.rows_affected() > 0 {
        Ok(StatusCode::OK)
    } else {
        Ok(StatusCode::NOT_FOUND)
    }
}

async fn get_access_control(
    State(state): State<AppState>,
    Path(key_id): Path<Uuid>,
) -> Result<Json<Vec<KeyAccessControlResponseV21>>, Response> {
    let rows = sqlx::query_as::<_, KeyAccessControlV21>(
        r#"SELECT id, key_id, principal_type, principal_id, permission, granted_by, granted_at, expires_at
         FROM encryption_key_access_control_v21
         WHERE key_id = $1
         ORDER BY granted_at DESC"#,
    )
    .bind(key_id)
    .fetch_all(state.db.pool())
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response())?;

    Ok(Json(rows.into_iter().map(|r| r.into()).collect()))
}

async fn log_audit_v13(
    State(state): State<AppState>,
    Json(input): Json<LogAuditRequestV13>,
) -> Result<(StatusCode, Json<EncryptionAuditLogResponseV21>), Response> {
    let row = sqlx::query_as::<_, EncryptionAuditLogV21>(
        r#"INSERT INTO encryption_audit_log_v21 (key_id, operation, principal_id, success, ip_address)
         VALUES ($1, $2, $3, $4, $5)
         RETURNING id, key_id, operation, principal_id, success, ip_address, created_at"#,
    )
    .bind(input.key_id)
    .bind(&input.operation)
    .bind(input.principal_id)
    .bind(input.success)
    .bind(&input.ip_address)
    .fetch_one(state.db.pool())
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response())?;

    Ok((StatusCode::CREATED, Json(row.into())))
}

async fn get_audit_logs_v13(
    State(state): State<AppState>,
    Path(key_id): Path<Uuid>,
) -> Result<Json<Vec<EncryptionAuditLogResponseV21>>, Response> {
    let rows = sqlx::query_as::<_, EncryptionAuditLogV21>(
        r#"SELECT id, key_id, operation, principal_id, success, ip_address, created_at
         FROM encryption_audit_log_v21
         WHERE key_id = $1
         ORDER BY created_at DESC
         LIMIT 100"#,
    )
    .bind(key_id)
    .fetch_all(state.db.pool())
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response())?;

    Ok(Json(rows.into_iter().map(|r| r.into()).collect()))
}

async fn get_key_usage_analytics(
    State(state): State<AppState>,
    Path(key_id): Path<Uuid>,
) -> Result<Json<KeyUsageAnalyticsResponseV23>, Response> {
    #[derive(sqlx::FromRow)]
    struct KeyInfo {
        id: Uuid,
        name: String,
    }

    let key = sqlx::query_as::<_, KeyInfo>(
        r#"SELECT id, name FROM encryption_keys WHERE id = $1"#,
    )
    .bind(key_id)
    .fetch_optional(state.db.pool())
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response())?
    .ok_or_else(|| (StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "key not found"}))).into_response())?;

    let total_ops: (i64,) = sqlx::query_as(
        r#"SELECT COUNT(*) FROM encryption_audit_log_v21 WHERE key_id = $1"#,
    )
    .bind(key_id)
    .fetch_one(state.db.pool())
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response())?;

    let successful_ops: (i64,) = sqlx::query_as(
        r#"SELECT COUNT(*) FROM encryption_audit_log_v21 WHERE key_id = $1 AND success = true"#,
    )
    .bind(key_id)
    .fetch_one(state.db.pool())
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response())?;

    let failed_ops: (i64,) = sqlx::query_as(
        r#"SELECT COUNT(*) FROM encryption_audit_log_v21 WHERE key_id = $1 AND success = false"#,
    )
    .bind(key_id)
    .fetch_one(state.db.pool())
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response())?;

    let unique_principals: (i64,) = sqlx::query_as(
        r#"SELECT COUNT(DISTINCT principal_id) FROM encryption_audit_log_v21 WHERE key_id = $1"#,
    )
    .bind(key_id)
    .fetch_one(state.db.pool())
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response())?;

    let last_op: Option<(Option<DateTime<Utc>>,)> = sqlx::query_as(
        r#"SELECT MAX(created_at) FROM encryption_audit_log_v21 WHERE key_id = $1"#,
    )
    .bind(key_id)
    .fetch_optional(state.db.pool())
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response())?;

    Ok(Json(KeyUsageAnalyticsResponseV23 {
        key_id: key.id,
        key_name: key.name,
        total_operations: total_ops.0,
        successful_operations: successful_ops.0,
        failed_operations: failed_ops.0,
        unique_principals: unique_principals.0,
        operations_by_type: serde_json::json!({}),
        last_operation_at: last_op.and_then(|l| l.0).map(|t| t.to_rfc3339()),
    }))
}

async fn generate_compliance_report_v13(
    State(state): State<AppState>,
) -> Result<Json<ComplianceReportResponseV23>, Response> {
    let total_keys: (i64,) = sqlx::query_as(
        r#"SELECT COUNT(*) FROM encryption_keys"#,
    )
    .fetch_one(state.db.pool())
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response())?;

    let keys_with_acl: (i64,) = sqlx::query_as(
        r#"SELECT COUNT(DISTINCT key_id) FROM encryption_key_access_control_v21"#,
    )
    .fetch_one(state.db.pool())
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response())?;

    let expired_permissions: (i64,) = sqlx::query_as(
        r#"SELECT COUNT(*) FROM encryption_key_access_control_v21
         WHERE expires_at IS NOT NULL AND expires_at < NOW()"#,
    )
    .fetch_one(state.db.pool())
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response())?;

    let total_audit_entries: (i64,) = sqlx::query_as(
        r#"SELECT COUNT(*) FROM encryption_audit_log_v21"#,
    )
    .fetch_one(state.db.pool())
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response())?;

    let keys_without_acl = total_keys.0 - keys_with_acl.0;
    let compliance_score = if total_keys.0 > 0 {
        let acl_coverage = (keys_with_acl.0 as f64 / total_keys.0 as f64) * 100.0;
        let expired_penalty = (expired_permissions.0 as f64 * 5.0).min(30.0);
        (acl_coverage - expired_penalty).max(0.0)
    } else {
        100.0
    };

    Ok(Json(ComplianceReportResponseV23 {
        report_id: Uuid::new_v4(),
        report_type: "compliance".to_string(),
        total_keys: total_keys.0,
        keys_with_access_control: keys_with_acl.0,
        keys_without_access_control: keys_without_acl,
        expired_permissions: expired_permissions.0,
        total_audit_entries: total_audit_entries.0,
        compliance_score,
        generated_at: Utc::now().to_rfc3339(),
        findings: serde_json::json!({
            "total_keys": total_keys.0,
            "keys_with_access_control": keys_with_acl.0,
            "keys_without_access_control": keys_without_acl,
            "expired_permissions": expired_permissions.0,
            "total_audit_entries": total_audit_entries.0,
            "compliance_score": compliance_score,
        }),
    }))
}

pub fn encryption_routes() -> Router<AppState> {
    Router::new()
        // v3 routes
        .route("/api/v1/encryption/rotate", post(rotate_key))
        .route("/api/v1/encryption/rotations", get(list_key_rotations))
        .route("/api/v1/encryption/audit-log", post(log_audit_event_v3).get(list_audit_logs_v3))
        .route("/api/v1/encryption/key-lifecycle/{key_id}", get(get_key_lifecycle_v3))
        .route("/api/v1/encryption/compliance-report", post(get_compliance_report))
        // v6 routes
        .route("/api/v1/encryption-v6/key-versions", post(create_key_version))
        .route("/api/v1/encryption-v6/key-versions/{key_id}", get(get_key_versions))
        .route("/api/v1/encryption-v6/compliance-checks", post(create_compliance_check).get(get_compliance_checks))
        .route("/api/v1/encryption-v6/compliance-checks/{check_id}", patch(update_compliance_check))
        .route("/api/v1/encryption-v6/key-lifecycle/{key_id}", get(get_key_lifecycle_v6))
        .route("/api/v1/encryption-v6/audit-log", post(log_audit_event_v6).get(list_audit_logs_v6))
        // v9 routes
        .route("/api/v1/encryption-v9/key-versions", post(create_key_version))
        .route("/api/v1/encryption-v9/key-versions/{key_id}", get(get_key_versions))
        .route("/api/v1/encryption-v9/compliance-checks", post(create_compliance_check).get(get_compliance_checks))
        .route("/api/v1/encryption-v9/compliance-checks/{check_id}", patch(update_compliance_check))
        .route("/api/v1/encryption-v9/key-lifecycle/{key_id}", get(get_key_lifecycle_v6))
        .route("/api/v1/encryption-v9/audit-log", post(log_audit_event_v6).get(list_audit_logs_v6))
        // v10 routes
        .route("/api/v1/encryption-v10/key-versions", post(create_key_version))
        .route("/api/v1/encryption-v10/key-versions/{key_id}", get(get_key_versions))
        .route("/api/v1/encryption-v10/compliance-checks", post(create_compliance_check).get(get_compliance_checks))
        .route("/api/v1/encryption-v10/compliance-checks/{check_id}", patch(update_compliance_check))
        .route("/api/v1/encryption-v10/key-lifecycle/{key_id}", get(get_key_lifecycle_v6))
        .route("/api/v1/encryption-v10/audit-log", post(log_audit_event_v6).get(list_audit_logs_v6))
        // v13 routes
        .route("/api/v1/encryption-v13/access-control", post(grant_access))
        .route("/api/v1/encryption-v13/access-control/{key_id}", get(get_access_control))
        .route("/api/v1/encryption-v13/access-control/{key_id}/{principal_type}/{principal_id}", delete(revoke_access))
        .route("/api/v1/encryption-v13/audit-log", post(log_audit_v13))
        .route("/api/v1/encryption-v13/audit-log/{key_id}", get(get_audit_logs_v13))
        .route("/api/v1/encryption-v13/usage-analytics/{key_id}", get(get_key_usage_analytics))
        .route("/api/v1/encryption-v13/compliance-report", get(generate_compliance_report_v13))
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

    #[test]
    fn test_key_access_control_response_conversion() {
        let acl = KeyAccessControlV21 {
            id: Uuid::nil(),
            key_id: Uuid::nil(),
            principal_type: "user".to_string(),
            principal_id: Uuid::nil(),
            permission: "use".to_string(),
            granted_by: Uuid::nil(),
            granted_at: Utc::now(),
            expires_at: None,
        };
        let response: KeyAccessControlResponseV21 = acl.into();
        assert_eq!(response.principal_type, "user");
        assert_eq!(response.permission, "use");
    }
}
