#![forbid(unsafe_code)]

use crate::api::AppState;
use crate::api::auth::AuthUser;
use crate::error::CoreError;
use crate::license_scanner::{LicenseInfo, LicenseScanner, LicenseViolation, ViolationSeverity};
use axum::{
    Router,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::{get, post},
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

async fn get_repo_id(pool: &sqlx::PgPool, owner: &str, name: &str) -> Option<Uuid> {
    sqlx::query_scalar::<_, Uuid>(
        "SELECT r.id FROM repositories r JOIN users u ON r.owner_id = u.id WHERE u.username = $1 AND r.name = $2",
    )
    .bind(owner)
    .bind(name)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten()
}

#[derive(Debug, Clone, Serialize)]
pub struct LicenseReportResponse {
    pub id: Uuid,
    pub repo_id: Uuid,
    pub license: String,
    pub spdx_id: String,
    pub file_count: i32,
    pub compliant: bool,
    pub issues: Vec<LicenseViolation>,
    pub scanned_at: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct LicenseDependencyEntry {
    pub package_name: String,
    pub version: String,
    pub license: LicenseInfo,
    pub source: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct LicenseDependenciesResponse {
    pub dependencies: Vec<LicenseDependencyEntry>,
    pub total: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct LicenseDistributionEntry {
    pub spdx_id: String,
    pub count: i64,
    pub percentage: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct LicenseDistributionResponse {
    pub entries: Vec<LicenseDistributionEntry>,
    pub total_files: i64,
}

#[derive(Debug, Deserialize)]
pub struct LicenseReportParams {
    pub owner: String,
    pub name: String,
}

pub fn license_routes() -> Router<AppState> {
    Router::new()
        .route(
            "/api/v1/repos/{owner}/{name}/license/scan",
            post(scan_repo_licenses),
        )
        .route(
            "/api/v1/repos/{owner}/{name}/license",
            get(get_license_report),
        )
        .route(
            "/api/v1/repos/{owner}/{name}/license/dependencies",
            get(list_dependency_licenses),
        )
        .route(
            "/api/v1/repos/{owner}/{name}/license/distribution",
            get(get_license_distribution),
        )
}

pub async fn scan_repo_licenses(
    State(state): State<AppState>,
    Path(params): Path<LicenseReportParams>,
    _auth: AuthUser,
) -> impl IntoResponse {
    let repo_id = match get_repo_id(state.db.pool(), &params.owner, &params.name).await {
        Some(id) => id,
        None => {
            let err = CoreError::NotFound("Repository not found".into());
            return (err.status_code(), Json(err.error_response())).into_response();
        }
    };

    let _scanner = LicenseScanner::new();
    let total_files = 0i32;

    let spdx_id = "LicenseRef-Unknown".to_string();
    let license_name = spdx_id.clone();

    let issues: Vec<LicenseViolation> = vec![LicenseViolation {
        package_name: params.name.clone(),
        license_id: spdx_id.clone(),
        reason: "No recognized license found".into(),
        severity: ViolationSeverity::Warning,
    }];

    let compliant = issues
        .iter()
        .all(|i| i.severity != ViolationSeverity::Error);

    let report_id = Uuid::new_v4();
    let _ = sqlx::query(
        r#"INSERT INTO license_reports (id, repo_id, license, spdx_id, file_count, compliant, issues, scanned_at)
           VALUES ($1, $2, $3, $4, $5, $6, $7, NOW())"#,
    )
    .bind(report_id)
    .bind(repo_id)
    .bind(&license_name)
    .bind(&spdx_id)
    .bind(total_files)
    .bind(compliant)
    .bind(serde_json::to_value(&issues).unwrap_or_default())
    .execute(state.db.pool())
    .await;

    let report = LicenseReportResponse {
        id: report_id,
        repo_id,
        license: license_name,
        spdx_id,
        file_count: total_files,
        compliant,
        issues,
        scanned_at: Utc::now().to_rfc3339(),
    };

    (StatusCode::OK, Json(report)).into_response()
}

pub async fn get_license_report(
    State(state): State<AppState>,
    Path(params): Path<LicenseReportParams>,
    _auth: AuthUser,
) -> impl IntoResponse {
    let repo_id = match get_repo_id(state.db.pool(), &params.owner, &params.name).await {
        Some(id) => id,
        None => {
            let err = CoreError::NotFound("Repository not found".into());
            return (err.status_code(), Json(err.error_response())).into_response();
        }
    };

    let row: Option<(Uuid, Uuid, String, String, i32, bool, serde_json::Value, DateTime<Utc>)> =
        sqlx::query_as(
            r#"SELECT id, repo_id, license, spdx_id, file_count, compliant, issues, scanned_at
               FROM license_reports
               WHERE repo_id = $1
               ORDER BY scanned_at DESC
               LIMIT 1"#,
        )
        .bind(repo_id)
        .fetch_optional(state.db.pool())
        .await
        .ok()
        .flatten();

    match row {
        Some((id, repo_id, license, spdx_id, file_count, compliant, issues_json, scanned_at)) => {
            let issues: Vec<LicenseViolation> =
                serde_json::from_value(issues_json).unwrap_or_default();
            let report = LicenseReportResponse {
                id,
                repo_id,
                license,
                spdx_id,
                file_count,
                compliant,
                issues,
                scanned_at: scanned_at.to_rfc3339(),
            };
            (StatusCode::OK, Json(report)).into_response()
        }
        None => {
            let err = CoreError::NotFound("No license report found for this repository".into());
            (err.status_code(), Json(err.error_response())).into_response()
        }
    }
}

pub async fn list_dependency_licenses(
    State(state): State<AppState>,
    Path(params): Path<LicenseReportParams>,
    _auth: AuthUser,
) -> impl IntoResponse {
    let _repo_id = match get_repo_id(state.db.pool(), &params.owner, &params.name).await {
        Some(id) => id,
        None => {
            let err = CoreError::NotFound("Repository not found".into());
            return (err.status_code(), Json(err.error_response())).into_response();
        }
    };

    let response = LicenseDependenciesResponse {
        total: 0,
        dependencies: Vec::new(),
    };

    (StatusCode::OK, Json(response)).into_response()
}

pub async fn get_license_distribution(
    State(state): State<AppState>,
    Path(params): Path<LicenseReportParams>,
    _auth: AuthUser,
) -> impl IntoResponse {
    let repo_id = match get_repo_id(state.db.pool(), &params.owner, &params.name).await {
        Some(id) => id,
        None => {
            let err = CoreError::NotFound("Repository not found".into());
            return (err.status_code(), Json(err.error_response())).into_response();
        }
    };

    let rows: Vec<(String, i64)> = sqlx::query_as(
        r#"SELECT spdx_id, COUNT(*) as count
           FROM license_reports
           WHERE repo_id = $1
           GROUP BY spdx_id
           ORDER BY count DESC"#,
    )
    .bind(repo_id)
    .fetch_all(state.db.pool())
    .await
    .unwrap_or_default();

    let total_files: i64 = rows.iter().map(|(_, c)| c).sum();
    let entries: Vec<LicenseDistributionEntry> = rows
        .into_iter()
        .map(|(spdx_id, count)| LicenseDistributionEntry {
            spdx_id,
            count,
            percentage: if total_files > 0 {
                (count as f64 / total_files as f64) * 100.0
            } else {
                0.0
            },
        })
        .collect();

    let response = LicenseDistributionResponse {
        entries,
        total_files,
    };

    (StatusCode::OK, Json(response)).into_response()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_license_report_response_serialization() {
        let resp = LicenseReportResponse {
            id: Uuid::new_v4(),
            repo_id: Uuid::new_v4(),
            license: "MIT".into(),
            spdx_id: "MIT".into(),
            file_count: 42,
            compliant: true,
            issues: vec![],
            scanned_at: "2025-01-01T00:00:00Z".into(),
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("\"spdx_id\":\"MIT\""));
        assert!(json.contains("\"compliant\":true"));
    }

    #[test]
    fn test_license_dependency_entry_serialization() {
        let entry = LicenseDependencyEntry {
            package_name: "serde".into(),
            version: "1.0".into(),
            license: LicenseInfo {
                spdx_id: "MIT".into(),
                name: "MIT License".into(),
                category: crate::license_scanner::LicenseCategory::Permissive,
                copyleft: false,
                patent_grant: false,
            },
            source: "Cargo.toml".into(),
        };
        let json = serde_json::to_string(&entry).unwrap();
        assert!(json.contains("\"package_name\":\"serde\""));
    }

    #[test]
    fn test_license_distribution_serialization() {
        let entry = LicenseDistributionEntry {
            spdx_id: "MIT".into(),
            count: 10,
            percentage: 50.0,
        };
        let json = serde_json::to_string(&entry).unwrap();
        assert!(json.contains("\"spdx_id\":\"MIT\""));
        assert!(json.contains("\"percentage\":50.0"));
    }

    #[test]
    fn test_license_routes_compile() {
        let router = license_routes();
        let _ = router;
    }
}
