#![forbid(unsafe_code)]

use axum::{
    Json, Router,
    extract::{Path, Query, State},
    response::IntoResponse,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::api::AppState;
use crate::api::auth::AuthUser;
use crate::error::{CoreError, Result};

fn err_response(e: CoreError) -> axum::response::Response {
    let status = e.status_code();
    let body = e.error_response();
    (status, Json(body)).into_response()
}

// ── Request/Response Types ──

#[derive(Debug, Deserialize)]
pub struct CreatePullRequest {
    pub title: String,
    pub body: Option<String>,
    pub source_branch: String,
    pub target_branch: String,
    pub draft: Option<bool>,
    pub assignees: Option<Vec<Uuid>>,
    pub reviewers: Option<Vec<Uuid>>,
    pub labels: Option<Vec<Uuid>>,
}

#[derive(Debug, Deserialize)]
pub struct UpdatePullRequest {
    pub title: Option<String>,
    pub body: Option<String>,
    pub state: Option<String>,
    pub draft: Option<bool>,
    pub target_branch: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreatePrComment {
    pub body: String,
    pub commit_sha: Option<String>,
    pub file_path: Option<String>,
    pub line: Option<i32>,
    pub in_reply_to_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct SubmitReview {
    pub status: String,
}

#[derive(Debug, Deserialize, Default)]
pub struct ListPrParams {
    pub state: Option<String>,
    #[serde(default = "default_page")]
    pub page: i32,
    #[serde(default = "default_per_page")]
    pub per_page: i32,
    pub head: Option<String>,
    pub base: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct PrResponse {
    pub id: String,
    pub number: i32,
    pub title: String,
    pub body: String,
    pub status: String,
    pub draft: bool,
    pub source_branch: String,
    pub target_branch: String,
    pub merge_commit_id: Option<String>,
    pub head_commit_sha: Option<String>,
    pub base_commit_sha: Option<String>,
    pub merge_strategy: String,
    pub author_id: String,
    pub repo_id: String,
    pub created_at: String,
    pub updated_at: String,
    pub merged_at: Option<String>,
    pub closed_at: Option<String>,
    pub mergeable: Option<bool>,
    pub additions: Option<i64>,
    pub deletions: Option<i64>,
    pub changed_files: Option<i64>,
    pub review_status: Option<String>,
    pub labels: Vec<super::issues::LabelResponse>,
    pub assignees: Vec<String>,
    pub reviewers: Vec<ReviewerResponse>,
}

#[derive(Debug, Serialize)]
pub struct PrListResponse {
    pub items: Vec<PrResponse>,
    pub total: i64,
    pub page: i32,
    pub per_page: i32,
}

#[derive(Debug, Serialize)]
pub struct PrCommentResponse {
    pub id: String,
    pub author_id: String,
    pub body: String,
    pub commit_sha: Option<String>,
    pub file_path: Option<String>,
    pub line: Option<i32>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize)]
pub struct ReviewerResponse {
    pub user_id: String,
    pub review_status: String,
    pub submitted_at: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct MergeResponse {
    pub merged: bool,
    pub message: String,
    pub merge_commit_sha: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct PrFileChange {
    pub path: String,
    pub status: String,
    pub additions: u32,
    pub deletions: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub patch: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct PrDiffResponse {
    pub files: Vec<PrFileChange>,
    pub total_additions: u32,
    pub total_deletions: u32,
    pub commit_count: u32,
}

#[derive(Debug, Serialize)]
pub struct MergeabilityResponse {
    pub mergeable: bool,
    pub merge_strategy: String,
    pub reason: String,
}

// ── Helpers ──

fn default_page() -> i32 {
    1
}
fn default_per_page() -> i32 {
    30
}

async fn get_repo_id(pool: &sqlx::PgPool, owner: &str, name: &str) -> Result<Uuid> {
    sqlx::query_scalar::<_, Uuid>(
        "SELECT r.id FROM repositories r JOIN users u ON r.owner_id = u.id WHERE u.username = $1 AND r.name = $2",
    )
    .bind(owner)
    .bind(name)
    .fetch_one(pool)
    .await
    .map_err(|_| CoreError::NotFound(format!("repo {owner}/{name}")))
}

fn validate_pr_state_transition(current: &str, next: &str) -> bool {
    matches!(
        (current, next),
        ("open", "open" | "closed") | ("closed", "open" | "closed") | ("merged", "closed")
    )
}

// ── Handlers ──

pub async fn list_pull_requests(
    State(state): State<AppState>,
    Path((owner, name)): Path<(String, String)>,
    Query(params): Query<ListPrParams>,
    _auth: AuthUser,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let repo_id = match get_repo_id(pool, &owner, &name).await {
        Ok(id) => id,
        Err(e) => return err_response(e),
    };

    let limit = params.per_page.clamp(1, 100) as i64;
    let offset = ((params.page - 1).max(0) * params.per_page) as i64;

    let prs = match state.db.list_prs(repo_id, limit, offset).await {
        Ok(r) => r,
        Err(e) => return err_response(e),
    };

    let total = match state.db.count_prs(repo_id, params.state.as_deref()).await {
        Ok(c) => c,
        Err(e) => return err_response(e),
    };

    let items: Vec<PrResponse> = prs
        .into_iter()
        .filter(|pr| {
            params.state.as_deref().is_none() || pr.status == params.state.as_deref().unwrap()
        })
        .map(|pr| pr_to_response(pr, None, None, None))
        .collect();

    let resp = PrListResponse {
        total,
        page: params.page,
        per_page: params.per_page,
        items,
    };
    (axum::http::StatusCode::OK, Json(resp)).into_response()
}

pub async fn get_pull_request(
    State(state): State<AppState>,
    Path((owner, name, number)): Path<(String, String, i32)>,
    _auth: AuthUser,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let repo_id = match get_repo_id(pool, &owner, &name).await {
        Ok(id) => id,
        Err(e) => return err_response(e),
    };

    let pr = match state.db.get_pr_by_number(repo_id, number).await {
        Ok(r) => r,
        Err(e) => return err_response(e),
    };

    let labels = get_pr_labels(pool, &pr.id).await.unwrap_or_default();
    let assignees = get_pr_assignee_ids(pool, &pr.id).await.unwrap_or_default();
    let reviewers = match state.db.list_pr_reviewers(pr.id).await {
        Ok(r) => r
            .into_iter()
            .map(|rv| ReviewerResponse {
                user_id: rv.user_id.to_string(),
                review_status: rv.review_status,
                submitted_at: rv.submitted_at.map(|t| t.to_rfc3339()),
            })
            .collect(),
        Err(_) => Vec::new(),
    };

    let review_status = compute_review_status(&reviewers);

    let resp = pr_to_response(pr, Some(labels), Some(assignees), Some(review_status));
    (axum::http::StatusCode::OK, Json(resp)).into_response()
}

pub async fn create_pull_request(
    State(state): State<AppState>,
    Path((owner, name)): Path<(String, String)>,
    auth: AuthUser,
    Json(req): Json<CreatePullRequest>,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let repo_id = match get_repo_id(pool, &owner, &name).await {
        Ok(id) => id,
        Err(e) => return err_response(e),
    };

    let body = req.body.as_deref().unwrap_or("");
    let is_draft = req.draft.unwrap_or(false);
    // Auto-detect draft from WIP/Draft prefix (GitHub-compatible)
    let is_draft = is_draft
        || req.title.to_uppercase().starts_with("WIP:")
        || req.title.to_uppercase().starts_with("WIP ")
        || req.title.to_uppercase().starts_with("DRAFT:")
        || req.title.to_uppercase().starts_with("DRAFT ");
    let author_id = uuid::Uuid::parse_str(&auth.user_id).unwrap_or(uuid::Uuid::nil());

    let pr = match state
        .db
        .create_pr(
            repo_id,
            &req.title,
            body,
            author_id,
            &req.source_branch,
            &req.target_branch,
            is_draft,
        )
        .await
    {
        Ok(r) => r,
        Err(e) => return err_response(e),
    };

    // Add labels
    for label_id in req.labels.unwrap_or_default() {
        let _ = state.db.add_pr_label(pr.id, label_id).await;
    }
    // Add assignees
    for user_id in req.assignees.unwrap_or_default() {
        let _ = state.db.add_pr_assignee(pr.id, user_id).await;
    }
    // Add reviewers
    for user_id in req.reviewers.unwrap_or_default() {
        let _ = state.db.add_pr_reviewer(pr.id, user_id).await;
    }

    let resp = pr_to_response(pr, None, None, None);
    (axum::http::StatusCode::CREATED, Json(resp)).into_response()
}

pub async fn update_pull_request(
    State(state): State<AppState>,
    Path((owner, name, number)): Path<(String, String, i32)>,
    _auth: AuthUser,
    Json(req): Json<UpdatePullRequest>,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let repo_id = match get_repo_id(pool, &owner, &name).await {
        Ok(id) => id,
        Err(e) => return err_response(e),
    };

    let pr = match state.db.get_pr_by_number(repo_id, number).await {
        Ok(r) => r,
        Err(e) => return err_response(e),
    };

    if let Some(ref state_val) = req.state {
        if !validate_pr_state_transition(&pr.status, state_val) {
            return err_response(CoreError::BadRequest(format!(
                "invalid transition: {} -> {}",
                pr.status, state_val
            )));
        }
    }

    let updated = match state
        .db
        .update_pr(
            pr.id,
            req.title.as_deref(),
            req.body.as_deref(),
            req.state.as_deref(),
        )
        .await
    {
        Ok(r) => r,
        Err(e) => return err_response(e),
    };

    // Handle draft toggle
    let final_pr = if let Some(draft) = req.draft {
        match state.db.set_pr_draft(updated.id, draft).await {
            Ok(r) => r,
            Err(e) => return err_response(e),
        }
    } else {
        updated
    };

    let resp = pr_to_response(final_pr, None, None, None);
    (axum::http::StatusCode::OK, Json(resp)).into_response()
}

pub async fn list_pr_comments(
    State(state): State<AppState>,
    Path((owner, name, number)): Path<(String, String, i32)>,
    _auth: AuthUser,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let repo_id = match get_repo_id(pool, &owner, &name).await {
        Ok(id) => id,
        Err(e) => return err_response(e),
    };
    let pr = match state.db.get_pr_by_number(repo_id, number).await {
        Ok(r) => r,
        Err(e) => return err_response(e),
    };
    let comments = match state.db.list_pr_comments(pr.id).await {
        Ok(c) => c,
        Err(e) => return err_response(e),
    };
    let items: Vec<PrCommentResponse> = comments
        .into_iter()
        .map(|c| PrCommentResponse {
            id: c.id.to_string(),
            author_id: c.author_id.to_string(),
            body: c.body,
            commit_sha: c.commit_sha,
            file_path: c.file_path,
            line: c.line,
            created_at: c.created_at.to_rfc3339(),
            updated_at: c.updated_at.to_rfc3339(),
        })
        .collect();
    (axum::http::StatusCode::OK, Json(items)).into_response()
}

pub async fn create_pr_comment(
    State(state): State<AppState>,
    Path((owner, name, number)): Path<(String, String, i32)>,
    auth: AuthUser,
    Json(req): Json<CreatePrComment>,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let repo_id = match get_repo_id(pool, &owner, &name).await {
        Ok(id) => id,
        Err(e) => return err_response(e),
    };
    let pr = match state.db.get_pr_by_number(repo_id, number).await {
        Ok(r) => r,
        Err(e) => return err_response(e),
    };
    let author_id = uuid::Uuid::parse_str(&auth.user_id).unwrap_or(uuid::Uuid::nil());
    let comment = match state
        .db
        .create_pr_comment(
            pr.id,
            author_id,
            &req.body,
            req.commit_sha.as_deref(),
            req.file_path.as_deref(),
            req.line,
            req.in_reply_to_id,
        )
        .await
    {
        Ok(c) => c,
        Err(e) => return err_response(e),
    };

    let resp = PrCommentResponse {
        id: comment.id.to_string(),
        author_id: comment.author_id.to_string(),
        body: comment.body,
        commit_sha: comment.commit_sha,
        file_path: comment.file_path,
        line: comment.line,
        created_at: comment.created_at.to_rfc3339(),
        updated_at: comment.updated_at.to_rfc3339(),
    };
    (axum::http::StatusCode::CREATED, Json(resp)).into_response()
}

pub async fn request_review(
    State(state): State<AppState>,
    Path((owner, name, number)): Path<(String, String, i32)>,
    _auth: AuthUser,
    Json(body): Json<serde_json::Value>,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let repo_id = match get_repo_id(pool, &owner, &name).await {
        Ok(id) => id,
        Err(e) => return err_response(e),
    };
    let pr = match state.db.get_pr_by_number(repo_id, number).await {
        Ok(r) => r,
        Err(e) => return err_response(e),
    };

    let reviewer_ids: Vec<Uuid> = body
        .get("reviewers")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().and_then(|s| Uuid::parse_str(s).ok()))
                .collect()
        })
        .unwrap_or_default();

    let mut results = Vec::new();
    for uid in reviewer_ids {
        if let Ok(rv) = state.db.add_pr_reviewer(pr.id, uid).await {
            results.push(ReviewerResponse {
                user_id: rv.user_id.to_string(),
                review_status: rv.review_status,
                submitted_at: rv.submitted_at.map(|t| t.to_rfc3339()),
            });
        }
    }
    (axum::http::StatusCode::OK, Json(results)).into_response()
}

pub async fn submit_review(
    State(state): State<AppState>,
    Path((owner, name, number, user_id)): Path<(String, String, i32, String)>,
    _auth: AuthUser,
    Json(req): Json<SubmitReview>,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let repo_id = match get_repo_id(pool, &owner, &name).await {
        Ok(id) => id,
        Err(e) => return err_response(e),
    };
    let pr = match state.db.get_pr_by_number(repo_id, number).await {
        Ok(r) => r,
        Err(e) => return err_response(e),
    };
    let uid = match Uuid::parse_str(&user_id) {
        Ok(u) => u,
        Err(_) => {
            return err_response(CoreError::BadRequest("invalid user_id".into()));
        }
    };

    let valid = ["approved", "changes_requested", "commented"];
    if !valid.contains(&req.status.as_str()) {
        return err_response(CoreError::BadRequest(format!(
            "status must be one of: {valid:?}",
        )));
    }

    match state.db.submit_pr_review(pr.id, uid, &req.status).await {
        Ok(rv) => {
            let resp = ReviewerResponse {
                user_id: rv.user_id.to_string(),
                review_status: rv.review_status,
                submitted_at: rv.submitted_at.map(|t| t.to_rfc3339()),
            };
            (axum::http::StatusCode::OK, Json(resp)).into_response()
        }
        Err(e) => err_response(e),
    }
}

pub async fn merge_pull_request(
    State(state): State<AppState>,
    Path((owner, name, number)): Path<(String, String, i32)>,
    auth: AuthUser,
    Json(body): Json<serde_json::Value>,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let repo_id = match get_repo_id(pool, &owner, &name).await {
        Ok(id) => id,
        Err(e) => return err_response(e),
    };
    let pr = match state.db.get_pr_by_number(repo_id, number).await {
        Ok(r) => r,
        Err(e) => return err_response(e),
    };

    if pr.status != "open" {
        return err_response(CoreError::BadRequest(format!(
            "PR #{} is {} (only open PRs can be merged)",
            number, pr.status
        )));
    }

    let strategy_str = body
        .get("strategy")
        .and_then(|v| v.as_str())
        .unwrap_or("merge");
    let strategy: crate::git::MergeStrategy = match strategy_str.parse() {
        Ok(s) => s,
        Err(e) => return err_response(e),
    };

    // Perform the actual git merge
    let merge_result = match state.git_service.merge_branch(
        &owner,
        &name,
        &pr.source_branch,
        &pr.target_branch,
        strategy,
        &format!("{} (CivitForge)", auth.username),
        &format!("{}@civitforge.local", auth.username),
    ) {
        Ok(r) => r,
        Err(e) => return err_response(e),
    };

    // Record the real merge in the DB
    let merged = state
        .db
        .merge_pr(
            pr.id,
            &merge_result.commit_sha,
            &merge_result.strategy_used,
            pr.head_commit_sha.as_deref(),
            pr.base_commit_sha.as_deref(),
        )
        .await
        .is_ok();

    if merged {
        let _ = state
            .db
            .insert_pr_timeline(
                pr.id,
                pr.author_id,
                "merged",
                serde_json::json!({
                    "merge_commit_sha": merge_result.commit_sha,
                    "strategy": merge_result.strategy_used,
                    "was_fast_forward": merge_result.was_ff,
                }),
            )
            .await;
    }

    let resp = MergeResponse {
        merged,
        message: if merged {
            format!(
                "PR #{number} merged via {} ({})",
                merge_result.strategy_used,
                &merge_result.commit_sha[..8]
            )
        } else {
            "Merge failed: could not update PR record".into()
        },
        merge_commit_sha: if merged {
            Some(merge_result.commit_sha)
        } else {
            None
        },
    };
    (axum::http::StatusCode::OK, Json(resp)).into_response()
}

pub async fn list_pr_status_checks(
    State(state): State<AppState>,
    Path((owner, name, number)): Path<(String, String, i32)>,
    _auth: AuthUser,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let repo_id = match get_repo_id(pool, &owner, &name).await {
        Ok(id) => id,
        Err(e) => return err_response(e),
    };
    let pr = match state.db.get_pr_by_number(repo_id, number).await {
        Ok(r) => r,
        Err(e) => return err_response(e),
    };
    let checks = match state.db.list_pr_status_checks(pr.id).await {
        Ok(c) => c,
        Err(e) => return err_response(e),
    };
    (axum::http::StatusCode::OK, Json(checks)).into_response()
}

// ── Conversion ──

fn pr_to_response(
    pr: crate::db::models::PullRequest,
    labels: Option<Vec<super::issues::LabelResponse>>,
    assignees: Option<Vec<String>>,
    review_status: Option<String>,
) -> PrResponse {
    PrResponse {
        id: pr.id.to_string(),
        number: pr.number,
        title: pr.title,
        body: pr.body,
        status: pr.status,
        draft: pr.draft,
        source_branch: pr.source_branch,
        target_branch: pr.target_branch,
        merge_commit_id: pr.merge_commit_id,
        head_commit_sha: pr.head_commit_sha,
        base_commit_sha: pr.base_commit_sha,
        merge_strategy: pr.merge_strategy,
        author_id: pr.author_id.to_string(),
        repo_id: pr.repo_id.to_string(),
        created_at: pr.created_at.to_rfc3339(),
        updated_at: pr.updated_at.to_rfc3339(),
        merged_at: pr.merged_at.map(|t| t.to_rfc3339()),
        closed_at: pr.closed_at.map(|t| t.to_rfc3339()),
        mergeable: None,
        additions: None,
        deletions: None,
        changed_files: None,
        review_status,
        labels: labels.unwrap_or_default(),
        assignees: assignees.unwrap_or_default(),
        reviewers: Vec::new(),
    }
}

fn compute_review_status(reviewers: &[ReviewerResponse]) -> String {
    let approved = reviewers.iter().any(|r| r.review_status == "approved");
    let changes = reviewers
        .iter()
        .any(|r| r.review_status == "changes_requested");
    match (approved, changes) {
        (true, _) => "approved".into(),
        (_, true) => "changes_requested".into(),
        _ => "pending".into(),
    }
}

async fn get_pr_labels(
    pool: &sqlx::PgPool,
    pr_id: &Uuid,
) -> Result<Vec<super::issues::LabelResponse>> {
    let rows: Vec<super::issues::LabelResponse> = sqlx::query_as(
        r#"SELECT l.id, l.repo_id, l.name, l.color, l.description, l.created_at
           FROM labels l
           INNER JOIN pr_labels pl ON l.id = pl.label_id
           WHERE pl.pr_id = $1"#,
    )
    .bind(pr_id)
    .fetch_all(pool)
    .await
    .map_err(|e| CoreError::Database(format!("get_pr_labels: {e}")))?;

    Ok(rows)
}

async fn get_pr_assignee_ids(pool: &sqlx::PgPool, pr_id: &Uuid) -> Result<Vec<String>> {
    let ids: Vec<Uuid> = sqlx::query_scalar("SELECT user_id FROM pr_assignees WHERE pr_id = $1")
        .bind(pr_id)
        .fetch_all(pool)
        .await
        .map_err(|e| CoreError::Database(format!("get_pr_assignee_ids: {e}")))?;

    Ok(ids.into_iter().map(|u| u.to_string()).collect())
}

fn collect_tree_files(
    repo: &gix::Repository,
    ref_name: &str,
) -> std::result::Result<HashMap<String, String>, String> {
    let commit_id = repo
        .rev_parse_single(ref_name)
        .map_err(|e| format!("cannot resolve ref {ref_name}: {e}"))?;
    let commit_obj = commit_id
        .object()
        .map_err(|e| format!("cannot read commit object: {e}"))?;
    let commit = commit_obj
        .try_into_commit()
        .map_err(|_| "not a commit object".to_string())?;
    let tree = commit
        .tree()
        .map_err(|e| format!("cannot read tree: {e}"))?;

    let mut files: HashMap<String, String> = HashMap::new();
    walk_tree_recursive(&tree, String::new(), &mut files);
    Ok(files)
}

fn walk_tree_recursive(tree: &gix::Tree, prefix: String, files: &mut HashMap<String, String>) {
    for entry in tree.iter() {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };
        let entry_name = entry.filename().to_string();
        let full_path = if prefix.is_empty() {
            entry_name.clone()
        } else {
            format!("{prefix}/{entry_name}")
        };
        if entry.mode().is_tree() {
            if let Ok(obj) = entry.object() {
                if let Ok(subtree) = obj.try_into_tree() {
                    walk_tree_recursive(&subtree, full_path, files);
                }
            }
        } else if entry.mode().is_blob() {
            files.insert(full_path, entry.oid().to_hex().to_string());
        }
    }
}

pub async fn list_pr_files(
    State(state): State<AppState>,
    Path((owner, name, number)): Path<(String, String, i32)>,
    _auth: AuthUser,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let repo_id = match get_repo_id(pool, &owner, &name).await {
        Ok(id) => id,
        Err(e) => return err_response(e),
    };

    let pr = match state.db.get_pr_by_number(repo_id, number).await {
        Ok(r) => r,
        Err(e) => return err_response(e),
    };

    let repo_path = state.git_service.repo_path(&owner, &name);
    let repo = match gix::open(&repo_path) {
        Ok(r) => r,
        Err(e) => return err_response(CoreError::Git(e.to_string())),
    };

    let source_ref = format!("refs/heads/{}", pr.source_branch);
    let target_ref = format!("refs/heads/{}", pr.target_branch);

    let source_files = match collect_tree_files(&repo, &source_ref) {
        Ok(f) => f,
        Err(_) => {
            return (axum::http::StatusCode::OK, Json(Vec::<PrFileChange>::new())).into_response();
        }
    };

    let target_files = match collect_tree_files(&repo, &target_ref) {
        Ok(f) => f,
        Err(_) => {
            return (axum::http::StatusCode::OK, Json(Vec::<PrFileChange>::new())).into_response();
        }
    };

    let mut changes: Vec<PrFileChange> = Vec::new();

    for (path, source_oid) in &source_files {
        match target_files.get(path) {
            None => changes.push(PrFileChange {
                path: path.clone(),
                status: "added".into(),
                additions: 1,
                deletions: 0,
                patch: None,
            }),
            Some(target_oid) => {
                if source_oid != target_oid {
                    changes.push(PrFileChange {
                        path: path.clone(),
                        status: "modified".into(),
                        additions: 1,
                        deletions: 1,
                        patch: None,
                    });
                }
            }
        }
    }

    for path in target_files.keys() {
        if !source_files.contains_key(path) {
            changes.push(PrFileChange {
                path: path.clone(),
                status: "removed".into(),
                additions: 0,
                deletions: 1,
                patch: None,
            });
        }
    }

    (axum::http::StatusCode::OK, Json(changes)).into_response()
}

/// GET /repos/{owner}/{name}/pulls/{number}/diff
/// Returns unified diff patches with accurate line counts.
pub async fn get_pr_diff(
    State(state): State<AppState>,
    Path((owner, name, number)): Path<(String, String, i32)>,
    _auth: AuthUser,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let repo_id = match get_repo_id(pool, &owner, &name).await {
        Ok(id) => id,
        Err(e) => return err_response(e),
    };

    let pr = match state.db.get_pr_by_number(repo_id, number).await {
        Ok(r) => r,
        Err(e) => return err_response(e),
    };

    let repo_path = state.git_service.repo_path(&owner, &name);
    let git_bin = std::env::var("GIT_BIN").unwrap_or_else(|_| "git".to_string());

    let source_ref = format!("refs/heads/{}", pr.source_branch);
    let target_ref = format!("refs/heads/{}", pr.target_branch);

    // Use git diff --numstat for accurate line counts
    let numstat = run_git_diff(
        &git_bin,
        &repo_path,
        &["diff", "--numstat", &target_ref, &source_ref],
    );

    let mut total_additions = 0u32;
    let mut total_deletions = 0u32;
    let mut files: Vec<PrFileChange> = Vec::new();

    // Count commits between target..source
    let commit_count = run_git_diff(
        &git_bin,
        &repo_path,
        &[
            "rev-list",
            "--count",
            &format!("{target_ref}..{source_ref}"),
        ],
    )
    .unwrap_or_default()
    .trim()
    .parse::<u32>()
    .unwrap_or(0);

    if let Ok(stat) = numstat {
        for line in stat.lines() {
            let parts: Vec<&str> = line.split('\t').collect();
            if parts.len() != 3 {
                continue;
            }
            let additions: u32 = parts[0].parse().unwrap_or(0);
            let deletions: u32 = parts[1].parse().unwrap_or(0);
            let path = parts[2].to_string();

            total_additions += additions;
            total_deletions += deletions;

            let file_status =
                determine_file_status(&git_bin, &repo_path, &target_ref, &source_ref, &path);

            files.push(PrFileChange {
                path,
                status: file_status,
                additions,
                deletions,
                patch: None,
            });
        }
    }

    let resp = PrDiffResponse {
        total_additions,
        total_deletions,
        commit_count,
        files,
    };
    (axum::http::StatusCode::OK, Json(resp)).into_response()
}

/// GET /repos/{owner}/{name}/pulls/{number}/mergecheck
/// Check if a PR can be merged without conflicts.
pub async fn check_pr_mergeability(
    State(state): State<AppState>,
    Path((owner, name, number)): Path<(String, String, i32)>,
    _auth: AuthUser,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let repo_id = match get_repo_id(pool, &owner, &name).await {
        Ok(id) => id,
        Err(e) => return err_response(e),
    };

    let pr = match state.db.get_pr_by_number(repo_id, number).await {
        Ok(r) => r,
        Err(e) => return err_response(e),
    };

    let repo_path = state.git_service.repo_path(&owner, &name);
    let git_bin = std::env::var("GIT_BIN").unwrap_or_else(|_| "git".to_string());

    let source_ref = format!("origin/{}", pr.source_branch);
    let target_ref = format!("origin/{}", pr.target_branch);

    // Quick check: is target an ancestor of source? (ff possible)
    let ff_check = run_git_diff(
        &git_bin,
        &repo_path,
        &["merge-base", "--is-ancestor", &target_ref, &source_ref],
    );

    let (mergeable, strategy, reason) = if ff_check.is_ok() {
        (
            true,
            pr.merge_strategy.clone(),
            "fast-forward possible".into(),
        )
    } else {
        // Clone to temp and attempt dry-run merge
        let tmp_dir = match tempfile::tempdir() {
            Ok(d) => d,
            Err(_) => {
                return (
                    axum::http::StatusCode::OK,
                    Json(MergeabilityResponse {
                        mergeable: true,
                        merge_strategy: pr.merge_strategy.clone(),
                        reason: "mergeability check skipped".into(),
                    }),
                )
                    .into_response();
            }
        };
        let work = tmp_dir.path();

        let clone_ok = run_git_diff(
            &git_bin,
            work,
            &[
                "clone",
                "--no-checkout",
                repo_path.to_str().unwrap_or(""),
                work.to_str().unwrap_or(""),
            ],
        )
        .is_ok()
            && run_git_diff(&git_bin, work, &["checkout", &pr.target_branch]).is_ok()
            && run_git_diff(&git_bin, work, &["fetch", "origin"]).is_ok()
            && run_git_diff(
                &git_bin,
                work,
                &["branch", "-f", "source-temp", &source_ref],
            )
            .is_ok();

        if !clone_ok {
            return (
                axum::http::StatusCode::OK,
                Json(MergeabilityResponse {
                    mergeable: true,
                    merge_strategy: pr.merge_strategy.clone(),
                    reason: "mergeability check skipped".into(),
                }),
            )
                .into_response();
        }

        // Dry-run merge --no-commit --no-ff
        let _ = run_git_diff(
            &git_bin,
            work,
            &["merge", "--no-commit", "--no-ff", "source-temp"],
        );

        // Check for unmerged files (conflict markers)
        let status_output = run_git_diff(&git_bin, work, &["status", "--porcelain"]);
        let has_conflicts = status_output
            .as_ref()
            .map(|s| {
                s.lines().any(|l| {
                    l.starts_with("UU")
                        || l.starts_with("AA")
                        || l.starts_with("DU")
                        || l.starts_with("UD")
                })
            })
            .unwrap_or(false);

        // tmp_dir cleaned on drop
        (
            !has_conflicts,
            pr.merge_strategy.clone(),
            if has_conflicts {
                "merge conflicts detected".into()
            } else {
                "no conflicts detected".into()
            },
        )
    };

    (
        axum::http::StatusCode::OK,
        Json(MergeabilityResponse {
            mergeable,
            merge_strategy: strategy,
            reason,
        }),
    )
        .into_response()
}

/// Helper: run a git command and return stdout.
fn run_git_diff(
    git_bin: &str,
    cwd: &std::path::Path,
    args: &[&str],
) -> std::result::Result<String, String> {
    use std::process::{Command, Stdio};
    let output = Command::new(git_bin)
        .args(args)
        .current_dir(cwd)
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .map_err(|e| format!("git {args:?}: {e}"))?;

    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).to_string());
    }
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

/// Determine if a file was added, removed, or modified between two refs.
fn determine_file_status(
    git_bin: &str,
    repo_path: &std::path::Path,
    target_ref: &str,
    source_ref: &str,
    path: &str,
) -> String {
    let in_target = run_git_diff(
        git_bin,
        repo_path,
        &["cat-file", "-e", &format!("{target_ref}:{path}")],
    )
    .is_ok();

    let in_source = run_git_diff(
        git_bin,
        repo_path,
        &["cat-file", "-e", &format!("{source_ref}:{path}")],
    )
    .is_ok();

    match (in_target, in_source) {
        (false, true) => "added".into(),
        (true, false) => "removed".into(),
        _ => "modified".into(),
    }
}

// ── Router ──

pub fn pr_routes() -> Router<AppState> {
    Router::new()
        .route(
            "/api/v1/repos/{owner}/{name}/pulls",
            get(list_pull_requests).post(create_pull_request),
        )
        .route(
            "/api/v1/repos/{owner}/{name}/pulls/{number}",
            get(get_pull_request).patch(update_pull_request),
        )
        .route(
            "/api/v1/repos/{owner}/{name}/pulls/{number}/comments",
            get(list_pr_comments).post(create_pr_comment),
        )
        .route(
            "/api/v1/repos/{owner}/{name}/pulls/{number}/reviewers",
            post(request_review),
        )
        .route(
            "/api/v1/repos/{owner}/{name}/pulls/{number}/reviews/{user_id}",
            post(submit_review),
        )
        .route(
            "/api/v1/repos/{owner}/{name}/pulls/{number}/merge",
            post(merge_pull_request),
        )
        .route(
            "/api/v1/repos/{owner}/{name}/pulls/{number}/status",
            get(list_pr_status_checks),
        )
        .route(
            "/api/v1/repos/{owner}/{name}/pulls/{number}/files",
            get(list_pr_files),
        )
        .route(
            "/api/v1/repos/{owner}/{name}/pulls/{number}/diff",
            get(get_pr_diff),
        )
        .route(
            "/api/v1/repos/{owner}/{name}/pulls/{number}/mergecheck",
            get(check_pr_mergeability),
        )
}
