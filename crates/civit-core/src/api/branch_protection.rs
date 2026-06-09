#![forbid(unsafe_code)]

use crate::api::AppState;
use crate::api::auth::AuthUser;
use crate::error::CoreError;
use axum::Router;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Json};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// --- Request/Response Types ---

#[derive(Debug, Deserialize)]
pub struct UpsertBranchProtectionRequest {
    pub branch_pattern: String,
    pub require_pull_request: Option<bool>,
    pub required_approving_reviews: Option<i32>,
    pub required_status_checks: Option<Vec<String>>,
    pub enforce_admins: Option<bool>,
    pub allow_force_pushes: Option<bool>,
    pub allow_deletions: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct BranchProtectionResponse {
    pub id: String,
    pub repo_id: String,
    pub branch_pattern: String,
    pub require_pull_request: bool,
    pub required_approving_reviews: i32,
    pub required_status_checks: Vec<String>,
    pub enforce_admins: bool,
    pub allow_force_pushes: bool,
    pub allow_deletions: bool,
    pub created_at: String,
    pub updated_at: String,
}

// --- Helpers ---

fn err_response(e: CoreError) -> axum::response::Response {
    let status = e.status_code();
    let body = e.error_response();
    (status, Json(body)).into_response()
}

async fn get_repo_id(pool: &sqlx::PgPool, owner: &str, name: &str) -> Result<Uuid, CoreError> {
    sqlx::query_scalar::<_, Uuid>(
        "SELECT r.id FROM repositories r JOIN users u ON r.owner_id = u.id WHERE u.username = $1 AND r.name = $2",
    )
    .bind(owner)
    .bind(name)
    .fetch_one(pool)
    .await
    .map_err(|_| CoreError::NotFound(format!("repo {owner}/{name}")))
}

// --- Handlers ---

pub async fn get_branch_protection(
    State(state): State<AppState>,
    Path((owner, name)): Path<(String, String)>,
    _auth: AuthUser,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let repo_id = match get_repo_id(pool, &owner, &name).await {
        Ok(id) => id,
        Err(e) => return err_response(e),
    };

    match state.db.get_branch_protection(repo_id).await {
        Ok(rules) => {
            let items: Vec<BranchProtectionResponse> = rules
                .into_iter()
                .map(|r| BranchProtectionResponse {
                    id: r.id.to_string(),
                    repo_id: r.repo_id.to_string(),
                    branch_pattern: r.branch_pattern,
                    require_pull_request: r.require_pull_request,
                    required_approving_reviews: r.required_approving_reviews,
                    required_status_checks: r.required_status_checks,
                    enforce_admins: r.enforce_admins,
                    allow_force_pushes: r.allow_force_pushes,
                    allow_deletions: r.allow_deletions,
                    created_at: r.created_at.to_rfc3339(),
                    updated_at: r.updated_at.to_rfc3339(),
                })
                .collect();
            (StatusCode::OK, Json(items)).into_response()
        }
        Err(e) => err_response(e),
    }
}

pub async fn set_branch_protection(
    State(state): State<AppState>,
    Path((owner, name)): Path<(String, String)>,
    _auth: AuthUser,
    Json(req): Json<UpsertBranchProtectionRequest>,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let repo_id = match get_repo_id(pool, &owner, &name).await {
        Ok(id) => id,
        Err(e) => return err_response(e),
    };

    if req.branch_pattern.trim().is_empty() {
        return err_response(CoreError::BadRequest("branch_pattern is required".into()));
    }

    match state
        .db
        .upsert_branch_protection(
            repo_id,
            &req.branch_pattern,
            req.require_pull_request.unwrap_or(false),
            req.required_approving_reviews.unwrap_or(0),
            &req.required_status_checks.clone().unwrap_or_default(),
            req.enforce_admins.unwrap_or(false),
            req.allow_force_pushes.unwrap_or(false),
            req.allow_deletions.unwrap_or(false),
        )
        .await
    {
        Ok(rule) => {
            let resp = BranchProtectionResponse {
                id: rule.id.to_string(),
                repo_id: rule.repo_id.to_string(),
                branch_pattern: rule.branch_pattern,
                require_pull_request: rule.require_pull_request,
                required_approving_reviews: rule.required_approving_reviews,
                required_status_checks: rule.required_status_checks,
                enforce_admins: rule.enforce_admins,
                allow_force_pushes: rule.allow_force_pushes,
                allow_deletions: rule.allow_deletions,
                created_at: rule.created_at.to_rfc3339(),
                updated_at: rule.updated_at.to_rfc3339(),
            };
            (StatusCode::OK, Json(resp)).into_response()
        }
        Err(e) => err_response(e),
    }
}

// --- Router ---

pub fn branch_protection_routes() -> Router<AppState> {
    use axum::routing::get;

    Router::new().route(
        "/api/v1/repos/{owner}/{name}/branch-protection",
        get(get_branch_protection).put(set_branch_protection),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_branch_protection_response_serialization() {
        let resp = BranchProtectionResponse {
            id: "00000000-0000-0000-0000-000000000001".into(),
            repo_id: "00000000-0000-0000-0000-000000000002".into(),
            branch_pattern: "main".into(),
            require_pull_request: true,
            required_approving_reviews: 2,
            required_status_checks: vec!["ci/test".into()],
            enforce_admins: true,
            allow_force_pushes: false,
            allow_deletions: false,
            created_at: "2024-01-01T00:00:00Z".into(),
            updated_at: "2024-01-01T00:00:00Z".into(),
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("main"));
        assert!(json.contains("require_pull_request"));
        assert!(json.contains("enforce_admins"));
    }

    #[test]
    fn test_upsert_request_parse() {
        let req: UpsertBranchProtectionRequest = serde_json::from_str(
            r#"{"branch_pattern":"main","require_pull_request":true,"required_approving_reviews":1,"enforce_admins":false}"#,
        )
        .unwrap();
        assert_eq!(req.branch_pattern, "main");
        assert!(req.require_pull_request.unwrap_or(false));
        assert_eq!(req.required_approving_reviews, Some(1));
    }

    #[test]
    fn test_branch_protection_routes_compile() {
        let router = branch_protection_routes();
        let _ = router;
    }
}
