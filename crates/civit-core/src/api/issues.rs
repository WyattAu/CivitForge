#![forbid(unsafe_code)]

use crate::api::AppState;
use crate::api::auth::AuthUser;
use crate::error::CoreError;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Json};
use chrono::{DateTime, Utc};
use civit_shared::{ListResponse, Pagination};
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Query / request param structs
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, Default)]
pub struct ListIssuesParams {
    pub state: Option<String>,
    pub label: Option<String>,
    pub assignee: Option<uuid::Uuid>,
    pub milestone: Option<uuid::Uuid>,
    pub sort: Option<String>,
    #[serde(default = "default_page")]
    pub page: i32,
    #[serde(default = "default_per_page")]
    pub per_page: i32,
}

#[derive(Debug, Deserialize, Default)]
pub struct CreateIssueRequest {
    pub title: String,
    pub description: Option<String>,
    pub assignee: Option<uuid::Uuid>,
    pub milestone: Option<uuid::Uuid>,
    pub labels: Option<Vec<uuid::Uuid>>,
}

#[derive(Debug, Deserialize, Default)]
pub struct UpdateIssueRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    pub state: Option<String>,
    pub assignee: Option<uuid::Uuid>,
    pub milestone: Option<uuid::Uuid>,
    pub labels: Option<Vec<uuid::Uuid>>,
}

#[derive(Debug, Deserialize, Default)]
pub struct CreateCommentRequest {
    pub body: String,
}

#[derive(Debug, Deserialize, Default)]
pub struct UpdateCommentRequest {
    pub body: String,
}

#[derive(Debug, Deserialize, Default)]
pub struct AddReactionRequest {
    pub emoji: String,
}

#[derive(Debug, Deserialize, Default)]
pub struct CreateLabelRequest {
    pub name: String,
    pub color: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
pub struct UpdateLabelRequest {
    pub name: Option<String>,
    pub color: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
pub struct CreateMilestoneRequest {
    pub title: String,
    pub description: Option<String>,
    pub due_on: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize, Default)]
pub struct UpdateMilestoneRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    pub state: Option<String>,
    pub due_on: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize, Default)]
pub struct ListMilestonesParams {
    pub state: Option<String>,
    #[serde(default = "default_page")]
    pub page: i32,
    #[serde(default = "default_per_page")]
    pub per_page: i32,
}

fn default_page() -> i32 {
    1
}

fn default_per_page() -> i32 {
    30
}

// ---------------------------------------------------------------------------
// Response structs
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize)]
pub struct IssueResponse {
    pub id: uuid::Uuid,
    pub repo_id: uuid::Uuid,
    pub number: i32,
    pub title: String,
    pub body: Option<String>,
    pub status: String,
    pub author_id: uuid::Uuid,
    pub assignee_id: Option<uuid::Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub closed_at: Option<DateTime<Utc>>,
    pub labels: Vec<LabelResponse>,
    pub assignees: Vec<IssueAssignee>,
    pub comments: Vec<CommentResponse>,
}

#[derive(Debug, sqlx::FromRow, Serialize)]
pub struct LabelResponse {
    pub id: uuid::Uuid,
    pub repo_id: uuid::Uuid,
    pub name: String,
    pub color: Option<String>,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, sqlx::FromRow, Serialize)]
pub struct IssueAssignee {
    pub user_id: uuid::Uuid,
    pub assigned_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct CommentResponse {
    pub id: uuid::Uuid,
    pub issue_id: uuid::Uuid,
    pub author_id: uuid::Uuid,
    pub body: String,
    pub is_edited: bool,
    pub edited_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub cross_references: Vec<CrossReferenceResponse>,
}

#[derive(Debug, sqlx::FromRow)]
struct CommentRow {
    pub id: uuid::Uuid,
    pub issue_id: uuid::Uuid,
    pub author_id: uuid::Uuid,
    pub body: String,
    pub is_edited: bool,
    pub edited_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

impl From<CommentRow> for CommentResponse {
    fn from(row: CommentRow) -> Self {
        Self {
            id: row.id,
            issue_id: row.issue_id,
            author_id: row.author_id,
            body: row.body,
            is_edited: row.is_edited,
            edited_at: row.edited_at,
            created_at: row.created_at,
            cross_references: Vec::new(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct CrossReferenceResponse {
    pub target_number: i32,
    pub target_type: String,
}

#[derive(Debug, Deserialize, Default)]
pub struct AddIssueAssigneeRequest {
    pub assignee_id: uuid::Uuid,
}

#[derive(Debug, sqlx::FromRow, Serialize)]
pub struct MilestoneResponse {
    pub id: uuid::Uuid,
    pub repo_id: uuid::Uuid,
    pub title: String,
    pub description: Option<String>,
    pub state: String,
    pub due_on: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, sqlx::FromRow, Serialize)]
pub struct ReactionResponse {
    pub id: uuid::Uuid,
    pub issue_id: uuid::Uuid,
    pub comment_id: Option<uuid::Uuid>,
    pub user_id: uuid::Uuid,
    pub emoji: String,
    pub created_at: DateTime<Utc>,
}

// ---------------------------------------------------------------------------
// Helper: get repo id
// ---------------------------------------------------------------------------

async fn get_repo_id(pool: &sqlx::PgPool, owner: &str, name: &str) -> Option<uuid::Uuid> {
    sqlx::query_scalar::<_, uuid::Uuid>(
        "SELECT r.id FROM repositories r JOIN users u ON r.owner_id = u.id WHERE u.username = $1 AND r.name = $2",
    )
    .bind(owner)
    .bind(name)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten()
}

// ---------------------------------------------------------------------------
// Helper: state machine validation
// ---------------------------------------------------------------------------

fn validate_state_transition(current: &str, next: &str) -> bool {
    matches!(
        (current, next),
        ("open", "open" | "in_progress")
            | ("in_progress", "in_progress" | "closed")
            | ("closed", "open")
    )
}

// ---------------------------------------------------------------------------
// Helper: insert timeline event
// ---------------------------------------------------------------------------

async fn insert_timeline(
    pool: &sqlx::PgPool,
    issue_id: uuid::Uuid,
    actor_id: uuid::Uuid,
    event_type: &str,
    event_detail: Option<&str>,
) {
    let _ = sqlx::query(
        "INSERT INTO issue_timeline (issue_id, actor_id, event_type, event_detail, created_at) VALUES ($1, $2, $3, $4, NOW())",
    )
    .bind(issue_id)
    .bind(actor_id)
    .bind(event_type)
    .bind(event_detail)
    .execute(pool)
    .await;
}

// ---------------------------------------------------------------------------
// Helper: error response shorthand
// ---------------------------------------------------------------------------

fn err_response(status: StatusCode, msg: &str) -> axum::response::Response {
    (
        status,
        Json(CoreError::NotFound(msg.to_string()).error_response()),
    )
        .into_response()
}

fn internal_err(msg: &str) -> axum::response::Response {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(CoreError::Database(msg.to_string()).error_response()),
    )
        .into_response()
}

// ---------------------------------------------------------------------------
// 1. GET /issues — list
// ---------------------------------------------------------------------------

pub async fn list_issues(
    State(state): State<AppState>,
    Path((owner, name)): Path<(String, String)>,
    Query(params): Query<ListIssuesParams>,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let repo_id = match get_repo_id(pool, &owner, &name).await {
        Some(id) => id,
        None => {
            return err_response(
                StatusCode::NOT_FOUND,
                &format!("repository {owner}/{name} not found"),
            );
        }
    };

    let offset = (params.page - 1) * params.per_page;
    let sort_col = params.sort.as_deref().unwrap_or("created_at");
    let sort_col = match sort_col {
        "updated_at" | "created_at" | "number" => sort_col,
        _ => "created_at",
    };

    let mut base = String::from(
        "SELECT id, repo_id, number, title, body, status, author_id, assignee_id, labels, created_at, updated_at, closed_at FROM issues WHERE repo_id = $1",
    );
    let mut count_base = String::from("SELECT COUNT(*) FROM issues WHERE repo_id = $1");
    let mut bind_idx = 2i32;

    if let Some(ref s) = params.state {
        if !s.is_empty() {
            let clause = format!(" AND status = ${bind_idx}");
            base.push_str(&clause);
            count_base.push_str(&clause);
            bind_idx += 1;
        }
    }
    if let Some(ref label_name) = params.label {
        if !label_name.is_empty() {
            let clause = format!(
                " AND id IN (SELECT issue_id FROM issue_labels WHERE label_id IN (SELECT id FROM labels WHERE repo_id = $1 AND name = ${bind_idx}))"
            );
            base.push_str(&clause);
            count_base.push_str(&clause);
            bind_idx += 1;
        }
    }
    if let Some(_assignee_id) = params.assignee {
        let clause = format!(
            " AND id IN (SELECT issue_id FROM issue_assignees WHERE user_id = ${bind_idx})"
        );
        base.push_str(&clause);
        count_base.push_str(&clause);
        bind_idx += 1;
    }

    let query_str = format!(
        "{base} ORDER BY {sort_col} DESC LIMIT ${bind_idx} OFFSET ${bind_idx_plus}",
        base = base,
        sort_col = sort_col,
        bind_idx = bind_idx,
        bind_idx_plus = bind_idx + 1,
    );

    let mut query = sqlx::query_as::<_, IssueRow>(&query_str).bind(repo_id);
    let mut count_query = sqlx::query_scalar::<_, i64>(&count_base).bind(repo_id);

    if let Some(ref s) = params.state {
        if !s.is_empty() {
            query = query.bind(s.clone());
            count_query = count_query.bind(s.clone());
        }
    }
    if let Some(ref label_name) = params.label {
        if !label_name.is_empty() {
            query = query.bind(label_name.clone());
            count_query = count_query.bind(label_name.clone());
        }
    }
    if let Some(assignee_id) = params.assignee {
        query = query.bind(assignee_id);
        count_query = count_query.bind(assignee_id);
    }
    query = query.bind(params.per_page).bind(offset);

    let rows = match query.fetch_all(pool).await {
        Ok(r) => r,
        Err(e) => return internal_err(&e.to_string()),
    };
    let total = match count_query.fetch_one(pool).await {
        Ok(t) => t,
        Err(e) => return internal_err(&e.to_string()),
    };

    let resp = ListResponse {
        data: rows,
        pagination: Pagination {
            page: params.page as u32,
            per_page: params.per_page as u32,
            total: total as u64,
            total_pages: if total == 0 {
                1
            } else {
                (total as u64).div_ceil(params.per_page as u64) as u32
            },
        },
    };

    (StatusCode::OK, Json(resp)).into_response()
}

#[derive(Debug, sqlx::FromRow, Serialize)]
struct IssueRow {
    pub id: uuid::Uuid,
    pub repo_id: uuid::Uuid,
    pub number: i32,
    pub title: String,
    pub body: Option<String>,
    pub status: String,
    pub author_id: uuid::Uuid,
    pub assignee_id: Option<uuid::Uuid>,
    pub labels: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub closed_at: Option<DateTime<Utc>>,
}

// ---------------------------------------------------------------------------
// 2. POST /issues — create
// ---------------------------------------------------------------------------

pub async fn create_issue(
    State(state): State<AppState>,
    Path((owner, name)): Path<(String, String)>,
    auth: AuthUser,
    Json(req): Json<CreateIssueRequest>,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let repo_id = match get_repo_id(pool, &owner, &name).await {
        Some(id) => id,
        None => {
            return err_response(
                StatusCode::NOT_FOUND,
                &format!("repository {owner}/{name} not found"),
            );
        }
    };

    if req.title.trim().is_empty() {
        return err_response(StatusCode::BAD_REQUEST, "title is required");
    }

    let author_id = match uuid::Uuid::parse_str(&auth.user_id) {
        Ok(id) => id,
        Err(_) => return err_response(StatusCode::UNAUTHORIZED, "invalid user id in token"),
    };

    let row = match sqlx::query_as::<_, IssueRow>(
        "INSERT INTO issues (repo_id, number, title, body, status, author_id, assignee_id, created_at, updated_at) VALUES ($1, (SELECT COALESCE(MAX(number),0)+1 FROM issues WHERE repo_id=$1), $2, $3, 'open', $4, $5, NOW(), NOW()) RETURNING id, repo_id, number, title, body, status, author_id, assignee_id, labels, created_at, updated_at, closed_at",
    )
    .bind(repo_id)
    .bind(&req.title)
    .bind(&req.description)
    .bind(author_id)
    .bind(req.assignee)
    .fetch_one(pool)
    .await
    {
        Ok(r) => r,
        Err(e) => return internal_err(&e.to_string()),
    };

    if let Some(ref label_ids) = req.labels {
        for lid in label_ids {
            let _ = sqlx::query("INSERT INTO issue_labels (issue_id, label_id) VALUES ($1, $2) ON CONFLICT DO NOTHING")
                .bind(row.id)
                .bind(*lid)
                .execute(pool)
                .await;
        }
    }

    insert_timeline(pool, row.id, row.author_id, "opened", None).await;

    let dispatcher = crate::webhooks::WebhookDispatcher::new();
    let pool_clone = state.db.pool().clone();
    let rid = repo_id;
    let evt = crate::webhooks::WebhookEvent::Issue;
    let pl = serde_json::json!({
        "action": "opened",
        "issue_number": row.number,
        "repo_id": rid.to_string(),
        "title": row.title,
        "status": row.status,
        "author_id": row.author_id.to_string(),
    });
    tokio::spawn(async move { dispatcher.dispatch(&pool_clone, rid, &evt, pl).await });

    // Deliver federation activity to followers
    if state.config.federation_enabled {
        let domain = &state.config.federation_instance_domain;
        let activity = crate::federation::activitypub::Activity {
            r#type: crate::federation::activitypub::ActivityType::Create,
            id: format!("https://{domain}/activities/{}", uuid::Uuid::new_v4()),
            actor: format!("https://{domain}/api/v1/users/{}", auth.user_id),
            object: crate::federation::activitypub::ActivityObject::Issue {
                id: row.id.to_string(),
                name: row.title.clone(),
                attributed_to: auth.username.clone(),
            },
            target: None,
            published: chrono::Utc::now().to_rfc3339(),
            to: vec![format!("https://{domain}/api/v1/federation/actor")],
            cc: vec![],
        };
        crate::api::federation_routes::deliver_to_followers(activity, state.db.pool().clone()).await;
    }

    (StatusCode::CREATED, Json(row)).into_response()
}

// ---------------------------------------------------------------------------
// 3. GET /issues/:number — detail
// ---------------------------------------------------------------------------

pub async fn get_issue(
    State(state): State<AppState>,
    Path((owner, name, number)): Path<(String, String, i32)>,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let repo_id = match get_repo_id(pool, &owner, &name).await {
        Some(id) => id,
        None => {
            return err_response(
                StatusCode::NOT_FOUND,
                &format!("repository {owner}/{name} not found"),
            );
        }
    };

    let row = match sqlx::query_as::<_, IssueRow>(
        "SELECT id, repo_id, number, title, body, status, author_id, assignee_id, labels, created_at, updated_at, closed_at FROM issues WHERE repo_id = $1 AND number = $2",
    )
    .bind(repo_id)
    .bind(number)
    .fetch_optional(pool)
    .await
    {
        Ok(Some(r)) => r,
        Ok(None) => {
            return err_response(
                StatusCode::NOT_FOUND,
                &format!("issue #{number} not found"),
            );
        }
        Err(e) => return internal_err(&e.to_string()),
    };

    let labels = match sqlx::query_as::<_, LabelResponse>(
        "SELECT l.id, l.repo_id, l.name, l.color, l.description, l.created_at FROM labels l INNER JOIN issue_labels il ON l.id = il.label_id WHERE il.issue_id = $1",
    )
    .bind(row.id)
    .fetch_all(pool)
    .await
    {
        Ok(r) => r,
        Err(e) => return internal_err(&e.to_string()),
    };

    let assignees = match sqlx::query_as::<_, IssueAssignee>(
        "SELECT user_id, assigned_at FROM issue_assignees WHERE issue_id = $1",
    )
    .bind(row.id)
    .fetch_all(pool)
    .await
    {
        Ok(r) => r,
        Err(e) => return internal_err(&e.to_string()),
    };

    let comments = match sqlx::query_as::<_, CommentRow>(
        "SELECT id, issue_id, author_id, body, is_edited, edited_at, created_at FROM issue_comments WHERE issue_id = $1 ORDER BY created_at",
    )
    .bind(row.id)
    .fetch_all(pool)
    .await
    {
        Ok(r) => r.into_iter().map(CommentResponse::from).collect(),
        Err(e) => return internal_err(&e.to_string()),
    };

    let resp = IssueResponse {
        id: row.id,
        repo_id: row.repo_id,
        number: row.number,
        title: row.title,
        body: row.body,
        status: row.status,
        author_id: row.author_id,
        assignee_id: row.assignee_id,
        created_at: row.created_at,
        updated_at: row.updated_at,
        closed_at: row.closed_at,
        labels,
        assignees,
        comments,
    };

    (StatusCode::OK, Json(resp)).into_response()
}

// ---------------------------------------------------------------------------
// 4. PATCH /issues/:number — update
// ---------------------------------------------------------------------------

pub async fn update_issue(
    State(state): State<AppState>,
    Path((owner, name, number)): Path<(String, String, i32)>,
    auth: AuthUser,
    Json(req): Json<UpdateIssueRequest>,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let repo_id = match get_repo_id(pool, &owner, &name).await {
        Some(id) => id,
        None => {
            return err_response(
                StatusCode::NOT_FOUND,
                &format!("repository {owner}/{name} not found"),
            );
        }
    };

    let existing = match sqlx::query_as::<_, IssueRow>(
        "SELECT id, repo_id, number, title, body, status, author_id, assignee_id, labels, created_at, updated_at, closed_at FROM issues WHERE repo_id = $1 AND number = $2",
    )
    .bind(repo_id)
    .bind(number)
    .fetch_optional(pool)
    .await
    {
        Ok(Some(r)) => r,
        Ok(None) => {
            return err_response(
                StatusCode::NOT_FOUND,
                &format!("issue #{number} not found"),
            );
        }
        Err(e) => return internal_err(&e.to_string()),
    };

    if let Some(ref new_state) = req.state {
        if !validate_state_transition(&existing.status, new_state) {
            return err_response(
                StatusCode::CONFLICT,
                &format!(
                    "invalid state transition: {old} -> {new}",
                    old = existing.status,
                    new = new_state,
                ),
            );
        }
    }

    let title = req.title.as_deref().unwrap_or(&existing.title);
    let body = req
        .description
        .as_deref()
        .unwrap_or(existing.body.as_deref().unwrap_or(""));
    let status = req.state.as_deref().unwrap_or(&existing.status);
    let assignee_id = req.assignee.or(existing.assignee_id);

    let is_closed = status == "closed";
    let row = match sqlx::query_as::<_, IssueRow>(
        "UPDATE issues SET title = $1, body = $2, status = $3, assignee_id = $4, closed_at = CASE WHEN $5 THEN NOW() ELSE NULL END, updated_at = NOW() WHERE id = $6 RETURNING id, repo_id, number, title, body, status, author_id, assignee_id, labels, created_at, updated_at, closed_at",
    )
    .bind(title)
    .bind(body)
    .bind(status)
    .bind(assignee_id)
    .bind(is_closed)
    .bind(existing.id)
    .fetch_one(pool)
    .await
    {
        Ok(r) => r,
        Err(e) => return internal_err(&e.to_string()),
    };

    if req.state.is_some() {
        insert_timeline(pool, row.id, row.author_id, "state_change", Some(status)).await;
    }
    if req.assignee.is_some() {
        insert_timeline(pool, row.id, row.author_id, "assignee_change", None).await;
    }

    if let Some(ref label_ids) = req.labels {
        let _ = sqlx::query("DELETE FROM issue_labels WHERE issue_id = $1")
            .bind(row.id)
            .execute(pool)
            .await;
        for lid in label_ids {
            let _ = sqlx::query("INSERT INTO issue_labels (issue_id, label_id) VALUES ($1, $2) ON CONFLICT DO NOTHING")
                .bind(row.id)
                .bind(*lid)
                .execute(pool)
                .await;
        }
        insert_timeline(pool, row.id, row.author_id, "label_change", None).await;
    }

    let dispatcher = crate::webhooks::WebhookDispatcher::new();
    let pool_clone = state.db.pool().clone();
    let rid = repo_id;
    let evt = crate::webhooks::WebhookEvent::Issue;
    let pl = serde_json::json!({
        "action": "updated",
        "issue_number": row.number,
        "repo_id": rid.to_string(),
        "title": row.title,
        "status": row.status,
        "author_id": row.author_id.to_string(),
    });
    tokio::spawn(async move { dispatcher.dispatch(&pool_clone, rid, &evt, pl).await });

    // Deliver federation activity on state change
    if state.config.federation_enabled && req.state.is_some() {
        let domain = &state.config.federation_instance_domain;
        let activity = crate::federation::activitypub::Activity {
            r#type: crate::federation::activitypub::ActivityType::Update,
            id: format!("https://{domain}/activities/{}", uuid::Uuid::new_v4()),
            actor: format!("https://{domain}/api/v1/users/{}", auth.user_id),
            object: crate::federation::activitypub::ActivityObject::Issue {
                id: row.id.to_string(),
                name: row.title.clone(),
                attributed_to: auth.username.clone(),
            },
            target: None,
            published: chrono::Utc::now().to_rfc3339(),
            to: vec![format!("https://{domain}/api/v1/federation/actor")],
            cc: vec![],
        };
        crate::api::federation_routes::deliver_to_followers(activity, state.db.pool().clone()).await;
    }

    (StatusCode::OK, Json(row)).into_response()
}

// ---------------------------------------------------------------------------
// 5. DELETE /issues/:number — delete
// ---------------------------------------------------------------------------

pub async fn delete_issue(
    State(state): State<AppState>,
    Path((owner, name, number)): Path<(String, String, i32)>,
    _auth: AuthUser,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let repo_id = match get_repo_id(pool, &owner, &name).await {
        Some(id) => id,
        None => {
            return err_response(
                StatusCode::NOT_FOUND,
                &format!("repository {owner}/{name} not found"),
            );
        }
    };

    let result = sqlx::query("DELETE FROM issues WHERE repo_id = $1 AND number = $2")
        .bind(repo_id)
        .bind(number)
        .execute(pool)
        .await;

    match result {
        Ok(rows) if rows.rows_affected() == 0 => {
            err_response(StatusCode::NOT_FOUND, &format!("issue #{number} not found"))
        }
        Ok(_) => (StatusCode::NO_CONTENT, ()).into_response(),
        Err(e) => internal_err(&e.to_string()),
    }
}

// ---------------------------------------------------------------------------
// 6. POST /issues/:number/comments — add comment
// ---------------------------------------------------------------------------

pub async fn add_comment(
    State(state): State<AppState>,
    Path((owner, name, number)): Path<(String, String, i32)>,
    auth: AuthUser,
    Json(req): Json<CreateCommentRequest>,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let repo_id = match get_repo_id(pool, &owner, &name).await {
        Some(id) => id,
        None => {
            return err_response(
                StatusCode::NOT_FOUND,
                &format!("repository {owner}/{name} not found"),
            );
        }
    };

    let issue_id = match sqlx::query_scalar::<_, uuid::Uuid>(
        "SELECT id FROM issues WHERE repo_id = $1 AND number = $2",
    )
    .bind(repo_id)
    .bind(number)
    .fetch_optional(pool)
    .await
    {
        Ok(Some(id)) => id,
        Ok(None) => {
            return err_response(StatusCode::NOT_FOUND, &format!("issue #{number} not found"));
        }
        Err(e) => return internal_err(&e.to_string()),
    };

    if req.body.trim().is_empty() {
        return err_response(StatusCode::BAD_REQUEST, "comment body is required");
    }

    let author_id = match uuid::Uuid::parse_str(&auth.user_id) {
        Ok(id) => id,
        Err(_) => return err_response(StatusCode::UNAUTHORIZED, "invalid user id in token"),
    };

    let comment = match sqlx::query_as::<_, CommentRow>(
        "INSERT INTO issue_comments (issue_id, author_id, body, is_edited, created_at) VALUES ($1, $2, $3, false, NOW()) RETURNING id, issue_id, author_id, body, is_edited, edited_at, created_at",
    )
    .bind(issue_id)
    .bind(author_id)
    .bind(&req.body)
    .fetch_one(pool)
    .await
    {
        Ok(row) => {
            let id = row.id;
            let issue_id_val = row.issue_id;
            let author_id_val = row.author_id;
            let body = row.body.clone();

            // Parse and store @mentions
            let mentioned_usernames = crate::api::mentions::parse_mentions(&body);
            for username in &mentioned_usernames {
                if let Ok(Some(mentioned_id)) = sqlx::query_scalar::<_, uuid::Uuid>(
                    "SELECT id FROM users WHERE username = $1",
                )
                .bind(username)
                .fetch_optional(pool)
                .await
                {
                    let _ = sqlx::query(
                        "INSERT INTO comment_mentions (comment_id, comment_type, mentioned_user_id, repo_id) VALUES ($1, 'issue', $2, $3) ON CONFLICT DO NOTHING",
                    )
                    .bind(id)
                    .bind(mentioned_id)
                    .bind(repo_id)
                    .execute(pool)
                    .await;

                    // Create notification for mentioned user (skip self-mentions)
                    if mentioned_id != author_id {
                        let _ = sqlx::query(
                            "INSERT INTO notifications (user_id, kind, title, body, repo_name) VALUES ($1, 'mention', $2, $3, $4)",
                        )
                        .bind(mentioned_id)
                        .bind(format!("New mention in issue #{number}"))
                        .bind(format!("{} mentioned you in a comment", auth.username))
                        .bind(format!("{owner}/{name}"))
                        .execute(pool)
                        .await;
                    }
                }
            }

            // Parse and store cross-references
            let target_numbers = crate::api::mentions::parse_cross_references(&body);
            let mut cross_references = Vec::new();
            for target_number in &target_numbers {
                let _ = sqlx::query(
                    "INSERT INTO comment_cross_references (source_comment_id, source_comment_type, source_repo_id, target_number, target_type) VALUES ($1, 'issue', $2, $3, 'issue')",
                )
                .bind(id)
                .bind(repo_id)
                .bind(target_number)
                .execute(pool)
                .await;
                cross_references.push(CrossReferenceResponse {
                    target_number: *target_number,
                    target_type: "issue".to_string(),
                });
            }

            // Insert timeline event
            insert_timeline(pool, issue_id_val, author_id_val, "commented", None).await;

            let dispatcher = crate::webhooks::WebhookDispatcher::new();
            let pool_clone = state.db.pool().clone();
            let rid = repo_id;
            let evt = crate::webhooks::WebhookEvent::IssueComment;
            let pl = serde_json::json!({
                "action": "created",
                "issue_id": issue_id_val.to_string(),
                "repo_id": rid.to_string(),
                "comment_id": id.to_string(),
                "author_id": author_id_val.to_string(),
                "body": body,
            });
            tokio::spawn(async move { dispatcher.dispatch(&pool_clone, rid, &evt, pl).await });

            let comment = CommentResponse {
                id,
                issue_id: issue_id_val,
                author_id: author_id_val,
                body: row.body,
                is_edited: row.is_edited,
                edited_at: row.edited_at,
                created_at: row.created_at,
                cross_references,
            };

            Ok(comment)
        }
        Err(e) => Err(internal_err(&e.to_string())),
    };

    match comment {
        Ok(comment) => (StatusCode::CREATED, Json(comment)).into_response(),
        Err(resp) => resp,
    }
}

// ---------------------------------------------------------------------------
// 7. PATCH /issues/:number/comments/:comment_id — edit comment
// ---------------------------------------------------------------------------

pub async fn edit_comment(
    State(state): State<AppState>,
    Path((owner, name, _number, comment_id)): Path<(String, String, i32, uuid::Uuid)>,
    _auth: AuthUser,
    Json(req): Json<UpdateCommentRequest>,
) -> impl IntoResponse {
    let pool = state.db.pool();

    if req.body.trim().is_empty() {
        return err_response(StatusCode::BAD_REQUEST, "comment body is required");
    }

    let repo_id = match get_repo_id(pool, &owner, &name).await {
        Some(id) => id,
        None => {
            return err_response(
                StatusCode::NOT_FOUND,
                &format!("repository {owner}/{name} not found"),
            );
        }
    };

    match sqlx::query_as::<_, CommentRow>(
        "UPDATE issue_comments SET body = $1, is_edited = true, edited_at = NOW() WHERE id = $2 AND issue_id IN (SELECT id FROM issues WHERE repo_id = $3) RETURNING id, issue_id, author_id, body, is_edited, edited_at, created_at",
    )
    .bind(&req.body)
    .bind(comment_id)
    .bind(repo_id)
    .fetch_optional(pool)
    .await
    {
        Ok(Some(row)) => {
            let resp = CommentResponse::from(row);
            (StatusCode::OK, Json(resp)).into_response()
        }
        Ok(None) => err_response(
            StatusCode::NOT_FOUND,
            &format!("comment #{comment_id} not found"),
        ),
        Err(e) => internal_err(&e.to_string()),
    }
}

// ---------------------------------------------------------------------------
// 8. DELETE /issues/:number/comments/:comment_id — delete comment
// ---------------------------------------------------------------------------

pub async fn delete_comment(
    State(state): State<AppState>,
    Path((owner, name, _number, comment_id)): Path<(String, String, i32, uuid::Uuid)>,
    _auth: AuthUser,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let repo_id = match get_repo_id(pool, &owner, &name).await {
        Some(id) => id,
        None => {
            return err_response(
                StatusCode::NOT_FOUND,
                &format!("repository {owner}/{name} not found"),
            );
        }
    };

    let result = sqlx::query(
        "DELETE FROM issue_comments WHERE id = $1 AND issue_id IN (SELECT id FROM issues WHERE repo_id = $2)",
    )
    .bind(comment_id)
    .bind(repo_id)
    .execute(pool)
    .await;

    match result {
        Ok(rows) if rows.rows_affected() == 0 => err_response(
            StatusCode::NOT_FOUND,
            &format!("comment #{comment_id} not found"),
        ),
        Ok(_) => (StatusCode::NO_CONTENT, ()).into_response(),
        Err(e) => internal_err(&e.to_string()),
    }
}

// ---------------------------------------------------------------------------
// 9. POST /issues/:number/reactions — add reaction
// ---------------------------------------------------------------------------

pub async fn add_reaction(
    State(state): State<AppState>,
    Path((owner, name, number)): Path<(String, String, i32)>,
    auth: AuthUser,
    Json(req): Json<AddReactionRequest>,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let repo_id = match get_repo_id(pool, &owner, &name).await {
        Some(id) => id,
        None => {
            return err_response(
                StatusCode::NOT_FOUND,
                &format!("repository {owner}/{name} not found"),
            );
        }
    };

    let issue_id = match sqlx::query_scalar::<_, uuid::Uuid>(
        "SELECT id FROM issues WHERE repo_id = $1 AND number = $2",
    )
    .bind(repo_id)
    .bind(number)
    .fetch_optional(pool)
    .await
    {
        Ok(Some(id)) => id,
        Ok(None) => {
            return err_response(StatusCode::NOT_FOUND, &format!("issue #{number} not found"));
        }
        Err(e) => return internal_err(&e.to_string()),
    };

    let user_id = match uuid::Uuid::parse_str(&auth.user_id) {
        Ok(id) => id,
        Err(_) => return err_response(StatusCode::UNAUTHORIZED, "invalid user id in token"),
    };

    match sqlx::query_as::<_, ReactionResponse>(
        "INSERT INTO issue_reactions (issue_id, comment_id, user_id, emoji, created_at) VALUES ($1, NULL, $2, $3, NOW()) RETURNING id, issue_id, comment_id, user_id, emoji, created_at",
    )
    .bind(issue_id)
    .bind(user_id)
    .bind(&req.emoji)
    .fetch_one(pool)
    .await
    {
        Ok(reaction) => (StatusCode::CREATED, Json(reaction)).into_response(),
        Err(e) => internal_err(&e.to_string()),
    }
}

// ---------------------------------------------------------------------------
// 10. DELETE /issues/:number/reactions/:emoji — remove reaction
// ---------------------------------------------------------------------------

pub async fn remove_reaction(
    State(state): State<AppState>,
    Path((owner, name, number, emoji)): Path<(String, String, i32, String)>,
    auth: AuthUser,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let repo_id = match get_repo_id(pool, &owner, &name).await {
        Some(id) => id,
        None => {
            return err_response(
                StatusCode::NOT_FOUND,
                &format!("repository {owner}/{name} not found"),
            );
        }
    };

    let user_id = match uuid::Uuid::parse_str(&auth.user_id) {
        Ok(id) => id,
        Err(_) => return err_response(StatusCode::UNAUTHORIZED, "invalid user id in token"),
    };

    let result = sqlx::query(
        "DELETE FROM issue_reactions WHERE issue_id IN (SELECT id FROM issues WHERE repo_id = $1 AND number = $2) AND user_id = $3 AND emoji = $4",
    )
    .bind(repo_id)
    .bind(number)
    .bind(user_id)
    .bind(&emoji)
    .execute(pool)
    .await;

    match result {
        Ok(rows) if rows.rows_affected() == 0 => err_response(
            StatusCode::NOT_FOUND,
            &format!("reaction '{emoji}' not found"),
        ),
        Ok(_) => (StatusCode::NO_CONTENT, ()).into_response(),
        Err(e) => internal_err(&e.to_string()),
    }
}

// ---------------------------------------------------------------------------
// 11. GET /labels — list labels
// ---------------------------------------------------------------------------

pub async fn list_labels(
    State(state): State<AppState>,
    Path((owner, name)): Path<(String, String)>,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let repo_id = match get_repo_id(pool, &owner, &name).await {
        Some(id) => id,
        None => {
            return err_response(
                StatusCode::NOT_FOUND,
                &format!("repository {owner}/{name} not found"),
            );
        }
    };

    match sqlx::query_as::<_, LabelResponse>(
        "SELECT id, repo_id, name, color, description, created_at FROM labels WHERE repo_id = $1 ORDER BY name",
    )
    .bind(repo_id)
    .fetch_all(pool)
    .await
    {
        Ok(labels) => (StatusCode::OK, Json(labels)).into_response(),
        Err(e) => internal_err(&e.to_string()),
    }
}

// ---------------------------------------------------------------------------
// 12. POST /labels — create label
// ---------------------------------------------------------------------------

pub async fn create_label(
    State(state): State<AppState>,
    Path((owner, name)): Path<(String, String)>,
    _auth: AuthUser,
    Json(req): Json<CreateLabelRequest>,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let repo_id = match get_repo_id(pool, &owner, &name).await {
        Some(id) => id,
        None => {
            return err_response(
                StatusCode::NOT_FOUND,
                &format!("repository {owner}/{name} not found"),
            );
        }
    };

    if req.name.trim().is_empty() {
        return err_response(StatusCode::BAD_REQUEST, "label name is required");
    }

    match sqlx::query_as::<_, LabelResponse>(
        "INSERT INTO labels (repo_id, name, color, description, created_at) VALUES ($1, $2, $3, $4, NOW()) RETURNING id, repo_id, name, color, description, created_at",
    )
    .bind(repo_id)
    .bind(&req.name)
    .bind(&req.color)
    .bind(&req.description)
    .fetch_one(pool)
    .await
    {
        Ok(label) => (StatusCode::CREATED, Json(label)).into_response(),
        Err(e) => internal_err(&e.to_string()),
    }
}

// ---------------------------------------------------------------------------
// 13. PATCH /labels/:id — update label
// ---------------------------------------------------------------------------

pub async fn update_label(
    State(state): State<AppState>,
    Path((owner, name, label_id)): Path<(String, String, uuid::Uuid)>,
    _auth: AuthUser,
    Json(req): Json<UpdateLabelRequest>,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let repo_id = match get_repo_id(pool, &owner, &name).await {
        Some(id) => id,
        None => {
            return err_response(
                StatusCode::NOT_FOUND,
                &format!("repository {owner}/{name} not found"),
            );
        }
    };

    let existing = match sqlx::query_as::<_, LabelResponse>(
        "SELECT id, repo_id, name, color, description, created_at FROM labels WHERE id = $1 AND repo_id = $2",
    )
    .bind(label_id)
    .bind(repo_id)
    .fetch_optional(pool)
    .await
    {
        Ok(Some(l)) => l,
        Ok(None) => {
            return err_response(
                StatusCode::NOT_FOUND,
                &format!("label #{label_id} not found"),
            );
        }
        Err(e) => return internal_err(&e.to_string()),
    };

    let name = req.name.as_deref().unwrap_or(&existing.name);
    let color = req.color.as_deref().or(existing.color.as_deref());
    let description = req
        .description
        .as_deref()
        .or(existing.description.as_deref());

    match sqlx::query_as::<_, LabelResponse>(
        "UPDATE labels SET name = $1, color = $2, description = $3 WHERE id = $4 RETURNING id, repo_id, name, color, description, created_at",
    )
    .bind(name)
    .bind(color)
    .bind(description)
    .bind(label_id)
    .fetch_one(pool)
    .await
    {
        Ok(label) => (StatusCode::OK, Json(label)).into_response(),
        Err(e) => internal_err(&e.to_string()),
    }
}

// ---------------------------------------------------------------------------
// 14. DELETE /labels/:id — delete label
// ---------------------------------------------------------------------------

pub async fn delete_label(
    State(state): State<AppState>,
    Path((owner, name, label_id)): Path<(String, String, uuid::Uuid)>,
    _auth: AuthUser,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let repo_id = match get_repo_id(pool, &owner, &name).await {
        Some(id) => id,
        None => {
            return err_response(
                StatusCode::NOT_FOUND,
                &format!("repository {owner}/{name} not found"),
            );
        }
    };

    let result = sqlx::query("DELETE FROM labels WHERE id = $1 AND repo_id = $2")
        .bind(label_id)
        .bind(repo_id)
        .execute(pool)
        .await;

    match result {
        Ok(rows) if rows.rows_affected() == 0 => err_response(
            StatusCode::NOT_FOUND,
            &format!("label #{label_id} not found"),
        ),
        Ok(_) => (StatusCode::NO_CONTENT, ()).into_response(),
        Err(e) => internal_err(&e.to_string()),
    }
}

// ---------------------------------------------------------------------------
// 15. GET /milestones — list milestones
// ---------------------------------------------------------------------------

pub async fn list_milestones(
    State(state): State<AppState>,
    Path((owner, name)): Path<(String, String)>,
    Query(params): Query<ListMilestonesParams>,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let repo_id = match get_repo_id(pool, &owner, &name).await {
        Some(id) => id,
        None => {
            return err_response(
                StatusCode::NOT_FOUND,
                &format!("repository {owner}/{name} not found"),
            );
        }
    };

    let mut query_str = String::from(
        "SELECT id, repo_id, title, description, state, due_on, created_at, updated_at FROM milestones WHERE repo_id = $1",
    );

    if let Some(ref s) = params.state {
        if !s.is_empty() {
            query_str.push_str(" AND state = $2");
        }
    }
    query_str.push_str(" ORDER BY created_at DESC");

    let offset = (params.page - 1) * params.per_page;
    let limit_offset_idx = if params.state.is_some() { 3i32 } else { 2i32 };
    query_str.push_str(&format!(
        " LIMIT ${idx} OFFSET ${idx2}",
        idx = limit_offset_idx,
        idx2 = limit_offset_idx + 1
    ));

    let mut query = sqlx::query_as::<_, MilestoneResponse>(&query_str).bind(repo_id);

    if let Some(ref s) = params.state {
        if !s.is_empty() {
            query = query.bind(s.as_str());
        }
    }
    query = query.bind(params.per_page).bind(offset);

    match query.fetch_all(pool).await {
        Ok(milestones) => (StatusCode::OK, Json(milestones)).into_response(),
        Err(e) => internal_err(&e.to_string()),
    }
}

// ---------------------------------------------------------------------------
// 16. POST /milestones — create milestone
// ---------------------------------------------------------------------------

pub async fn create_milestone(
    State(state): State<AppState>,
    Path((owner, name)): Path<(String, String)>,
    _auth: AuthUser,
    Json(req): Json<CreateMilestoneRequest>,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let repo_id = match get_repo_id(pool, &owner, &name).await {
        Some(id) => id,
        None => {
            return err_response(
                StatusCode::NOT_FOUND,
                &format!("repository {owner}/{name} not found"),
            );
        }
    };

    if req.title.trim().is_empty() {
        return err_response(StatusCode::BAD_REQUEST, "milestone title is required");
    }

    match sqlx::query_as::<_, MilestoneResponse>(
        "INSERT INTO milestones (repo_id, title, description, state, due_on, created_at, updated_at) VALUES ($1, $2, $3, 'open', $4, NOW(), NOW()) RETURNING id, repo_id, title, description, state, due_on, created_at, updated_at",
    )
    .bind(repo_id)
    .bind(&req.title)
    .bind(&req.description)
    .bind(req.due_on)
    .fetch_one(pool)
    .await
    {
        Ok(milestone) => (StatusCode::CREATED, Json(milestone)).into_response(),
        Err(e) => internal_err(&e.to_string()),
    }
}

// ---------------------------------------------------------------------------
// 17. PATCH /milestones/:id — update milestone
// ---------------------------------------------------------------------------

pub async fn update_milestone(
    State(state): State<AppState>,
    Path((owner, name, milestone_id)): Path<(String, String, uuid::Uuid)>,
    _auth: AuthUser,
    Json(req): Json<UpdateMilestoneRequest>,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let repo_id = match get_repo_id(pool, &owner, &name).await {
        Some(id) => id,
        None => {
            return err_response(
                StatusCode::NOT_FOUND,
                &format!("repository {owner}/{name} not found"),
            );
        }
    };

    let existing = match sqlx::query_as::<_, MilestoneResponse>(
        "SELECT id, repo_id, title, description, state, due_on, created_at, updated_at FROM milestones WHERE id = $1 AND repo_id = $2",
    )
    .bind(milestone_id)
    .bind(repo_id)
    .fetch_optional(pool)
    .await
    {
        Ok(Some(m)) => m,
        Ok(None) => {
            return err_response(
                StatusCode::NOT_FOUND,
                &format!("milestone #{milestone_id} not found"),
            );
        }
        Err(e) => return internal_err(&e.to_string()),
    };

    let title = req.title.as_deref().unwrap_or(&existing.title);
    let description = req
        .description
        .as_deref()
        .or(existing.description.as_deref());
    let state_val = req.state.as_deref().unwrap_or(&existing.state);
    let due_on = req.due_on.or(existing.due_on);

    match sqlx::query_as::<_, MilestoneResponse>(
        "UPDATE milestones SET title = $1, description = $2, state = $3, due_on = $4, updated_at = NOW() WHERE id = $5 RETURNING id, repo_id, title, description, state, due_on, created_at, updated_at",
    )
    .bind(title)
    .bind(description)
    .bind(state_val)
    .bind(due_on)
    .bind(milestone_id)
    .fetch_one(pool)
    .await
    {
        Ok(milestone) => (StatusCode::OK, Json(milestone)).into_response(),
        Err(e) => internal_err(&e.to_string()),
    }
}

// ---------------------------------------------------------------------------
// 18. DELETE /milestones/:id — delete milestone
// ---------------------------------------------------------------------------

pub async fn delete_milestone(
    State(state): State<AppState>,
    Path((owner, name, milestone_id)): Path<(String, String, uuid::Uuid)>,
    _auth: AuthUser,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let repo_id = match get_repo_id(pool, &owner, &name).await {
        Some(id) => id,
        None => {
            return err_response(
                StatusCode::NOT_FOUND,
                &format!("repository {owner}/{name} not found"),
            );
        }
    };

    let result = sqlx::query("DELETE FROM milestones WHERE id = $1 AND repo_id = $2")
        .bind(milestone_id)
        .bind(repo_id)
        .execute(pool)
        .await;

    match result {
        Ok(rows) if rows.rows_affected() == 0 => err_response(
            StatusCode::NOT_FOUND,
            &format!("milestone #{milestone_id} not found"),
        ),
        Ok(_) => (StatusCode::NO_CONTENT, ()).into_response(),
        Err(e) => internal_err(&e.to_string()),
    }
}

// ---------------------------------------------------------------------------
// 19. POST /issues/:number/assignees — add assignee
// ---------------------------------------------------------------------------

pub async fn add_issue_assignee(
    State(state): State<AppState>,
    Path((owner, name, number)): Path<(String, String, i32)>,
    auth: AuthUser,
    Json(req): Json<AddIssueAssigneeRequest>,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let repo_id = match get_repo_id(pool, &owner, &name).await {
        Some(id) => id,
        None => {
            return err_response(
                StatusCode::NOT_FOUND,
                &format!("repository {owner}/{name} not found"),
            );
        }
    };

    let issue_id = match sqlx::query_scalar::<_, uuid::Uuid>(
        "SELECT id FROM issues WHERE repo_id = $1 AND number = $2",
    )
    .bind(repo_id)
    .bind(number)
    .fetch_optional(pool)
    .await
    {
        Ok(Some(id)) => id,
        Ok(None) => {
            return err_response(StatusCode::NOT_FOUND, &format!("issue #{number} not found"));
        }
        Err(e) => return internal_err(&e.to_string()),
    };

    let actor_id = match uuid::Uuid::parse_str(&auth.user_id) {
        Ok(id) => id,
        Err(_) => return err_response(StatusCode::UNAUTHORIZED, "invalid user id in token"),
    };

    match sqlx::query(
        "INSERT INTO issue_assignees (issue_id, user_id, assigned_at) VALUES ($1, $2, NOW()) ON CONFLICT DO NOTHING",
    )
    .bind(issue_id)
    .bind(req.assignee_id)
    .execute(pool)
    .await
    {
        Ok(_) => {
            insert_timeline(pool, issue_id, actor_id, "assigned", Some(&req.assignee_id.to_string())).await;

            // Notify the assignee
            if req.assignee_id != actor_id {
                let _ = sqlx::query(
                    "INSERT INTO notifications (user_id, kind, title, body, repo_name) VALUES ($1, 'assignment', $2, $3, $4)",
                )
                .bind(req.assignee_id)
                .bind(format!("Assigned to issue #{number}"))
                .bind(format!("{} assigned you to issue #{number}", auth.username))
                .bind(format!("{owner}/{name}"))
                .execute(pool)
                .await;
            }

            (StatusCode::CREATED, Json(serde_json::json!({"status": "assigned"}))).into_response()
        }
        Err(e) => internal_err(&e.to_string()),
    }
}

// ---------------------------------------------------------------------------
// 20. DELETE /issues/:number/assignees/:user_id — remove assignee
// ---------------------------------------------------------------------------

pub async fn remove_issue_assignee(
    State(state): State<AppState>,
    Path((owner, name, number, user_id)): Path<(String, String, i32, uuid::Uuid)>,
    auth: AuthUser,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let repo_id = match get_repo_id(pool, &owner, &name).await {
        Some(id) => id,
        None => {
            return err_response(
                StatusCode::NOT_FOUND,
                &format!("repository {owner}/{name} not found"),
            );
        }
    };

    let issue_id = match sqlx::query_scalar::<_, uuid::Uuid>(
        "SELECT id FROM issues WHERE repo_id = $1 AND number = $2",
    )
    .bind(repo_id)
    .bind(number)
    .fetch_optional(pool)
    .await
    {
        Ok(Some(id)) => id,
        Ok(None) => {
            return err_response(StatusCode::NOT_FOUND, &format!("issue #{number} not found"));
        }
        Err(e) => return internal_err(&e.to_string()),
    };

    let actor_id = match uuid::Uuid::parse_str(&auth.user_id) {
        Ok(id) => id,
        Err(_) => return err_response(StatusCode::UNAUTHORIZED, "invalid user id in token"),
    };

    let result = sqlx::query("DELETE FROM issue_assignees WHERE issue_id = $1 AND user_id = $2")
        .bind(issue_id)
        .bind(user_id)
        .execute(pool)
        .await;

    match result {
        Ok(rows) if rows.rows_affected() == 0 => err_response(
            StatusCode::NOT_FOUND,
            &format!("assignee {user_id} not found on issue #{number}"),
        ),
        Ok(_) => {
            insert_timeline(
                pool,
                issue_id,
                actor_id,
                "unassigned",
                Some(&user_id.to_string()),
            )
            .await;
            (StatusCode::NO_CONTENT, ()).into_response()
        }
        Err(e) => internal_err(&e.to_string()),
    }
}

// ---------------------------------------------------------------------------
// Route builder
// ---------------------------------------------------------------------------

pub fn issue_routes() -> axum::Router<AppState> {
    use axum::routing::{delete, get, patch, post};

    axum::Router::new()
        .route(
            "/api/v1/repos/{owner}/{name}/issues",
            get(list_issues).post(create_issue),
        )
        .route(
            "/api/v1/repos/{owner}/{name}/issues/{number}",
            get(get_issue).patch(update_issue).delete(delete_issue),
        )
        .route(
            "/api/v1/repos/{owner}/{name}/issues/{number}/comments",
            post(add_comment),
        )
        .route(
            "/api/v1/repos/{owner}/{name}/issues/{number}/comments/{comment_id}",
            patch(edit_comment).delete(delete_comment),
        )
        .route(
            "/api/v1/repos/{owner}/{name}/issues/{number}/reactions",
            post(add_reaction),
        )
        .route(
            "/api/v1/repos/{owner}/{name}/issues/{number}/reactions/{emoji}",
            delete(remove_reaction),
        )
        .route(
            "/api/v1/repos/{owner}/{name}/issues/{number}/assignees",
            post(add_issue_assignee),
        )
        .route(
            "/api/v1/repos/{owner}/{name}/issues/{number}/assignees/{user_id}",
            delete(remove_issue_assignee),
        )
        .route(
            "/api/v1/repos/{owner}/{name}/labels",
            get(list_labels).post(create_label),
        )
        .route(
            "/api/v1/repos/{owner}/{name}/labels/{id}",
            patch(update_label).delete(delete_label),
        )
        .route(
            "/api/v1/repos/{owner}/{name}/milestones",
            get(list_milestones).post(create_milestone),
        )
        .route(
            "/api/v1/repos/{owner}/{name}/milestones/{id}",
            patch(update_milestone).delete(delete_milestone),
        )
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_issues_params_default() {
        let p: ListIssuesParams = serde_json::from_str("{}").unwrap();
        assert!(p.state.is_none());
        assert!(p.label.is_none());
        assert!(p.assignee.is_none());
        assert!(p.milestone.is_none());
        assert!(p.sort.is_none());
        assert_eq!(p.page, 1);
        assert_eq!(p.per_page, 30);
    }

    #[test]
    fn test_list_issues_params_with_values() {
        let p: ListIssuesParams = serde_json::from_str(
            r#"{"state":"open","label":"bug","assignee":"00000000-0000-0000-0000-000000000005","milestone":"00000000-0000-0000-0000-000000000003","sort":"updated_at","page":2,"per_page":10}"#,
        )
        .unwrap();
        assert_eq!(p.state.as_deref(), Some("open"));
        assert_eq!(p.label.as_deref(), Some("bug"));
        assert_eq!(
            p.assignee,
            Some(uuid::Uuid::parse_str("00000000-0000-0000-0000-000000000005").unwrap())
        );
        assert_eq!(
            p.milestone,
            Some(uuid::Uuid::parse_str("00000000-0000-0000-0000-000000000003").unwrap())
        );
        assert_eq!(p.sort.as_deref(), Some("updated_at"));
        assert_eq!(p.page, 2);
        assert_eq!(p.per_page, 10);
    }

    #[test]
    fn test_create_issue_request_parse() {
        let req: CreateIssueRequest = serde_json::from_str(
            r#"{"title":"Fix bug","description":"A nasty bug","assignee":"00000000-0000-0000-0000-00000000002a","milestone":"00000000-0000-0000-0000-000000000007","labels":["00000000-0000-0000-0000-000000000001","00000000-0000-0000-0000-000000000002","00000000-0000-0000-0000-000000000003"]}"#,
        )
        .unwrap();
        assert_eq!(req.title, "Fix bug");
        assert_eq!(req.description.as_deref(), Some("A nasty bug"));
        assert_eq!(
            req.assignee,
            Some(uuid::Uuid::parse_str("00000000-0000-0000-0000-00000000002a").unwrap())
        );
        assert_eq!(
            req.milestone,
            Some(uuid::Uuid::parse_str("00000000-0000-0000-0000-000000000007").unwrap())
        );
        assert_eq!(
            req.labels.as_deref(),
            Some(
                &[
                    uuid::Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap(),
                    uuid::Uuid::parse_str("00000000-0000-0000-0000-000000000002").unwrap(),
                    uuid::Uuid::parse_str("00000000-0000-0000-0000-000000000003").unwrap(),
                ][..]
            )
        );
    }

    #[test]
    fn test_create_issue_request_defaults() {
        let req: CreateIssueRequest = serde_json::from_str(r#"{"title":"Title"}"#).unwrap();
        assert!(req.description.is_none());
        assert!(req.assignee.is_none());
        assert!(req.milestone.is_none());
        assert!(req.labels.is_none());
    }

    #[test]
    fn test_create_label_request_parse() {
        let req: CreateLabelRequest =
            serde_json::from_str(r#"{"name":"bug","color":"ff0000","description":"Bug reports"}"#)
                .unwrap();
        assert_eq!(req.name, "bug");
        assert_eq!(req.color.as_deref(), Some("ff0000"));
        assert_eq!(req.description.as_deref(), Some("Bug reports"));
    }

    #[test]
    fn test_update_label_request_partial() {
        let req: UpdateLabelRequest = serde_json::from_str(r#"{"color":"00ff00"}"#).unwrap();
        assert!(req.name.is_none());
        assert_eq!(req.color.as_deref(), Some("00ff00"));
        assert!(req.description.is_none());
    }

    #[test]
    fn test_create_comment_request_parse() {
        let req: CreateCommentRequest =
            serde_json::from_str(r#"{"body":"This is a comment"}"#).unwrap();
        assert_eq!(req.body, "This is a comment");
    }

    #[test]
    fn test_update_comment_request_parse() {
        let req: UpdateCommentRequest =
            serde_json::from_str(r#"{"body":"Updated comment"}"#).unwrap();
        assert_eq!(req.body, "Updated comment");
    }

    #[test]
    fn test_create_milestone_request_parse() {
        let req: CreateMilestoneRequest =
            serde_json::from_str(r#"{"title":"v1.0","description":"First release"}"#).unwrap();
        assert_eq!(req.title, "v1.0");
        assert_eq!(req.description.as_deref(), Some("First release"));
        assert!(req.due_on.is_none());
    }

    #[test]
    fn test_state_machine_valid_transitions() {
        assert!(validate_state_transition("open", "open"));
        assert!(validate_state_transition("open", "in_progress"));
        assert!(!validate_state_transition("open", "closed"));
        assert!(validate_state_transition("in_progress", "in_progress"));
        assert!(validate_state_transition("in_progress", "closed"));
        assert!(!validate_state_transition("in_progress", "open"));
        assert!(validate_state_transition("closed", "open"));
        assert!(!validate_state_transition("closed", "in_progress"));
        assert!(!validate_state_transition("closed", "closed"));
    }

    #[test]
    fn test_state_machine_same_state_allowed() {
        assert!(validate_state_transition("open", "open"));
        assert!(validate_state_transition("in_progress", "in_progress"));
    }

    #[test]
    fn test_timeline_event_types() {
        let events = [
            "opened",
            "state_change",
            "assignee_change",
            "milestone_change",
            "label_change",
        ];
        for event in events {
            assert!(!event.is_empty());
        }
        assert_eq!(events.len(), 5);
    }

    #[test]
    fn test_issue_routes_compile() {
        let router = issue_routes();
        let _ = router;
    }

    #[test]
    fn test_list_milestones_params_default() {
        let p: ListMilestonesParams = serde_json::from_str("{}").unwrap();
        assert!(p.state.is_none());
        assert_eq!(p.page, 1);
        assert_eq!(p.per_page, 30);
    }

    #[test]
    fn test_update_issue_request_partial() {
        let req: UpdateIssueRequest =
            serde_json::from_str(r#"{"title":"New title","state":"in_progress"}"#).unwrap();
        assert_eq!(req.title.as_deref(), Some("New title"));
        assert_eq!(req.state.as_deref(), Some("in_progress"));
        assert!(req.description.is_none());
        assert!(req.assignee.is_none());
        assert!(req.milestone.is_none());
        assert!(req.labels.is_none());
    }

    #[test]
    fn test_add_reaction_request_parse() {
        let req: AddReactionRequest = serde_json::from_str(r#"{"emoji":"thumbs_up"}"#).unwrap();
        assert_eq!(req.emoji, "thumbs_up");
    }

    #[test]
    fn test_update_milestone_request_partial() {
        let req: UpdateMilestoneRequest = serde_json::from_str(r#"{"state":"closed"}"#).unwrap();
        assert!(req.title.is_none());
        assert!(req.description.is_none());
        assert_eq!(req.state.as_deref(), Some("closed"));
        assert!(req.due_on.is_none());
    }
}
