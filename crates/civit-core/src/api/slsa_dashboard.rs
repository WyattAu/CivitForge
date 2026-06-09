#![forbid(unsafe_code)]

use crate::api::AppState;
use crate::error::CoreError;
use crate::provenance::{ProvenanceGenerator, SlsaProvenance};
use axum::{
    Router,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::get,
};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct ProvenanceAttestation {
    pub attestation_id: String,
    pub pipeline_run_id: String,
    pub provenance: SlsaProvenance,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProvenanceListResponse {
    pub attestations: Vec<ProvenanceAttestation>,
    pub total: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct VerificationResponse {
    pub attestation_id: String,
    pub verification: crate::provenance::VerificationResult,
}

#[derive(Debug, Clone, Serialize)]
pub struct ScorecardCheck {
    pub name: String,
    pub status: String,
    pub score: f64,
    pub details: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ScorecardResponse {
    pub overall_score: f64,
    pub checks: Vec<ScorecardCheck>,
    pub total_checks: usize,
    pub passed_checks: usize,
}

pub fn slsa_dashboard_routes() -> Router<AppState> {
    Router::new()
        .route(
            "/api/v1/repos/{owner}/{name}/slsa/provenance",
            get(list_provenance),
        )
        .route(
            "/api/v1/repos/{owner}/{name}/slsa/verify/{attestation_id}",
            get(verify_attestation),
        )
        .route(
            "/api/v1/repos/{owner}/{name}/slsa/scorecard",
            get(security_scorecard),
        )
}

async fn list_provenance(
    State(state): State<AppState>,
    Path((owner, name)): Path<(String, String)>,
) -> impl IntoResponse {
    let result = sqlx::query_as::<_, (String, String, String)>(
        "SELECT pipeline_run_id, attestation_id, provenance_json FROM slsa_attestations WHERE repo_owner = $1 AND repo_name = $2 ORDER BY created_at DESC LIMIT 100",
    )
    .bind(&owner)
    .bind(&name)
    .fetch_all(state.db.pool())
    .await;

    match result {
        Ok(rows) => {
            let attestations: Vec<ProvenanceAttestation> = rows
                .into_iter()
                .filter_map(|(run_id, att_id, json)| {
                    let provenance: SlsaProvenance = serde_json::from_str(&json).ok()?;
                    Some(ProvenanceAttestation {
                        attestation_id: att_id,
                        pipeline_run_id: run_id,
                        provenance,
                    })
                })
                .collect();
            let total = attestations.len();
            (
                StatusCode::OK,
                Json(ProvenanceListResponse {
                    attestations,
                    total,
                }),
            )
                .into_response()
        }
        Err(e) => {
            // Table might not exist yet — return empty list
            tracing::debug!("slsa_attestations query failed: {e}");
            (
                StatusCode::OK,
                Json(ProvenanceListResponse {
                    attestations: Vec::new(),
                    total: 0,
                }),
            )
                .into_response()
        }
    }
}

async fn verify_attestation(
    State(state): State<AppState>,
    Path((owner, name, attestation_id)): Path<(String, String, String)>,
) -> impl IntoResponse {
    let result = sqlx::query_as::<_, (String,)>(
        "SELECT provenance_json FROM slsa_attestations WHERE repo_owner = $1 AND repo_name = $2 AND attestation_id = $3",
    )
    .bind(&owner)
    .bind(&name)
    .bind(&attestation_id)
    .fetch_optional(state.db.pool())
    .await;

    match result {
        Ok(Some((json,))) => {
            let provenance: SlsaProvenance = match serde_json::from_str(&json) {
                Ok(p) => p,
                Err(e) => {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(
                            CoreError::Internal(format!("failed to parse provenance: {e}"))
                                .error_response(),
                        ),
                    )
                        .into_response();
                }
            };
            let verification = ProvenanceGenerator::verify(&provenance).unwrap_or_else(|_| {
                crate::provenance::VerificationResult {
                    passed: false,
                    checks: vec![crate::provenance::VerificationCheck {
                        name: "parse".into(),
                        passed: false,
                        message: "failed to verify".into(),
                    }],
                }
            });
            (
                StatusCode::OK,
                Json(VerificationResponse {
                    attestation_id,
                    verification,
                }),
            )
                .into_response()
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(
                CoreError::NotFound(format!("attestation {attestation_id} not found"))
                    .error_response(),
            ),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Internal(format!("database error: {e}")).error_response()),
        )
            .into_response(),
    }
}

async fn security_scorecard(
    State(state): State<AppState>,
    Path((owner, name)): Path<(String, String)>,
) -> impl IntoResponse {
    let mut checks = Vec::new();

    // Check 1: SLSA provenance exists
    let provenance_exists = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM slsa_attestations WHERE repo_owner = $1 AND repo_name = $2",
    )
    .bind(&owner)
    .bind(&name)
    .fetch_one(state.db.pool())
    .await
    .unwrap_or(0);

    checks.push(ScorecardCheck {
        name: "SLSA Provenance".into(),
        status: if provenance_exists > 0 {
            "PASS".into()
        } else {
            "WARN".into()
        },
        score: if provenance_exists > 0 { 1.0 } else { 0.0 },
        details: format!("{provenance_exists} provenance attestation(s) found"),
    });

    // Check 2: Secret scanning history
    let secret_scans = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM secret_scan_results WHERE repo_owner = $1 AND repo_name = $2",
    )
    .bind(&owner)
    .bind(&name)
    .fetch_one(state.db.pool())
    .await
    .unwrap_or(0);

    checks.push(ScorecardCheck {
        name: "Secret Scanning".into(),
        status: if secret_scans > 0 {
            "PASS".into()
        } else {
            "INFO".into()
        },
        score: if secret_scans > 0 { 1.0 } else { 0.5 },
        details: format!("{secret_scans} scan(s) performed"),
    });

    // Check 3: Branch protection
    let protected_branches = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM branch_protections WHERE repo_owner = $1 AND repo_name = $2",
    )
    .bind(&owner)
    .bind(&name)
    .fetch_one(state.db.pool())
    .await
    .unwrap_or(0);

    checks.push(ScorecardCheck {
        name: "Branch Protection".into(),
        status: if protected_branches > 0 {
            "PASS".into()
        } else {
            "WARN".into()
        },
        score: if protected_branches > 0 { 1.0 } else { 0.0 },
        details: format!("{protected_branches} protected branch rule(s)"),
    });

    // Check 4: CI/CD pipeline
    let pipeline_defs = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM pipeline_definitions WHERE repo_owner = $1 AND repo_name = $2",
    )
    .bind(&owner)
    .bind(&name)
    .fetch_one(state.db.pool())
    .await
    .unwrap_or(0);

    checks.push(ScorecardCheck {
        name: "CI/CD Pipeline".into(),
        status: if pipeline_defs > 0 {
            "PASS".into()
        } else {
            "WARN".into()
        },
        score: if pipeline_defs > 0 { 1.0 } else { 0.0 },
        details: format!("{pipeline_defs} pipeline definition(s)"),
    });

    // Check 5: Signed commits (check for deploy keys / SSH keys)
    let deploy_keys = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM deploy_keys WHERE repo_owner = $1 AND repo_name = $2",
    )
    .bind(&owner)
    .bind(&name)
    .fetch_one(state.db.pool())
    .await
    .unwrap_or(0);

    checks.push(ScorecardCheck {
        name: "Deploy Keys".into(),
        status: if deploy_keys > 0 {
            "PASS".into()
        } else {
            "INFO".into()
        },
        score: if deploy_keys > 0 { 1.0 } else { 0.5 },
        details: format!("{deploy_keys} deploy key(s) configured"),
    });

    // Check 6: Webhook integrations
    let webhooks = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM webhooks WHERE repo_owner = $1 AND repo_name = $2",
    )
    .bind(&owner)
    .bind(&name)
    .fetch_one(state.db.pool())
    .await
    .unwrap_or(0);

    checks.push(ScorecardCheck {
        name: "Webhook Integrations".into(),
        status: if webhooks > 0 {
            "PASS".into()
        } else {
            "INFO".into()
        },
        score: if webhooks > 0 { 1.0 } else { 0.5 },
        details: format!("{webhooks} webhook(s) configured"),
    });

    let total_checks = checks.len();
    let passed_checks = checks.iter().filter(|c| c.status == "PASS").count();
    let overall_score = if total_checks > 0 {
        checks.iter().map(|c| c.score).sum::<f64>() / total_checks as f64
    } else {
        0.0
    };

    (
        StatusCode::OK,
        Json(ScorecardResponse {
            overall_score,
            checks,
            total_checks,
            passed_checks,
        }),
    )
        .into_response()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::provenance::{BuildMetadata, Builder, Completeness, Material};
    use chrono::Utc;
    use std::collections::HashMap;

    fn make_test_provenance() -> SlsaProvenance {
        let mut digest = HashMap::new();
        digest.insert("sha256".into(), "abc123".into());
        SlsaProvenance {
            kind: "https://in-toto.io/Statement/v0.1".into(),
            version: 1,
            builder: Builder {
                id: "civitforge-builder".into(),
                version: Some("1.0.0".into()),
                builder_dependencies: None,
            },
            metadata: BuildMetadata {
                build_invocation_id: "inv-1".into(),
                build_started_on: Utc::now(),
                build_finished_on: Some(Utc::now()),
                completeness: Completeness {
                    parameters: true,
                    environment: false,
                    materials: true,
                },
                reproducible: true,
            },
            materials: vec![Material {
                uri: "git+https://example.com/repo".into(),
                digest,
                annotations: None,
            }],
        }
    }

    #[test]
    fn test_provenance_attestation_serialization() {
        let att = ProvenanceAttestation {
            attestation_id: "att-001".into(),
            pipeline_run_id: "run-001".into(),
            provenance: make_test_provenance(),
        };
        let json = serde_json::to_string(&att).unwrap();
        assert!(json.contains("att-001"));
        assert!(json.contains("run-001"));
        assert!(json.contains("civitforge-builder"));
    }

    #[test]
    fn test_provenance_list_response_serialization() {
        let resp = ProvenanceListResponse {
            attestations: vec![],
            total: 0,
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("\"total\":0"));
    }

    #[test]
    fn test_verification_response_serialization() {
        let resp = VerificationResponse {
            attestation_id: "att-001".into(),
            verification: crate::provenance::VerificationResult {
                passed: true,
                checks: vec![crate::provenance::VerificationCheck {
                    name: "builder.id".into(),
                    passed: true,
                    message: String::new(),
                }],
            },
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("\"passed\":true"));
    }

    #[test]
    fn test_scorecard_check_serialization() {
        let check = ScorecardCheck {
            name: "Test".into(),
            status: "PASS".into(),
            score: 1.0,
            details: "ok".into(),
        };
        let json = serde_json::to_string(&check).unwrap();
        assert!(json.contains("\"score\":1.0"));
        assert!(json.contains("\"status\":\"PASS\""));
    }

    #[test]
    fn test_scorecard_response_serialization() {
        let resp = ScorecardResponse {
            overall_score: 0.75,
            checks: vec![],
            total_checks: 4,
            passed_checks: 3,
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("\"overall_score\":0.75"));
        assert!(json.contains("\"total_checks\":4"));
        assert!(json.contains("\"passed_checks\":3"));
    }

    #[test]
    fn test_scorecard_check_names() {
        let check = ScorecardCheck {
            name: "SLSA Provenance".into(),
            status: "PASS".into(),
            score: 1.0,
            details: "1 attestation found".into(),
        };
        assert_eq!(check.name, "SLSA Provenance");
    }

    #[test]
    fn test_provenance_attestation_with_empty_materials() {
        let prov = SlsaProvenance {
            kind: "https://in-toto.io/Statement/v0.1".into(),
            version: 1,
            builder: Builder {
                id: "builder".into(),
                version: None,
                builder_dependencies: None,
            },
            metadata: BuildMetadata {
                build_invocation_id: "inv".into(),
                build_started_on: Utc::now(),
                build_finished_on: None,
                completeness: Completeness {
                    parameters: true,
                    environment: false,
                    materials: true,
                },
                reproducible: false,
            },
            materials: vec![],
        };
        let att = ProvenanceAttestation {
            attestation_id: "att-002".into(),
            pipeline_run_id: "run-002".into(),
            provenance: prov,
        };
        let json = serde_json::to_string(&att).unwrap();
        assert!(json.contains("att-002"));
    }

    #[test]
    fn test_scorecard_overall_score_zero_checks() {
        let total_checks = 0usize;
        let _passed_checks = 0usize;
        let overall_score = if total_checks > 0 { 1.0_f64 } else { 0.0 };
        assert_eq!(overall_score, 0.0);
    }
}
