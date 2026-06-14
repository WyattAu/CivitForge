#![forbid(unsafe_code)]

use crate::api::AppState;
use crate::api::auth::{AuthUser, OptionalAuthUser};
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
    pub due_date: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize, Default)]
pub struct UpdateIssueRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    pub state: Option<String>,
    pub assignee: Option<uuid::Uuid>,
    pub milestone: Option<uuid::Uuid>,
    pub labels: Option<Vec<uuid::Uuid>>,
    pub due_date: Option<Option<DateTime<Utc>>>,
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
pub struct CreateTimeEntryRequest {
    pub hours: f64,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
pub struct AddDependencyRequest {
    pub blocked_by_issue_number: i32,
    pub dependency_type: Option<String>,
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
    pub is_pinned: bool,
    pub is_locked: bool,
    pub due_date: Option<DateTime<Utc>>,
    pub labels: Vec<LabelResponse>,
    pub assignees: Vec<IssueAssignee>,
    pub comments: Vec<CommentResponse>,
    pub task_lists: Option<TaskListSummary>,
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

#[derive(Debug, Deserialize, Default)]
pub struct LogTimeRequest {
    pub hours: f64,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
pub struct CreateBranchRequest {
    pub branch_name: String,
}

#[derive(Debug, Serialize)]
pub struct TimeEntriesResponse {
    pub entries: Vec<TimeEntryResponse>,
    pub total_hours: f64,
}

#[derive(Debug, Serialize)]
pub struct DependenciesResponse {
    pub blocking: Vec<DependencyResponse>,
    pub blocked_by: Vec<DependencyResponse>,
}

#[derive(Debug, Serialize)]
pub struct TaskListSummary {
    pub total: i32,
    pub completed: i32,
    pub items: Vec<TaskItem>,
}

#[derive(Debug, Serialize)]
pub struct TaskItem {
    pub description: String,
    pub completed: bool,
}

#[derive(Debug, sqlx::FromRow, Serialize)]
pub struct TimeEntryResponse {
    pub id: uuid::Uuid,
    pub issue_id: uuid::Uuid,
    pub user_id: uuid::Uuid,
    pub hours: f64,
    pub description: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, sqlx::FromRow, Serialize)]
pub struct DependencyResponse {
    pub id: uuid::Uuid,
    pub blocking_issue_id: uuid::Uuid,
    pub blocked_by_issue_id: uuid::Uuid,
    pub dependency_type: String,
    pub created_at: DateTime<Utc>,
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
// Helper: parse task lists from markdown body
// ---------------------------------------------------------------------------

fn parse_task_lists(body: &str) -> Option<TaskListSummary> {
    let mut items = Vec::new();
    for line in body.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("- [x] ") {
            items.push(TaskItem {
                description: rest.to_string(),
                completed: true,
            });
        } else if let Some(rest) = trimmed.strip_prefix("- [ ] ") {
            items.push(TaskItem {
                description: rest.to_string(),
                completed: false,
            });
        }
    }
    if items.is_empty() {
        return None;
    }
    let total = items.len() as i32;
    let completed = items.iter().filter(|i| i.completed).count() as i32;
    Some(TaskListSummary {
        total,
        completed,
        items,
    })
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
        "SELECT id, repo_id, number, title, body, status, author_id, assignee_id, labels, created_at, updated_at, closed_at, is_pinned, is_locked, due_date FROM issues WHERE repo_id = $1",
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
        "{base} ORDER BY is_pinned DESC, {sort_col} DESC LIMIT ${bind_idx} OFFSET ${bind_idx_plus}",
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
    pub labels: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub closed_at: Option<DateTime<Utc>>,
    pub is_pinned: bool,
    pub is_locked: bool,
    pub due_date: Option<DateTime<Utc>>,
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

    let description = req.description.unwrap_or_default();

    let row = match sqlx::query_as::<_, IssueRow>(
        "INSERT INTO issues (repo_id, number, title, body, status, author_id, assignee_id, due_date, created_at, updated_at) VALUES ($1, (SELECT COALESCE(MAX(number),0)+1 FROM issues WHERE repo_id=$1), $2, $3, 'open', $4, $5, $6, NOW(), NOW()) RETURNING id, repo_id, number, title, body, status, author_id, assignee_id, labels, created_at, updated_at, closed_at, is_pinned, is_locked, due_date",
    )
    .bind(repo_id)
    .bind(&req.title)
    .bind(&description)
    .bind(author_id)
    .bind(req.assignee)
    .bind(req.due_date)
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
        crate::api::federation_routes::deliver_to_followers(activity, state.db.pool().clone())
            .await;
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
        "SELECT id, repo_id, number, title, body, status, author_id, assignee_id, labels, created_at, updated_at, closed_at, is_pinned, is_locked, due_date FROM issues WHERE repo_id = $1 AND number = $2",
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

    let task_lists = row.body.as_deref().and_then(parse_task_lists);

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
        is_pinned: row.is_pinned,
        is_locked: row.is_locked,
        due_date: row.due_date,
        labels,
        assignees,
        comments,
        task_lists,
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
        "SELECT id, repo_id, number, title, body, status, author_id, assignee_id, labels, created_at, updated_at, closed_at, is_pinned, is_locked, due_date FROM issues WHERE repo_id = $1 AND number = $2",
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
    let due_date = match &req.due_date {
        Some(d) => *d,
        None => existing.due_date,
    };

    let is_closed = status == "closed";
    let row = match sqlx::query_as::<_, IssueRow>(
        "UPDATE issues SET title = $1, body = $2, status = $3, assignee_id = $4, due_date = $5, closed_at = CASE WHEN $6 THEN NOW() ELSE NULL END, updated_at = NOW() WHERE id = $7 RETURNING id, repo_id, number, title, body, status, author_id, assignee_id, labels, created_at, updated_at, closed_at, is_pinned, is_locked, due_date",
    )
    .bind(title)
    .bind(body)
    .bind(status)
    .bind(assignee_id)
    .bind(due_date)
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
        crate::api::federation_routes::deliver_to_followers(activity, state.db.pool().clone())
            .await;
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

    // Check if issue is locked — reject comments from non-write users
    let is_locked: bool = sqlx::query_scalar("SELECT is_locked FROM issues WHERE id = $1")
        .bind(issue_id)
        .fetch_one(pool)
        .await
        .unwrap_or(false);
    if is_locked {
        let user_id = uuid::Uuid::parse_str(&auth.user_id).ok();
        let has_write = match user_id {
            Some(uid) => sqlx::query_scalar::<_, bool>(
                "SELECT EXISTS(SELECT 1 FROM repo_collaborators WHERE repo_id = $1 AND user_id = $2 AND permission IN ('write', 'admin', 'owner'))",
            )
            .bind(repo_id)
            .bind(uid)
            .fetch_one(pool)
            .await
            .unwrap_or(false),
            None => false,
        };
        if !has_write {
            return err_response(
                StatusCode::FORBIDDEN,
                "issue is locked; only users with write access can comment",
            );
        }
    }

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
// 21. POST /issues/:number/time — log time entry
// ---------------------------------------------------------------------------

pub async fn log_time(
    State(state): State<AppState>,
    Path((owner, name, number)): Path<(String, String, i32)>,
    auth: AuthUser,
    Json(req): Json<LogTimeRequest>,
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

    if req.hours <= 0.0 {
        return err_response(StatusCode::BAD_REQUEST, "hours must be positive");
    }

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

    let description = req.description.unwrap_or_default();

    match sqlx::query_as::<_, TimeEntryResponse>(
        "INSERT INTO issue_time_entries (issue_id, user_id, hours, description, created_at) VALUES ($1, $2, $3, $4, NOW()) RETURNING id, issue_id, user_id, hours, description, created_at",
    )
    .bind(issue_id)
    .bind(user_id)
    .bind(req.hours)
    .bind(&description)
    .fetch_one(pool)
    .await
    {
        Ok(entry) => (StatusCode::CREATED, Json(entry)).into_response(),
        Err(e) => internal_err(&e.to_string()),
    }
}

// ---------------------------------------------------------------------------
// 22. GET /issues/:number/time — list time entries
// ---------------------------------------------------------------------------

pub async fn get_time_entries(
    State(state): State<AppState>,
    Path((owner, name, number)): Path<(String, String, i32)>,
    _auth: OptionalAuthUser,
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

    let entries = match sqlx::query_as::<_, TimeEntryResponse>(
        "SELECT id, issue_id, user_id, hours, description, created_at FROM issue_time_entries WHERE issue_id = $1 ORDER BY created_at DESC",
    )
    .bind(issue_id)
    .fetch_all(pool)
    .await
    {
        Ok(e) => e,
        Err(e) => return internal_err(&e.to_string()),
    };

    let total_hours: f64 = entries.iter().map(|e| e.hours).sum();

    let resp = TimeEntriesResponse {
        entries,
        total_hours,
    };

    (StatusCode::OK, Json(resp)).into_response()
}

// ---------------------------------------------------------------------------
// 23. POST /issues/:number/dependencies — add dependency
// ---------------------------------------------------------------------------

pub async fn add_dependency(
    State(state): State<AppState>,
    Path((owner, name, number)): Path<(String, String, i32)>,
    auth: AuthUser,
    Json(req): Json<AddDependencyRequest>,
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

    let blocking_issue_id = match sqlx::query_scalar::<_, uuid::Uuid>(
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

    let blocked_by_issue_id = match sqlx::query_scalar::<_, uuid::Uuid>(
        "SELECT id FROM issues WHERE repo_id = $1 AND number = $2",
    )
    .bind(repo_id)
    .bind(req.blocked_by_issue_number)
    .fetch_optional(pool)
    .await
    {
        Ok(Some(id)) => id,
        Ok(None) => {
            return err_response(
                StatusCode::NOT_FOUND,
                &format!("blocking issue #{} not found", req.blocked_by_issue_number),
            );
        }
        Err(e) => return internal_err(&e.to_string()),
    };

    if blocking_issue_id == blocked_by_issue_id {
        return err_response(StatusCode::BAD_REQUEST, "issue cannot block itself");
    }

    let dependency_type = req.dependency_type.unwrap_or_else(|| "blocks".to_string());

    let actor_id = match uuid::Uuid::parse_str(&auth.user_id) {
        Ok(id) => id,
        Err(_) => return err_response(StatusCode::UNAUTHORIZED, "invalid user id in token"),
    };

    match sqlx::query_as::<_, DependencyResponse>(
        "INSERT INTO issue_dependencies (blocking_issue_id, blocked_by_issue_id, dependency_type, created_at) VALUES ($1, $2, $3, NOW()) ON CONFLICT (blocking_issue_id, blocked_by_issue_id) DO UPDATE SET dependency_type = $3 RETURNING id, blocking_issue_id, blocked_by_issue_id, dependency_type, created_at",
    )
    .bind(blocking_issue_id)
    .bind(blocked_by_issue_id)
    .bind(&dependency_type)
    .fetch_one(pool)
    .await
    {
        Ok(dep) => {
            insert_timeline(
                pool,
                blocking_issue_id,
                actor_id,
                "dependency_added",
                Some(&format!(
                    "blocked by #{} ({})",
                    req.blocked_by_issue_number, dependency_type
                )),
            )
            .await;
            (StatusCode::CREATED, Json(dep)).into_response()
        }
        Err(e) => internal_err(&e.to_string()),
    }
}

// ---------------------------------------------------------------------------
// 24. GET /issues/:number/dependencies — list dependencies
// ---------------------------------------------------------------------------

pub async fn list_dependencies(
    State(state): State<AppState>,
    Path((owner, name, number)): Path<(String, String, i32)>,
    _auth: OptionalAuthUser,
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

    let blocking = match sqlx::query_as::<_, DependencyResponse>(
        "SELECT id, blocking_issue_id, blocked_by_issue_id, dependency_type, created_at FROM issue_dependencies WHERE blocking_issue_id = $1",
    )
    .bind(issue_id)
    .fetch_all(pool)
    .await
    {
        Ok(d) => d,
        Err(e) => return internal_err(&e.to_string()),
    };

    let blocked_by = match sqlx::query_as::<_, DependencyResponse>(
        "SELECT id, blocking_issue_id, blocked_by_issue_id, dependency_type, created_at FROM issue_dependencies WHERE blocked_by_issue_id = $1",
    )
    .bind(issue_id)
    .fetch_all(pool)
    .await
    {
        Ok(d) => d,
        Err(e) => return internal_err(&e.to_string()),
    };

    let resp = DependenciesResponse {
        blocking,
        blocked_by,
    };

    (StatusCode::OK, Json(resp)).into_response()
}

// ---------------------------------------------------------------------------
// 25. DELETE /issues/:number/dependencies/:dep_id — remove dependency
// ---------------------------------------------------------------------------

pub async fn remove_dependency(
    State(state): State<AppState>,
    Path((owner, name, number, dep_id)): Path<(String, String, i32, uuid::Uuid)>,
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

    let result = sqlx::query(
        "DELETE FROM issue_dependencies WHERE id = $1 AND (blocking_issue_id = $2 OR blocked_by_issue_id = $2)",
    )
    .bind(dep_id)
    .bind(issue_id)
    .execute(pool)
    .await;

    match result {
        Ok(rows) if rows.rows_affected() == 0 => err_response(
            StatusCode::NOT_FOUND,
            &format!("dependency #{dep_id} not found"),
        ),
        Ok(_) => {
            insert_timeline(
                pool,
                issue_id,
                actor_id,
                "dependency_removed",
                Some(&dep_id.to_string()),
            )
            .await;
            (StatusCode::NO_CONTENT, ()).into_response()
        }
        Err(e) => internal_err(&e.to_string()),
    }
}

// ---------------------------------------------------------------------------
// 26. POST /issues/:number/pin — toggle pin
// ---------------------------------------------------------------------------

pub async fn toggle_pin(
    State(state): State<AppState>,
    Path((owner, name, number)): Path<(String, String, i32)>,
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

    let actor_id = match uuid::Uuid::parse_str(&auth.user_id) {
        Ok(id) => id,
        Err(_) => return err_response(StatusCode::UNAUTHORIZED, "invalid user id in token"),
    };

    match sqlx::query_scalar::<_, bool>(
        "UPDATE issues SET is_pinned = NOT is_pinned, updated_at = NOW() WHERE repo_id = $1 AND number = $2 RETURNING is_pinned",
    )
    .bind(repo_id)
    .bind(number)
    .fetch_optional(pool)
    .await
    {
        Ok(Some(is_pinned)) => {
            insert_timeline(
                pool,
                uuid::Uuid::nil(),
                actor_id,
                if is_pinned { "pinned" } else { "unpinned" },
                None,
            )
            .await;
            (
                StatusCode::OK,
                Json(serde_json::json!({"is_pinned": is_pinned})),
            )
                .into_response()
        }
        Ok(None) => err_response(StatusCode::NOT_FOUND, &format!("issue #{number} not found")),
        Err(e) => internal_err(&e.to_string()),
    }
}

// ---------------------------------------------------------------------------
// 27. POST /issues/:number/lock — toggle lock
// ---------------------------------------------------------------------------

pub async fn toggle_lock(
    State(state): State<AppState>,
    Path((owner, name, number)): Path<(String, String, i32)>,
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

    let actor_id = match uuid::Uuid::parse_str(&auth.user_id) {
        Ok(id) => id,
        Err(_) => return err_response(StatusCode::UNAUTHORIZED, "invalid user id in token"),
    };

    match sqlx::query_scalar::<_, bool>(
        "UPDATE issues SET is_locked = NOT is_locked, updated_at = NOW() WHERE repo_id = $1 AND number = $2 RETURNING is_locked",
    )
    .bind(repo_id)
    .bind(number)
    .fetch_optional(pool)
    .await
    {
        Ok(Some(is_locked)) => {
            insert_timeline(
                pool,
                uuid::Uuid::nil(),
                actor_id,
                if is_locked { "locked" } else { "unlocked" },
                None,
            )
            .await;
            (
                StatusCode::OK,
                Json(serde_json::json!({"is_locked": is_locked})),
            )
                .into_response()
        }
        Ok(None) => err_response(StatusCode::NOT_FOUND, &format!("issue #{number} not found")),
        Err(e) => internal_err(&e.to_string()),
    }
}

// ---------------------------------------------------------------------------
// 28. POST /issues/:number/create-branch — create branch from issue
// ---------------------------------------------------------------------------

pub async fn create_branch_from_issue(
    State(state): State<AppState>,
    Path((owner, name, number)): Path<(String, String, i32)>,
    auth: AuthUser,
    Json(req): Json<CreateBranchRequest>,
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

    let issue = match sqlx::query_as::<_, IssueRow>(
        "SELECT id, repo_id, number, title, body, status, author_id, assignee_id, labels, created_at, updated_at, closed_at, is_pinned, is_locked, due_date FROM issues WHERE repo_id = $1 AND number = $2",
    )
    .bind(repo_id)
    .bind(number)
    .fetch_optional(pool)
    .await
    {
        Ok(Some(r)) => r,
        Ok(None) => {
            return err_response(StatusCode::NOT_FOUND, &format!("issue #{number} not found"));
        }
        Err(e) => return internal_err(&e.to_string()),
    };

    let branch_name = req.branch_name.trim().to_string();
    if branch_name.is_empty() {
        return err_response(StatusCode::BAD_REQUEST, "branch_name is required");
    }

    let repo_path = state.git_service.repo_path(&owner, &name);
    if !repo_path.exists() {
        return err_response(StatusCode::NOT_FOUND, "repository git data not found");
    }

    let output = std::process::Command::new("git")
        .arg("-C")
        .arg(&repo_path)
        .arg("branch")
        .arg(&branch_name)
        .output();

    match output {
        Ok(o) => {
            if !o.status.success() {
                let stderr = String::from_utf8_lossy(&o.stderr);
                return err_response(
                    StatusCode::CONFLICT,
                    &format!("failed to create branch: {stderr}"),
                );
            }

            let actor_id = match uuid::Uuid::parse_str(&auth.user_id) {
                Ok(id) => id,
                Err(_) => {
                    return err_response(StatusCode::UNAUTHORIZED, "invalid user id in token");
                }
            };

            insert_timeline(
                pool,
                issue.id,
                actor_id,
                "branch_created",
                Some(&branch_name),
            )
            .await;

            (
                StatusCode::CREATED,
                Json(serde_json::json!({
                    "branch_name": branch_name,
                    "issue_number": number,
                })),
            )
                .into_response()
        }
        Err(e) => internal_err(&format!("failed to run git: {e}")),
    }
}

// ---------------------------------------------------------------------------
// Issue analytics
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize)]
pub struct IssueAnalyticsResponse {
    pub total: i64,
    pub open_count: i64,
    pub closed_count: i64,
    pub in_progress_count: i64,
    pub by_label: Vec<LabelCount>,
    pub by_author: Vec<AuthorCount>,
    pub created_per_week: Vec<WeekCount>,
}

#[derive(Debug, Serialize)]
pub struct LabelCount {
    pub label: String,
    pub count: i64,
}

#[derive(Debug, Serialize)]
pub struct AuthorCount {
    pub author_id: String,
    pub count: i64,
}

#[derive(Debug, Serialize)]
pub struct WeekCount {
    pub week_start: String,
    pub count: i64,
}

pub async fn issue_analytics(
    State(state): State<AppState>,
    Path((owner, name)): Path<(String, String)>,
    _auth: AuthUser,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let repo_id = match crate::api::merge_queue::get_repo_id(pool, &owner, &name).await {
        Ok(id) => id,
        Err(e) => return err_response(StatusCode::NOT_FOUND, &e.to_string()),
    };

    let total: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM issues WHERE repo_id = $1")
        .bind(repo_id)
        .fetch_one(pool)
        .await
        .unwrap_or((0,));

    let open_count: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM issues WHERE repo_id = $1 AND state = 'open'")
            .bind(repo_id)
            .fetch_one(pool)
            .await
            .unwrap_or((0,));

    let closed_count: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM issues WHERE repo_id = $1 AND state = 'closed'")
            .bind(repo_id)
            .fetch_one(pool)
            .await
            .unwrap_or((0,));

    let in_progress_count: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM issues WHERE repo_id = $1 AND state = 'in_progress'")
            .bind(repo_id)
            .fetch_one(pool)
            .await
            .unwrap_or((0,));

    let by_label_rows: Vec<(String, i64)> = sqlx::query_as(
        r#"SELECT il.name AS label, COUNT(*) AS count
           FROM issue_labels il
           JOIN issues i ON il.issue_id = i.id
           WHERE i.repo_id = $1
           GROUP BY il.name
           ORDER BY count DESC
           LIMIT 20"#,
    )
    .bind(repo_id)
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    let by_author_rows: Vec<(String, i64)> = sqlx::query_as(
        r#"SELECT author_id::text, COUNT(*) AS count
           FROM issues WHERE repo_id = $1
           GROUP BY author_id
           ORDER BY count DESC
           LIMIT 20"#,
    )
    .bind(repo_id)
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    let created_per_week_rows: Vec<(chrono::NaiveDate, i64)> = sqlx::query_as(
        r#"SELECT DATE_TRUNC('week', created_at)::date AS week_start, COUNT(*) AS count
           FROM issues WHERE repo_id = $1 AND created_at >= NOW() - INTERVAL '12 weeks'
           GROUP BY week_start
           ORDER BY week_start"#,
    )
    .bind(repo_id)
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    (
        StatusCode::OK,
        Json(IssueAnalyticsResponse {
            total: total.0,
            open_count: open_count.0,
            closed_count: closed_count.0,
            in_progress_count: in_progress_count.0,
            by_label: by_label_rows
                .into_iter()
                .map(|(label, count)| LabelCount { label, count })
                .collect(),
            by_author: by_author_rows
                .into_iter()
                .map(|(author_id, count)| AuthorCount { author_id, count })
                .collect(),
            created_per_week: created_per_week_rows
                .into_iter()
                .map(|(week_start, count)| WeekCount {
                    week_start: week_start.to_string(),
                    count,
                })
                .collect(),
        }),
    )
        .into_response()
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
            "/api/v1/repos/{owner}/{name}/issues/analytics",
            get(issue_analytics),
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
            "/api/v1/repos/{owner}/{name}/issues/{number}/time",
            get(get_time_entries).post(log_time),
        )
        .route(
            "/api/v1/repos/{owner}/{name}/issues/{number}/dependencies",
            get(list_dependencies).post(add_dependency),
        )
        .route(
            "/api/v1/repos/{owner}/{name}/issues/{number}/dependencies/{dep_id}",
            delete(remove_dependency),
        )
        .route(
            "/api/v1/repos/{owner}/{name}/issues/{number}/pin",
            post(toggle_pin),
        )
        .route(
            "/api/v1/repos/{owner}/{name}/issues/{number}/lock",
            post(toggle_lock),
        )
        .route(
            "/api/v1/repos/{owner}/{name}/issues/{number}/create-branch",
            post(create_branch_from_issue),
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

    #[test]
    fn test_log_time_request_parse() {
        let req: LogTimeRequest =
            serde_json::from_str(r#"{"hours":2.5,"description":"Fixed login bug"}"#).unwrap();
        assert_eq!(req.hours, 2.5);
        assert_eq!(req.description.as_deref(), Some("Fixed login bug"));
    }

    #[test]
    fn test_log_time_request_defaults() {
        let req: LogTimeRequest = serde_json::from_str(r#"{"hours":1.0}"#).unwrap();
        assert_eq!(req.hours, 1.0);
        assert!(req.description.is_none());
    }

    #[test]
    fn test_add_dependency_request_parse() {
        let req: AddDependencyRequest =
            serde_json::from_str(r#"{"blocked_by_issue_number":5,"dependency_type":"blocks"}"#)
                .unwrap();
        assert_eq!(req.blocked_by_issue_number, 5);
        assert_eq!(req.dependency_type.as_deref(), Some("blocks"));
    }

    #[test]
    fn test_add_dependency_request_defaults() {
        let req: AddDependencyRequest =
            serde_json::from_str(r#"{"blocked_by_issue_number":3}"#).unwrap();
        assert_eq!(req.blocked_by_issue_number, 3);
        assert!(req.dependency_type.is_none());
    }

    #[test]
    fn test_create_branch_request_parse() {
        let req: CreateBranchRequest =
            serde_json::from_str(r#"{"branch_name":"fix/issue-42-login"}"#).unwrap();
        assert_eq!(req.branch_name, "fix/issue-42-login");
    }

    #[test]
    fn test_parse_task_lists_none() {
        assert!(parse_task_lists("no task lists here").is_none());
    }

    #[test]
    fn test_parse_task_lists_empty() {
        assert!(parse_task_lists("").is_none());
    }

    #[test]
    fn test_parse_task_lists_all_uncompleted() {
        let result = parse_task_lists("- [ ] Task 1\n- [ ] Task 2").unwrap();
        assert_eq!(result.total, 2);
        assert_eq!(result.completed, 0);
        assert_eq!(result.items[0].description, "Task 1");
        assert!(!result.items[0].completed);
        assert_eq!(result.items[1].description, "Task 2");
        assert!(!result.items[1].completed);
    }

    #[test]
    fn test_parse_task_lists_mixed() {
        let result = parse_task_lists("- [x] Done\n- [ ] Todo\n- [x] Also done").unwrap();
        assert_eq!(result.total, 3);
        assert_eq!(result.completed, 2);
        assert!(result.items[0].completed);
        assert!(!result.items[1].completed);
        assert!(result.items[2].completed);
    }

    #[test]
    fn test_parse_task_lists_with_whitespace() {
        let result = parse_task_lists("  - [x] Trimmed\n  - [ ] Also trimmed").unwrap();
        assert_eq!(result.total, 2);
        assert!(result.items[0].completed);
        assert!(!result.items[1].completed);
    }

    #[test]
    fn test_parse_task_lists_no_prefix_dash() {
        assert!(parse_task_lists("[ ] Not a task list").is_none());
    }

    #[test]
    fn test_time_entries_response_serialize() {
        let resp = TimeEntriesResponse {
            entries: vec![],
            total_hours: 5.5,
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("5.5"));
        assert!(json.contains("entries"));
    }

    #[test]
    fn test_dependencies_response_serialize() {
        let resp = DependenciesResponse {
            blocking: vec![],
            blocked_by: vec![],
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("blocking"));
        assert!(json.contains("blocked_by"));
    }

    #[test]
    fn test_task_list_summary_serialize() {
        let summary = TaskListSummary {
            total: 3,
            completed: 1,
            items: vec![
                TaskItem {
                    description: "a".into(),
                    completed: true,
                },
                TaskItem {
                    description: "b".into(),
                    completed: false,
                },
                TaskItem {
                    description: "c".into(),
                    completed: false,
                },
            ],
        };
        let json = serde_json::to_string(&summary).unwrap();
        assert!(json.contains("\"total\":3"));
        assert!(json.contains("\"completed\":1"));
    }
}
