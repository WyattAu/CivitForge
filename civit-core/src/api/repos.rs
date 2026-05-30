#![forbid(unsafe_code)]

use crate::api::AppState;
use crate::error::CoreError;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use chrono::Utc;
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

pub async fn list_repos(State(_state): State<AppState>) -> impl IntoResponse {
    let repos = vec![Repo {
        id: Uuid::new_v4().to_string(),
        name: "example-repo".into(),
        owner: "example-org".into(),
        description: "An example repository".into(),
        visibility: Visibility::Public,
        created_at: Utc::now().to_rfc3339(),
        updated_at: Utc::now().to_rfc3339(),
    }];
    (StatusCode::OK, Json(repos)).into_response()
}

pub async fn get_repo(
    State(_state): State<AppState>,
    Path((owner, name)): Path<(String, String)>,
) -> impl IntoResponse {
    let repo = Repo {
        id: Uuid::new_v4().to_string(),
        name: name.clone(),
        owner,
        description: "Repository details".into(),
        visibility: Visibility::Public,
        created_at: Utc::now().to_rfc3339(),
        updated_at: Utc::now().to_rfc3339(),
    };
    (StatusCode::OK, Json(repo)).into_response()
}

pub async fn create_repo(
    State(_state): State<AppState>,
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
    let repo = Repo {
        id: Uuid::new_v4().to_string(),
        name: req.name,
        owner: req.owner,
        description: req.description,
        visibility: req.visibility,
        created_at: Utc::now().to_rfc3339(),
        updated_at: Utc::now().to_rfc3339(),
    };
    (StatusCode::CREATED, Json(repo)).into_response()
}

pub async fn delete_repo(
    State(_state): State<AppState>,
    Path((_owner, _name)): Path<(String, String)>,
) -> impl IntoResponse {
    tracing::info!("deleting repo");
    (StatusCode::NO_CONTENT, ()).into_response()
}

pub async fn list_commits(
    State(_state): State<AppState>,
    Path((_owner, _name)): Path<(String, String)>,
) -> impl IntoResponse {
    use crate::git::CommitInfo;
    let commits = vec![
        CommitInfo {
            id: "abc123".into(),
            message: "Initial commit".into(),
            author: "alice".into(),
            timestamp: "2025-01-01 00:00:00".into(),
            parents: vec![],
        },
        CommitInfo {
            id: "def456".into(),
            message: "Add feature X".into(),
            author: "bob".into(),
            timestamp: "2025-01-02 00:00:00".into(),
            parents: vec!["abc123".into()],
        },
    ];
    (StatusCode::OK, Json(commits)).into_response()
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
}
