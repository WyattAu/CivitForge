#![forbid(unsafe_code)]

use crate::api::AppState;
use crate::error::CoreError;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Repo {
    pub id: String,
    pub name: String,
    pub owner: String,
    pub description: String,
    pub visibility: Visibility,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Visibility {
    Public,
    Private,
    Internal,
}

#[derive(Debug, Deserialize)]
pub struct CreateRepoRequest {
    pub name: String,
    pub owner: String,
    pub description: String,
    pub visibility: Visibility,
}

#[derive(Debug, Deserialize)]
pub struct PaginationParams {
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default = "default_offset")]
    pub offset: i64,
}

fn default_limit() -> i64 {
    50
}
fn default_offset() -> i64 {
    0
}

impl From<crate::db::Repository> for Repo {
    fn from(r: crate::db::Repository) -> Self {
        Self {
            id: r.id.to_string(),
            name: r.name,
            owner: r.owner_id.to_string(),
            description: r.description,
            visibility: Visibility::from_str(&r.visibility),
            created_at: r.created_at.to_rfc3339(),
            updated_at: r.updated_at.to_rfc3339(),
        }
    }
}

impl Visibility {
    fn from_str(s: &str) -> Self {
        match s {
            "public" => Visibility::Public,
            "internal" => Visibility::Internal,
            _ => Visibility::Private,
        }
    }

    fn as_str(&self) -> &str {
        match self {
            Visibility::Public => "public",
            Visibility::Private => "private",
            Visibility::Internal => "internal",
        }
    }
}

pub async fn list_repos(
    State(state): State<AppState>,
    Query(params): Query<PaginationParams>,
) -> impl IntoResponse {
    match state.db.list_repos(params.limit, params.offset).await {
        Ok(repos) => {
            let out: Vec<Repo> = repos.into_iter().map(Into::into).collect();
            (StatusCode::OK, Json(out)).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub async fn get_repo(
    State(state): State<AppState>,
    Path((owner, name)): Path<(String, String)>,
) -> impl IntoResponse {
    // Resolve owner string to UUID (username lookup)
    let owner_uuid = match Uuid::parse_str(&owner) {
        Ok(id) => id,
        Err(_) => {
            // Try username lookup
            match state.db.get_user_by_username(&owner).await {
                Ok(user) => user.id,
                Err(_) => {
                    return (
                        StatusCode::NOT_FOUND,
                        Json(CoreError::NotFound("user not found".into()).error_response()),
                    )
                        .into_response();
                }
            }
        }
    };

    match state.db.get_repo_by_owner_name(owner_uuid, &name).await {
        Ok(repo) => (StatusCode::OK, Json(Repo::from(repo))).into_response(),
        Err(_) => (
            StatusCode::NOT_FOUND,
            Json(CoreError::NotFound("repository not found".into()).error_response()),
        )
            .into_response(),
    }
}

pub async fn create_repo(
    State(state): State<AppState>,
    Json(req): Json<CreateRepoRequest>,
) -> impl IntoResponse {
    if req.name.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(CoreError::Config("name required".into()).error_response()),
        )
            .into_response();
    }
    if !req
        .name
        .chars()
        .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
    {
        return (
            StatusCode::BAD_REQUEST,
            Json(CoreError::Config("invalid repo name".into()).error_response()),
        )
            .into_response();
    }

    let owner_uuid = match Uuid::parse_str(&req.owner) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(CoreError::Config("owner must be a valid UUID".into()).error_response()),
            )
                .into_response();
        }
    };

    match state
        .db
        .create_repo(
            &req.name,
            &req.description,
            owner_uuid,
            None,
            req.visibility.as_str(),
            "main",
        )
        .await
    {
        Ok(repo) => (StatusCode::CREATED, Json(Repo::from(repo))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub async fn delete_repo(
    State(state): State<AppState>,
    Path((owner, name)): Path<(String, String)>,
) -> impl IntoResponse {
    let owner_uuid = match Uuid::parse_str(&owner) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(CoreError::Config("owner must be a valid UUID".into()).error_response()),
            )
                .into_response();
        }
    };

    match state.db.get_repo_by_owner_name(owner_uuid, &name).await {
        Ok(repo) => {
            if let Err(e) = state.db.delete_repo(repo.id).await {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(CoreError::Database(e.to_string()).error_response()),
                )
                    .into_response();
            }
            (StatusCode::NO_CONTENT, ()).into_response()
        }
        Err(_) => (
            StatusCode::NOT_FOUND,
            Json(CoreError::NotFound("repository not found".into()).error_response()),
        )
            .into_response(),
    }
}

pub async fn list_commits(
    State(state): State<AppState>,
    Path((owner, name)): Path<(String, String)>,
) -> impl IntoResponse {
    let owner_uuid = match Uuid::parse_str(&owner) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(CoreError::Config("owner must be a valid UUID".into()).error_response()),
            )
                .into_response();
        }
    };

    match state.db.get_repo_by_owner_name(owner_uuid, &name).await {
        Ok(_) => {
            let git_svc =
                crate::git::GitService::new(std::path::PathBuf::from(&state.config.storage_path));
            // Storage path will be configurable; for now return real git walk
            // or empty if repo not yet cloned
            match git_svc.list_commits(&owner, &name, 50) {
                Ok(commits) => (StatusCode::OK, Json(commits)).into_response(),
                Err(e) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(CoreError::Git(e.to_string()).error_response()),
                )
                    .into_response(),
            }
        }
        Err(_) => (
            StatusCode::NOT_FOUND,
            Json(CoreError::NotFound("repository not found".into()).error_response()),
        )
            .into_response(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_visibility_serialization() {
        assert_eq!(
            serde_json::to_string(&Visibility::Public).unwrap(),
            "\"public\""
        );
        assert_eq!(
            serde_json::to_string(&Visibility::Private).unwrap(),
            "\"private\""
        );
        assert_eq!(
            serde_json::to_string(&Visibility::Internal).unwrap(),
            "\"internal\""
        );
    }

    #[test]
    fn test_visibility_deserialization() {
        let v: Visibility = serde_json::from_str("\"public\"").unwrap();
        assert_eq!(v, Visibility::Public);
        let v: Visibility = serde_json::from_str("\"private\"").unwrap();
        assert_eq!(v, Visibility::Private);
    }

    #[test]
    fn test_create_repo_request_parse() {
        let json =
            r#"{"name":"my-repo","owner":"myorg","description":"A repo","visibility":"public"}"#;
        let req: CreateRepoRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.name, "my-repo");
        assert_eq!(req.owner, "myorg");
        assert_eq!(req.visibility, Visibility::Public);
    }

    #[test]
    fn test_repo_serialization() {
        let repo = Repo {
            id: "test-id".into(),
            name: "test".into(),
            owner: "owner".into(),
            description: "desc".into(),
            visibility: Visibility::Private,
            created_at: "2025-01-01T00:00:00Z".into(),
            updated_at: "2025-01-01T00:00:00Z".into(),
        };
        let json = serde_json::to_string(&repo).unwrap();
        assert!(json.contains("test-id"));
        assert!(json.contains("private"));
    }

    #[test]
    fn test_visibility_from_str() {
        assert_eq!(Visibility::from_str("public"), Visibility::Public);
        assert_eq!(Visibility::from_str("private"), Visibility::Private);
        assert_eq!(Visibility::from_str("internal"), Visibility::Internal);
        assert_eq!(Visibility::from_str("unknown"), Visibility::Private);
    }

    #[test]
    fn test_visibility_as_str() {
        assert_eq!(Visibility::Public.as_str(), "public");
        assert_eq!(Visibility::Private.as_str(), "private");
        assert_eq!(Visibility::Internal.as_str(), "internal");
    }

    #[test]
    fn test_pagination_defaults() {
        assert_eq!(default_limit(), 50);
        assert_eq!(default_offset(), 0);
    }

    #[test]
    fn test_pagination_params_deserialize() {
        let p: PaginationParams = serde_json::from_str("{}").unwrap();
        assert_eq!(p.limit, 50);
        assert_eq!(p.offset, 0);
        let p: PaginationParams = serde_json::from_str(r#"{"limit":10,"offset":20}"#).unwrap();
        assert_eq!(p.limit, 10);
        assert_eq!(p.offset, 20);
    }
}
