#![forbid(unsafe_code)]

use crate::api::AppState;
use crate::api::auth::require_admin;
use crate::api::auth::{AuthUser, OptionalAuthUser};
use crate::error::CoreError;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize)]
pub struct UserResponse {
    pub id: String,
    pub username: String,
    pub email: String,
    pub display_name: String,
    pub bio: Option<String>,
    pub role: String,
    pub created_at: String,
    pub updated_at: String,
}

impl From<crate::db::User> for UserResponse {
    fn from(u: crate::db::User) -> Self {
        Self {
            id: u.id.to_string(),
            username: u.username,
            email: u.email,
            display_name: u.display_name,
            bio: if u.bio.is_empty() { None } else { Some(u.bio) },
            role: u.role,
            created_at: u.created_at.to_rfc3339(),
            updated_at: u.updated_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateUserRequest {
    pub username: String,
    pub email: String,
    pub display_name: String,
    pub role: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateUserRequest {
    pub display_name: Option<String>,
    pub bio: Option<String>,
    pub role: Option<String>,
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

pub async fn list_users(
    State(state): State<AppState>,
    Query(params): Query<PaginationParams>,
    auth: AuthUser,
) -> impl IntoResponse {
    let is_admin = auth.role.as_str() == "admin";
    match state.db.list_users(params.limit, params.offset).await {
        Ok(users) => {
            let out: Vec<UserResponse> = users
                .into_iter()
                .map(|u| {
                    let mut resp = UserResponse::from(u);
                    if !is_admin {
                        resp.email = String::new();
                    }
                    resp
                })
                .collect();
            (StatusCode::OK, Json(out)).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub async fn get_user(
    State(state): State<AppState>,
    Path(id): Path<String>,
    _auth: OptionalAuthUser,
) -> impl IntoResponse {
    let user_uuid = match Uuid::parse_str(&id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(CoreError::Config("invalid user id".into()).error_response()),
            )
                .into_response();
        }
    };

    match state.db.get_user_by_id(user_uuid).await {
        Ok(user) => (StatusCode::OK, Json(UserResponse::from(user))).into_response(),
        Err(_) => (
            StatusCode::NOT_FOUND,
            Json(CoreError::NotFound("user not found".into()).error_response()),
        )
            .into_response(),
    }
}

pub async fn create_user(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<CreateUserRequest>,
) -> impl IntoResponse {
    // Admin-only: user creation
    if let Err(rejection) = require_admin(&auth) {
        return rejection.into_response();
    }
    if req.username.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(CoreError::Config("username required".into()).error_response()),
        )
            .into_response();
    }
    if req.email.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(CoreError::Config("email required".into()).error_response()),
        )
            .into_response();
    }

    let role = req.role.as_deref().unwrap_or("member");

    match state
        .db
        .create_user(&req.username, &req.email, &req.display_name, role, "")
        .await
    {
        Ok(user) => (StatusCode::CREATED, Json(UserResponse::from(user))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub async fn update_user(
    State(state): State<AppState>,
    Path(id): Path<String>,
    auth: AuthUser,
    Json(req): Json<UpdateUserRequest>,
) -> impl IntoResponse {
    // Admin-only: user update (changing roles etc.)
    if let Err(rejection) = require_admin(&auth) {
        return rejection.into_response();
    }
    let user_uuid = match Uuid::parse_str(&id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(CoreError::Config("invalid user id".into()).error_response()),
            )
                .into_response();
        }
    };

    match state
        .db
        .update_user(
            user_uuid,
            req.display_name.as_deref(),
            req.bio.as_deref(),
            req.role.as_deref(),
        )
        .await
    {
        Ok(user) => (StatusCode::OK, Json(UserResponse::from(user))).into_response(),
        Err(_) => (
            StatusCode::NOT_FOUND,
            Json(CoreError::NotFound("user not found".into()).error_response()),
        )
            .into_response(),
    }
}

pub async fn delete_user(
    State(state): State<AppState>,
    Path(id): Path<String>,
    auth: AuthUser,
) -> impl IntoResponse {
    // Admin-only: user deletion
    if let Err(rejection) = require_admin(&auth) {
        return rejection.into_response();
    }
    let user_uuid = match Uuid::parse_str(&id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(CoreError::Config("invalid user id".into()).error_response()),
            )
                .into_response();
        }
    };

    match state.db.delete_user(user_uuid).await {
        Ok(()) => (StatusCode::NO_CONTENT, ()).into_response(),
        Err(_) => (
            StatusCode::NOT_FOUND,
            Json(CoreError::NotFound("user not found".into()).error_response()),
        )
            .into_response(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn make_db_user() -> crate::db::User {
        crate::db::User {
            id: Uuid::nil(),
            username: "alice".into(),
            email: "alice@example.com".into(),
            display_name: "Alice Smith".into(),
            bio: "Developer".into(),
            role: "admin".into(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[test]
    fn test_user_response_from_db_user() {
        let user = make_db_user();
        let resp = UserResponse::from(user);
        assert_eq!(resp.id, Uuid::nil().to_string());
        assert_eq!(resp.username, "alice");
        assert_eq!(resp.email, "alice@example.com");
        assert_eq!(resp.display_name, "Alice Smith");
        assert_eq!(resp.bio.as_deref(), Some("Developer"));
        assert_eq!(resp.role, "admin");
    }

    #[test]
    fn test_user_response_empty_bio_is_none() {
        let mut user = make_db_user();
        user.bio = String::new();
        let resp = UserResponse::from(user);
        assert!(resp.bio.is_none());
    }

    #[test]
    fn test_user_response_serialization() {
        let user = make_db_user();
        let resp = UserResponse::from(user);
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("\"username\":\"alice\""));
        assert!(json.contains("\"email\":\"alice@example.com\""));
        assert!(json.contains("\"role\":\"admin\""));
        assert!(json.contains("\"bio\":\"Developer\""));
    }

    #[test]
    fn test_create_user_request_parse() {
        let json = r#"{"username":"alice","email":"alice@example.com","display_name":"Alice"}"#;
        let req: CreateUserRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.username, "alice");
        assert_eq!(req.email, "alice@example.com");
        assert_eq!(req.display_name, "Alice");
        assert!(req.role.is_none());
    }

    #[test]
    fn test_create_user_request_with_role() {
        let json =
            r#"{"username":"bob","email":"bob@example.com","display_name":"Bob","role":"admin"}"#;
        let req: CreateUserRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.role.as_deref(), Some("admin"));
    }

    #[test]
    fn test_update_user_request_parse() {
        let json = r#"{"display_name":"New Name"}"#;
        let req: UpdateUserRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.display_name.as_deref(), Some("New Name"));
        assert!(req.bio.is_none());
        assert!(req.role.is_none());
    }

    #[test]
    fn test_update_user_request_all_fields() {
        let json = r#"{"display_name":"Alice","bio":"Updated bio","role":"member"}"#;
        let req: UpdateUserRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.display_name.as_deref(), Some("Alice"));
        assert_eq!(req.bio.as_deref(), Some("Updated bio"));
        assert_eq!(req.role.as_deref(), Some("member"));
    }

    #[test]
    fn test_pagination_defaults() {
        assert_eq!(default_limit(), 50);
        assert_eq!(default_offset(), 0);
    }

    #[test]
    fn test_pagination_params_deserialize_empty() {
        let p: PaginationParams = serde_json::from_str("{}").unwrap();
        assert_eq!(p.limit, 50);
        assert_eq!(p.offset, 0);
    }

    #[test]
    fn test_pagination_params_deserialize_custom() {
        let p: PaginationParams = serde_json::from_str(r#"{"limit":10,"offset":20}"#).unwrap();
        assert_eq!(p.limit, 10);
        assert_eq!(p.offset, 20);
    }

    #[test]
    fn test_user_response_json_contains_fields() {
        let user = make_db_user();
        let resp = UserResponse::from(user);
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("\"username\":\"alice\""));
        assert!(json.contains("\"email\":\"alice@example.com\""));
        assert!(json.contains("\"role\":\"admin\""));
    }
}
