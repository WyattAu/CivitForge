//! Pipeline Actions Marketplace API endpoints.

#![forbid(unsafe_code)]

use crate::api::AppState;
use crate::api::auth::AuthUser;
use crate::error::CoreError;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{delete, get, post};
use axum::{Json, Router};
use civit_ci::action_installations;
use civit_ci::actions;
use civit_ci::categories;
use serde::Deserialize;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct CreateActionRequest {
    pub name: String,
    pub description: Option<String>,
    pub action_type: String,
    pub config: Option<serde_json::Value>,
    pub version: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateActionRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub action_type: Option<String>,
    pub config: Option<serde_json::Value>,
    pub version: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ActionListParams {
    #[serde(default = "default_page")]
    pub page: u32,
    #[serde(default = "default_per_page")]
    pub per_page: u32,
    pub action_type: Option<String>,
    pub search: Option<String>,
    pub sort_by: Option<String>,
}

fn default_page() -> u32 {
    1
}
fn default_per_page() -> u32 {
    20
}

#[derive(Debug, Deserialize)]
pub struct RateActionRequest {
    pub rating: f64,
}

#[derive(Debug, Deserialize)]
pub struct ReviewActionRequest {
    pub rating: i32,
    pub review: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ForkActionRequest {
    pub new_name: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateCategoryRequest {
    pub name: String,
    pub description: Option<String>,
    pub parent_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateCategoryRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub parent_id: Option<Option<String>>,
}

#[derive(Debug, Deserialize)]
pub struct CategorySearchParams {
    pub search: Option<String>,
    #[serde(default = "default_limit")]
    pub limit: i64,
}

fn default_limit() -> i64 {
    50
}

#[derive(Debug, Deserialize)]
pub struct InstallActionRequest {
    pub version: String,
    #[serde(default)]
    pub config: serde_json::Value,
}

pub fn pipeline_action_routes() -> Router<AppState> {
    Router::new()
        .route(
            "/api/v1/pipeline-actions",
            get(list_actions).post(create_action),
        )
        .route(
            "/api/v1/pipeline-actions/{action_id}",
            get(get_action)
                .patch(update_action)
                .delete(delete_action),
        )
        .route(
            "/api/v1/pipeline-actions/{action_id}/download",
            post(record_download),
        )
        .route(
            "/api/v1/pipeline-actions/{action_id}/rate",
            post(rate_action),
        )
        .route(
            "/api/v1/pipeline-actions/{action_id}/reviews",
            get(list_reviews).post(upsert_review),
        )
        .route(
            "/api/v1/pipeline-actions/{action_id}/reviews/me",
            get(get_my_review).delete(delete_review),
        )
        .route(
            "/api/v1/pipeline-actions/{action_id}/fork",
            post(fork_action_handler),
        )
        .route(
            "/api/v1/pipeline-actions/{action_id}/forks",
            get(list_forks),
        )
        .route(
            "/api/v1/pipeline-actions/{action_id}/recommendations",
            get(get_recommendations),
        )
        .route(
            "/api/v1/pipeline-actions/{action_id}/analytics",
            get(get_analytics),
        )
        .route(
            "/api/v1/pipeline-actions/{action_id}/categories",
            get(list_action_cats).post(add_action_to_category),
        )
        .route(
            "/api/v1/pipeline-actions/{action_id}/categories/{category_id}",
            delete(remove_action_from_cat),
        )
        .route(
            "/api/v1/pipeline-categories",
            get(list_categories).post(create_category),
        )
        .route(
            "/api/v1/pipeline-categories/search",
            get(search_categories),
        )
        .route(
            "/api/v1/pipeline-categories/{category_id}",
            get(get_category)
                .patch(update_category)
                .delete(delete_category),
        )
        .route(
            "/api/v1/pipeline-categories/{category_id}/actions",
            get(list_category_actions),
        )
        .route(
            "/api/v1/pipeline-categories/{category_id}/analytics",
            get(get_category_analytics),
        )
        .route(
            "/api/v1/repos/{owner}/{name}/action-installations",
            get(list_repo_installations).post(install_action_to_repo),
        )
        .route(
            "/api/v1/repos/{owner}/{name}/action-installations/{installation_id}",
            delete(uninstall_action_from_repo),
        )
        // Reviews V2 endpoints
        .route(
            "/api/v1/pipeline-actions/{action_id}/reviews/v2",
            get(list_reviews_v2).post(upsert_review_v2),
        )
        .route(
            "/api/v1/pipeline-actions/{action_id}/reviews/v2/me",
            get(get_my_review_v2).delete(delete_review_v2),
        )
        .route(
            "/api/v1/pipeline-actions/{action_id}/reviews/v2/{review_id}/helpful",
            post(toggle_helpful),
        )
        .route(
            "/api/v1/pipeline-actions/{action_id}/reviews/v2/analytics",
            get(get_review_analytics_endpoint),
        )
        .route(
            "/api/v1/pipeline-actions/{action_id}/reviews/v2/{review_id}/moderate",
            post(submit_for_moderation),
        )
        .route(
            "/api/v1/pipeline-actions/{action_id}/reviews/v2/{moderation_id}/moderate-action",
            post(moderate_review_endpoint),
        )
        .route(
            "/api/v1/pipeline-actions/recommendations",
            get(get_user_recommendations),
        )
        // Reviews V19 endpoints
        .route(
            "/api/v1/pipeline-actions/{action_id}/reviews/v19",
            get(list_reviews_v19).post(upsert_review_v19),
        )
        .route(
            "/api/v1/pipeline-actions/{action_id}/reviews/v19/me",
            get(get_my_review_v19).delete(delete_review_v19),
        )
        .route(
            "/api/v1/pipeline-actions/{action_id}/reviews/v19/{review_id}/helpful",
            post(toggle_helpful_v19),
        )
        .route(
            "/api/v1/pipeline-actions/{action_id}/reviews/v19/analytics",
            get(get_review_analytics_v22_endpoint),
        )
        .route(
            "/api/v1/pipeline-actions/{action_id}/reviews/v19/{review_id}/moderate",
            post(moderate_review_v22_endpoint),
        )
        .route(
            "/api/v1/pipeline-actions/{action_id}/reviews/v19/recommendations",
            get(get_review_recommendations_v22_endpoint),
        )
}

pub async fn list_actions(
    State(state): State<AppState>,
    Query(params): Query<ActionListParams>,
    _auth: AuthUser,
) -> Response {
    let pool = state.db.pool();
    let offset = ((params.page.saturating_sub(1)) * params.per_page) as i64;

    match actions::list_pipeline_actions(
        pool,
        params.per_page as i64,
        offset,
        params.action_type.as_deref(),
        params.search.as_deref(),
        params.sort_by.as_deref(),
    )
    .await
    {
        Ok(actions_list) => (StatusCode::OK, Json(actions_list)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub async fn create_action(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<CreateActionRequest>,
) -> Response {
    let pool = state.db.pool();
    let author_id = Uuid::parse_str(&auth.user_id).ok();
    let config = req.config.unwrap_or(serde_json::json!({}));
    let version = req.version.as_deref().unwrap_or("1.0.0");
    let description = req.description.unwrap_or_default();

    match actions::create_pipeline_action(
        pool,
        &req.name,
        &description,
        &req.action_type,
        &config,
        version,
        author_id,
    )
    .await
    {
        Ok(action) => (StatusCode::CREATED, Json(action)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub async fn get_action(
    State(state): State<AppState>,
    Path(action_id): Path<String>,
    _auth: AuthUser,
) -> Response {
    let pool = state.db.pool();
    let aid = match Uuid::parse_str(&action_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(CoreError::BadRequest("invalid action ID".into()).error_response()),
            )
                .into_response();
        }
    };

    match actions::get_pipeline_action(pool, aid).await {
        Ok(Some(action)) => (StatusCode::OK, Json(action)).into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(CoreError::NotFound("action not found".into()).error_response()),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub async fn update_action(
    State(state): State<AppState>,
    Path(action_id): Path<String>,
    _auth: AuthUser,
    Json(req): Json<UpdateActionRequest>,
) -> Response {
    let pool = state.db.pool();
    let aid = match Uuid::parse_str(&action_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(CoreError::BadRequest("invalid action ID".into()).error_response()),
            )
                .into_response();
        }
    };

    let config_ref = req.config.as_ref();

    match actions::update_pipeline_action(
        pool,
        aid,
        req.name.as_deref(),
        req.description.as_deref(),
        req.action_type.as_deref(),
        config_ref,
        req.version.as_deref(),
    )
    .await
    {
        Ok(action) => (StatusCode::OK, Json(action)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub async fn delete_action(
    State(state): State<AppState>,
    Path(action_id): Path<String>,
    _auth: AuthUser,
) -> Response {
    let pool = state.db.pool();
    let aid = match Uuid::parse_str(&action_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(CoreError::BadRequest("invalid action ID".into()).error_response()),
            )
                .into_response();
        }
    };

    match actions::delete_pipeline_action(pool, aid).await {
        Ok(true) => (
            StatusCode::OK,
            Json(serde_json::json!({"status": "deleted"})),
        )
            .into_response(),
        Ok(false) => (
            StatusCode::NOT_FOUND,
            Json(CoreError::NotFound("action not found".into()).error_response()),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub async fn record_download(
    State(state): State<AppState>,
    Path(action_id): Path<String>,
    _auth: AuthUser,
) -> Response {
    let pool = state.db.pool();
    let aid = match Uuid::parse_str(&action_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(CoreError::BadRequest("invalid action ID".into()).error_response()),
            )
                .into_response();
        }
    };

    match actions::track_download(pool, aid).await {
        Ok(()) => (
            StatusCode::OK,
            Json(serde_json::json!({"status": "download recorded"})),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub async fn rate_action(
    State(state): State<AppState>,
    Path(action_id): Path<String>,
    _auth: AuthUser,
    Json(req): Json<RateActionRequest>,
) -> Response {
    let pool = state.db.pool();
    let aid = match Uuid::parse_str(&action_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(CoreError::BadRequest("invalid action ID".into()).error_response()),
            )
                .into_response();
        }
    };

    if req.rating < 0.0 || req.rating > 5.0 {
        return (
            StatusCode::BAD_REQUEST,
            Json(CoreError::BadRequest("rating must be between 0 and 5".into()).error_response()),
        )
            .into_response();
    }

    match actions::update_rating(pool, aid, req.rating).await {
        Ok(action) => (StatusCode::OK, Json(action)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub async fn list_reviews(
    State(state): State<AppState>,
    Path(action_id): Path<String>,
    _auth: AuthUser,
) -> Response {
    let pool = state.db.pool();
    let aid = match Uuid::parse_str(&action_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(CoreError::BadRequest("invalid action ID".into()).error_response()),
            )
                .into_response();
        }
    };

    match actions::list_action_reviews(pool, aid, 50, 0).await {
        Ok(reviews) => (StatusCode::OK, Json(reviews)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub async fn upsert_review(
    State(state): State<AppState>,
    Path(action_id): Path<String>,
    auth: AuthUser,
    Json(req): Json<ReviewActionRequest>,
) -> Response {
    let pool = state.db.pool();
    let aid = match Uuid::parse_str(&action_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(CoreError::BadRequest("invalid action ID".into()).error_response()),
            )
                .into_response();
        }
    };
    let uid = match Uuid::parse_str(&auth.user_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(CoreError::BadRequest("invalid user ID".into()).error_response()),
            )
                .into_response();
        }
    };

    if req.rating < 1 || req.rating > 5 {
        return (
            StatusCode::BAD_REQUEST,
            Json(CoreError::BadRequest("rating must be between 1 and 5".into()).error_response()),
        )
            .into_response();
    }

    let review = req.review.unwrap_or_default();
    match actions::upsert_action_review(pool, aid, uid, req.rating, &review).await {
        Ok(r) => {
            let _ = actions::refresh_action_rating(pool, aid).await;
            (StatusCode::OK, Json(r)).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub async fn get_my_review(
    State(state): State<AppState>,
    Path(action_id): Path<String>,
    auth: AuthUser,
) -> Response {
    let pool = state.db.pool();
    let aid = match Uuid::parse_str(&action_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(CoreError::BadRequest("invalid action ID".into()).error_response()),
            )
                .into_response();
        }
    };
    let uid = match Uuid::parse_str(&auth.user_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(CoreError::BadRequest("invalid user ID".into()).error_response()),
            )
                .into_response();
        }
    };

    match actions::get_action_review(pool, aid, uid).await {
        Ok(Some(review)) => (StatusCode::OK, Json(review)).into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(CoreError::NotFound("no review found".into()).error_response()),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub async fn delete_review(
    State(state): State<AppState>,
    Path(action_id): Path<String>,
    auth: AuthUser,
) -> Response {
    let pool = state.db.pool();
    let aid = match Uuid::parse_str(&action_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(CoreError::BadRequest("invalid action ID".into()).error_response()),
            )
                .into_response();
        }
    };
    let uid = match Uuid::parse_str(&auth.user_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(CoreError::BadRequest("invalid user ID".into()).error_response()),
            )
                .into_response();
        }
    };

    match actions::delete_action_review(pool, aid, uid).await {
        Ok(true) => {
            let _ = actions::refresh_action_rating(pool, aid).await;
            (
                StatusCode::OK,
                Json(serde_json::json!({"status": "deleted"})),
            )
                .into_response()
        }
        Ok(false) => (
            StatusCode::NOT_FOUND,
            Json(CoreError::NotFound("review not found".into()).error_response()),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub async fn fork_action_handler(
    State(state): State<AppState>,
    Path(action_id): Path<String>,
    auth: AuthUser,
    Json(req): Json<ForkActionRequest>,
) -> Response {
    let pool = state.db.pool();
    let aid = match Uuid::parse_str(&action_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(CoreError::BadRequest("invalid action ID".into()).error_response()),
            )
                .into_response();
        }
    };
    let uid = match Uuid::parse_str(&auth.user_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(CoreError::BadRequest("invalid user ID".into()).error_response()),
            )
                .into_response();
        }
    };

    match actions::fork_action(pool, aid, uid, &req.new_name).await {
        Ok(action) => (StatusCode::CREATED, Json(action)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub async fn list_forks(
    State(state): State<AppState>,
    Path(action_id): Path<String>,
    _auth: AuthUser,
) -> Response {
    let pool = state.db.pool();
    let aid = match Uuid::parse_str(&action_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(CoreError::BadRequest("invalid action ID".into()).error_response()),
            )
                .into_response();
        }
    };

    match actions::list_action_forks(pool, aid).await {
        Ok(forks) => (StatusCode::OK, Json(forks)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub async fn get_recommendations(
    State(state): State<AppState>,
    Path(action_id): Path<String>,
    _auth: AuthUser,
) -> Response {
    let pool = state.db.pool();
    let aid = match Uuid::parse_str(&action_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(CoreError::BadRequest("invalid action ID".into()).error_response()),
            )
                .into_response();
        }
    };

    match actions::get_recommended_actions(pool, aid, 10).await {
        Ok(actions) => (StatusCode::OK, Json(actions)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub async fn get_analytics(
    State(state): State<AppState>,
    Path(action_id): Path<String>,
    _auth: AuthUser,
) -> Response {
    let pool = state.db.pool();
    let aid = match Uuid::parse_str(&action_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(CoreError::BadRequest("invalid action ID".into()).error_response()),
            )
                .into_response();
        }
    };

    match actions::get_action_analytics(pool, aid).await {
        Ok(analytics) => (StatusCode::OK, Json(analytics)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

// ---------------------------------------------------------------------------
// Category handlers
// ---------------------------------------------------------------------------

pub async fn list_categories(
    State(state): State<AppState>,
    _auth: AuthUser,
) -> Response {
    let pool = state.db.pool();
    match categories::list_categories(pool).await {
        Ok(cats) => (StatusCode::OK, Json(cats)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub async fn create_category(
    State(state): State<AppState>,
    _auth: AuthUser,
    Json(req): Json<CreateCategoryRequest>,
) -> Response {
    let pool = state.db.pool();
    let parent_id = req.parent_id.as_deref().and_then(|s| Uuid::parse_str(s).ok());
    let description = req.description.unwrap_or_default();

    match categories::create_category(pool, &req.name, &description, parent_id).await {
        Ok(cat) => (StatusCode::CREATED, Json(cat)).into_response(),
        Err(e) => {
            let msg = e.to_string();
            if msg.contains("duplicate key") || msg.contains("unique") {
                (
                    StatusCode::CONFLICT,
                    Json(CoreError::BadRequest("category name already exists".into()).error_response()),
                )
                    .into_response()
            } else {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(CoreError::Database(msg).error_response()),
                )
                    .into_response()
            }
        }
    }
}

pub async fn get_category(
    State(state): State<AppState>,
    Path(category_id): Path<String>,
    _auth: AuthUser,
) -> Response {
    let pool = state.db.pool();
    let cid = match Uuid::parse_str(&category_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(CoreError::BadRequest("invalid category ID".into()).error_response()),
            )
                .into_response();
        }
    };

    match categories::get_category(pool, cid).await {
        Ok(Some(cat)) => (StatusCode::OK, Json(cat)).into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(CoreError::NotFound("category not found".into()).error_response()),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub async fn update_category(
    State(state): State<AppState>,
    Path(category_id): Path<String>,
    _auth: AuthUser,
    Json(req): Json<UpdateCategoryRequest>,
) -> Response {
    let pool = state.db.pool();
    let cid = match Uuid::parse_str(&category_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(CoreError::BadRequest("invalid category ID".into()).error_response()),
            )
                .into_response();
        }
    };

    let parent_id = req.parent_id.map(|opt| opt.and_then(|s| Uuid::parse_str(&s).ok()));

    match categories::update_category(pool, cid, req.name.as_deref(), req.description.as_deref(), parent_id).await {
        Ok(cat) => (StatusCode::OK, Json(cat)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub async fn delete_category(
    State(state): State<AppState>,
    Path(category_id): Path<String>,
    _auth: AuthUser,
) -> Response {
    let pool = state.db.pool();
    let cid = match Uuid::parse_str(&category_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(CoreError::BadRequest("invalid category ID".into()).error_response()),
            )
                .into_response();
        }
    };

    match categories::delete_category(pool, cid).await {
        Ok(true) => (
            StatusCode::OK,
            Json(serde_json::json!({"status": "deleted"})),
        )
            .into_response(),
        Ok(false) => (
            StatusCode::NOT_FOUND,
            Json(CoreError::NotFound("category not found".into()).error_response()),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub async fn search_categories(
    State(state): State<AppState>,
    Query(params): Query<CategorySearchParams>,
    _auth: AuthUser,
) -> Response {
    let pool = state.db.pool();
    let search = params.search.unwrap_or_default();

    match categories::search_categories(pool, &search, params.limit).await {
        Ok(cats) => (StatusCode::OK, Json(cats)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub async fn list_action_cats(
    State(state): State<AppState>,
    Path(action_id): Path<String>,
    _auth: AuthUser,
) -> Response {
    let pool = state.db.pool();
    let aid = match Uuid::parse_str(&action_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(CoreError::BadRequest("invalid action ID".into()).error_response()),
            )
                .into_response();
        }
    };

    match categories::list_action_categories(pool, aid).await {
        Ok(cats) => (StatusCode::OK, Json(cats)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub async fn add_action_to_category(
    State(state): State<AppState>,
    Path((action_id, category_id)): Path<(String, String)>,
    _auth: AuthUser,
) -> Response {
    let pool = state.db.pool();
    let aid = match Uuid::parse_str(&action_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(CoreError::BadRequest("invalid action ID".into()).error_response()),
            )
                .into_response();
        }
    };
    let cid = match Uuid::parse_str(&category_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(CoreError::BadRequest("invalid category ID".into()).error_response()),
            )
                .into_response();
        }
    };

    match categories::add_action_to_category(pool, aid, cid).await {
        Ok(member) => (StatusCode::CREATED, Json(member)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub async fn remove_action_from_cat(
    State(state): State<AppState>,
    Path((action_id, category_id)): Path<(String, String)>,
    _auth: AuthUser,
) -> Response {
    let pool = state.db.pool();
    let aid = match Uuid::parse_str(&action_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(CoreError::BadRequest("invalid action ID".into()).error_response()),
            )
                .into_response();
        }
    };
    let cid = match Uuid::parse_str(&category_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(CoreError::BadRequest("invalid category ID".into()).error_response()),
            )
                .into_response();
        }
    };

    match categories::remove_action_from_category(pool, aid, cid).await {
        Ok(true) => (
            StatusCode::OK,
            Json(serde_json::json!({"status": "removed"})),
        )
            .into_response(),
        Ok(false) => (
            StatusCode::NOT_FOUND,
            Json(CoreError::NotFound("membership not found".into()).error_response()),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub async fn list_category_actions(
    State(state): State<AppState>,
    Path(category_id): Path<String>,
    _auth: AuthUser,
) -> Response {
    let pool = state.db.pool();
    let cid = match Uuid::parse_str(&category_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(CoreError::BadRequest("invalid category ID".into()).error_response()),
            )
                .into_response();
        }
    };

    match categories::list_category_actions(pool, cid).await {
        Ok(actions) => (StatusCode::OK, Json(actions)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub async fn get_category_analytics(
    State(state): State<AppState>,
    Path(category_id): Path<String>,
    _auth: AuthUser,
) -> Response {
    let pool = state.db.pool();
    let cid = match Uuid::parse_str(&category_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(CoreError::BadRequest("invalid category ID".into()).error_response()),
            )
                .into_response();
        }
    };

    match categories::get_category_analytics(pool, cid).await {
        Ok(analytics) => (StatusCode::OK, Json(analytics)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

// ---------------------------------------------------------------------------
// Reviews V2 handlers
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct HelpfulRequest {
    pub helpful: bool,
}

#[derive(Debug, Deserialize)]
pub struct ModerateReviewRequest {
    pub reason: String,
}

#[derive(Debug, Deserialize)]
pub struct ModerateActionRequest {
    pub status: String,
}

#[derive(Debug, Deserialize)]
pub struct RecommendationsQuery {
    #[serde(default = "default_limit")]
    pub limit: i64,
}

pub async fn list_reviews_v2(
    State(state): State<AppState>,
    Path(action_id): Path<String>,
    _auth: AuthUser,
) -> Response {
    let pool = state.db.pool();
    let aid = match Uuid::parse_str(&action_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(CoreError::BadRequest("invalid action ID".into()).error_response()),
            )
                .into_response();
        }
    };

    match actions::list_action_reviews_v2(pool, aid, 50, 0).await {
        Ok(reviews) => (StatusCode::OK, Json(reviews)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub async fn upsert_review_v2(
    State(state): State<AppState>,
    Path(action_id): Path<String>,
    auth: AuthUser,
    Json(req): Json<ReviewActionRequest>,
) -> Response {
    let pool = state.db.pool();
    let aid = match Uuid::parse_str(&action_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(CoreError::BadRequest("invalid action ID".into()).error_response()),
            )
                .into_response();
        }
    };
    let uid = match Uuid::parse_str(&auth.user_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(CoreError::BadRequest("invalid user ID".into()).error_response()),
            )
                .into_response();
        }
    };

    if req.rating < 1 || req.rating > 5 {
        return (
            StatusCode::BAD_REQUEST,
            Json(CoreError::BadRequest("rating must be between 1 and 5".into()).error_response()),
        )
            .into_response();
    }

    let review = req.review.unwrap_or_default();
    match actions::upsert_action_review_v2(pool, aid, uid, req.rating, &review).await {
        Ok(r) => (StatusCode::OK, Json(r)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub async fn get_my_review_v2(
    State(state): State<AppState>,
    Path(action_id): Path<String>,
    auth: AuthUser,
) -> Response {
    let pool = state.db.pool();
    let aid = match Uuid::parse_str(&action_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(CoreError::BadRequest("invalid action ID".into()).error_response()),
            )
                .into_response();
        }
    };
    let uid = match Uuid::parse_str(&auth.user_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(CoreError::BadRequest("invalid user ID".into()).error_response()),
            )
                .into_response();
        }
    };

    match actions::get_action_review_v2(pool, aid, uid).await {
        Ok(Some(review)) => (StatusCode::OK, Json(review)).into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(CoreError::NotFound("no review found".into()).error_response()),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub async fn delete_review_v2(
    State(state): State<AppState>,
    Path(action_id): Path<String>,
    auth: AuthUser,
) -> Response {
    let pool = state.db.pool();
    let aid = match Uuid::parse_str(&action_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(CoreError::BadRequest("invalid action ID".into()).error_response()),
            )
                .into_response();
        }
    };
    let uid = match Uuid::parse_str(&auth.user_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(CoreError::BadRequest("invalid user ID".into()).error_response()),
            )
                .into_response();
        }
    };

    match actions::delete_action_review_v2(pool, aid, uid).await {
        Ok(true) => (
            StatusCode::OK,
            Json(serde_json::json!({"status": "deleted"})),
        )
            .into_response(),
        Ok(false) => (
            StatusCode::NOT_FOUND,
            Json(CoreError::NotFound("review not found".into()).error_response()),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub async fn toggle_helpful(
    State(state): State<AppState>,
    Path((action_id, review_id)): Path<(String, String)>,
    auth: AuthUser,
    Json(req): Json<HelpfulRequest>,
) -> Response {
    let pool = state.db.pool();
    let _aid = match Uuid::parse_str(&action_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(CoreError::BadRequest("invalid action ID".into()).error_response()),
            )
                .into_response();
        }
    };
    let rid = match Uuid::parse_str(&review_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(CoreError::BadRequest("invalid review ID".into()).error_response()),
            )
                .into_response();
        }
    };
    let uid = match Uuid::parse_str(&auth.user_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(CoreError::BadRequest("invalid user ID".into()).error_response()),
            )
                .into_response();
        }
    };

    match actions::toggle_review_helpfulness(pool, rid, uid, req.helpful).await {
        Ok(()) => (
            StatusCode::OK,
            Json(serde_json::json!({"status": "updated"})),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub async fn get_review_analytics_endpoint(
    State(state): State<AppState>,
    Path(action_id): Path<String>,
    _auth: AuthUser,
) -> Response {
    let pool = state.db.pool();
    let aid = match Uuid::parse_str(&action_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(CoreError::BadRequest("invalid action ID".into()).error_response()),
            )
                .into_response();
        }
    };

    match actions::get_review_analytics(pool, aid).await {
        Ok(analytics) => (StatusCode::OK, Json(analytics)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub async fn submit_for_moderation(
    State(state): State<AppState>,
    Path((action_id, review_id)): Path<(String, String)>,
    _auth: AuthUser,
    Json(req): Json<ModerateReviewRequest>,
) -> Response {
    let pool = state.db.pool();
    let _aid = match Uuid::parse_str(&action_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(CoreError::BadRequest("invalid action ID".into()).error_response()),
            )
                .into_response();
        }
    };
    let rid = match Uuid::parse_str(&review_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(CoreError::BadRequest("invalid review ID".into()).error_response()),
            )
                .into_response();
        }
    };

    match actions::submit_review_for_moderation(pool, rid, &req.reason).await {
        Ok(()) => (
            StatusCode::CREATED,
            Json(serde_json::json!({"status": "submitted_for_moderation"})),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub async fn moderate_review_endpoint(
    State(state): State<AppState>,
    Path((action_id, moderation_id)): Path<(String, String)>,
    auth: AuthUser,
    Json(req): Json<ModerateActionRequest>,
) -> Response {
    let pool = state.db.pool();
    let _aid = match Uuid::parse_str(&action_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(CoreError::BadRequest("invalid action ID".into()).error_response()),
            )
                .into_response();
        }
    };
    let mid = match Uuid::parse_str(&moderation_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(CoreError::BadRequest("invalid moderation ID".into()).error_response()),
            )
                .into_response();
        }
    };
    let moderator_id = match Uuid::parse_str(&auth.user_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(CoreError::BadRequest("invalid user ID".into()).error_response()),
            )
                .into_response();
        }
    };

    match actions::moderate_review(pool, mid, moderator_id, &req.status).await {
        Ok(()) => (
            StatusCode::OK,
            Json(serde_json::json!({"status": "moderated"})),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub async fn get_user_recommendations(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(params): Query<RecommendationsQuery>,
) -> Response {
    let pool = state.db.pool();
    let uid = match Uuid::parse_str(&auth.user_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(CoreError::BadRequest("invalid user ID".into()).error_response()),
            )
                .into_response();
        }
    };

    match actions::get_recommendations_for_user(pool, uid, params.limit).await {
        Ok(actions) => (StatusCode::OK, Json(actions)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

// ---------------------------------------------------------------------------
// Action Installation handlers
// ---------------------------------------------------------------------------

async fn resolve_repo_id_for_install(
    pool: &sqlx::PgPool,
    owner: &str,
    name: &str,
) -> std::result::Result<Uuid, Response> {
    let owner_uuid = if let Ok(id) = Uuid::parse_str(owner) {
        id
    } else {
        let user_row = sqlx::query_as::<_, (Uuid,)>("SELECT id FROM users WHERE username = $1")
            .bind(owner)
            .fetch_optional(pool)
            .await;

        match user_row {
            Ok(Some((id,))) => id,
            Ok(None) => {
                return Err((
                    StatusCode::NOT_FOUND,
                    Json(CoreError::NotFound("user not found".into()).error_response())
                        .into_response(),
                )
                    .into_response());
            }
            Err(e) => {
                return Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(CoreError::Database(e.to_string()).error_response())
                        .into_response(),
                )
                    .into_response());
            }
        }
    };

    let repo_row = sqlx::query_as::<_, (Uuid,)>(
        "SELECT id FROM repositories WHERE owner_id = $1 AND name = $2",
    )
    .bind(owner_uuid)
    .bind(name)
    .fetch_optional(pool)
    .await;

    match repo_row {
        Ok(Some((id,))) => Ok(id),
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json(CoreError::NotFound("repository not found".into()).error_response())
                .into_response(),
        )
            .into_response()),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response())
                .into_response(),
        )
            .into_response()),
    }
}

pub async fn list_repo_installations(
    State(state): State<AppState>,
    Path((owner, name)): Path<(String, String)>,
    _auth: AuthUser,
) -> Response {
    let pool = state.db.pool();
    let repo_id = match resolve_repo_id_for_install(pool, &owner, &name).await {
        Ok(id) => id,
        Err(resp) => return resp,
    };

    match action_installations::list_installations(pool, repo_id).await {
        Ok(installations) => (StatusCode::OK, Json(installations)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub async fn install_action_to_repo(
    State(state): State<AppState>,
    Path((owner, name, action_id)): Path<(String, String, String)>,
    auth: AuthUser,
    Json(req): Json<InstallActionRequest>,
) -> Response {
    let pool = state.db.pool();
    let repo_id = match resolve_repo_id_for_install(pool, &owner, &name).await {
        Ok(id) => id,
        Err(resp) => return resp,
    };

    let aid = match Uuid::parse_str(&action_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(CoreError::BadRequest("invalid action ID".into()).error_response()),
            )
                .into_response();
        }
    };

    let uid = match Uuid::parse_str(&auth.user_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(CoreError::BadRequest("invalid user ID".into()).error_response()),
            )
                .into_response();
        }
    };

    match action_installations::install_action(pool, aid, repo_id, uid, &req.version, &req.config).await {
        Ok(installation) => (StatusCode::CREATED, Json(installation)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub async fn uninstall_action_from_repo(
    State(state): State<AppState>,
    Path((owner, name, installation_id)): Path<(String, String, String)>,
    _auth: AuthUser,
) -> Response {
    let pool = state.db.pool();
    let _repo_id = match resolve_repo_id_for_install(pool, &owner, &name).await {
        Ok(id) => id,
        Err(resp) => return resp,
    };

    let iid = match Uuid::parse_str(&installation_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(CoreError::BadRequest("invalid installation ID".into()).error_response()),
            )
                .into_response();
        }
    };

    match action_installations::uninstall_action(pool, iid).await {
        Ok(true) => (
            StatusCode::OK,
            Json(serde_json::json!({"status": "uninstalled"})),
        )
            .into_response(),
        Ok(false) => (
            StatusCode::NOT_FOUND,
            Json(CoreError::NotFound("installation not found".into()).error_response()),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

// ---------------------------------------------------------------------------
// Reviews V19 handlers
// ---------------------------------------------------------------------------

pub async fn list_reviews_v19(
    State(state): State<AppState>,
    Path(action_id): Path<String>,
    _auth: AuthUser,
) -> Response {
    let pool = state.db.pool();
    let aid = match Uuid::parse_str(&action_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(CoreError::BadRequest("invalid action ID".into()).error_response()),
            )
                .into_response();
        }
    };

    match actions::list_action_reviews_v19(pool, aid, 50, 0).await {
        Ok(reviews) => (StatusCode::OK, Json(reviews)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub async fn upsert_review_v19(
    State(state): State<AppState>,
    Path(action_id): Path<String>,
    auth: AuthUser,
    Json(req): Json<ReviewActionRequest>,
) -> Response {
    let pool = state.db.pool();
    let aid = match Uuid::parse_str(&action_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(CoreError::BadRequest("invalid action ID".into()).error_response()),
            )
                .into_response();
        }
    };
    let uid = match Uuid::parse_str(&auth.user_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(CoreError::BadRequest("invalid user ID".into()).error_response()),
            )
                .into_response();
        }
    };

    if req.rating < 1 || req.rating > 5 {
        return (
            StatusCode::BAD_REQUEST,
            Json(CoreError::BadRequest("rating must be between 1 and 5".into()).error_response()),
        )
            .into_response();
    }

    let review = req.review.unwrap_or_default();
    match actions::upsert_action_review_v19(pool, aid, uid, req.rating, &review).await {
        Ok(r) => (StatusCode::OK, Json(r)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub async fn get_my_review_v19(
    State(state): State<AppState>,
    Path(action_id): Path<String>,
    auth: AuthUser,
) -> Response {
    let pool = state.db.pool();
    let aid = match Uuid::parse_str(&action_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(CoreError::BadRequest("invalid action ID".into()).error_response()),
            )
                .into_response();
        }
    };
    let uid = match Uuid::parse_str(&auth.user_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(CoreError::BadRequest("invalid user ID".into()).error_response()),
            )
                .into_response();
        }
    };

    match actions::get_action_review_v19(pool, aid, uid).await {
        Ok(Some(review)) => (StatusCode::OK, Json(review)).into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(CoreError::NotFound("no review found".into()).error_response()),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub async fn delete_review_v19(
    State(state): State<AppState>,
    Path(action_id): Path<String>,
    auth: AuthUser,
) -> Response {
    let pool = state.db.pool();
    let aid = match Uuid::parse_str(&action_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(CoreError::BadRequest("invalid action ID".into()).error_response()),
            )
                .into_response();
        }
    };
    let uid = match Uuid::parse_str(&auth.user_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(CoreError::BadRequest("invalid user ID".into()).error_response()),
            )
                .into_response();
        }
    };

    match actions::delete_action_review_v19(pool, aid, uid).await {
        Ok(true) => (
            StatusCode::OK,
            Json(serde_json::json!({"status": "deleted"})),
        )
            .into_response(),
        Ok(false) => (
            StatusCode::NOT_FOUND,
            Json(CoreError::NotFound("review not found".into()).error_response()),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub async fn toggle_helpful_v19(
    State(state): State<AppState>,
    Path((action_id, review_id)): Path<(String, String)>,
    auth: AuthUser,
    Json(req): Json<HelpfulRequest>,
) -> Response {
    let pool = state.db.pool();
    let _aid = match Uuid::parse_str(&action_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(CoreError::BadRequest("invalid action ID".into()).error_response()),
            )
                .into_response();
        }
    };
    let rid = match Uuid::parse_str(&review_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(CoreError::BadRequest("invalid review ID".into()).error_response()),
            )
                .into_response();
        }
    };
    let uid = match Uuid::parse_str(&auth.user_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(CoreError::BadRequest("invalid user ID".into()).error_response()),
            )
                .into_response();
        }
    };

    match actions::toggle_review_helpfulness_v19(pool, rid, uid, req.helpful).await {
        Ok(()) => (
            StatusCode::OK,
            Json(serde_json::json!({"status": "updated"})),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub async fn get_review_analytics_v22_endpoint(
    State(state): State<AppState>,
    Path(action_id): Path<String>,
    _auth: AuthUser,
) -> Response {
    let pool = state.db.pool();
    let aid = match Uuid::parse_str(&action_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(CoreError::BadRequest("invalid action ID".into()).error_response()),
            )
                .into_response();
        }
    };

    match actions::get_review_analytics_v22(pool, aid).await {
        Ok(analytics) => (StatusCode::OK, Json(analytics)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub async fn moderate_review_v22_endpoint(
    State(state): State<AppState>,
    Path((action_id, review_id)): Path<(String, String)>,
    auth: AuthUser,
    Json(req): Json<ModerateActionRequest>,
) -> Response {
    let pool = state.db.pool();
    let _aid = match Uuid::parse_str(&action_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(CoreError::BadRequest("invalid action ID".into()).error_response()),
            )
                .into_response();
        }
    };
    let rid = match Uuid::parse_str(&review_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(CoreError::BadRequest("invalid review ID".into()).error_response()),
            )
                .into_response();
        }
    };
    let moderator_id = match Uuid::parse_str(&auth.user_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(CoreError::BadRequest("invalid user ID".into()).error_response()),
            )
                .into_response();
        }
    };

    match actions::moderate_review_v22(pool, rid, moderator_id, &req.status).await {
        Ok(()) => (
            StatusCode::OK,
            Json(serde_json::json!({"status": "moderated"})),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub async fn get_review_recommendations_v22_endpoint(
    State(state): State<AppState>,
    Path(action_id): Path<String>,
    _auth: AuthUser,
) -> Response {
    let pool = state.db.pool();
    let aid = match Uuid::parse_str(&action_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(CoreError::BadRequest("invalid action ID".into()).error_response()),
            )
                .into_response();
        }
    };

    match actions::get_review_recommendations_v22(pool, aid, 10).await {
        Ok(actions) => (StatusCode::OK, Json(actions)).into_response(),
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
    fn test_create_action_request() {
        let json = r#"{"name": "docker-build", "action_type": "docker", "description": "Build Docker images"}"#;
        let req: CreateActionRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.name, "docker-build");
        assert_eq!(req.action_type, "docker");
    }

    #[test]
    fn test_update_action_request() {
        let json = r#"{"name": "updated-name"}"#;
        let req: UpdateActionRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.name.as_deref(), Some("updated-name"));
    }

    #[test]
    fn test_list_params_defaults() {
        let json = r#"{}"#;
        let params: ActionListParams = serde_json::from_str(json).unwrap();
        assert_eq!(params.page, 1);
        assert_eq!(params.per_page, 20);
    }

    #[test]
    fn test_rate_action_request() {
        let json = r#"{"rating": 4.5}"#;
        let req: RateActionRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.rating, 4.5);
    }

    #[test]
    fn test_pipeline_action_routes_compile() {
        let _ = pipeline_action_routes();
    }
}
