#![forbid(unsafe_code)]

use crate::api::AppState;
use crate::error::CoreError;
use axum::{
    Router,
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityScan {
    pub id: String,
    pub repo_id: String,
    pub scan_type: String,
    pub status: String,
    pub findings: serde_json::Value,
    pub score: i32,
    pub started_at: String,
    pub completed_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityFinding {
    pub id: String,
    pub scan_id: String,
    pub severity: String,
    pub category: String,
    pub title: String,
    pub description: String,
    pub file_path: Option<String>,
    pub line_number: Option<i32>,
    pub recommendation: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ScanListResponse {
    pub scans: Vec<SecurityScan>,
    pub total: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct FindingListResponse {
    pub findings: Vec<SecurityFinding>,
    pub total: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct SecurityDashboardResponse {
    pub total_scans: i64,
    pub completed_scans: i64,
    pub pending_scans: i64,
    pub average_score: f64,
    pub total_findings: i64,
    pub critical_findings: i64,
    pub high_findings: i64,
    pub medium_findings: i64,
    pub low_findings: i64,
    pub recent_scans: Vec<SecurityScan>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TriggerScanRequest {
    pub scan_type: String,
}

async fn get_repo_id(
    state: &AppState,
    owner: &str,
    name: &str,
) -> Result<Uuid, CoreError> {
    let result = sqlx::query_scalar::<_, Uuid>(
        "SELECT id FROM repositories WHERE owner_id = (SELECT id FROM users WHERE username = $1) AND name = $2",
    )
    .bind(owner)
    .bind(name)
    .fetch_optional(state.db.pool())
    .await;

    match result {
        Ok(Some(id)) => Ok(id),
        Ok(None) => Err(CoreError::NotFound("repository not found".into())),
        Err(e) => Err(CoreError::Internal(format!("database error: {e}"))),
    }
}

pub fn security_dashboard_routes() -> Router<AppState> {
    Router::new()
        .route(
            "/api/v1/repos/{owner}/{name}/security/scan",
            post(trigger_scan),
        )
        .route(
            "/api/v1/repos/{owner}/{name}/security/scans",
            get(list_scans),
        )
        .route(
            "/api/v1/repos/{owner}/{name}/security/findings",
            get(list_findings),
        )
        .route(
            "/api/v1/repos/{owner}/{name}/security/dashboard",
            get(security_dashboard),
        )
}

pub async fn trigger_scan(
    State(state): State<AppState>,
    Path((owner, name)): Path<(String, String)>,
    Json(req): Json<TriggerScanRequest>,
) -> Response {
    let repo_id = match get_repo_id(&state, &owner, &name).await {
        Ok(id) => id,
        Err(CoreError::NotFound(msg)) => {
            return (
                StatusCode::NOT_FOUND,
                Json(CoreError::NotFound(msg).error_response()),
            )
                .into_response();
        }
        Err(e) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(e.error_response())).into_response();
        }
    };

    let scan_id = Uuid::new_v4();
    let result = sqlx::query_as::<_, (String, String, String, String, serde_json::Value, i32, String, Option<String>)>(
        r#"INSERT INTO security_scans (id, repo_id, scan_type, status, findings, score)
           VALUES ($1, $2, $3, 'pending', '[]', 0)
           RETURNING id::text, repo_id::text, scan_type, status, findings, score, started_at::text, completed_at::text"#,
    )
    .bind(scan_id)
    .bind(repo_id)
    .bind(&req.scan_type)
    .fetch_one(state.db.pool())
    .await;

    match result {
        Ok(row) => {
            let scan = SecurityScan {
                id: row.0,
                repo_id: row.1,
                scan_type: row.2,
                status: row.3,
                findings: row.4,
                score: row.5,
                started_at: row.6,
                completed_at: row.7,
            };
            (StatusCode::CREATED, Json(scan)).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Internal(format!("database error: {e}")).error_response()),
        )
            .into_response(),
    }
}

pub async fn list_scans(
    State(state): State<AppState>,
    Path((owner, name)): Path<(String, String)>,
) -> Response {
    let repo_id = match get_repo_id(&state, &owner, &name).await {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::OK,
                Json(ScanListResponse {
                    scans: Vec::new(),
                    total: 0,
                }),
            )
                .into_response();
        }
    };

    let result = sqlx::query_as::<_, (String, String, String, String, serde_json::Value, i32, String, Option<String>)>(
        r#"SELECT id::text, repo_id::text, scan_type, status, findings, score, started_at::text, completed_at::text
           FROM security_scans WHERE repo_id = $1 ORDER BY started_at DESC LIMIT 50"#,
    )
    .bind(repo_id)
    .fetch_all(state.db.pool())
    .await;

    match result {
        Ok(rows) => {
            let scans: Vec<SecurityScan> = rows
                .into_iter()
                .map(|r| SecurityScan {
                    id: r.0,
                    repo_id: r.1,
                    scan_type: r.2,
                    status: r.3,
                    findings: r.4,
                    score: r.5,
                    started_at: r.6,
                    completed_at: r.7,
                })
                .collect();
            let total = scans.len();
            (StatusCode::OK, Json(ScanListResponse { scans, total })).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Internal(format!("database error: {e}")).error_response()),
        )
            .into_response(),
    }
}

pub async fn list_findings(
    State(state): State<AppState>,
    Path((owner, name)): Path<(String, String)>,
) -> Response {
    let repo_id = match get_repo_id(&state, &owner, &name).await {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::OK,
                Json(FindingListResponse {
                    findings: Vec::new(),
                    total: 0,
                }),
            )
                .into_response();
        }
    };

    let result = sqlx::query_as::<_, (String, String, String, String, String, String, Option<String>, Option<i32>, String, String)>(
        r#"SELECT sf.id::text, sf.scan_id::text, sf.severity, sf.category, sf.title, sf.description,
                  sf.file_path, sf.line_number, sf.recommendation, sf.created_at::text
           FROM security_findings sf
           JOIN security_scans ss ON sf.scan_id = ss.id
           WHERE ss.repo_id = $1
           ORDER BY sf.created_at DESC LIMIT 100"#,
    )
    .bind(repo_id)
    .fetch_all(state.db.pool())
    .await;

    match result {
        Ok(rows) => {
            let findings: Vec<SecurityFinding> = rows
                .into_iter()
                .map(|r| SecurityFinding {
                    id: r.0,
                    scan_id: r.1,
                    severity: r.2,
                    category: r.3,
                    title: r.4,
                    description: r.5,
                    file_path: r.6,
                    line_number: r.7,
                    recommendation: r.8,
                    created_at: r.9,
                })
                .collect();
            let total = findings.len();
            (StatusCode::OK, Json(FindingListResponse { findings, total })).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Internal(format!("database error: {e}")).error_response()),
        )
            .into_response(),
    }
}

pub async fn security_dashboard(
    State(state): State<AppState>,
    Path((owner, name)): Path<(String, String)>,
) -> Response {
    let repo_id = match get_repo_id(&state, &owner, &name).await {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::OK,
                Json(SecurityDashboardResponse {
                    total_scans: 0,
                    completed_scans: 0,
                    pending_scans: 0,
                    average_score: 0.0,
                    total_findings: 0,
                    critical_findings: 0,
                    high_findings: 0,
                    medium_findings: 0,
                    low_findings: 0,
                    recent_scans: Vec::new(),
                }),
            )
                .into_response();
        }
    };

    let total_scans: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM security_scans WHERE repo_id = $1",
    )
    .bind(repo_id)
    .fetch_one(state.db.pool())
    .await
    .unwrap_or(0);

    let completed_scans: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM security_scans WHERE repo_id = $1 AND status = 'completed'",
    )
    .bind(repo_id)
    .fetch_one(state.db.pool())
    .await
    .unwrap_or(0);

    let pending_scans: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM security_scans WHERE repo_id = $1 AND status = 'pending'",
    )
    .bind(repo_id)
    .fetch_one(state.db.pool())
    .await
    .unwrap_or(0);

    let average_score: f64 = sqlx::query_scalar(
        "SELECT COALESCE(AVG(score), 0.0) FROM security_scans WHERE repo_id = $1 AND status = 'completed'",
    )
    .bind(repo_id)
    .fetch_one(state.db.pool())
    .await
    .unwrap_or(0.0);

    let total_findings: i64 = sqlx::query_scalar(
        r#"SELECT COUNT(*) FROM security_findings sf
           JOIN security_scans ss ON sf.scan_id = ss.id
           WHERE ss.repo_id = $1"#,
    )
    .bind(repo_id)
    .fetch_one(state.db.pool())
    .await
    .unwrap_or(0);

    let critical_findings: i64 = sqlx::query_scalar(
        r#"SELECT COUNT(*) FROM security_findings sf
           JOIN security_scans ss ON sf.scan_id = ss.id
           WHERE ss.repo_id = $1 AND sf.severity = 'critical'"#,
    )
    .bind(repo_id)
    .fetch_one(state.db.pool())
    .await
    .unwrap_or(0);

    let high_findings: i64 = sqlx::query_scalar(
        r#"SELECT COUNT(*) FROM security_findings sf
           JOIN security_scans ss ON sf.scan_id = ss.id
           WHERE ss.repo_id = $1 AND sf.severity = 'high'"#,
    )
    .bind(repo_id)
    .fetch_one(state.db.pool())
    .await
    .unwrap_or(0);

    let medium_findings: i64 = sqlx::query_scalar(
        r#"SELECT COUNT(*) FROM security_findings sf
           JOIN security_scans ss ON sf.scan_id = ss.id
           WHERE ss.repo_id = $1 AND sf.severity = 'medium'"#,
    )
    .bind(repo_id)
    .fetch_one(state.db.pool())
    .await
    .unwrap_or(0);

    let low_findings: i64 = sqlx::query_scalar(
        r#"SELECT COUNT(*) FROM security_findings sf
           JOIN security_scans ss ON sf.scan_id = ss.id
           WHERE ss.repo_id = $1 AND sf.severity = 'low'"#,
    )
    .bind(repo_id)
    .fetch_one(state.db.pool())
    .await
    .unwrap_or(0);

    let recent_scans_result = sqlx::query_as::<_, (String, String, String, String, serde_json::Value, i32, String, Option<String>)>(
        r#"SELECT id::text, repo_id::text, scan_type, status, findings, score, started_at::text, completed_at::text
           FROM security_scans WHERE repo_id = $1 ORDER BY started_at DESC LIMIT 5"#,
    )
    .bind(repo_id)
    .fetch_all(state.db.pool())
    .await;

    let recent_scans = match recent_scans_result {
        Ok(rows) => rows
            .into_iter()
            .map(|r| SecurityScan {
                id: r.0,
                repo_id: r.1,
                scan_type: r.2,
                status: r.3,
                findings: r.4,
                score: r.5,
                started_at: r.6,
                completed_at: r.7,
            })
            .collect(),
        Err(_) => Vec::new(),
    };

    (
        StatusCode::OK,
        Json(SecurityDashboardResponse {
            total_scans,
            completed_scans,
            pending_scans,
            average_score,
            total_findings,
            critical_findings,
            high_findings,
            medium_findings,
            low_findings,
            recent_scans,
        }),
    )
        .into_response()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_security_scan_serialization() {
        let scan = SecurityScan {
            id: "test-id".into(),
            repo_id: "repo-id".into(),
            scan_type: "full".into(),
            status: "completed".into(),
            findings: serde_json::json!([]),
            score: 85,
            started_at: "2025-01-01T00:00:00Z".into(),
            completed_at: Some("2025-01-01T00:01:00Z".into()),
        };
        let json = serde_json::to_string(&scan).unwrap();
        assert!(json.contains("test-id"));
        assert!(json.contains("\"score\":85"));
    }

    #[test]
    fn test_security_finding_serialization() {
        let finding = SecurityFinding {
            id: "finding-1".into(),
            scan_id: "scan-1".into(),
            severity: "critical".into(),
            category: "injection".into(),
            title: "SQL Injection".into(),
            description: "Potential SQL injection vulnerability".into(),
            file_path: Some("src/db.rs".into()),
            line_number: Some(42),
            recommendation: "Use parameterized queries".into(),
            created_at: "2025-01-01T00:00:00Z".into(),
        };
        let json = serde_json::to_string(&finding).unwrap();
        assert!(json.contains("SQL Injection"));
        assert!(json.contains("\"severity\":\"critical\""));
    }

    #[test]
    fn test_scan_list_response_serialization() {
        let resp = ScanListResponse {
            scans: Vec::new(),
            total: 0,
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("\"total\":0"));
    }

    #[test]
    fn test_finding_list_response_serialization() {
        let resp = FindingListResponse {
            findings: Vec::new(),
            total: 0,
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("\"total\":0"));
    }

    #[test]
    fn test_security_dashboard_response_serialization() {
        let resp = SecurityDashboardResponse {
            total_scans: 10,
            completed_scans: 8,
            pending_scans: 2,
            average_score: 75.5,
            total_findings: 25,
            critical_findings: 2,
            high_findings: 5,
            medium_findings: 10,
            low_findings: 8,
            recent_scans: Vec::new(),
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("\"total_scans\":10"));
        assert!(json.contains("\"critical_findings\":2"));
        assert!(json.contains("\"average_score\":75.5"));
    }

    #[test]
    fn test_trigger_scan_request_deserialization() {
        let json = r#"{"scan_type": "full"}"#;
        let req: TriggerScanRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.scan_type, "full");
    }
}
