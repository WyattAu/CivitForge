#![forbid(unsafe_code)]

use crate::api::AppState;
use crate::api::auth::{AuthUser, require_admin};
use crate::error::CoreError;
use crate::multi_region::MultiRegionService;
use axum::{
    Router,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize)]
pub struct RegionConfigResponse {
    pub id: Uuid,
    pub region: String,
    pub endpoint: String,
    pub status: String,
    pub failover_strategy: String,
    pub data_residency_required: bool,
    pub compliance_frameworks: Vec<String>,
    pub max_latency_ms: u64,
    pub capacity_weight: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct RegionStatusResponse {
    pub region: String,
    pub status: String,
    pub replication_links: Vec<ReplicationLinkResponse>,
    pub compliance_rules: Vec<ComplianceRuleResponse>,
    pub latency_routes: Vec<LatencyRouteResponse>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ReplicationLinkResponse {
    pub source_region: String,
    pub target_region: String,
    pub status: String,
    pub lag_bytes: i64,
    pub lag_seconds: f64,
    pub items_pending: i64,
    pub last_synced_at: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ComplianceRuleResponse {
    pub id: Uuid,
    pub framework: String,
    pub rule_name: String,
    pub rule_description: String,
    pub enabled: bool,
    pub last_result: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct LatencyRouteResponse {
    pub target_region: String,
    pub latency_ms: f64,
    pub healthy: bool,
    pub last_measured_at: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FailoverBody {
    pub target_region: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct FailoverResponse {
    pub id: Uuid,
    pub source_region: String,
    pub target_region: String,
    pub reason: String,
    pub status: String,
    pub initiated_by: Uuid,
    pub started_at: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ReplicationOverviewResponse {
    pub links: Vec<ReplicationLinkResponse>,
    pub total_links: usize,
    pub active_links: usize,
    pub degraded_links: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct RegionOverviewResponse {
    pub regions: Vec<RegionConfigResponse>,
    pub total_regions: usize,
    pub healthy_regions: usize,
    pub degraded_regions: usize,
    pub unavailable_regions: usize,
}

pub fn region_routes() -> Router<AppState> {
    Router::new()
        .route("/api/v1/regions", get(list_regions))
        .route("/api/v1/regions/{id}/status", get(get_region_status))
        .route("/api/v1/regions/{id}/failover", post(trigger_failover))
        .route("/api/v1/regions/replication", get(get_replication_status))
}

pub async fn list_regions(
    State(state): State<AppState>,
    _auth: AuthUser,
) -> Response {
    let pool = state.db.pool().clone();
    let svc = MultiRegionService::new(pool);

    match svc.list_region_configs().await {
        Ok(configs) => {
            let responses: Vec<RegionConfigResponse> = configs
                .into_iter()
                .map(|c| {
                    let frameworks = c.compliance_frameworks_list();
                    RegionConfigResponse {
                        id: c.id,
                        region: c.region,
                        endpoint: c.endpoint,
                        status: c.status,
                        failover_strategy: c.failover_strategy,
                        data_residency_required: c.data_residency_required,
                        compliance_frameworks: frameworks,
                        max_latency_ms: c.max_latency_ms as u64,
                        capacity_weight: c.capacity_weight,
                    }
                })
                .collect();
            (StatusCode::OK, Json(responses)).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Internal(format!("database error: {e}")).error_response()),
        )
            .into_response(),
    }
}

pub async fn get_region_status(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(region): Path<String>,
) -> Response {
    let pool = state.db.pool().clone();
    let svc = MultiRegionService::new(pool);

    match svc.get_region_config(&region).await {
        Ok(Some(config)) => {
            let replication = svc
                .get_replication_status(Some(&region), None)
                .await
                .unwrap_or_default();
            let rules = svc
                .get_compliance_rules(&region, None)
                .await
                .unwrap_or_default();

            let replication_links: Vec<ReplicationLinkResponse> = replication
                .into_iter()
                .map(|r| ReplicationLinkResponse {
                    source_region: r.source_region,
                    target_region: r.target_region,
                    status: r.status,
                    lag_bytes: r.lag_bytes,
                    lag_seconds: r.lag_seconds,
                    items_pending: r.items_pending,
                    last_synced_at: r.last_synced_at.map(|dt| dt.to_rfc3339()),
                })
                .collect();

            let compliance_rules: Vec<ComplianceRuleResponse> = rules
                .into_iter()
                .map(|r| ComplianceRuleResponse {
                    id: r.id,
                    framework: r.framework,
                    rule_name: r.rule_name,
                    rule_description: r.rule_description,
                    enabled: r.enabled,
                    last_result: r.last_result,
                })
                .collect();

            let latency_routes: Vec<LatencyRouteResponse> = Vec::new();

            let response = RegionStatusResponse {
                region: config.region,
                status: config.status,
                replication_links,
                compliance_rules,
                latency_routes,
            };
            (StatusCode::OK, Json(response)).into_response()
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(CoreError::NotFound(format!("region '{region}' not found")).error_response()),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Internal(format!("database error: {e}")).error_response()),
        )
            .into_response(),
    }
}

pub async fn trigger_failover(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(source_region): Path<String>,
    Json(req): Json<FailoverBody>,
) -> Response {
    if let Err(rejection) = require_admin(&auth) {
        return rejection.into_response();
    }

    let pool = state.db.pool().clone();
    let svc = MultiRegionService::new(pool);

    let user_id = match Uuid::parse_str(&auth.user_id) {
        Ok(id) => id,
        Err(_) => Uuid::nil(),
    };

    match svc
        .create_failover_record(&source_region, &req.target_region, &req.reason, user_id)
        .await
    {
        Ok(record) => {
            let response = FailoverResponse {
                id: record.id,
                source_region: record.source_region,
                target_region: record.target_region,
                reason: record.reason,
                status: record.status,
                initiated_by: record.initiated_by,
                started_at: record.started_at.to_rfc3339(),
            };
            (StatusCode::ACCEPTED, Json(response)).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Internal(format!("failed to create failover: {e}")).error_response()),
        )
            .into_response(),
    }
}

pub async fn get_replication_status(
    State(state): State<AppState>,
    _auth: AuthUser,
) -> Response {
    let pool = state.db.pool().clone();
    let svc = MultiRegionService::new(pool);

    match svc.get_replication_status(None, None).await {
        Ok(links) => {
            let responses: Vec<ReplicationLinkResponse> = links
                .into_iter()
                .map(|r| ReplicationLinkResponse {
                    source_region: r.source_region,
                    target_region: r.target_region,
                    status: r.status.clone(),
                    lag_bytes: r.lag_bytes,
                    lag_seconds: r.lag_seconds,
                    items_pending: r.items_pending,
                    last_synced_at: r.last_synced_at.map(|dt| dt.to_rfc3339()),
                })
                .collect();

            let total = responses.len();
            let active = responses.iter().filter(|r| r.status == "active").count();
            let degraded = responses.iter().filter(|r| r.status == "degraded").count();

            let overview = ReplicationOverviewResponse {
                links: responses,
                total_links: total,
                active_links: active,
                degraded_links: degraded,
            };
            (StatusCode::OK, Json(overview)).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Internal(format!("database error: {e}")).error_response()),
        )
            .into_response(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_region_config_response_serialization() {
        let response = RegionConfigResponse {
            id: Uuid::new_v4(),
            region: "eu".into(),
            endpoint: "https://eu.civitforge.com".into(),
            status: "healthy".into(),
            failover_strategy: "automatic".into(),
            data_residency_required: true,
            compliance_frameworks: vec!["GDPR".into()],
            max_latency_ms: 150,
            capacity_weight: 1.5,
        };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"region\":\"eu\""));
        assert!(json.contains("\"data_residency_required\":true"));
    }

    #[test]
    fn test_replication_link_response_serialization() {
        let response = ReplicationLinkResponse {
            source_region: "us".into(),
            target_region: "eu".into(),
            status: "active".into(),
            lag_bytes: 0,
            lag_seconds: 0.1,
            items_pending: 0,
            last_synced_at: Some(Utc::now().to_rfc3339()),
        };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"source_region\":\"us\""));
        assert!(json.contains("\"lag_seconds\":0.1"));
    }

    #[test]
    fn test_failover_body_deserialization() {
        let json = r#"{
            "target_region": "eu",
            "reason": "US region outage"
        }"#;
        let body: FailoverBody = serde_json::from_str(json).unwrap();
        assert_eq!(body.target_region, "eu");
        assert_eq!(body.reason, "US region outage");
    }

    #[test]
    fn test_failover_response_serialization() {
        let response = FailoverResponse {
            id: Uuid::new_v4(),
            source_region: "us".into(),
            target_region: "eu".into(),
            reason: "region outage".into(),
            status: "in_progress".into(),
            initiated_by: Uuid::new_v4(),
            started_at: Utc::now().to_rfc3339(),
        };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"status\":\"in_progress\""));
    }

    #[test]
    fn test_replication_overview_response_serialization() {
        let overview = ReplicationOverviewResponse {
            links: vec![],
            total_links: 2,
            active_links: 2,
            degraded_links: 0,
        };
        let json = serde_json::to_string(&overview).unwrap();
        assert!(json.contains("\"total_links\":2"));
        assert!(json.contains("\"active_links\":2"));
    }

    #[test]
    fn test_region_routes_compile() {
        let router = region_routes();
        let _ = router;
    }
}
