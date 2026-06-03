#![forbid(unsafe_code)]

use crate::api::AppState;
use crate::api::auth::{AuthUser, OptionalAuthUser, require_permission};
use crate::error::CoreError;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use civit_shared::permissions::{Action, Resource};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize)]
pub struct OrgResponse {
    pub id: String,
    pub name: String,
    pub display_name: String,
    pub description: String,
    pub visibility: String,
    pub owner_id: String,
    pub created_at: String,
    pub updated_at: String,
}

impl From<crate::db::Org> for OrgResponse {
    fn from(o: crate::db::Org) -> Self {
        Self {
            id: o.id.to_string(),
            name: o.name,
            display_name: o.display_name,
            description: o.description,
            visibility: o.visibility,
            owner_id: o.owner_id.to_string(),
            created_at: o.created_at.to_rfc3339(),
            updated_at: o.updated_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateOrgRequest {
    pub name: String,
    pub display_name: String,
    pub description: String,
    pub visibility: String,
    pub owner_id: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateOrgRequest {
    pub display_name: Option<String>,
    pub description: Option<String>,
    pub visibility: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ListOrgsParams {
    pub owner_id: String,
}

pub async fn list_orgs(
    State(state): State<AppState>,
    Query(params): Query<ListOrgsParams>,
    _auth: OptionalAuthUser,
) -> impl IntoResponse {
    let owner_uuid = match Uuid::parse_str(&params.owner_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(CoreError::Config("invalid owner_id".into()).error_response()),
            )
                .into_response();
        }
    };

    match state.db.list_orgs_by_owner(owner_uuid).await {
        Ok(orgs) => {
            let out: Vec<OrgResponse> = orgs.into_iter().map(Into::into).collect();
            (StatusCode::OK, Json(out)).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub async fn get_org(
    State(state): State<AppState>,
    Path(id): Path<String>,
    _auth: OptionalAuthUser,
) -> impl IntoResponse {
    let org_uuid = match Uuid::parse_str(&id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(CoreError::Config("invalid org id".into()).error_response()),
            )
                .into_response();
        }
    };

    match state.db.get_org(org_uuid).await {
        Ok(org) => (StatusCode::OK, Json(OrgResponse::from(org))).into_response(),
        Err(_) => (
            StatusCode::NOT_FOUND,
            Json(CoreError::NotFound("org not found".into()).error_response()),
        )
            .into_response(),
    }
}

pub async fn create_org(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<CreateOrgRequest>,
) -> impl IntoResponse {
    // Require create permission on organizations
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
    if req.name.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(CoreError::Config("name required".into()).error_response()),
        )
            .into_response();
    }

    let owner_uuid = match Uuid::parse_str(&req.owner_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(CoreError::Config("invalid owner_id".into()).error_response()),
            )
                .into_response();
        }
    };

    match state
        .db
        .create_org(
            &req.name,
            &req.display_name,
            &req.description,
            &req.visibility,
            owner_uuid,
        )
        .await
    {
        Ok(org) => (StatusCode::CREATED, Json(OrgResponse::from(org))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub async fn update_org(
    State(state): State<AppState>,
    Path(id): Path<String>,
    auth: AuthUser,
    Json(req): Json<UpdateOrgRequest>,
) -> impl IntoResponse {
    // Require update permission on organizations
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
    let org_uuid = match Uuid::parse_str(&id) {
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
        .update_org(
            org_uuid,
            req.display_name.as_deref(),
            req.description.as_deref(),
            req.visibility.as_deref(),
        )
        .await
    {
        Ok(org) => (StatusCode::OK, Json(OrgResponse::from(org))).into_response(),
        Err(_) => (
            StatusCode::NOT_FOUND,
            Json(CoreError::NotFound("org not found".into()).error_response()),
        )
            .into_response(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn make_db_org() -> crate::db::Org {
        crate::db::Org {
            id: Uuid::nil(),
            name: "myorg".into(),
            display_name: "My Org".into(),
            description: "An organization".into(),
            visibility: "public".into(),
            owner_id: Uuid::new_v4(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[test]
    fn test_org_response_from_db_org() {
        let org = make_db_org();
        let resp = OrgResponse::from(org);
        assert_eq!(resp.id, Uuid::nil().to_string());
        assert_eq!(resp.name, "myorg");
        assert_eq!(resp.display_name, "My Org");
        assert_eq!(resp.description, "An organization");
        assert_eq!(resp.visibility, "public");
    }

    #[test]
    fn test_org_response_serialization() {
        let org = make_db_org();
        let resp = OrgResponse::from(org);
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("\"name\":\"myorg\""));
        assert!(json.contains("\"visibility\":\"public\""));
    }

    #[test]
    fn test_create_org_request_parse() {
        let json = r#"{"name":"myorg","display_name":"My Org","description":"Desc","visibility":"public","owner_id":"00000000-0000-0000-0000-000000000000"}"#;
        let req: CreateOrgRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.name, "myorg");
        assert_eq!(req.visibility, "public");
        assert_eq!(req.owner_id, Uuid::nil().to_string());
    }

    #[test]
    fn test_create_org_request_missing_fields() {
        let json = r#"{"name":"myorg"}"#;
        let result = serde_json::from_str::<CreateOrgRequest>(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_update_org_request_parse() {
        let json = r#"{"description":"Updated description"}"#;
        let req: UpdateOrgRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.description.as_deref(), Some("Updated description"));
        assert!(req.display_name.is_none());
        assert!(req.visibility.is_none());
    }

    #[test]
    fn test_update_org_request_all_fields() {
        let json = r#"{"display_name":"New Name","description":"Desc","visibility":"private"}"#;
        let req: UpdateOrgRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.display_name.as_deref(), Some("New Name"));
        assert_eq!(req.visibility.as_deref(), Some("private"));
    }

    #[test]
    fn test_list_orgs_params_parse() {
        let json = r#"{"owner_id":"00000000-0000-0000-0000-000000000000"}"#;
        let params: ListOrgsParams = serde_json::from_str(json).unwrap();
        assert_eq!(params.owner_id, Uuid::nil().to_string());
    }

    #[test]
    fn test_org_response_json_contains_fields() {
        let org = make_db_org();
        let resp = OrgResponse::from(org);
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("\"name\":\"myorg\""));
        assert!(json.contains("\"visibility\":\"public\""));
    }
}
