#![forbid(unsafe_code)]

use crate::api::AppState;
use crate::api::auth::{AuthUser, require_admin};
use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post, put},
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureFlagResponse {
    pub id: String,
    pub name: String,
    pub description: String,
    pub enabled: bool,
    pub enabled_for_users: Vec<String>,
    pub enabled_for_percentage: i32,
    pub enabled_for_orgs: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
    /// Lifecycle category; sets the staleness deadline (ADR-0008).
    pub kind: String,
    /// Accountable party.
    pub owner: String,
    /// External issue reference.
    pub ticket: String,
    /// Rollout-cycle identifier; change it to re-draw the cohort.
    pub salt: String,
    /// Null means never evaluated, which is a staleness signal.
    pub last_evaluated_at: Option<String>,
    /// Staleness clock origin.
    pub last_changed_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureFlagsListResponse {
    pub flags: Vec<FeatureFlagResponse>,
    pub total: usize,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateFeatureFlagRequest {
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub enabled_for_percentage: i32,
    /// Lifecycle category; defaults to `release`.
    #[serde(default)]
    pub kind: String,
    /// Accountable party. Required: an unowned flag cannot be cleaned up.
    #[serde(default)]
    pub owner: String,
    /// External issue reference.
    #[serde(default)]
    pub ticket: String,
    /// Rollout-cycle identifier. Defaults to empty, which evaluates
    /// identically to the pre-rollout-cycle unsalted bucket.
    #[serde(default)]
    pub salt: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateFeatureFlagRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled_for_percentage: Option<i32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ToggleUserRequest {
    pub user_id: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ToggleOrgRequest {
    pub org_id: String,
}

fn flag_to_response(flag: &civit_db::models::FeatureFlag) -> FeatureFlagResponse {
    FeatureFlagResponse {
        id: flag.id.to_string(),
        name: flag.name.clone(),
        description: flag.description.clone(),
        enabled: flag.enabled,
        enabled_for_users: flag
            .enabled_for_users
            .iter()
            .map(|u| u.to_string())
            .collect(),
        enabled_for_percentage: flag.enabled_for_percentage,
        enabled_for_orgs: flag
            .enabled_for_orgs
            .iter()
            .map(|o| o.to_string())
            .collect(),
        created_at: flag.created_at.to_rfc3339(),
        updated_at: flag.updated_at.to_rfc3339(),
        kind: flag.kind.clone(),
        owner: flag.owner.clone(),
        ticket: flag.ticket.clone(),
        salt: flag.salt.clone(),
        last_evaluated_at: flag.last_evaluated_at.map(|t| t.to_rfc3339()),
        last_changed_at: flag.last_changed_at.to_rfc3339(),
    }
}

pub async fn list_feature_flags_for_user(
    State(state): State<AppState>,
    auth: AuthUser,
) -> impl IntoResponse {
    let user_id: Uuid = match auth.user_id.parse() {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({"error": "invalid user id"})),
            )
                .into_response();
        }
    };

    match state.db.list_enabled_feature_flags_for_user(user_id).await {
        Ok(flags) => {
            // This *is* the evaluation event, so it is where the evidence
            // for the `never_evaluated` staleness signal comes from.
            // Without it every flag looks unevaluated and the signal fires
            // on all of them.
            let names: Vec<String> = flags.iter().map(|f| f.name.clone()).collect();
            if let Err(e) = state.db.touch_feature_flags_evaluated(&names).await {
                tracing::warn!("could not record flag evaluation: {e}");
            }
            let response = FeatureFlagsListResponse {
                flags: flags.iter().map(flag_to_response).collect(),
                total: flags.len(),
            };
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

pub async fn list_all_feature_flags(
    State(state): State<AppState>,
    auth: AuthUser,
) -> impl IntoResponse {
    if let Err(rejection) = require_admin(&auth) {
        return rejection.into_response();
    }

    match state.db.list_feature_flags().await {
        Ok(flags) => {
            let response = FeatureFlagsListResponse {
                flags: flags.iter().map(flag_to_response).collect(),
                total: flags.len(),
            };
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

pub async fn create_feature_flag(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<CreateFeatureFlagRequest>,
) -> impl IntoResponse {
    if let Err(rejection) = require_admin(&auth) {
        return rejection.into_response();
    }

    if let Some(rejection) = validate_governance(&req) {
        return rejection.into_response();
    }

    // Conflict before insert: a duplicate name is the client's mistake and
    // must read as 409, not as a 500 carrying a raw constraint violation.
    match state.db.get_feature_flag_by_name(&req.name).await {
        Ok(Some(_)) => {
            return (
                StatusCode::CONFLICT,
                Json(serde_json::json!({"error": format!("flag {} already exists", req.name)})),
            )
                .into_response();
        }
        Ok(None) => {}
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": e.to_string()})),
            )
                .into_response();
        }
    }

    let kind = if req.kind.is_empty() {
        flag_kit::FlagKind::Release.as_str()
    } else {
        req.kind.as_str()
    };

    match state
        .db
        .create_feature_flag_governed(
            &req.name,
            &req.description,
            req.enabled,
            req.enabled_for_percentage,
            kind,
            &req.owner,
            &req.ticket,
            &req.salt,
        )
        .await
    {
        Ok(flag) => (StatusCode::CREATED, Json(flag_to_response(&flag))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

pub async fn update_feature_flag(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateFeatureFlagRequest>,
) -> impl IntoResponse {
    if let Err(rejection) = require_admin(&auth) {
        return rejection.into_response();
    }

    match state
        .db
        .update_feature_flag(
            id,
            req.name.as_deref(),
            req.description.as_deref(),
            req.enabled,
            req.enabled_for_percentage,
        )
        .await
    {
        Ok(flag) => (StatusCode::OK, Json(flag_to_response(&flag))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

pub async fn delete_feature_flag(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    if let Err(rejection) = require_admin(&auth) {
        return rejection.into_response();
    }

    match state.db.delete_feature_flag(id).await {
        Ok(()) => (StatusCode::NO_CONTENT).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

pub async fn toggle_feature_flag(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    if let Err(rejection) = require_admin(&auth) {
        return rejection.into_response();
    }

    match state.db.toggle_feature_flag(id).await {
        Ok(flag) => {
            let _ = state
                .db
                .record_feature_flag_event(id, None, flag.enabled)
                .await;
            (StatusCode::OK, Json(flag_to_response(&flag))).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

pub async fn add_feature_flag_user(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    Json(req): Json<ToggleUserRequest>,
) -> impl IntoResponse {
    if let Err(rejection) = require_admin(&auth) {
        return rejection.into_response();
    }

    let user_id: Uuid = match req.user_id.parse() {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({"error": "invalid user id"})),
            )
                .into_response();
        }
    };

    match state.db.add_feature_flag_user(id, user_id).await {
        Ok(()) => (StatusCode::NO_CONTENT).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

pub async fn remove_feature_flag_user(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((id, user_id)): Path<(Uuid, Uuid)>,
) -> impl IntoResponse {
    if let Err(rejection) = require_admin(&auth) {
        return rejection.into_response();
    }

    match state.db.remove_feature_flag_user(id, user_id).await {
        Ok(()) => (StatusCode::NO_CONTENT).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

pub async fn add_feature_flag_org(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    Json(req): Json<ToggleOrgRequest>,
) -> impl IntoResponse {
    if let Err(rejection) = require_admin(&auth) {
        return rejection.into_response();
    }

    let org_id: Uuid = match req.org_id.parse() {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({"error": "invalid org id"})),
            )
                .into_response();
        }
    };

    match state.db.add_feature_flag_org(id, org_id).await {
        Ok(()) => (StatusCode::NO_CONTENT).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

pub async fn remove_feature_flag_org(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((id, org_id)): Path<(Uuid, Uuid)>,
) -> impl IntoResponse {
    if let Err(rejection) = require_admin(&auth) {
        return rejection.into_response();
    }

    match state.db.remove_feature_flag_org(id, org_id).await {
        Ok(()) => (StatusCode::NO_CONTENT).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

/// Governance rules, from the Harness FME model (owner, description, and
/// default-off in production). Enforced on creation so an unowned,
/// undescribed flag cannot exist in the first place.
fn validate_governance(req: &CreateFeatureFlagRequest) -> Option<axum::response::Response> {
    let bad = |msg: &str| {
        (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": msg})),
        )
            .into_response()
    };

    if req.name.is_empty() {
        return Some(bad("name is required"));
    }
    if let Err(e) = flag_kit::FlagName::new(req.name.clone()) {
        return Some(bad(&format!("invalid flag name: {e}")));
    }
    if !(0..=100).contains(&req.enabled_for_percentage) {
        return Some(bad("enabled_for_percentage must be 0..=100"));
    }
    if req.description.trim().is_empty() {
        return Some(bad(
            "description is required: flags must explain their purpose",
        ));
    }
    if req.owner.trim().is_empty() {
        return Some(bad(
            "owner is required: an unowned flag cannot be cleaned up",
        ));
    }
    if !req.kind.is_empty() && flag_kit::FlagKind::parse(&req.kind).is_err() {
        return Some(bad(
            "kind must be one of release, experiment, operational, permission",
        ));
    }
    None
}

fn days_since(then: chrono::DateTime<chrono::Utc>) -> i64 {
    (chrono::Utc::now() - then).num_days().max(0)
}

fn signal_detail(signal: flag_kit::StaleSignal) -> civit_db::models::FlagSignalView {
    use civit_db::models::FlagSignalView;
    let detail = match signal {
        flag_kit::StaleSignal::AgedPastDeadline {
            age_days,
            deadline_days,
        } => format!("{age_days} days past a {deadline_days} day deadline"),
        flag_kit::StaleSignal::FullyRolledOut => {
            "rollout reached 100% but the flag still exists".to_string()
        }
        flag_kit::StaleSignal::NeverEvaluated => {
            "no recorded evaluation since creation".to_string()
        }
    };
    FlagSignalView {
        signal: signal.as_str().to_string(),
        detail,
    }
}

/// Derives the policy's inputs from a stored row and returns its verdict.
///
/// Pure and separate from the handler so the mapping from a DB row to a
/// staleness verdict can be tested without a database or a request.
fn staleness_view(
    f: &civit_db::models::FeatureFlag,
    policy: &flag_kit::FlagPolicy,
) -> civit_db::models::FlagStalenessView {
    // An unrecognized kind falls back to `Release`: a short deadline, not a
    // silent exemption.
    let kind = flag_kit::FlagKind::parse(&f.kind).unwrap_or_default();
    let last_evaluated_age_days = f.last_evaluated_at.map(days_since);
    let c = policy.classify(flag_kit::FlagFacts {
        kind,
        age_days: days_since(f.created_at) as u32,
        last_changed_age_days: Some(days_since(f.last_changed_at) as u32),
        percentage: f.enabled_for_percentage.clamp(0, 100) as u8,
        last_evaluated_age_days: last_evaluated_age_days.map(|d| d as u32),
        // This endpoint reads `last_evaluated_at`, which the evaluation path
        // writes, so absence really does mean "never evaluated" here.
        evaluation_tracked: true,
    });
    civit_db::models::FlagStalenessView {
        id: f.id,
        name: f.name.clone(),
        kind: kind.as_str().to_string(),
        owner: f.owner.clone(),
        ticket: f.ticket.clone(),
        staleness: c.staleness.as_str().to_string(),
        is_removal_candidate: c.is_stale(),
        age_days: days_since(f.created_at),
        last_changed_age_days: Some(days_since(f.last_changed_at)),
        last_evaluated_age_days,
        enabled: f.enabled,
        percentage: f.enabled_for_percentage,
        signals: c.signals.iter().copied().map(signal_detail).collect(),
    }
}

/// Classifies every flag with the kit's policy and returns the evidence.
///
/// This is the "what needs cleanup" endpoint that the research says is
/// missing everywhere: Piranha needs to be told which flags are stale, and
/// Uber's production answer is a weekly job querying the flag system.
pub async fn audit_flag_staleness(
    State(state): State<AppState>,
    auth: AuthUser,
) -> impl IntoResponse {
    if let Err(rejection) = require_admin(&auth) {
        return rejection.into_response();
    }

    let flags = match state.db.list_feature_flags().await {
        Ok(f) => f,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": e.to_string()})),
            )
                .into_response();
        }
    };

    let policy = flag_kit::FlagPolicy::default();
    let views: Vec<civit_db::models::FlagStalenessView> =
        flags.iter().map(|f| staleness_view(f, &policy)).collect();

    let removal_candidates = views.iter().filter(|v| v.is_removal_candidate).count();
    let aging = views.iter().filter(|v| v.staleness == "aging").count();
    let permanent = views.iter().filter(|v| v.staleness == "permanent").count();

    Json(serde_json::json!({
        "flags": views,
        "total": views.len(),
        "removal_candidates": removal_candidates,
        "aging": aging,
        "permanent": permanent,
    }))
    .into_response()
}

pub fn feature_flag_routes() -> Router<AppState> {
    Router::new()
        .route("/api/v1/feature-flags", get(list_feature_flags_for_user))
        .route(
            "/api/v1/admin/feature-flags",
            get(list_all_feature_flags).post(create_feature_flag),
        )
        .route(
            "/api/v1/admin/feature-flags/stale",
            get(audit_flag_staleness),
        )
        .route(
            "/api/v1/admin/feature-flags/{id}",
            put(update_feature_flag).delete(delete_feature_flag),
        )
        .route(
            "/api/v1/admin/feature-flags/{id}/toggle",
            post(toggle_feature_flag),
        )
        .route(
            "/api/v1/admin/feature-flags/{id}/users",
            post(add_feature_flag_user),
        )
        .route(
            "/api/v1/admin/feature-flags/{id}/users/{user_id}",
            delete(remove_feature_flag_user),
        )
        .route(
            "/api/v1/admin/feature-flags/{id}/orgs",
            post(add_feature_flag_org),
        )
        .route(
            "/api/v1/admin/feature-flags/{id}/orgs/{org_id}",
            delete(remove_feature_flag_org),
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    use chrono::Duration;

    fn db_flag(
        name: &str,
        kind: &str,
        percentage: i32,
        age_days: i64,
        evaluated: bool,
    ) -> civit_db::models::FeatureFlag {
        let created = chrono::Utc::now() - Duration::days(age_days);
        civit_db::models::FeatureFlag {
            id: Uuid::nil(),
            name: name.into(),
            description: "d".into(),
            enabled: true,
            enabled_for_users: Vec::new(),
            enabled_for_percentage: percentage,
            enabled_for_orgs: Vec::new(),
            created_at: created,
            updated_at: created,
            kind: kind.into(),
            owner: "team".into(),
            ticket: "T-1".into(),
            salt: String::new(),
            last_evaluated_at: evaluated.then(|| chrono::Utc::now() - Duration::days(1)),
            last_changed_at: created,
        }
    }

    /// The four lifecycle cases, matching the rows verified against a live
    /// Postgres. A stale verdict with the wrong evidence is worse than no
    /// verdict, so the signals are asserted too.
    #[test]
    fn staleness_view_classifies_the_four_lifecycle_cases() {
        let policy = flag_kit::FlagPolicy::default();

        let done = staleness_view(
            &db_flag("fully_rolled_old", "release", 100, 400, true),
            &policy,
        );
        assert_eq!(done.staleness, "stale");
        assert!(done.is_removal_candidate);
        assert!(
            done.signals.iter().any(|s| s.signal == "fully_rolled_out"),
            "rolled out is the actionable signal here: {:?}",
            done.signals
        );

        let halfway = staleness_view(&db_flag("half_way", "release", 10, 16, true), &policy);
        assert_eq!(halfway.staleness, "aging");
        assert!(!halfway.is_removal_candidate);
        assert!(halfway.signals.is_empty(), "aging fires no stale signal");

        let kill = staleness_view(&db_flag("kill_switch", "permission", 0, 900, true), &policy);
        assert_eq!(kill.staleness, "permanent");
        assert!(!kill.is_removal_candidate);

        let unused = staleness_view(&db_flag("never_used", "release", 0, 5, false), &policy);
        assert_eq!(unused.staleness, "stale");
        assert!(unused.signals.iter().any(|s| s.signal == "never_evaluated"));
    }

    /// Rollout evidence must be reported even for a young flag: that is the
    /// difference between "old" and "finished".
    #[test]
    fn fully_rolled_out_is_stale_regardless_of_age() {
        let v = staleness_view(
            &db_flag("brand_new", "release", 100, 1, true),
            &flag_kit::FlagPolicy::default(),
        );
        assert!(v.is_removal_candidate);
    }

    /// A flag still being edited has not finished its life.
    #[test]
    fn last_change_restarts_the_staleness_clock() {
        let mut f = db_flag("actively_edited", "release", 10, 400, true);
        f.last_changed_at = chrono::Utc::now() - Duration::days(2);
        let v = staleness_view(&f, &flag_kit::FlagPolicy::default());
        assert_eq!(v.staleness, "fresh");
        assert!(v.age_days > 300, "age is still reported for visibility");
        assert_eq!(v.last_changed_age_days, Some(2));
    }

    /// An unparseable kind must not silently exempt the flag.
    #[test]
    fn unknown_kind_falls_back_to_a_short_deadline() {
        let v = staleness_view(
            &db_flag("weird_kind", "not_a_kind", 0, 60, true),
            &flag_kit::FlagPolicy::default(),
        );
        assert_eq!(v.kind, "release");
        assert!(v.is_removal_candidate);
    }

    /// Out-of-range percentages are clamped, not wrapped.
    #[test]
    fn percentage_is_clamped() {
        let v = staleness_view(
            &db_flag("over", "release", 250, 1, true),
            &flag_kit::FlagPolicy::default(),
        );
        assert_eq!(v.percentage, 250, "raw value is reported as stored");
        assert!(v.is_removal_candidate, "250 is certainly fully rolled out");
    }

    #[test]
    fn test_flag_response_serialization() {
        let response = FeatureFlagResponse {
            id: "test-id".into(),
            name: "dark-mode".into(),
            description: "Enable dark mode".into(),
            enabled: true,
            enabled_for_users: vec!["user1".into()],
            enabled_for_percentage: 50,
            enabled_for_orgs: vec!["org1".into()],
            created_at: "2024-01-01T00:00:00Z".into(),
            updated_at: "2024-01-01T00:00:00Z".into(),
            kind: "release".into(),
            owner: "core".into(),
            ticket: "CF-1".into(),
            salt: String::new(),
            last_evaluated_at: Some("2024-01-02T00:00:00Z".into()),
            last_changed_at: "2024-01-01T00:00:00Z".into(),
        };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"dark-mode\""));
        assert!(json.contains("\"enabled\":true"));
    }

    #[test]
    fn test_create_request_deserialization() {
        let json = r#"{"name": "new-feature", "description": "A new feature", "enabled": false, "enabled_for_percentage": 0}"#;
        let req: CreateFeatureFlagRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.name, "new-feature");
        assert!(!req.enabled);
    }

    #[test]
    fn test_update_request_deserialization() {
        let json = r#"{"enabled": true, "enabled_for_percentage": 75}"#;
        let req: UpdateFeatureFlagRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.enabled, Some(true));
        assert_eq!(req.enabled_for_percentage, Some(75));
    }
}
