#![forbid(unsafe_code)]

use crate::api::AppState;
use crate::api::auth::AuthUser;
use crate::error::CoreError;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiTransform {
    pub id: String,
    pub route: String,
    pub request_transform: Option<serde_json::Value>,
    pub response_transform: Option<serde_json::Value>,
    pub enabled: bool,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateTransformRequest {
    pub route: String,
    pub request_transform: Option<serde_json::Value>,
    pub response_transform: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateTransformRequest {
    pub route: Option<String>,
    pub request_transform: Option<serde_json::Value>,
    pub response_transform: Option<serde_json::Value>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct ListParams {
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

/// List API transforms (admin only)
pub async fn list_transforms(
    State(state): State<AppState>,
    _auth: AuthUser,
    Query(params): Query<ListParams>,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let offset = (params.page.saturating_sub(1) * params.per_page) as i64;

    let rows = sqlx::query_as::<
        _,
        (
            Uuid,
            String,
            Option<serde_json::Value>,
            Option<serde_json::Value>,
            bool,
            DateTime<Utc>,
        ),
    >(
        "SELECT id, route, request_transform, response_transform, enabled, created_at
         FROM api_transforms ORDER BY created_at DESC LIMIT $1 OFFSET $2",
    )
    .bind(params.per_page as i64)
    .bind(offset)
    .fetch_all(pool)
    .await;

    match rows {
        Ok(rows) => {
            let transforms: Vec<ApiTransform> = rows
                .into_iter()
                .map(|(id, route, request_transform, response_transform, enabled, created_at)| {
                    ApiTransform {
                        id: id.to_string(),
                        route,
                        request_transform,
                        response_transform,
                        enabled,
                        created_at: created_at.to_rfc3339(),
                    }
                })
                .collect();
            (StatusCode::OK, Json(transforms)).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

/// Create an API transform (admin only)
pub async fn create_transform(
    State(state): State<AppState>,
    _auth: AuthUser,
    Json(req): Json<CreateTransformRequest>,
) -> impl IntoResponse {
    let pool = state.db.pool();

    if req.route.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(CoreError::BadRequest("route is required".into()).error_response()),
        )
            .into_response();
    }

    let result = sqlx::query_as::<_, (Uuid, DateTime<Utc>)>(
        "INSERT INTO api_transforms (route, request_transform, response_transform)
         VALUES ($1, $2, $3) RETURNING id, created_at",
    )
    .bind(&req.route)
    .bind(&req.request_transform)
    .bind(&req.response_transform)
    .fetch_one(pool)
    .await;

    match result {
        Ok((id, created_at)) => {
            let route = req.route.clone();
            (
                StatusCode::CREATED,
                Json(ApiTransform {
                    id: id.to_string(),
                    route,
                    request_transform: req.request_transform,
                    response_transform: req.response_transform,
                    enabled: true,
                    created_at: created_at.to_rfc3339(),
                }),
            )
                .into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

/// Update an API transform (admin only)
pub async fn update_transform(
    State(state): State<AppState>,
    Path(transform_id): Path<String>,
    _auth: AuthUser,
    Json(req): Json<UpdateTransformRequest>,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let transform_uuid = match Uuid::parse_str(&transform_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(CoreError::BadRequest("invalid transform id".into()).error_response()),
            )
                .into_response();
        }
    };

    let result = sqlx::query_as::<_, (Uuid, DateTime<Utc>)>(
        "UPDATE api_transforms SET
            route = COALESCE($2, route),
            request_transform = COALESCE($3, request_transform),
            response_transform = COALESCE($4, response_transform),
            enabled = COALESCE($5, enabled)
         WHERE id = $1 RETURNING id, created_at",
    )
    .bind(transform_uuid)
    .bind(req.route.clone())
    .bind(&req.request_transform)
    .bind(&req.response_transform)
    .bind(req.enabled)
    .fetch_one(pool)
    .await;

    match result {
        Ok((id, created_at)) => (
            StatusCode::OK,
            Json(ApiTransform {
                id: id.to_string(),
                route: req.route.unwrap_or_default(),
                request_transform: req.request_transform,
                response_transform: req.response_transform,
                enabled: req.enabled.unwrap_or(true),
                created_at: created_at.to_rfc3339(),
            }),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

/// Delete an API transform (admin only)
pub async fn delete_transform(
    State(state): State<AppState>,
    Path(transform_id): Path<String>,
    _auth: AuthUser,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let transform_uuid = match Uuid::parse_str(&transform_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(CoreError::BadRequest("invalid transform id".into()).error_response()),
            )
                .into_response();
        }
    };

    let result = sqlx::query("DELETE FROM api_transforms WHERE id = $1")
        .bind(transform_uuid)
        .execute(pool)
        .await;

    match result {
        Ok(row) if row.rows_affected() > 0 => (
            StatusCode::OK,
            Json(serde_json::json!({"status": "deleted"})),
        )
            .into_response(),
        Ok(_) => (
            StatusCode::NOT_FOUND,
            Json(CoreError::NotFound("transform not found".into()).error_response()),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

/// Get a transform for a specific route
pub async fn get_transform_for_route(
    pool: &sqlx::PgPool,
    route: &str,
) -> Result<Option<ApiTransform>, sqlx::Error> {
    let row = sqlx::query_as::<
        _,
        (
            Uuid,
            String,
            Option<serde_json::Value>,
            Option<serde_json::Value>,
            bool,
            DateTime<Utc>,
        ),
    >(
        "SELECT id, route, request_transform, response_transform, enabled, created_at
         FROM api_transforms WHERE route = $1 AND enabled = true",
    )
    .bind(route)
    .fetch_optional(pool)
    .await?;

    match row {
        Some((id, route, request_transform, response_transform, enabled, created_at)) => {
            Ok(Some(ApiTransform {
                id: id.to_string(),
                route,
                request_transform,
                response_transform,
                enabled,
                created_at: created_at.to_rfc3339(),
            }))
        }
        None => Ok(None),
    }
}

/// Apply request transformation to headers/body
pub fn apply_request_transform(
    transform: &serde_json::Value,
    headers: &mut axum::http::HeaderMap,
    body: &mut serde_json::Value,
) {
    if let Some(header_transforms) = transform.get("headers")
        && let Some(obj) = header_transforms.as_object() {
            for (key, value) in obj {
                if let Some(val_str) = value.as_str()
                    && let Ok(header_value) = axum::http::HeaderValue::from_str(val_str) {
                        headers.insert(
                            axum::http::HeaderName::from_bytes(key.as_bytes()).unwrap_or_else(|_| {
                                axum::http::HeaderName::from_static("x-transform")
                            }),
                            header_value,
                        );
                    }
            }
        }

    if let Some(body_transform) = transform.get("body")
        && let Some(merge_obj) = body_transform.as_object()
            && let Some(body_obj) = body.as_object_mut() {
                for (key, value) in merge_obj {
                    body_obj.insert(key.clone(), value.clone());
                }
            }
}

/// Apply response transformation to body
pub fn apply_response_transform(
    transform: &serde_json::Value,
    body: &mut serde_json::Value,
) {
    if let Some(body_transform) = transform.get("body")
        && let Some(merge_obj) = body_transform.as_object()
            && let Some(body_obj) = body.as_object_mut() {
                for (key, value) in merge_obj {
                    body_obj.insert(key.clone(), value.clone());
                }
            }
}

pub fn transform_routes() -> axum::Router<AppState> {
    use axum::routing::get;
    axum::Router::new()
        .route(
            "/api/v1/admin/transforms",
            get(list_transforms).post(create_transform),
        )
        .route(
            "/api/v1/admin/transforms/{transform_id}",
            get(get_transform)
                .put(update_transform)
                .delete(delete_transform),
        )
}

async fn get_transform(
    State(state): State<AppState>,
    Path(transform_id): Path<String>,
    _auth: AuthUser,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let transform_uuid = match Uuid::parse_str(&transform_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(CoreError::BadRequest("invalid transform id".into()).error_response()),
            )
                .into_response();
        }
    };

    let row = sqlx::query_as::<
        _,
        (
            Uuid,
            String,
            Option<serde_json::Value>,
            Option<serde_json::Value>,
            bool,
            DateTime<Utc>,
        ),
    >(
        "SELECT id, route, request_transform, response_transform, enabled, created_at
         FROM api_transforms WHERE id = $1",
    )
    .bind(transform_uuid)
    .fetch_optional(pool)
    .await;

    match row {
        Ok(Some((id, route, request_transform, response_transform, enabled, created_at))) => {
            (
                StatusCode::OK,
                Json(ApiTransform {
                    id: id.to_string(),
                    route,
                    request_transform,
                    response_transform,
                    enabled,
                    created_at: created_at.to_rfc3339(),
                }),
            )
                .into_response()
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(CoreError::NotFound("transform not found".into()).error_response()),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apply_request_transform_headers() {
        let transform = serde_json::json!({
            "headers": {
                "X-Custom-Header": "test-value",
                "X-Another": "another-value"
            }
        });
        let mut headers = axum::http::HeaderMap::new();
        let mut body = serde_json::json!({});

        apply_request_transform(&transform, &mut headers, &mut body);

        assert_eq!(
            headers.get("x-custom-header").unwrap().to_str().unwrap(),
            "test-value"
        );
        assert_eq!(
            headers.get("x-another").unwrap().to_str().unwrap(),
            "another-value"
        );
    }

    #[test]
    fn test_apply_request_transform_body() {
        let transform = serde_json::json!({
            "body": {
                "added_field": "added_value"
            }
        });
        let mut headers = axum::http::HeaderMap::new();
        let mut body = serde_json::json!({"existing": "value"});

        apply_request_transform(&transform, &mut headers, &mut body);

        assert_eq!(body["existing"], "value");
        assert_eq!(body["added_field"], "added_value");
    }

    #[test]
    fn test_apply_response_transform() {
        let transform = serde_json::json!({
            "body": {
                "response_field": "response_value"
            }
        });
        let mut body = serde_json::json!({"original": "data"});

        apply_response_transform(&transform, &mut body);

        assert_eq!(body["original"], "data");
        assert_eq!(body["response_field"], "response_value");
    }
}
