#![forbid(unsafe_code)]

use crate::api::AppState;
use crate::api::auth::{AuthUser, require_admin};
use crate::error::CoreError;
use axum::{
    Router,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::{get, put},
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ---------------------------------------------------------------------------
// SCIM 2.0 Types
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize)]
pub struct ScimUser {
    pub schemas: Vec<String>,
    pub id: Option<String>,
    pub external_id: Option<String>,
    #[serde(rename = "userName")]
    pub user_name: String,
    pub display_name: Option<String>,
    pub emails: Option<Vec<ScimEmail>>,
    pub active: Option<bool>,
    pub groups: Option<Vec<ScimGroupRef>>,
    pub meta: Option<ScimMeta>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ScimEmail {
    pub value: String,
    #[serde(rename = "type")]
    pub email_type: Option<String>,
    pub primary: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ScimGroupRef {
    pub value: String,
    pub display: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ScimMeta {
    pub resource_type: String,
    pub created: Option<String>,
    pub last_modified: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ScimGroup {
    pub schemas: Vec<String>,
    pub id: Option<String>,
    pub display_name: String,
    pub members: Option<Vec<ScimMemberRef>>,
    pub meta: Option<ScimMeta>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ScimMemberRef {
    pub value: String,
    pub display: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ScimListResponse<T: Serialize> {
    pub schemas: Vec<String>,
    pub total_results: usize,
    pub start_index: usize,
    pub items_per_page: usize,
    pub resources: Vec<T>,
}

#[derive(Debug, Serialize)]
pub struct ScimErrorResponse {
    pub schemas: Vec<String>,
    pub scim_type: Option<String>,
    pub detail: String,
    pub status: u16,
}

// ---------------------------------------------------------------------------
// SCIM Token Authentication
// ---------------------------------------------------------------------------

#[allow(dead_code)]
async fn authenticate_scim_token(state: &AppState, auth_header: &str) -> Result<Uuid, CoreError> {
    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or_else(|| CoreError::Auth("invalid SCIM authorization scheme".into()))?;

    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    let token_hash = format!("{:x}", hasher.finalize());

    let pool = state.db.pool();
    let row: Option<(Uuid, bool)> = sqlx::query_as::<_, (Uuid, bool)>(
        "SELECT provider_id, enabled FROM scim_tokens WHERE token_hash = $1",
    )
    .bind(&token_hash)
    .fetch_optional(pool)
    .await
    .map_err(|e| CoreError::Database(e.to_string()))?;

    match row {
        Some((provider_id, true)) => Ok(provider_id),
        _ => Err(CoreError::Auth("invalid or disabled SCIM token".into())),
    }
}

// ---------------------------------------------------------------------------
// GET /api/v1/scim/v2/Users
// ---------------------------------------------------------------------------
pub async fn scim_list_users(
    State(state): State<AppState>,
    auth: AuthUser,
) -> impl IntoResponse {
    if let Err(rejection) = require_admin(&auth) {
        return rejection.into_response();
    }

    let pool = state.db.pool();
    let rows: Vec<(Uuid, String, String, String, bool, chrono::DateTime<chrono::Utc>)> =
        sqlx::query_as(
            "SELECT id, username, email, display_name, banned, created_at
             FROM users ORDER BY username LIMIT 1000",
        )
        .fetch_all(pool)
        .await
        .unwrap_or_default();

    let users: Vec<ScimUser> = rows
        .into_iter()
        .map(|(id, username, email, display_name, active, created_at)| ScimUser {
            schemas: vec!["urn:ietf:params:scim:schemas:core:2.0:User".into()],
            id: Some(id.to_string()),
            external_id: None,
            user_name: username,
            display_name: Some(display_name),
            emails: Some(vec![ScimEmail {
                value: email,
                email_type: Some("work".into()),
                primary: Some(true),
            }]),
            active: Some(!active), // banned = true means active = false
            groups: None,
            meta: Some(ScimMeta {
                resource_type: "User".into(),
                created: Some(created_at.to_rfc3339()),
                last_modified: None,
            }),
        })
        .collect();

    let resp = ScimListResponse {
        schemas: vec!["urn:ietf:params:scim:api:messages:2.0:ListResponse".into()],
        total_results: users.len(),
        start_index: 1,
        items_per_page: users.len(),
        resources: users,
    };
    (StatusCode::OK, Json(resp)).into_response()
}

// ---------------------------------------------------------------------------
// POST /api/v1/scim/v2/Users
// ---------------------------------------------------------------------------
pub async fn scim_create_user(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(scim_user): Json<ScimUser>,
) -> impl IntoResponse {
    if let Err(rejection) = require_admin(&auth) {
        return rejection.into_response();
    }

    if scim_user.user_name.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ScimErrorResponse {
                schemas: vec!["urn:ietf:params:scim:api:messages:2.0:Error".into()],
                scim_type: Some("invalidValue".into()),
                detail: "userName is required".into(),
                status: 400,
            }),
        )
            .into_response();
    }

    let email = scim_user
        .emails
        .as_ref()
        .and_then(|e| e.first())
        .map(|e| e.value.as_str())
        .unwrap_or("");

    let display_name = scim_user
        .display_name
        .as_deref()
        .unwrap_or(&scim_user.user_name);

    let password_hash = civit_auth::password::hash_password("changeme123!").unwrap_or_default();

    let result = state
        .db
        .create_user(
            &scim_user.user_name,
            email,
            display_name,
            "member",
            &password_hash,
        )
        .await;

    match result {
        Ok(user) => {
            let resp = ScimUser {
                schemas: vec!["urn:ietf:params:scim:schemas:core:2.0:User".into()],
                id: Some(user.id.to_string()),
                external_id: scim_user.external_id,
                user_name: user.username,
                display_name: Some(user.display_name),
                emails: Some(vec![ScimEmail {
                    value: user.email,
                    email_type: Some("work".into()),
                    primary: Some(true),
                }]),
                active: Some(true),
                groups: None,
                meta: Some(ScimMeta {
                    resource_type: "User".into(),
                    created: Some(user.created_at.to_rfc3339()),
                    last_modified: None,
                }),
            };
            (StatusCode::CREATED, Json(resp)).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ScimErrorResponse {
                schemas: vec!["urn:ietf:params:scim:api:messages:2.0:Error".into()],
                scim_type: None,
                detail: format!("user creation failed: {e}"),
                status: 500,
            }),
        )
            .into_response(),
    }
}

// ---------------------------------------------------------------------------
// PUT /api/v1/scim/v2/Users/:id
// ---------------------------------------------------------------------------
pub async fn scim_update_user(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<String>,
    Json(scim_user): Json<ScimUser>,
) -> impl IntoResponse {
    if let Err(rejection) = require_admin(&auth) {
        return rejection.into_response();
    }

    let user_uuid = match Uuid::parse_str(&id) {
        Ok(u) => u,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ScimErrorResponse {
                    schemas: vec!["urn:ietf:params:scim:api:messages:2.0:Error".into()],
                    scim_type: Some("invalidValue".into()),
                    detail: "invalid user id".into(),
                    status: 400,
                }),
            )
                .into_response();
        }
    };

    let display_name = scim_user
        .display_name
        .clone()
        .unwrap_or_else(|| scim_user.user_name.clone());

    let pool = state.db.pool();
    let result = sqlx::query(
        "UPDATE users SET display_name = $1 WHERE id = $2",
    )
    .bind(&display_name)
    .bind(user_uuid)
    .execute(pool)
    .await;

    match result {
        Ok(_) => {
            let resp = ScimUser {
                schemas: vec!["urn:ietf:params:scim:schemas:core:2.0:User".into()],
                id: Some(id),
                external_id: scim_user.external_id,
                user_name: scim_user.user_name,
                display_name: Some(display_name),
                emails: scim_user.emails,
                active: scim_user.active,
                groups: scim_user.groups,
                meta: Some(ScimMeta {
                    resource_type: "User".into(),
                    created: None,
                    last_modified: Some(chrono::Utc::now().to_rfc3339()),
                }),
            };
            (StatusCode::OK, Json(resp)).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ScimErrorResponse {
                schemas: vec!["urn:ietf:params:scim:api:messages:2.0:Error".into()],
                scim_type: None,
                detail: format!("user update failed: {e}"),
                status: 500,
            }),
        )
            .into_response(),
    }
}

// ---------------------------------------------------------------------------
// DELETE /api/v1/scim/v2/Users/:id
// ---------------------------------------------------------------------------
pub async fn scim_delete_user(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<String>,
) -> impl IntoResponse {
    if let Err(rejection) = require_admin(&auth) {
        return rejection.into_response();
    }

    let user_uuid = match Uuid::parse_str(&id) {
        Ok(u) => u,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ScimErrorResponse {
                    schemas: vec!["urn:ietf:params:scim:api:messages:2.0:Error".into()],
                    scim_type: Some("invalidValue".into()),
                    detail: "invalid user id".into(),
                    status: 400,
                }),
            )
                .into_response();
        }
    };

    let pool = state.db.pool();
    let result = sqlx::query("DELETE FROM users WHERE id = $1")
        .bind(user_uuid)
        .execute(pool)
        .await;

    match result {
        Ok(r) => {
            if r.rows_affected() == 0 {
                StatusCode::NOT_FOUND.into_response()
            } else {
                StatusCode::NO_CONTENT.into_response()
            }
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ScimErrorResponse {
                schemas: vec!["urn:ietf:params:scim:api:messages:2.0:Error".into()],
                scim_type: None,
                detail: format!("user deletion failed: {e}"),
                status: 500,
            }),
        )
            .into_response(),
    }
}

// ---------------------------------------------------------------------------
// GET /api/v1/scim/v2/Groups
// ---------------------------------------------------------------------------
pub async fn scim_list_groups(
    State(_state): State<AppState>,
    auth: AuthUser,
) -> impl IntoResponse {
    if let Err(rejection) = require_admin(&auth) {
        return rejection.into_response();
    }

    // Return CivitForge roles as SCIM groups
    let roles = vec![
        ("admin".into(), "Administrator".into()),
        ("member".into(), "Member".into()),
        ("guest".into(), "Guest".into()),
    ];

    let groups: Vec<ScimGroup> = roles
        .into_iter()
        .map(|(id, display_name)| ScimGroup {
            schemas: vec!["urn:ietf:params:scim:schemas:core:2.0:Group".into()],
            id: Some(id),
            display_name,
            members: None,
            meta: Some(ScimMeta {
                resource_type: "Group".into(),
                created: None,
                last_modified: None,
            }),
        })
        .collect();

    let resp = ScimListResponse {
        schemas: vec!["urn:ietf:params:scim:api:messages:2.0:ListResponse".into()],
        total_results: groups.len(),
        start_index: 1,
        items_per_page: groups.len(),
        resources: groups,
    };
    (StatusCode::OK, Json(resp)).into_response()
}

// ---------------------------------------------------------------------------
// POST /api/v1/scim/v2/Groups
// ---------------------------------------------------------------------------
pub async fn scim_create_group(
    State(_state): State<AppState>,
    auth: AuthUser,
    Json(scim_group): Json<ScimGroup>,
) -> impl IntoResponse {
    if let Err(rejection) = require_admin(&auth) {
        return rejection.into_response();
    }

    // Groups are mapped through sso_group_mappings; this creates a placeholder
    let resp = ScimGroup {
        schemas: vec!["urn:ietf:params:scim:schemas:core:2.0:Group".into()],
        id: Some(scim_group.display_name.to_lowercase().replace(' ', "-")),
        display_name: scim_group.display_name,
        members: scim_group.members,
        meta: Some(ScimMeta {
            resource_type: "Group".into(),
            created: Some(chrono::Utc::now().to_rfc3339()),
            last_modified: None,
        }),
    };
    (StatusCode::CREATED, Json(resp)).into_response()
}

pub fn scim_routes() -> Router<AppState> {
    Router::new()
        .route(
            "/api/v1/scim/v2/Users",
            get(scim_list_users).post(scim_create_user),
        )
        .route(
            "/api/v1/scim/v2/Users/{id}",
            put(scim_update_user).delete(scim_delete_user),
        )
        .route(
            "/api/v1/scim/v2/Groups",
            get(scim_list_groups).post(scim_create_group),
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scim_user_serialize() {
        let user = ScimUser {
            schemas: vec!["urn:ietf:params:scim:schemas:core:2.0:User".into()],
            id: Some("123".into()),
            external_id: None,
            user_name: "alice".into(),
            display_name: Some("Alice Smith".into()),
            emails: Some(vec![ScimEmail {
                value: "alice@example.com".into(),
                email_type: Some("work".into()),
                primary: Some(true),
            }]),
            active: Some(true),
            groups: None,
            meta: None,
        };
        let json = serde_json::to_string(&user).unwrap();
        assert!(json.contains("\"userName\":\"alice\""));
    }

    #[test]
    fn test_scim_routes_compile() {
        let _ = scim_routes();
    }
}
