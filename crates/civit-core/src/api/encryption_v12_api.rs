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
pub struct KeyUsageV20 {
    pub id: Uuid,
    pub key_id: Uuid,
    pub operation: String,
    pub success: bool,
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct RotationScheduleV20 {
    pub id: Uuid,
    pub key_id: Uuid,
    pub rotation_days: i32,
    pub last_rotated_at: Option<DateTime<Utc>>,
    pub next_rotation_at: Option<DateTime<Utc>>,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LogKeyUsageRequest {
    pub key_id: Uuid,
    pub operation: String,
    pub success: bool,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateRotationScheduleRequest {
    pub key_id: Uuid,
    pub rotation_days: i32,
}

#[derive(Debug, Clone, Serialize)]
pub struct KeyUsageResponseV20 {
    pub id: Uuid,
    pub key_id: Uuid,
    pub operation: String,
    pub success: bool,
    pub error_message: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct RotationScheduleResponseV20 {
    pub id: Uuid,
    pub key_id: Uuid,
    pub rotation_days: i32,
    pub last_rotated_at: Option<String>,
    pub next_rotation_at: Option<String>,
    pub enabled: bool,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct KeyPerformanceMetricsResponseV22 {
    pub key_id: Uuid,
    pub key_name: String,
    pub total_operations: i64,
    pub successful_operations: i64,
    pub failed_operations: i64,
    pub success_rate: f64,
    pub avg_operation_time_ms: f64,
    pub last_operation_at: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ComplianceReportResponseV22 {
    pub report_id: Uuid,
    pub report_type: String,
    pub key_count: i64,
    pub keys_needing_rotation: i64,
    pub average_compliance_score: f64,
    pub generated_at: String,
    pub findings: serde_json::Value,
}

impl From<KeyUsageV20> for KeyUsageResponseV20 {
    fn from(u: KeyUsageV20) -> Self {
        Self {
            id: u.id,
            key_id: u.key_id,
            operation: u.operation,
            success: u.success,
            error_message: u.error_message,
            created_at: u.created_at.to_rfc3339(),
        }
    }
}

impl From<RotationScheduleV20> for RotationScheduleResponseV20 {
    fn from(r: RotationScheduleV20) -> Self {
        Self {
            id: r.id,
            key_id: r.key_id,
            rotation_days: r.rotation_days,
            last_rotated_at: r.last_rotated_at.map(|t| t.to_rfc3339()),
            next_rotation_at: r.next_rotation_at.map(|t| t.to_rfc3339()),
            enabled: r.enabled,
            created_at: r.created_at.to_rfc3339(),
        }
    }
}

async fn log_key_usage(
    State(state): State<AppState>,
    Json(input): Json<LogKeyUsageRequest>,
) -> Result<(StatusCode, Json<KeyUsageResponseV20>), Response> {
    let usage = sqlx::query_as::<_, KeyUsageV20>(
        r#"INSERT INTO encryption_key_usage_v20 (key_id, operation, success, error_message)
         VALUES ($1, $2, $3, $4)
         RETURNING id, key_id, operation, success, error_message, created_at"#,
    )
    .bind(input.key_id)
    .bind(&input.operation)
    .bind(input.success)
    .bind(&input.error_message)
    .fetch_one(state.db.pool())
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response())?;

    Ok((StatusCode::CREATED, Json(usage.into())))
}

async fn get_key_usage(
    State(state): State<AppState>,
    Path(key_id): Path<Uuid>,
) -> Result<Json<Vec<KeyUsageResponseV20>>, Response> {
    let usages = sqlx::query_as::<_, KeyUsageV20>(
        r#"SELECT id, key_id, operation, success, error_message, created_at
         FROM encryption_key_usage_v20
         WHERE key_id = $1
         ORDER BY created_at DESC"#,
    )
    .bind(key_id)
    .fetch_all(state.db.pool())
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response())?;

    Ok(Json(usages.into_iter().map(|u| u.into()).collect()))
}

async fn create_rotation_schedule(
    State(state): State<AppState>,
    Json(input): Json<CreateRotationScheduleRequest>,
) -> Result<(StatusCode, Json<RotationScheduleResponseV20>), Response> {
    let schedule = sqlx::query_as::<_, RotationScheduleV20>(
        r#"INSERT INTO encryption_key_rotation_schedules_v20 (key_id, rotation_days, next_rotation_at)
         VALUES ($1, $2, NOW() + ($3 || ' days')::INTERVAL)
         ON CONFLICT (key_id) DO UPDATE SET rotation_days = $2, next_rotation_at = NOW() + ($3 || ' days')::INTERVAL
         RETURNING id, key_id, rotation_days, last_rotated_at, next_rotation_at, enabled, created_at"#,
    )
    .bind(input.key_id)
    .bind(input.rotation_days)
    .bind(input.rotation_days.to_string())
    .fetch_one(state.db.pool())
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response())?;

    Ok((StatusCode::CREATED, Json(schedule.into())))
}

async fn get_rotation_schedules(
    State(state): State<AppState>,
) -> Result<Json<Vec<RotationScheduleResponseV20>>, Response> {
    let schedules = sqlx::query_as::<_, RotationScheduleV20>(
        r#"SELECT id, key_id, rotation_days, last_rotated_at, next_rotation_at, enabled, created_at
         FROM encryption_key_rotation_schedules_v20
         WHERE enabled = true
         ORDER BY next_rotation_at ASC"#,
    )
    .fetch_all(state.db.pool())
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response())?;

    Ok(Json(schedules.into_iter().map(|s| s.into()).collect()))
}

async fn get_key_performance_metrics(
    State(state): State<AppState>,
    Path(key_id): Path<Uuid>,
) -> Result<Json<KeyPerformanceMetricsResponseV22>, Response> {
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
        r#"SELECT COUNT(*) FROM encryption_key_usage_v20 WHERE key_id = $1"#,
    )
    .bind(key_id)
    .fetch_one(state.db.pool())
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response())?;

    let successful_ops: (i64,) = sqlx::query_as(
        r#"SELECT COUNT(*) FROM encryption_key_usage_v20 WHERE key_id = $1 AND success = true"#,
    )
    .bind(key_id)
    .fetch_one(state.db.pool())
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response())?;

    let failed_ops: (i64,) = sqlx::query_as(
        r#"SELECT COUNT(*) FROM encryption_key_usage_v20 WHERE key_id = $1 AND success = false"#,
    )
    .bind(key_id)
    .fetch_one(state.db.pool())
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response())?;

    let last_op: Option<(Option<DateTime<Utc>>,)> = sqlx::query_as(
        r#"SELECT MAX(created_at) FROM encryption_key_usage_v20 WHERE key_id = $1"#,
    )
    .bind(key_id)
    .fetch_optional(state.db.pool())
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response())?;

    let total = total_ops.0;
    let successful = successful_ops.0;
    let failed = failed_ops.0;
    let success_rate = if total > 0 {
        (successful as f64 / total as f64) * 100.0
    } else {
        100.0
    };

    Ok(Json(KeyPerformanceMetricsResponseV22 {
        key_id: key.id,
        key_name: key.name,
        total_operations: total,
        successful_operations: successful,
        failed_operations: failed,
        success_rate,
        avg_operation_time_ms: 0.0,
        last_operation_at: last_op.and_then(|l| l.0).map(|t| t.to_rfc3339()),
    }))
}

async fn generate_compliance_report(
    State(state): State<AppState>,
) -> Result<Json<ComplianceReportResponseV22>, Response> {
    let key_count: (i64,) = sqlx::query_as(
        r#"SELECT COUNT(*) FROM encryption_keys"#,
    )
    .fetch_one(state.db.pool())
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response())?;

    let keys_needing_rotation: (i64,) = sqlx::query_as(
        r#"SELECT COUNT(*) FROM encryption_key_rotation_schedules_v20
         WHERE enabled = true AND next_rotation_at < NOW()"#,
    )
    .fetch_one(state.db.pool())
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response())?;

    let average_score: (f64,) = sqlx::query_as(
        r#"SELECT COALESCE(AVG(score), 0) FROM encryption_compliance_checks_v18"#,
    )
    .fetch_one(state.db.pool())
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response())?;

    Ok(Json(ComplianceReportResponseV22 {
        report_id: Uuid::new_v4(),
        report_type: "compliance".to_string(),
        key_count: key_count.0,
        keys_needing_rotation: keys_needing_rotation.0,
        average_compliance_score: average_score.0,
        generated_at: Utc::now().to_rfc3339(),
        findings: serde_json::json!({
            "total_keys": key_count.0,
            "keys_needing_rotation": keys_needing_rotation.0,
            "average_compliance_score": average_score.0,
        }),
    }))
}

pub fn encryption_v12_routes() -> Router<AppState> {
    Router::new()
        .route(
            "/api/v1/encryption-v12/key-usage",
            post(log_key_usage),
        )
        .route(
            "/api/v1/encryption-v12/key-usage/{key_id}",
            get(get_key_usage),
        )
        .route(
            "/api/v1/encryption-v12/rotation-schedules",
            post(create_rotation_schedule).get(get_rotation_schedules),
        )
        .route(
            "/api/v1/encryption-v12/performance/{key_id}",
            get(get_key_performance_metrics),
        )
        .route(
            "/api/v1/encryption-v12/compliance-report",
            get(generate_compliance_report),
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_usage_response_conversion() {
        let usage = KeyUsageV20 {
            id: Uuid::nil(),
            key_id: Uuid::nil(),
            operation: "encrypt".to_string(),
            success: true,
            error_message: None,
            created_at: Utc::now(),
        };
        let response: KeyUsageResponseV20 = usage.into();
        assert_eq!(response.operation, "encrypt");
        assert!(response.success);
    }

    #[test]
    fn test_rotation_schedule_response_conversion() {
        let schedule = RotationScheduleV20 {
            id: Uuid::nil(),
            key_id: Uuid::nil(),
            rotation_days: 90,
            last_rotated_at: None,
            next_rotation_at: Some(Utc::now()),
            enabled: true,
            created_at: Utc::now(),
        };
        let response: RotationScheduleResponseV20 = schedule.into();
        assert_eq!(response.rotation_days, 90);
        assert!(response.enabled);
    }
}
