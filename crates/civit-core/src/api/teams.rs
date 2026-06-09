#![forbid(unsafe_code)]

use crate::api::AppState;
use crate::api::auth::{AuthUser, OptionalAuthUser, require_permission};
use crate::error::CoreError;
use axum::{
    Router,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::{delete, get},
};
use civit_shared::{
    ListResponse,
    permissions::{Action, Resource},
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize)]
pub struct TeamResponse {
    pub id: String,
    pub org_id: String,
    pub name: String,
    pub description: String,
    pub privacy: String,
    pub created_at: String,
    pub updated_at: String,
}

impl From<crate::db::Team> for TeamResponse {
    fn from(t: crate::db::Team) -> Self {
        Self {
            id: t.id.to_string(),
            org_id: t.org_id.to_string(),
            name: t.name,
            description: t.description,
            privacy: t.privacy,
            created_at: t.created_at.to_rfc3339(),
            updated_at: t.updated_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct TeamMemberResponse {
    pub team_id: String,
    pub user_id: String,
    pub role: String,
    pub joined_at: String,
}

impl From<crate::db::TeamMember> for TeamMemberResponse {
    fn from(m: crate::db::TeamMember) -> Self {
        Self {
            team_id: m.team_id.to_string(),
            user_id: m.user_id.to_string(),
            role: m.role,
            joined_at: m.joined_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateTeamRequest {
    pub name: String,
    pub description: Option<String>,
    pub privacy: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateTeamRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub privacy: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AddMemberRequest {
    pub user_id: String,
    pub role: Option<String>,
}

pub async fn list_teams(
    State(state): State<AppState>,
    Path(org_id): Path<String>,
    _auth: OptionalAuthUser,
) -> impl IntoResponse {
    let org_uuid = match Uuid::parse_str(&org_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(CoreError::Config("invalid org id".into()).error_response()),
            )
                .into_response();
        }
    };

    match state.db.list_teams(org_uuid).await {
        Ok(teams) => {
            let total = teams.len() as u64;
            let out: Vec<TeamResponse> = teams.into_iter().map(Into::into).collect();
            let resp = ListResponse {
                data: out,
                pagination: civit_shared::Pagination {
                    page: 1,
                    per_page: 100,
                    total,
                    total_pages: 1,
                },
            };
            (StatusCode::OK, Json(resp)).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub async fn create_team(
    State(state): State<AppState>,
    Path(org_id): Path<String>,
    auth: AuthUser,
    Json(req): Json<CreateTeamRequest>,
) -> impl IntoResponse {
    if let Err(rejection) = require_permission(
        &state,
        &auth,
        Resource::Organization,
        Action::Create,
        None,
        None,
        None,
    )
    .await
    {
        return rejection.into_response();
    }

    let org_uuid = match Uuid::parse_str(&org_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(CoreError::Config("invalid org id".into()).error_response()),
            )
                .into_response();
        }
    };

    if req.name.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(CoreError::Config("name required".into()).error_response()),
        )
            .into_response();
    }

    let privacy = req.privacy.as_deref().unwrap_or("visible");

    match state
        .db
        .create_team(
            org_uuid,
            &req.name,
            req.description.as_deref().unwrap_or(""),
            privacy,
        )
        .await
    {
        Ok(team) => (StatusCode::CREATED, Json(TeamResponse::from(team))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub async fn get_team(
    State(state): State<AppState>,
    Path((org_id, team_id)): Path<(String, String)>,
    _auth: OptionalAuthUser,
) -> impl IntoResponse {
    let team_uuid = match Uuid::parse_str(&team_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(CoreError::Config("invalid team id".into()).error_response()),
            )
                .into_response();
        }
    };

    let _org_uuid = match Uuid::parse_str(&org_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(CoreError::Config("invalid org id".into()).error_response()),
            )
                .into_response();
        }
    };

    match state.db.get_team(team_uuid).await {
        Ok(team) => (StatusCode::OK, Json(TeamResponse::from(team))).into_response(),
        Err(_) => (
            StatusCode::NOT_FOUND,
            Json(CoreError::NotFound("team not found".into()).error_response()),
        )
            .into_response(),
    }
}

pub async fn update_team(
    State(state): State<AppState>,
    Path((org_id, team_id)): Path<(String, String)>,
    auth: AuthUser,
    Json(req): Json<UpdateTeamRequest>,
) -> impl IntoResponse {
    if let Err(rejection) = require_permission(
        &state,
        &auth,
        Resource::Organization,
        Action::Update,
        None,
        None,
        None,
    )
    .await
    {
        return rejection.into_response();
    }

    let team_uuid = match Uuid::parse_str(&team_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(CoreError::Config("invalid team id".into()).error_response()),
            )
                .into_response();
        }
    };

    let _org_uuid = match Uuid::parse_str(&org_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(CoreError::Config("invalid org id".into()).error_response()),
            )
                .into_response();
        }
    };

    match state
        .db
        .update_team(
            team_uuid,
            req.name.as_deref(),
            req.description.as_deref(),
            req.privacy.as_deref(),
        )
        .await
    {
        Ok(team) => (StatusCode::OK, Json(TeamResponse::from(team))).into_response(),
        Err(_) => (
            StatusCode::NOT_FOUND,
            Json(CoreError::NotFound("team not found".into()).error_response()),
        )
            .into_response(),
    }
}

pub async fn delete_team(
    State(state): State<AppState>,
    Path((org_id, team_id)): Path<(String, String)>,
    auth: AuthUser,
) -> impl IntoResponse {
    if let Err(rejection) = require_permission(
        &state,
        &auth,
        Resource::Organization,
        Action::Delete,
        None,
        None,
        None,
    )
    .await
    {
        return rejection.into_response();
    }

    let team_uuid = match Uuid::parse_str(&team_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(CoreError::Config("invalid team id".into()).error_response()),
            )
                .into_response();
        }
    };

    let _org_uuid = match Uuid::parse_str(&org_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(CoreError::Config("invalid org id".into()).error_response()),
            )
                .into_response();
        }
    };

    match state.db.delete_team(team_uuid).await {
        Ok(_) => (StatusCode::NO_CONTENT, ()).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub async fn add_member(
    State(state): State<AppState>,
    Path((org_id, team_id)): Path<(String, String)>,
    auth: AuthUser,
    Json(req): Json<AddMemberRequest>,
) -> impl IntoResponse {
    if let Err(rejection) = require_permission(
        &state,
        &auth,
        Resource::Organization,
        Action::Update,
        None,
        None,
        None,
    )
    .await
    {
        return rejection.into_response();
    }

    let team_uuid = match Uuid::parse_str(&team_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(CoreError::Config("invalid team id".into()).error_response()),
            )
                .into_response();
        }
    };

    let _org_uuid = match Uuid::parse_str(&org_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(CoreError::Config("invalid org id".into()).error_response()),
            )
                .into_response();
        }
    };

    let user_uuid = match Uuid::parse_str(&req.user_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(CoreError::Config("invalid user_id".into()).error_response()),
            )
                .into_response();
        }
    };

    let role = req.role.as_deref().unwrap_or("member");

    match state.db.add_team_member(team_uuid, user_uuid, role).await {
        Ok(member) => (StatusCode::CREATED, Json(TeamMemberResponse::from(member))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub async fn remove_member(
    State(state): State<AppState>,
    Path((_org_id, team_id, user_id)): Path<(String, String, String)>,
    auth: AuthUser,
) -> impl IntoResponse {
    if let Err(rejection) = require_permission(
        &state,
        &auth,
        Resource::Organization,
        Action::Update,
        None,
        None,
        None,
    )
    .await
    {
        return rejection.into_response();
    }

    let team_uuid = match Uuid::parse_str(&team_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(CoreError::Config("invalid team id".into()).error_response()),
            )
                .into_response();
        }
    };

    let user_uuid = match Uuid::parse_str(&user_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(CoreError::Config("invalid user_id".into()).error_response()),
            )
                .into_response();
        }
    };

    match state.db.remove_team_member(team_uuid, user_uuid).await {
        Ok(_) => (
            StatusCode::OK,
            Json(serde_json::json!({"status": "removed"})),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub async fn list_members(
    State(state): State<AppState>,
    Path((org_id, team_id)): Path<(String, String)>,
    _auth: OptionalAuthUser,
) -> impl IntoResponse {
    let team_uuid = match Uuid::parse_str(&team_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(CoreError::Config("invalid team id".into()).error_response()),
            )
                .into_response();
        }
    };

    let _org_uuid = match Uuid::parse_str(&org_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(CoreError::Config("invalid org id".into()).error_response()),
            )
                .into_response();
        }
    };

    match state.db.list_team_members(team_uuid).await {
        Ok(members) => {
            let total = members.len() as u64;
            let out: Vec<TeamMemberResponse> = members.into_iter().map(Into::into).collect();
            let resp = ListResponse {
                data: out,
                pagination: civit_shared::Pagination {
                    page: 1,
                    per_page: 100,
                    total,
                    total_pages: 1,
                },
            };
            (StatusCode::OK, Json(resp)).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub fn team_routes() -> Router<AppState> {
    Router::new()
        .route(
            "/api/v1/orgs/{org_id}/teams",
            get(list_teams).post(create_team),
        )
        .route(
            "/api/v1/orgs/{org_id}/teams/{team_id}",
            get(get_team).patch(update_team).delete(delete_team),
        )
        .route(
            "/api/v1/orgs/{org_id}/teams/{team_id}/members",
            get(list_members).post(add_member),
        )
        .route(
            "/api/v1/orgs/{org_id}/teams/{team_id}/members/{user_id}",
            delete(remove_member),
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_team_response_from_db_team() {
        let team = crate::db::Team {
            id: Uuid::nil(),
            org_id: Uuid::nil(),
            name: "backend".into(),
            description: "Backend team".into(),
            privacy: "visible".into(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        let resp = TeamResponse::from(team);
        assert_eq!(resp.id, Uuid::nil().to_string());
        assert_eq!(resp.name, "backend");
        assert_eq!(resp.privacy, "visible");
    }

    #[test]
    fn test_team_response_serialization() {
        let resp = TeamResponse {
            id: Uuid::nil().to_string(),
            org_id: Uuid::nil().to_string(),
            name: "frontend".into(),
            description: "Frontend team".into(),
            privacy: "secret".into(),
            created_at: "2025-01-01T00:00:00+00:00".into(),
            updated_at: "2025-01-01T00:00:00+00:00".into(),
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("\"name\":\"frontend\""));
        assert!(json.contains("\"privacy\":\"secret\""));
    }

    #[test]
    fn test_team_member_response_from_db() {
        let member = crate::db::TeamMember {
            team_id: Uuid::nil(),
            user_id: Uuid::new_v4(),
            role: "maintainer".into(),
            joined_at: chrono::Utc::now(),
        };
        let resp = TeamMemberResponse::from(member);
        assert_eq!(resp.role, "maintainer");
    }

    #[test]
    fn test_create_team_request_parse() {
        let req: CreateTeamRequest = serde_json::from_str(
            r#"{"name":"devops","description":"DevOps team","privacy":"secret"}"#,
        )
        .unwrap();
        assert_eq!(req.name, "devops");
        assert_eq!(req.description.as_deref(), Some("DevOps team"));
        assert_eq!(req.privacy.as_deref(), Some("secret"));
    }

    #[test]
    fn test_create_team_request_defaults() {
        let req: CreateTeamRequest = serde_json::from_str(r#"{"name":"qa"}"#).unwrap();
        assert!(req.description.is_none());
        assert!(req.privacy.is_none());
    }

    #[test]
    fn test_update_team_request_parse() {
        let req: UpdateTeamRequest =
            serde_json::from_str(r#"{"description":"Updated desc"}"#).unwrap();
        assert_eq!(req.description.as_deref(), Some("Updated desc"));
        assert!(req.name.is_none());
        assert!(req.privacy.is_none());
    }

    #[test]
    fn test_add_member_request_parse() {
        let req: AddMemberRequest = serde_json::from_str(
            r#"{"user_id":"00000000-0000-0000-0000-000000000001","role":"owner"}"#,
        )
        .unwrap();
        assert_eq!(req.user_id, "00000000-0000-0000-0000-000000000001");
        assert_eq!(req.role.as_deref(), Some("owner"));
    }

    #[test]
    fn test_add_member_request_default_role() {
        let req: AddMemberRequest =
            serde_json::from_str(r#"{"user_id":"00000000-0000-0000-0000-000000000001"}"#).unwrap();
        assert!(req.role.is_none());
    }

    #[test]
    fn test_team_routes_compile() {
        let router = team_routes();
        let _ = router;
    }
}
