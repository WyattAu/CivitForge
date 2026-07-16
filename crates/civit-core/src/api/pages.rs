#![forbid(unsafe_code)]

use crate::api::AppState;
use crate::api::auth::{AuthUser, OptionalAuthUser};
use crate::error::CoreError;
use axum::{
    Router,
    extract::{Path, Query, State},
    response::{IntoResponse, Json},
    routing::{get, patch, post},
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct EnablePagesRequest {
    pub url: String,
    #[serde(default = "default_branch")]
    pub branch: String,
    #[serde(default = "default_path")]
    pub path: String,
    #[serde(default = "default_true")]
    pub public: bool,
}

fn default_branch() -> String {
    "main".into()
}

fn default_path() -> String {
    "/".into()
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Deserialize)]
pub struct UpdateCustomDomainRequest {
    pub custom_domain: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct TriggerBuildRequest {
    pub sha: String,
}

#[derive(Debug, Deserialize)]
pub struct ListDeploymentsParams {
    #[serde(default = "default_page")]
    pub page: u32,
    #[serde(default = "default_per_page")]
    pub per_page: u32,
}

fn default_page() -> u32 {
    1
}
fn default_per_page() -> u32 {
    20
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct PagesSiteResponse {
    pub id: Uuid,
    pub repo_id: Uuid,
    pub url: String,
    pub branch: String,
    pub path: String,
    pub public: bool,
    pub custom_domain: Option<String>,
    pub https_enabled: bool,
    pub last_built_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct PagesDeploymentResponse {
    pub id: Uuid,
    pub site_id: Uuid,
    pub sha: String,
    pub url: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

fn err_response(status: axum::http::StatusCode, msg: &str) -> axum::response::Response {
    (
        status,
        Json(CoreError::NotFound(msg.to_string()).error_response()),
    )
        .into_response()
}

fn internal_err(msg: &str) -> axum::response::Response {
    (
        axum::http::StatusCode::INTERNAL_SERVER_ERROR,
        Json(CoreError::Database(msg.to_string()).error_response()),
    )
        .into_response()
}

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

pub async fn enable_pages(
    State(state): State<AppState>,
    Path((owner, name)): Path<(String, String)>,
    auth: AuthUser,
    Json(req): Json<EnablePagesRequest>,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let repo_id = match get_repo_id(pool, &owner, &name).await {
        Some(id) => id,
        None => {
            return err_response(
                axum::http::StatusCode::NOT_FOUND,
                &format!("repository {owner}/{name} not found"),
            );
        }
    };

    if req.url.trim().is_empty() {
        return err_response(axum::http::StatusCode::BAD_REQUEST, "url is required");
    }

    let _auth_id = uuid::Uuid::parse_str(&auth.user_id).unwrap_or(uuid::Uuid::nil());

    match state
        .db
        .enable_pages(repo_id, &req.url, &req.branch, &req.path, req.public)
        .await
    {
        Ok(site) => {
            let resp = PagesSiteResponse {
                id: site.id,
                repo_id: site.repo_id,
                url: site.url,
                branch: site.branch,
                path: site.path,
                public: site.public,
                custom_domain: site.custom_domain,
                https_enabled: site.https_enabled,
                last_built_at: site.last_built_at,
                created_at: site.created_at,
                updated_at: site.updated_at,
            };
            (axum::http::StatusCode::CREATED, Json(resp)).into_response()
        }
        Err(e) => internal_err(&e.to_string()),
    }
}

pub async fn disable_pages(
    State(state): State<AppState>,
    Path((owner, name)): Path<(String, String)>,
    _auth: AuthUser,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let repo_id = match get_repo_id(pool, &owner, &name).await {
        Some(id) => id,
        None => {
            return err_response(
                axum::http::StatusCode::NOT_FOUND,
                &format!("repository {owner}/{name} not found"),
            );
        }
    };

    match state.db.disable_pages(repo_id).await {
        Ok(()) => (axum::http::StatusCode::NO_CONTENT, ()).into_response(),
        Err(e) => internal_err(&e.to_string()),
    }
}

pub async fn get_pages(
    State(state): State<AppState>,
    Path((owner, name)): Path<(String, String)>,
    _auth: OptionalAuthUser,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let repo_id = match get_repo_id(pool, &owner, &name).await {
        Some(id) => id,
        None => {
            return err_response(
                axum::http::StatusCode::NOT_FOUND,
                &format!("repository {owner}/{name} not found"),
            );
        }
    };

    match state.db.get_pages_site(repo_id).await {
        Ok(Some(site)) => {
            let resp = PagesSiteResponse {
                id: site.id,
                repo_id: site.repo_id,
                url: site.url,
                branch: site.branch,
                path: site.path,
                public: site.public,
                custom_domain: site.custom_domain,
                https_enabled: site.https_enabled,
                last_built_at: site.last_built_at,
                created_at: site.created_at,
                updated_at: site.updated_at,
            };
            (axum::http::StatusCode::OK, Json(resp)).into_response()
        }
        Ok(None) => err_response(
            axum::http::StatusCode::NOT_FOUND,
            &format!("Pages not enabled for {owner}/{name}"),
        ),
        Err(e) => internal_err(&e.to_string()),
    }
}

pub async fn update_custom_domain(
    State(state): State<AppState>,
    Path((owner, name)): Path<(String, String)>,
    _auth: AuthUser,
    Json(req): Json<UpdateCustomDomainRequest>,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let repo_id = match get_repo_id(pool, &owner, &name).await {
        Some(id) => id,
        None => {
            return err_response(
                axum::http::StatusCode::NOT_FOUND,
                &format!("repository {owner}/{name} not found"),
            );
        }
    };

    // Verify pages is enabled
    match state.db.get_pages_site(repo_id).await {
        Ok(Some(_)) => {}
        Ok(None) => {
            return err_response(
                axum::http::StatusCode::NOT_FOUND,
                &format!("Pages not enabled for {owner}/{name}"),
            );
        }
        Err(e) => return internal_err(&e.to_string()),
    }

    // When setting a custom domain, enable HTTPS; when clearing, disable it
    let https_enabled = req.custom_domain.is_some();

    match state
        .db
        .update_pages_custom_domain(repo_id, req.custom_domain.as_deref(), https_enabled)
        .await
    {
        Ok(site) => {
            let resp = PagesSiteResponse {
                id: site.id,
                repo_id: site.repo_id,
                url: site.url,
                branch: site.branch,
                path: site.path,
                public: site.public,
                custom_domain: site.custom_domain,
                https_enabled: site.https_enabled,
                last_built_at: site.last_built_at,
                created_at: site.created_at,
                updated_at: site.updated_at,
            };
            (axum::http::StatusCode::OK, Json(resp)).into_response()
        }
        Err(e) => internal_err(&e.to_string()),
    }
}

pub async fn trigger_build(
    State(state): State<AppState>,
    Path((owner, name)): Path<(String, String)>,
    _auth: AuthUser,
    Json(req): Json<TriggerBuildRequest>,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let repo_id = match get_repo_id(pool, &owner, &name).await {
        Some(id) => id,
        None => {
            return err_response(
                axum::http::StatusCode::NOT_FOUND,
                &format!("repository {owner}/{name} not found"),
            );
        }
    };

    let site = match state.db.get_pages_site(repo_id).await {
        Ok(Some(site)) => site,
        Ok(None) => {
            return err_response(
                axum::http::StatusCode::NOT_FOUND,
                &format!("Pages not enabled for {owner}/{name}"),
            );
        }
        Err(e) => return internal_err(&e.to_string()),
    };

    let build_url = format!("{}/{}", site.url.trim_end_matches('/'), req.sha);

    match state
        .db
        .create_pages_deployment(site.id, &req.sha, &build_url)
        .await
    {
        Ok(deployment) => {
            let _ = state.db.update_pages_last_built(site.id).await;
            let resp = PagesDeploymentResponse {
                id: deployment.id,
                site_id: deployment.site_id,
                sha: deployment.sha,
                url: deployment.url,
                status: deployment.status,
                created_at: deployment.created_at,
            };
            (axum::http::StatusCode::CREATED, Json(resp)).into_response()
        }
        Err(e) => internal_err(&e.to_string()),
    }
}

pub async fn list_deployments(
    State(state): State<AppState>,
    Path((owner, name)): Path<(String, String)>,
    _auth: AuthUser,
    Query(params): Query<ListDeploymentsParams>,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let repo_id = match get_repo_id(pool, &owner, &name).await {
        Some(id) => id,
        None => {
            return err_response(
                axum::http::StatusCode::NOT_FOUND,
                &format!("repository {owner}/{name} not found"),
            );
        }
    };

    let site = match state.db.get_pages_site(repo_id).await {
        Ok(Some(site)) => site,
        Ok(None) => {
            return err_response(
                axum::http::StatusCode::NOT_FOUND,
                &format!("Pages not enabled for {owner}/{name}"),
            );
        }
        Err(e) => return internal_err(&e.to_string()),
    };

    let offset = (params.page.saturating_sub(1) * params.per_page) as i64;

    match state
        .db
        .list_pages_deployments(site.id, params.per_page as i64, offset)
        .await
    {
        Ok(deployments) => {
            let resp: Vec<PagesDeploymentResponse> = deployments
                .into_iter()
                .map(|d| PagesDeploymentResponse {
                    id: d.id,
                    site_id: d.site_id,
                    sha: d.sha,
                    url: d.url,
                    status: d.status,
                    created_at: d.created_at,
                })
                .collect();
            (axum::http::StatusCode::OK, Json(resp)).into_response()
        }
        Err(e) => internal_err(&e.to_string()),
    }
}

pub async fn update_deployment_status(
    State(state): State<AppState>,
    Path((owner, name, deployment_id)): Path<(String, String, String)>,
    _auth: AuthUser,
    Json(req): Json<TriggerBuildRequest>,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let _repo_id = match get_repo_id(pool, &owner, &name).await {
        Some(id) => id,
        None => {
            return err_response(
                axum::http::StatusCode::NOT_FOUND,
                &format!("repository {owner}/{name} not found"),
            );
        }
    };

    let did = match Uuid::parse_str(&deployment_id) {
        Ok(id) => id,
        Err(_) => {
            return err_response(axum::http::StatusCode::BAD_REQUEST, "invalid deployment ID");
        }
    };

    let valid_statuses = ["pending", "in_progress", "success", "failure"];
    if !valid_statuses.contains(&req.sha.as_str()) {
        return err_response(
            axum::http::StatusCode::BAD_REQUEST,
            &format!(
                "invalid status: must be one of {}",
                valid_statuses.join(", ")
            ),
        );
    }

    match state
        .db
        .update_pages_deployment_status(did, &req.sha)
        .await
    {
        Ok(deployment) => {
            let resp = PagesDeploymentResponse {
                id: deployment.id,
                site_id: deployment.site_id,
                sha: deployment.sha,
                url: deployment.url,
                status: deployment.status,
                created_at: deployment.created_at,
            };
            (axum::http::StatusCode::OK, Json(resp)).into_response()
        }
        Err(e) => {
            let msg = e.to_string();
            if msg.contains("returned no rows") {
                err_response(axum::http::StatusCode::NOT_FOUND, "deployment not found")
            } else {
                internal_err(&msg)
            }
        }
    }
}

pub fn pages_routes() -> Router<AppState> {
    Router::new()
        .route(
            "/api/v1/repos/{owner}/{name}/pages",
            get(get_pages)
                .post(enable_pages)
                .delete(disable_pages),
        )
        .route(
            "/api/v1/repos/{owner}/{name}/pages/custom-domain",
            patch(update_custom_domain),
        )
        .route(
            "/api/v1/repos/{owner}/{name}/pages/build",
            post(trigger_build),
        )
        .route(
            "/api/v1/repos/{owner}/{name}/pages/deployments",
            get(list_deployments),
        )
        .route(
            "/api/v1/repos/{owner}/{name}/pages/deployments/{deployment_id}/status",
            patch(update_deployment_status),
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enable_pages_request_deserialize() {
        let json = r#"{"url":"https://example.pages.dev","branch":"main","path":"/docs","public":true}"#;
        let req: EnablePagesRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.url, "https://example.pages.dev");
        assert_eq!(req.branch, "main");
        assert_eq!(req.path, "/docs");
        assert!(req.public);
    }

    #[test]
    fn test_enable_pages_request_defaults() {
        let json = r#"{"url":"https://example.pages.dev"}"#;
        let req: EnablePagesRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.branch, "main");
        assert_eq!(req.path, "/");
        assert!(req.public);
    }

    #[test]
    fn test_update_custom_domain_request() {
        let json = r#"{"custom_domain":"blog.example.com"}"#;
        let req: UpdateCustomDomainRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.custom_domain.as_deref(), Some("blog.example.com"));
    }

    #[test]
    fn test_update_custom_domain_clear() {
        let json = r#"{"custom_domain":null}"#;
        let req: UpdateCustomDomainRequest = serde_json::from_str(json).unwrap();
        assert!(req.custom_domain.is_none());
    }

    #[test]
    fn test_trigger_build_request() {
        let json = r#"{"sha":"abc123"}"#;
        let req: TriggerBuildRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.sha, "abc123");
    }

    #[test]
    fn test_list_deployments_params_defaults() {
        let json = r#"{}"#;
        let params: ListDeploymentsParams = serde_json::from_str(json).unwrap();
        assert_eq!(params.page, 1);
        assert_eq!(params.per_page, 20);
    }
}
