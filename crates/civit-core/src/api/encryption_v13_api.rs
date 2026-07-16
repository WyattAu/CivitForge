#![forbid(unsafe_code)]

use crate::api::AppState;
use axum::{
    Router,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{delete, get, post},
    Json,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

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
pub struct LogAuditRequest {
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

async fn log_audit(
    State(state): State<AppState>,
    Json(input): Json<LogAuditRequest>,
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

async fn get_audit_logs(
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

async fn generate_compliance_report(
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

pub fn encryption_v13_routes() -> Router<AppState> {
    Router::new()
        .route(
            "/api/v1/encryption-v13/access-control",
            post(grant_access),
        )
        .route(
            "/api/v1/encryption-v13/access-control/{key_id}",
            get(get_access_control),
        )
        .route(
            "/api/v1/encryption-v13/access-control/{key_id}/{principal_type}/{principal_id}",
            delete(revoke_access),
        )
        .route(
            "/api/v1/encryption-v13/audit-log",
            post(log_audit),
        )
        .route(
            "/api/v1/encryption-v13/audit-log/{key_id}",
            get(get_audit_logs),
        )
        .route(
            "/api/v1/encryption-v13/usage-analytics/{key_id}",
            get(get_key_usage_analytics),
        )
        .route(
            "/api/v1/encryption-v13/compliance-report",
            get(generate_compliance_report),
        )
}

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn test_encryption_audit_log_response_conversion() {
        let log = EncryptionAuditLogV21 {
            id: Uuid::nil(),
            key_id: Uuid::nil(),
            operation: "encrypt".to_string(),
            principal_id: Uuid::nil(),
            success: true,
            ip_address: Some("192.168.1.1".to_string()),
            created_at: Utc::now(),
        };
        let response: EncryptionAuditLogResponseV21 = log.into();
        assert_eq!(response.operation, "encrypt");
        assert!(response.success);
    }
}
