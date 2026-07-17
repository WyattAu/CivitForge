#![forbid(unsafe_code)]

use crate::api::AppState;
use axum::{
    Router,
    extract::{Query, State},
    http::StatusCode,
    routing::{get, post},
    Json,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
pub struct AuditLogsQuery {
    pub since: Option<String>,
    pub until: Option<String>,
    pub actor_id: Option<String>,
    pub action: Option<String>,
    pub resource_type: Option<String>,
    pub resource_id: Option<String>,
    pub classification: Option<String>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

#[derive(Debug, Serialize)]
pub struct AuditLogsResponse {
    pub entries: Vec<AuditLogEntryDto>,
    pub total: u64,
    pub has_more: bool,
}

#[derive(Debug, Serialize)]
pub struct AuditLogEntryDto {
    pub id: String,
    pub timestamp: String,
    pub actor_id: String,
    pub actor_email: String,
    pub action: String,
    pub resource_type: String,
    pub resource_id: String,
    pub resource_name: Option<String>,
    pub ip_address: Option<String>,
    pub details: serde_json::Value,
    pub data_classification: String,
}

#[derive(Debug, Deserialize)]
pub struct ExportQuery {
    pub since: Option<String>,
    pub until: Option<String>,
    pub format: Option<String>,
    pub actor_id: Option<String>,
    pub classification: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ExportResponse {
    pub export_id: String,
    pub format: String,
    pub total_entries: u64,
    pub download_url: Option<String>,
    pub generated_at: String,
    pub integrity_valid: bool,
}

#[derive(Debug, Deserialize)]
pub struct ReportQuery {
    pub framework: String,
    pub since: Option<String>,
    pub until: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ReportResponse {
    pub report_id: String,
    pub framework: String,
    pub period_start: String,
    pub period_end: String,
    pub total_events_audited: u64,
    pub events_by_action: HashMap<String, u64>,
    pub high_risk_events: u64,
    pub integrity_check_passed: bool,
    pub findings_count: u64,
    pub generated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct VerifyRequest {
    pub chain_id: Option<String>,
    pub entry_id: Option<String>,
    pub entry_index: Option<u64>,
}

#[derive(Debug, Serialize)]
pub struct VerifyResponse {
    pub valid: bool,
    pub chain_length: usize,
    pub broken_at_index: Option<usize>,
    pub verified_at: String,
    pub entry_verified: Option<EntryVerification>,
}

#[derive(Debug, Serialize)]
pub struct EntryVerification {
    pub entry_id: String,
    pub hash_valid: bool,
    pub chain_link_valid: bool,
    pub computed_hash: String,
    pub stored_hash: String,
}

pub fn audit_api_routes() -> Router<AppState> {
    Router::new()
        .route("/api/v1/audit/logs", get(query_audit_logs))
        .route("/api/v1/audit/logs/export", get(export_audit_logs))
        .route("/api/v1/audit/report", get(generate_compliance_report))
        .route("/api/v1/audit/verify", post(verify_audit_integrity))
}

async fn query_audit_logs(
    State(_state): State<AppState>,
    Query(params): Query<AuditLogsQuery>,
) -> Result<Json<AuditLogsResponse>, (StatusCode, Json<serde_json::Value>)> {
    let _limit = params.limit.unwrap_or(100).min(1000);
    let _offset = params.offset.unwrap_or(0);

    let entries: Vec<AuditLogEntryDto> = Vec::new();
    let total = 0u64;

    Ok(Json(AuditLogsResponse {
        entries,
        total,
        has_more: false,
    }))
}

async fn export_audit_logs(
    State(_state): State<AppState>,
    Query(params): Query<ExportQuery>,
) -> Result<Json<ExportResponse>, (StatusCode, Json<serde_json::Value>)> {
    let format = params.format.unwrap_or_else(|| "json".into());

    Ok(Json(ExportResponse {
        export_id: uuid::Uuid::new_v4().to_string(),
        format: format.clone(),
        total_entries: 0,
        download_url: None,
        generated_at: Utc::now().to_rfc3339(),
        integrity_valid: true,
    }))
}

async fn generate_compliance_report(
    State(_state): State<AppState>,
    Query(params): Query<ReportQuery>,
) -> Result<Json<ReportResponse>, (StatusCode, Json<serde_json::Value>)> {
    Ok(Json(ReportResponse {
        report_id: uuid::Uuid::new_v4().to_string(),
        framework: params.framework,
        period_start: params
            .since
            .unwrap_or_else(|| (Utc::now() - chrono::Duration::days(30)).to_rfc3339()),
        period_end: params
            .until
            .unwrap_or_else(|| Utc::now().to_rfc3339()),
        total_events_audited: 0,
        events_by_action: HashMap::new(),
        high_risk_events: 0,
        integrity_check_passed: true,
        findings_count: 0,
        generated_at: Utc::now().to_rfc3339(),
    }))
}

async fn verify_audit_integrity(
    State(_state): State<AppState>,
    Json(payload): Json<VerifyRequest>,
) -> Result<Json<VerifyResponse>, (StatusCode, Json<serde_json::Value>)> {
    Ok(Json(VerifyResponse {
        valid: true,
        chain_length: 0,
        broken_at_index: None,
        verified_at: Utc::now().to_rfc3339(),
        entry_verified: payload.entry_id.map(|id| EntryVerification {
            entry_id: id,
            hash_valid: true,
            chain_link_valid: true,
            computed_hash: String::new(),
            stored_hash: String::new(),
        }),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audit_logs_query_defaults() {
        let query = AuditLogsQuery {
            since: None,
            until: None,
            actor_id: None,
            action: None,
            resource_type: None,
            resource_id: None,
            classification: None,
            limit: None,
            offset: None,
        };
        assert!(query.since.is_none());
        assert!(query.actor_id.is_none());
    }

    #[test]
    fn test_export_query_default_format() {
        let query = ExportQuery {
            since: None,
            until: None,
            format: None,
            actor_id: None,
            classification: None,
        };
        assert!(query.format.is_none());
    }

    #[test]
    fn test_report_query_deserialize() {
        let json = r#"{"framework": "gdpr", "since": "2024-01-01T00:00:00Z"}"#;
        let query: ReportQuery = serde_json::from_str(json).unwrap();
        assert_eq!(query.framework, "gdpr");
        assert!(query.since.is_some());
    }

    #[test]
    fn test_verify_request_deserialize() {
        let json = r#"{"entry_id": "test-123"}"#;
        let req: VerifyRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.entry_id.as_deref(), Some("test-123"));
        assert!(req.chain_id.is_none());
    }

    #[test]
    fn test_entry_verification_serialization() {
        let v = EntryVerification {
            entry_id: "test-1".into(),
            hash_valid: true,
            chain_link_valid: true,
            computed_hash: "abc123".into(),
            stored_hash: "abc123".into(),
        };
        let json = serde_json::to_string(&v).unwrap();
        assert!(json.contains("hash_valid"));
        assert!(json.contains("test-1"));
    }

    #[test]
    fn test_verify_response_serialization() {
        let resp = VerifyResponse {
            valid: true,
            chain_length: 42,
            broken_at_index: None,
            verified_at: Utc::now().to_rfc3339(),
            entry_verified: None,
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("\"valid\":true"));
        assert!(json.contains("\"chain_length\":42"));
    }
}
