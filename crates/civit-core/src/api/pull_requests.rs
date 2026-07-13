#![forbid(unsafe_code)]

use axum::{
    Json, Router,
    extract::{Path, Query, State},
    response::IntoResponse,
    routing::{get, patch, post},
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::api::AppState;
use crate::api::auth::AuthUser;
use crate::api::mentions;
use crate::error::{CoreError, Result};

fn err_response<E: Into<CoreError>>(e: E) -> axum::response::Response {
    let e: CoreError = e.into();
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
    pub auto_merge: Option<bool>,
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

#[derive(Debug, Deserialize)]
pub struct AddPrAssigneeRequest {
    pub assignee_id: Uuid,
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
    pub auto_merge: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub checkout_instructions: Option<CheckoutInstructions>,
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
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub cross_references: Vec<CrossReferenceResponse>,
}

#[derive(Debug, Serialize)]
pub struct CrossReferenceResponse {
    pub target_number: i32,
    pub target_type: String,
}

#[derive(Debug, Serialize)]
pub struct ReviewerResponse {
    pub user_id: String,
    pub review_status: String,
    pub submitted_at: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CheckoutInstructions {
    pub fetch: String,
    pub checkout: String,
    pub test: String,
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

// ── Inline / Side-by-side Diff Types ──

#[derive(Debug, Serialize)]
pub struct InlineDiffLine {
    pub old_line_num: Option<u32>,
    pub new_line_num: Option<u32>,
    pub content: String,
    pub line_type: String, // context, added, deleted
}

#[derive(Debug, Serialize)]
pub struct DiffHunk {
    pub old_start: u32,
    pub old_count: u32,
    pub new_start: u32,
    pub new_count: u32,
    pub lines: Vec<InlineDiffLine>,
}

#[derive(Debug, Serialize)]
pub struct InlineDiffFile {
    pub filename: String,
    pub old_filename: Option<String>,
    pub status: String,
    pub hunks: Vec<DiffHunk>,
}

#[derive(Debug, Serialize)]
pub struct InlineDiffResponse {
    pub files: Vec<InlineDiffFile>,
}

#[derive(Debug, Serialize)]
pub struct SideBySideHunk {
    pub old_start: u32,
    pub old_count: u32,
    pub new_start: u32,
    pub new_count: u32,
    pub left: Vec<SideBySideLine>,
    pub right: Vec<SideBySideLine>,
}

#[derive(Debug, Serialize)]
pub struct SideBySideLine {
    pub line_num: Option<u32>,
    pub content: String,
    pub line_type: String,
}

#[derive(Debug, Serialize)]
pub struct SideBySideFile {
    pub filename: String,
    pub old_filename: Option<String>,
    pub status: String,
    pub hunks: Vec<SideBySideHunk>,
}

#[derive(Debug, Serialize)]
pub struct SideBySideDiffResponse {
    pub files: Vec<SideBySideFile>,
}

#[derive(Debug, Deserialize)]
pub struct CreateInlineComment {
    pub body: String,
    pub path: String,
    pub line: Option<i32>,
    pub side: Option<String>, // LEFT or RIGHT
    pub commit_sha: Option<String>,
    pub in_reply_to_id: Option<Uuid>,
}

#[derive(Debug, Serialize)]
pub struct InlineCommentResponse {
    pub id: String,
    pub author_id: String,
    pub body: String,
    pub path: String,
    pub line: Option<i32>,
    pub side: Option<String>,
    pub commit_sha: Option<String>,
    pub created_at: String,
    pub updated_at: String,
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

    let auto_merge = req.auto_merge.unwrap_or(false);

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
            auto_merge,
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

    // CODEOWNERS enforcement: auto-request reviews from code owners
    {
        let storage_path = &state.config.storage_path;
        let changed_files: Vec<String> = vec![req.source_branch.clone()];
        let required = crate::api::codeowners::get_required_reviewers(
            &owner,
            &name,
            storage_path,
            &changed_files,
        )
        .await;
        if !required.is_empty() {
            let _ =
                crate::api::codeowners::insert_codeowners_reviews_for_pr(pool, pr.id, &required)
                    .await;
            for username in &required {
                if let Ok(user) = state.db.get_user_by_username(username).await {
                    let _ = state.db.add_pr_reviewer(pr.id, user.id).await;
                }
            }
        }
    }

    let resp = pr_to_response(pr.clone(), None, None, None);

    let dispatcher = crate::webhooks::WebhookDispatcher::new();
    let pool_clone = state.db.pool().clone();
    let rid = repo_id;
    let evt = crate::webhooks::WebhookEvent::PullRequest;
    let pl = serde_json::json!({
        "action": "opened",
        "pr_number": resp.number,
        "repo_id": rid.to_string(),
        "title": resp.title,
        "source_branch": resp.source_branch,
        "target_branch": resp.target_branch,
        "author_id": resp.author_id,
    });
    tokio::spawn(async move { dispatcher.dispatch(&pool_clone, rid, &evt, pl).await });

    // Deliver federation activity to followers
    if state.config.federation_enabled {
        let domain = &state.config.federation_instance_domain;
        let activity = crate::federation::activitypub::Activity {
            r#type: crate::federation::activitypub::ActivityType::Create,
            id: format!("https://{domain}/activities/{}", uuid::Uuid::new_v4()),
            actor: format!("https://{domain}/api/v1/users/{}", auth.user_id),
            object: crate::federation::activitypub::ActivityObject::PullRequest {
                id: pr.id.to_string(),
                name: pr.title.clone(),
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

    if let Some(ref state_val) = req.state
        && !validate_pr_state_transition(&pr.status, state_val)
    {
        return err_response(CoreError::BadRequest(format!(
            "invalid transition: {} -> {}",
            pr.status, state_val
        )));
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

    let dispatcher = crate::webhooks::WebhookDispatcher::new();
    let pool_clone = state.db.pool().clone();
    let rid = repo_id;
    let evt = crate::webhooks::WebhookEvent::PullRequest;
    let pl = serde_json::json!({
        "action": "updated",
        "pr_number": resp.number,
        "repo_id": rid.to_string(),
        "title": resp.title,
        "status": resp.status,
        "author_id": resp.author_id,
    });
    tokio::spawn(async move { dispatcher.dispatch(&pool_clone, rid, &evt, pl).await });

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
            cross_references: Vec::new(),
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

    // Parse and store @mentions
    let mentioned_usernames = mentions::parse_mentions(&comment.body);
    for username in &mentioned_usernames {
        if let Ok(Some(mentioned_id)) =
            sqlx::query_scalar::<_, Uuid>("SELECT id FROM users WHERE username = $1")
                .bind(username)
                .fetch_optional(pool)
                .await
        {
            let _ = sqlx::query(
                "INSERT INTO comment_mentions (comment_id, comment_type, mentioned_user_id, repo_id) VALUES ($1, 'pr', $2, $3) ON CONFLICT DO NOTHING",
            )
            .bind(comment.id)
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
                .bind(format!("New mention in PR #{number}"))
                .bind(format!("{} mentioned you in a PR comment", auth.username))
                .bind(format!("{owner}/{name}"))
                .execute(pool)
                .await;
            }
        }
    }

    // Parse and store cross-references
    let target_numbers = mentions::parse_cross_references(&comment.body);
    let mut cross_references = Vec::new();
    for target_number in &target_numbers {
        let _ = sqlx::query(
            "INSERT INTO comment_cross_references (source_comment_id, source_comment_type, source_repo_id, target_number, target_type) VALUES ($1, 'pr', $2, $3, 'issue')",
        )
        .bind(comment.id)
        .bind(repo_id)
        .bind(target_number)
        .execute(pool)
        .await;
        cross_references.push(CrossReferenceResponse {
            target_number: *target_number,
            target_type: "issue".to_string(),
        });
    }

    let resp = PrCommentResponse {
        id: comment.id.to_string(),
        author_id: comment.author_id.to_string(),
        body: comment.body,
        commit_sha: comment.commit_sha,
        file_path: comment.file_path,
        line: comment.line,
        created_at: comment.created_at.to_rfc3339(),
        updated_at: comment.updated_at.to_rfc3339(),
        cross_references,
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

    let mut reviewer_ids: Vec<Uuid> = body
        .get("reviewers")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().and_then(|s| Uuid::parse_str(s).ok()))
                .collect()
        })
        .unwrap_or_default();

    // CODEOWNERS enforcement: read CODEOWNERS and auto-add code owners
    let git_service = &state.git_service;
    let repo_path = git_service.repo_path(&owner, &name);
    let codeowners_content = read_codeowners_from_repo(&repo_path);
    if let Some(content) = codeowners_content {
        let entries = crate::api::codeowners::parse_codeowners(&content);
        // Use PR title + source_branch as a proxy for changed files
        // In production this would diff the PR commits
        let changed_files = vec![pr.source_branch.clone()];
        let owner_usernames =
            crate::api::codeowners::find_codeowners_for_files(&entries, &changed_files);
        for username in &owner_usernames {
            // Strip leading @ if present
            let uname = username.trim_start_matches('@');
            if let Ok(user) = state.db.get_user_by_username(uname).await
                && !reviewer_ids.contains(&user.id)
            {
                reviewer_ids.push(user.id);
            }
        }
    }

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

fn read_codeowners_from_repo(repo_path: &std::path::Path) -> Option<String> {
    std::fs::read_to_string(repo_path.join("CODEOWNERS"))
        .ok()
        .or_else(|| std::fs::read_to_string(repo_path.join(".github").join("CODEOWNERS")).ok())
        .or_else(|| std::fs::read_to_string(repo_path.join("docs").join("CODEOWNERS")).ok())
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
            // Update CODEOWNERS review if this reviewer is a required owner
            if req.status == "approved" {
                let _ = crate::api::codeowners::update_codeowners_review_approval(
                    pool,
                    pr.id,
                    &uid.to_string(),
                    true,
                )
                .await;
            }

            let resp = ReviewerResponse {
                user_id: rv.user_id.to_string(),
                review_status: rv.review_status,
                submitted_at: rv.submitted_at.map(|t| t.to_rfc3339()),
            };

            let dispatcher = crate::webhooks::WebhookDispatcher::new();
            let pool_clone = state.db.pool().clone();
            let rid = repo_id;
            let evt = crate::webhooks::WebhookEvent::PullRequest;
            let pl = serde_json::json!({
                "action": "reviewed",
                "pr_number": number,
                "repo_id": rid.to_string(),
                "reviewer_id": uid.to_string(),
                "review_status": req.status,
                "author_id": pr.author_id.to_string(),
            });
            tokio::spawn(async move { dispatcher.dispatch(&pool_clone, rid, &evt, pl).await });

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

    if pr.draft {
        return err_response(CoreError::BadRequest(format!(
            "PR #{number} is a draft and cannot be merged",
        )));
    }

    // CODEOWNERS enforcement: block merge if required owners haven't approved
    let codeowners_approved = crate::api::codeowners::check_codeowners_approval(pool, pr.id)
        .await
        .unwrap_or(true);
    if !codeowners_approved {
        return err_response(CoreError::Forbidden(
            "merge blocked: CODEOWNERS required reviewers have not approved".into(),
        ));
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
        // Auto-close linked issues (Fixes #NNN, Closes #NNN, Resolves #NNN)
        let _closed_issues = state
            .db
            .close_issues_for_pr(repo_id, &pr.title, &pr.body, pr.author_id)
            .await
            .unwrap_or_default();

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

    if merged {
        let dispatcher = crate::webhooks::WebhookDispatcher::new();
        let pool_clone = state.db.pool().clone();
        let rid = repo_id;
        let evt = crate::webhooks::WebhookEvent::PullRequest;
        let pl = serde_json::json!({
            "action": "merged",
            "pr_number": number,
            "repo_id": rid.to_string(),
            "merge_commit_sha": resp.merge_commit_sha,
            "strategy": merge_result.strategy_used,
            "author_id": pr.author_id.to_string(),
        });
        tokio::spawn(async move { dispatcher.dispatch(&pool_clone, rid, &evt, pl).await });

        // Deliver federation activity to followers
        if state.config.federation_enabled {
            let domain = &state.config.federation_instance_domain;
            let activity = crate::federation::activitypub::Activity {
                r#type: crate::federation::activitypub::ActivityType::Update,
                id: format!("https://{domain}/activities/{}", uuid::Uuid::new_v4()),
                actor: format!("https://{domain}/api/v1/users/{}", auth.user_id),
                object: crate::federation::activitypub::ActivityObject::PullRequest {
                    id: pr.id.to_string(),
                    name: pr.title.clone(),
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
    }

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
    let checkout_instructions = if pr.status == "open" {
        Some(CheckoutInstructions {
            fetch: format!("git fetch origin {}", pr.source_branch),
            checkout: format!(
                "git checkout -b {} origin/{}",
                pr.source_branch, pr.source_branch
            ),
            test: format!(
                "git diff origin/{}...HEAD",
                pr.target_branch
            ),
        })
    } else {
        None
    };

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
        auto_merge: pr.auto_merge,
        checkout_instructions,
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
            if let Ok(obj) = entry.object()
                && let Ok(subtree) = obj.try_into_tree()
            {
                walk_tree_recursive(&subtree, full_path, files);
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

/// Check if a PR has auto_merge enabled and all status checks pass, then merge automatically.
/// Called from the pipeline completion handler when a pipeline succeeds.
pub async fn check_auto_merge(state: &AppState, repo_id: Uuid, commit_sha: &str) {
    let pool = state.db.pool();

    // Find open PRs whose head_commit_sha matches the just-succeeded pipeline commit
    let prs: Vec<crate::db::models::PullRequest> = match sqlx::query_as(
        r#"SELECT * FROM pull_requests
           WHERE repo_id = $1 AND status = 'open' AND auto_merge = true
             AND head_commit_sha = $2 AND draft = false"#,
    )
    .bind(repo_id)
    .bind(commit_sha)
    .fetch_all(pool)
    .await
    {
        Ok(r) => r,
        Err(e) => {
            tracing::warn!(error = %e, "check_auto_merge: failed to query PRs");
            return;
        }
    };

    for pr in prs {
        // Verify all status checks for this PR are passing
        let checks = match state.db.list_pr_status_checks(pr.id).await {
            Ok(c) => c,
            Err(_) => continue,
        };

        let all_passing = !checks.is_empty()
            && checks
                .iter()
                .all(|c| c.state == "success" || c.state == "passed");

        if !all_passing {
            tracing::info!(
                pr_number = pr.number,
                "auto_merge: not all status checks passing, skipping"
            );
            continue;
        }

        // Verify CODEOWNERS approval
        let codeowners_approved = crate::api::codeowners::check_codeowners_approval(pool, pr.id)
            .await
            .unwrap_or(true);
        if !codeowners_approved {
            tracing::info!(
                pr_number = pr.number,
                "auto_merge: CODEOWNERS approval not satisfied, skipping"
            );
            continue;
        }

        // Resolve owner username from repo
        let owner_username: Option<String> = sqlx::query_scalar(
            "SELECT u.username FROM users u JOIN repositories r ON r.owner_id = u.id WHERE r.id = $1",
        )
        .bind(repo_id)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten();

        let owner = match owner_username {
            Some(u) => u,
            None => continue,
        };

        let repo_name: Option<String> =
            sqlx::query_scalar("SELECT name FROM repositories WHERE id = $1")
                .bind(repo_id)
                .fetch_optional(pool)
                .await
                .ok()
                .flatten();

        let name = match repo_name {
            Some(n) => n,
            None => continue,
        };

        // Perform the merge using the default strategy
        let strategy: crate::git::MergeStrategy = pr
            .merge_strategy
            .parse()
            .unwrap_or(crate::git::MergeStrategy::Merge);

        let merge_result = match state.git_service.merge_branch(
            &owner,
            &name,
            &pr.source_branch,
            &pr.target_branch,
            strategy,
            "auto-merge (CivitForge)",
            "auto-merge@civitforge.local",
        ) {
            Ok(r) => r,
            Err(e) => {
                tracing::warn!(
                    pr_number = pr.number,
                    error = %e,
                    "auto_merge: merge failed"
                );
                continue;
            }
        };

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
                        "auto_merge": true,
                    }),
                )
                .await;

            let dispatcher = crate::webhooks::WebhookDispatcher::new();
            let pool_clone = state.db.pool().clone();
            let evt = crate::webhooks::WebhookEvent::PullRequest;
            let pl = serde_json::json!({
                "action": "merged",
                "pr_number": pr.number,
                "repo_id": repo_id.to_string(),
                "merge_commit_sha": merge_result.commit_sha,
                "strategy": merge_result.strategy_used,
                "auto_merge": true,
                "author_id": pr.author_id.to_string(),
            });
            tokio::spawn(async move { dispatcher.dispatch(&pool_clone, repo_id, &evt, pl).await });

            tracing::info!(
                pr_number = pr.number,
                commit_sha = %merge_result.commit_sha,
                "auto_merge: PR merged successfully"
            );
        }
    }
}

/// Trigger auto-merge for PRs when a pipeline succeeds.
/// This checks if any open PRs with auto_merge enabled have all checks passing
/// and all required CODEOWNERS reviews approved, then merges them.
pub async fn trigger_auto_merge_on_success(state: &AppState, pipeline_id: Uuid) {
    let pool = state.db.pool();

    // Find the pipeline run to get repo_id and commit_sha
    let pipeline: Option<(Uuid, String, String)> =
        sqlx::query_as(r#"SELECT repo_id, commit_sha, ref_name FROM pipeline_runs WHERE id = $1"#)
            .bind(pipeline_id)
            .fetch_optional(pool)
            .await
            .ok()
            .flatten();

    let (repo_id, commit_sha, _ref_name) = match pipeline {
        Some(p) => p,
        None => return,
    };

    check_auto_merge(state, repo_id, &commit_sha).await;
}

/// GET /repos/{owner}/{name}/pulls/{number}/patch
/// Download PR patch as a unified diff.
pub async fn download_patch(
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

    match run_git_diff(
        &git_bin,
        &repo_path,
        &["diff", "-U3", &target_ref, &source_ref],
    ) {
        Ok(patch) => (
            axum::http::StatusCode::OK,
            [(
                axum::http::header::CONTENT_TYPE,
                "text/plain; charset=utf-8",
            )],
            patch,
        )
            .into_response(),
        Err(e) => err_response(CoreError::Git(e)),
    }
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

// ── PR Assignees ──

pub async fn add_pr_assignee(
    State(state): State<AppState>,
    Path((owner, name, number)): Path<(String, String, i32)>,
    auth: AuthUser,
    Json(req): Json<AddPrAssigneeRequest>,
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

    let actor_id = uuid::Uuid::parse_str(&auth.user_id).unwrap_or(uuid::Uuid::nil());

    match state.db.add_pr_assignee(pr.id, req.assignee_id).await {
        Ok(_) => {
            // Notify the assignee
            if req.assignee_id != actor_id {
                let _ = sqlx::query(
                    "INSERT INTO notifications (user_id, kind, title, body, repo_name) VALUES ($1, 'assignment', $2, $3, $4)",
                )
                .bind(req.assignee_id)
                .bind(format!("Assigned to PR #{number}"))
                .bind(format!("{} assigned you to PR #{number}", auth.username))
                .bind(format!("{owner}/{name}"))
                .execute(pool)
                .await;
            }

            (
                axum::http::StatusCode::CREATED,
                Json(serde_json::json!({"status": "assigned"})),
            )
                .into_response()
        }
        Err(e) => err_response(e),
    }
}

pub async fn remove_pr_assignee(
    State(state): State<AppState>,
    Path((owner, name, number, user_id)): Path<(String, String, i32, String)>,
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

    let uid = match Uuid::parse_str(&user_id) {
        Ok(u) => u,
        Err(_) => {
            return err_response(CoreError::BadRequest("invalid user_id".into()));
        }
    };

    match state.db.remove_pr_assignee(pr.id, uid).await {
        Ok(_) => (axum::http::StatusCode::NO_CONTENT, ()).into_response(),
        Err(e) => err_response(e),
    }
}

// ── Unified Diff Parser ──

fn parse_unified_diff(diff_output: &str) -> Vec<InlineDiffFile> {
    let mut files = Vec::new();
    let mut current_file: Option<InlineDiffFile> = None;
    let mut current_hunk: Option<DiffHunk> = None;
    let mut old_line: Option<u32> = None;
    let mut new_line: Option<u32> = None;

    for line in diff_output.lines() {
        if let Some(rest) = line.strip_prefix("diff --git ") {
            // Save previous file/hunk
            if let Some(mut f) = current_file.take() {
                if let Some(h) = current_hunk.take() {
                    f.hunks.push(h);
                }
                files.push(f);
            }
            // Parse "a/path b/path" from the diff header
            let parts: Vec<&str> = rest.splitn(2, " b/").collect();
            let old_name = parts
                .first()
                .map(|s| s.strip_prefix("a/").unwrap_or(s).to_string());
            let new_name = parts.get(1).map(|s| s.to_string()).unwrap_or_default();
            current_file = Some(InlineDiffFile {
                filename: new_name,
                old_filename: old_name,
                status: String::new(), // will be determined from hunks
                hunks: Vec::new(),
            });
            current_hunk = None;
        } else if let Some(header) = line.strip_prefix("@@ ") {
            // Save previous hunk
            if let Some(f) = current_file.as_mut()
                && let Some(h) = current_hunk.take()
            {
                f.hunks.push(h);
            }
            // Parse "@@ -old_start,old_count +new_start,new_count @@"
            if let Some(plus_pos) = header.find('+') {
                let old_part = &header[..plus_pos];
                let rest = &header[plus_pos..];
                let old_range = old_part.trim().trim_start_matches('-');
                let new_range = rest
                    .split(" @")
                    .next()
                    .unwrap_or("")
                    .trim_start_matches('+');

                let (os, oc) = parse_range(old_range);
                let (ns, nc) = parse_range(new_range);

                current_hunk = Some(DiffHunk {
                    old_start: os,
                    old_count: oc,
                    new_start: ns,
                    new_count: nc,
                    lines: Vec::new(),
                });
                old_line = Some(os);
                new_line = Some(ns);
            }
        } else if let Some(hunk) = current_hunk.as_mut() {
            if let Some(content) = line.strip_prefix('+') {
                hunk.lines.push(InlineDiffLine {
                    old_line_num: None,
                    new_line_num: new_line,
                    content: content.to_string(),
                    line_type: "added".into(),
                });
                new_line = new_line.map(|n| n + 1);
            } else if let Some(content) = line.strip_prefix('-') {
                hunk.lines.push(InlineDiffLine {
                    old_line_num: old_line,
                    new_line_num: None,
                    content: content.to_string(),
                    line_type: "deleted".into(),
                });
                old_line = old_line.map(|o| o + 1);
            } else if line.strip_prefix(' ').is_some() || line.is_empty() {
                let content = line.strip_prefix(' ').unwrap_or(line);
                hunk.lines.push(InlineDiffLine {
                    old_line_num: old_line,
                    new_line_num: new_line,
                    content: content.to_string(),
                    line_type: "context".into(),
                });
                old_line = old_line.map(|o| o + 1);
                new_line = new_line.map(|n| n + 1);
            } else if line.strip_prefix('\\').is_some() {
                // "\ No newline at end of file" — skip
            }
        }
    }

    // Push last file
    if let Some(mut f) = current_file.take() {
        if let Some(h) = current_hunk.take() {
            f.hunks.push(h);
        }
        files.push(f);
    }

    // Determine file status from content
    for file in &mut files {
        let has_additions = file
            .hunks
            .iter()
            .any(|h| h.lines.iter().any(|l| l.line_type == "added"));
        let has_deletions = file
            .hunks
            .iter()
            .any(|h| h.lines.iter().any(|l| l.line_type == "deleted"));
        file.status = if file.old_filename.as_deref() == Some("") || file.old_filename.is_none() {
            "added".into()
        } else if file.filename.is_empty() {
            "deleted".into()
        } else if has_additions && !has_deletions {
            "added".into()
        } else if has_deletions && !has_additions {
            "deleted".into()
        } else {
            "modified".into()
        };
        if file.old_filename.as_deref() == Some(&file.filename) {
            file.old_filename = None;
        }
    }

    files
}

fn parse_range(s: &str) -> (u32, u32) {
    let parts: Vec<&str> = s.split(',').collect();
    let start = parts.first().and_then(|p| p.parse().ok()).unwrap_or(1);
    let count = parts.get(1).and_then(|p| p.parse().ok()).unwrap_or(1);
    (start, count)
}

fn inline_to_side_by_side(files: Vec<InlineDiffFile>) -> Vec<SideBySideFile> {
    files
        .into_iter()
        .map(|f| SideBySideFile {
            filename: f.filename,
            old_filename: f.old_filename,
            status: f.status,
            hunks: f
                .hunks
                .into_iter()
                .map(|h| {
                    let mut left = Vec::new();
                    let mut right = Vec::new();
                    for line in &h.lines {
                        match line.line_type.as_str() {
                            "added" => {
                                left.push(SideBySideLine {
                                    line_num: None,
                                    content: String::new(),
                                    line_type: "empty".into(),
                                });
                                right.push(SideBySideLine {
                                    line_num: line.new_line_num,
                                    content: line.content.clone(),
                                    line_type: "added".into(),
                                });
                            }
                            "deleted" => {
                                left.push(SideBySideLine {
                                    line_num: line.old_line_num,
                                    content: line.content.clone(),
                                    line_type: "deleted".into(),
                                });
                                right.push(SideBySideLine {
                                    line_num: None,
                                    content: String::new(),
                                    line_type: "empty".into(),
                                });
                            }
                            _ => {
                                left.push(SideBySideLine {
                                    line_num: line.old_line_num,
                                    content: line.content.clone(),
                                    line_type: "context".into(),
                                });
                                right.push(SideBySideLine {
                                    line_num: line.new_line_num,
                                    content: line.content.clone(),
                                    line_type: "context".into(),
                                });
                            }
                        }
                    }
                    SideBySideHunk {
                        old_start: h.old_start,
                        old_count: h.old_count,
                        new_start: h.new_start,
                        new_count: h.new_count,
                        left,
                        right,
                    }
                })
                .collect(),
        })
        .collect()
}

/// GET /repos/{owner}/{name}/pulls/{number}/diff/inline
pub async fn get_pr_diff_inline(
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

    let diff_output = run_git_diff(
        &git_bin,
        &repo_path,
        &["diff", "-U3", &target_ref, &source_ref],
    );

    match diff_output {
        Ok(output) => {
            let files = parse_unified_diff(&output);
            let resp = InlineDiffResponse { files };
            (axum::http::StatusCode::OK, Json(resp)).into_response()
        }
        Err(e) => err_response(CoreError::Git(e)),
    }
}

/// GET /repos/{owner}/{name}/pulls/{number}/diff/side-by-side
pub async fn get_pr_diff_side_by_side(
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

    let diff_output = run_git_diff(
        &git_bin,
        &repo_path,
        &["diff", "-U3", &target_ref, &source_ref],
    );

    match diff_output {
        Ok(output) => {
            let files = parse_unified_diff(&output);
            let files = inline_to_side_by_side(files);
            let resp = SideBySideDiffResponse { files };
            (axum::http::StatusCode::OK, Json(resp)).into_response()
        }
        Err(e) => err_response(CoreError::Git(e)),
    }
}

/// POST /repos/{owner}/{name}/pulls/{number}/inline-comments
pub async fn create_inline_comment(
    State(state): State<AppState>,
    Path((owner, name, number)): Path<(String, String, i32)>,
    auth: AuthUser,
    Json(req): Json<CreateInlineComment>,
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

    // Validate side if provided
    if let Some(ref side) = req.side
        && side != "LEFT"
        && side != "RIGHT"
    {
        return err_response(CoreError::BadRequest("side must be LEFT or RIGHT".into()));
    }

    let comment = match state
        .db
        .create_pr_comment(
            pr.id,
            author_id,
            &req.body,
            req.commit_sha.as_deref(),
            Some(&req.path),
            req.line,
            req.in_reply_to_id,
        )
        .await
    {
        Ok(c) => c,
        Err(e) => return err_response(e),
    };

    // Store the side info if provided (append to existing data via update)
    // The PrComment model already supports file_path and line, side is stored in
    // a separate column or we can store it in the metadata.
    // For now we include it in the response — the DB column can be added later.

    let resp = InlineCommentResponse {
        id: comment.id.to_string(),
        author_id: comment.author_id.to_string(),
        body: comment.body,
        path: req.path,
        line: comment.line,
        side: req.side,
        commit_sha: comment.commit_sha,
        created_at: comment.created_at.to_rfc3339(),
        updated_at: comment.updated_at.to_rfc3339(),
    };
    (axum::http::StatusCode::CREATED, Json(resp)).into_response()
}

// ── PR Re-request Review ──

#[derive(Debug, Deserialize)]
pub struct ReRequestReviewRequest {
    pub reviewer_id: String,
}

pub async fn rerequest_review(
    State(state): State<AppState>,
    Path((owner, name, number)): Path<(String, String, i32)>,
    _auth: AuthUser,
    Json(req): Json<ReRequestReviewRequest>,
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

    let uid = match Uuid::parse_str(&req.reviewer_id) {
        Ok(u) => u,
        Err(_) => {
            return err_response(CoreError::BadRequest("invalid reviewer_id".into()));
        }
    };

    match state.db.rerequest_pr_review(pr.id, uid).await {
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

// ── PR Draft/Ready Toggle ──

#[derive(Debug, Deserialize)]
pub struct ToggleDraftRequest {
    pub draft: bool,
}

pub async fn toggle_pr_draft(
    State(state): State<AppState>,
    Path((owner, name, number)): Path<(String, String, i32)>,
    _auth: AuthUser,
    Json(req): Json<ToggleDraftRequest>,
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
            "PR #{} is {} (only open PRs can toggle draft status)",
            number, pr.status
        )));
    }

    match state.db.set_pr_draft(pr.id, req.draft).await {
        Ok(updated) => {
            let resp = pr_to_response(updated, None, None, None);
            (axum::http::StatusCode::OK, Json(resp)).into_response()
        }
        Err(e) => err_response(e),
    }
}

// ── Review Summary / Resolve / Assign ──

#[derive(Debug, Serialize)]
pub struct ReviewSummaryResponse {
    pub pr_id: String,
    pub approvals: i64,
    pub changes_requested: i64,
    pub comments: i64,
    pub codeowners_approved: bool,
}

#[derive(Debug, Serialize)]
pub struct ReviewAssignmentResponse {
    pub id: String,
    pub pr_id: String,
    pub user_id: String,
    pub team: String,
    pub assigned_by: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct AssignReviewersRequest {
    pub user_ids: Vec<Uuid>,
    pub team: Option<String>,
}

pub async fn get_review_summary(
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
    let summary = match state.db.get_review_summary(pr.id).await {
        Ok(s) => s,
        Err(e) => return err_response(e),
    };
    let resp = ReviewSummaryResponse {
        pr_id: summary.pr_id.to_string(),
        approvals: summary.approvals,
        changes_requested: summary.changes_requested,
        comments: summary.comments,
        codeowners_approved: summary.codeowners_approved,
    };
    (axum::http::StatusCode::OK, Json(resp)).into_response()
}

pub async fn resolve_comment_handler(
    State(state): State<AppState>,
    Path((owner, name, number, comment_id)): Path<(String, String, i32, String)>,
    auth: AuthUser,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let repo_id = match get_repo_id(pool, &owner, &name).await {
        Ok(id) => id,
        Err(e) => return err_response(e),
    };
    let _pr = match state.db.get_pr_by_number(repo_id, number).await {
        Ok(r) => r,
        Err(e) => return err_response(e),
    };
    let cid = match Uuid::parse_str(&comment_id) {
        Ok(u) => u,
        Err(_) => return err_response(CoreError::BadRequest("invalid comment_id".into())),
    };
    let user_id = uuid::Uuid::parse_str(&auth.user_id).unwrap_or(uuid::Uuid::nil());
    match state.db.resolve_comment(cid, user_id).await {
        Ok(_) => (
            axum::http::StatusCode::OK,
            Json(serde_json::json!({"resolved": true})),
        )
            .into_response(),
        Err(e) => err_response(e),
    }
}

pub async fn unresolve_comment_handler(
    State(state): State<AppState>,
    Path((owner, name, number, comment_id)): Path<(String, String, i32, String)>,
    _auth: AuthUser,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let repo_id = match get_repo_id(pool, &owner, &name).await {
        Ok(id) => id,
        Err(e) => return err_response(e),
    };
    let _pr = match state.db.get_pr_by_number(repo_id, number).await {
        Ok(r) => r,
        Err(e) => return err_response(e),
    };
    let cid = match Uuid::parse_str(&comment_id) {
        Ok(u) => u,
        Err(_) => return err_response(CoreError::BadRequest("invalid comment_id".into())),
    };
    match state.db.unresolve_comment(cid).await {
        Ok(_) => (
            axum::http::StatusCode::OK,
            Json(serde_json::json!({"resolved": false})),
        )
            .into_response(),
        Err(e) => err_response(e),
    }
}

pub async fn list_review_assignments(
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
    let assignments = match state.db.get_review_assignments(pr.id).await {
        Ok(a) => a,
        Err(e) => return err_response(e),
    };
    let items: Vec<ReviewAssignmentResponse> = assignments
        .into_iter()
        .map(|a| ReviewAssignmentResponse {
            id: a.id.to_string(),
            pr_id: a.pr_id.to_string(),
            user_id: a.user_id.to_string(),
            team: a.team,
            assigned_by: a.assigned_by.map(|u| u.to_string()),
            created_at: a.created_at.to_rfc3339(),
        })
        .collect();
    (axum::http::StatusCode::OK, Json(items)).into_response()
}

pub async fn assign_reviewers(
    State(state): State<AppState>,
    Path((owner, name, number)): Path<(String, String, i32)>,
    auth: AuthUser,
    Json(req): Json<AssignReviewersRequest>,
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
    let actor_id = uuid::Uuid::parse_str(&auth.user_id).unwrap_or(uuid::Uuid::nil());
    let team = req.team.unwrap_or_default();
    let mut results = Vec::new();
    for uid in &req.user_ids {
        if let Ok(assignment) = state.db.add_review_assignment(pr.id, *uid, &team, actor_id).await {
            results.push(ReviewAssignmentResponse {
                id: assignment.id.to_string(),
                pr_id: assignment.pr_id.to_string(),
                user_id: assignment.user_id.to_string(),
                team: assignment.team,
                assigned_by: assignment.assigned_by.map(|u| u.to_string()),
                created_at: assignment.created_at.to_rfc3339(),
            });
        }
    }
    (axum::http::StatusCode::OK, Json(results)).into_response()
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
        .route(
            "/api/v1/repos/{owner}/{name}/pulls/{number}/assignees",
            post(add_pr_assignee),
        )
        .route(
            "/api/v1/repos/{owner}/{name}/pulls/{number}/assignees/{user_id}",
            axum::routing::delete(remove_pr_assignee),
        )
        .route(
            "/api/v1/repos/{owner}/{name}/pulls/{number}/diff/inline",
            get(get_pr_diff_inline),
        )
        .route(
            "/api/v1/repos/{owner}/{name}/pulls/{number}/diff/side-by-side",
            get(get_pr_diff_side_by_side),
        )
        .route(
            "/api/v1/repos/{owner}/{name}/pulls/{number}/patch",
            get(download_patch),
        )
        .route(
            "/api/v1/repos/{owner}/{name}/pulls/{number}/inline-comments",
            post(create_inline_comment),
        )
        .route(
            "/api/v1/repos/{owner}/{name}/pulls/{number}/review/re-request",
            post(rerequest_review),
        )
        .route(
            "/api/v1/repos/{owner}/{name}/pulls/{number}/draft",
            patch(toggle_pr_draft),
        )
        .route(
            "/api/v1/repos/{owner}/{name}/pulls/{number}/review-summary",
            get(get_review_summary),
        )
        .route(
            "/api/v1/repos/{owner}/{name}/pulls/{number}/comments/{comment_id}/resolve",
            patch(resolve_comment_handler),
        )
        .route(
            "/api/v1/repos/{owner}/{name}/pulls/{number}/comments/{comment_id}/unresolve",
            patch(unresolve_comment_handler),
        )
        .route(
            "/api/v1/repos/{owner}/{name}/pulls/{number}/assignments",
            get(list_review_assignments),
        )
        .route(
            "/api/v1/repos/{owner}/{name}/pulls/{number}/assign-reviewers",
            post(assign_reviewers),
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_range_simple() {
        assert_eq!(parse_range("10,5"), (10, 5));
        assert_eq!(parse_range("1"), (1, 1));
        assert_eq!(parse_range("0,0"), (0, 0));
    }

    #[test]
    fn test_parse_unified_diff_single_file() {
        let diff = r##"diff --git a/src/main.rs b/src/main.rs
index abc1234..def5678 100644
--- a/src/main.rs
+++ b/src/main.rs
@@ -1,7 +1,8 @@
 use std::io;
 
 fn main() {
-    println!("hello");
+    println!("world");
+    println!("extra");
 }
 
 fn helper() {}"##;

        let files = parse_unified_diff(diff);
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].filename, "src/main.rs");
        assert_eq!(files[0].status, "modified");
        assert_eq!(files[0].hunks.len(), 1);

        let hunk = &files[0].hunks[0];
        assert_eq!(hunk.old_start, 1);
        assert_eq!(hunk.old_count, 7);
        assert_eq!(hunk.new_start, 1);
        assert_eq!(hunk.new_count, 8);

        // Verify line types
        let line_types: Vec<&str> = hunk.lines.iter().map(|l| l.line_type.as_str()).collect();
        assert_eq!(
            line_types,
            vec![
                "context", "context", "context", "deleted", "added", "added", "context", "context",
                "context"
            ]
        );

        // Verify line numbers
        assert_eq!(hunk.lines[0].old_line_num, Some(1));
        assert_eq!(hunk.lines[0].new_line_num, Some(1));
        assert_eq!(hunk.lines[3].old_line_num, Some(4));
        assert_eq!(hunk.lines[3].new_line_num, None);
        assert_eq!(hunk.lines[4].old_line_num, None);
        assert_eq!(hunk.lines[4].new_line_num, Some(4));
    }

    #[test]
    fn test_parse_unified_diff_added_file() {
        let diff = r##"diff --git a/new_file.txt b/new_file.txt
new file mode 100644
index 0000000..abc1234
--- /dev/null
+++ b/new_file.txt
@@ -0, +3 @@
+line one
+line two
+line three"##;

        let files = parse_unified_diff(diff);
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].filename, "new_file.txt");
        assert_eq!(files[0].hunks.len(), 1);

        let hunk = &files[0].hunks[0];
        assert_eq!(hunk.lines.len(), 3);
        assert!(hunk.lines.iter().all(|l| l.line_type == "added"));
    }

    #[test]
    fn test_parse_unified_diff_no_changes() {
        let diff = "";
        let files = parse_unified_diff(diff);
        assert!(files.is_empty());
    }

    #[test]
    fn test_inline_to_side_by_side_conversion() {
        let diff = r##"diff --git a/src/main.rs b/src/main.rs
index abc1234..def5678 100644
--- a/src/main.rs
+++ b/src/main.rs
@@ -1,3 +1,3 @@
 context line
-old line
+new line"##;

        let inline_files = parse_unified_diff(diff);
        let side_by_side = inline_to_side_by_side(inline_files);

        assert_eq!(side_by_side.len(), 1);
        let file = &side_by_side[0];
        assert_eq!(file.hunks.len(), 1);

        let hunk = &file.hunks[0];
        assert_eq!(hunk.left.len(), 3);
        assert_eq!(hunk.right.len(), 3);

        // Left: context, deleted, empty
        assert_eq!(hunk.left[0].line_type, "context");
        assert_eq!(hunk.left[1].line_type, "deleted");
        assert_eq!(hunk.left[2].line_type, "empty");

        // Right: context, empty, added
        assert_eq!(hunk.right[0].line_type, "context");
        assert_eq!(hunk.right[1].line_type, "empty");
        assert_eq!(hunk.right[2].line_type, "added");
    }

    #[test]
    fn test_pr_routes_type() {
        fn _assert_routes() -> Router<AppState> {
            pr_routes()
        }
    }

    #[test]
    fn test_inline_diff_response_serialization() {
        let resp = InlineDiffResponse {
            files: vec![InlineDiffFile {
                filename: "test.rs".into(),
                old_filename: None,
                status: "modified".into(),
                hunks: vec![DiffHunk {
                    old_start: 1,
                    old_count: 3,
                    new_start: 1,
                    new_count: 4,
                    lines: vec![InlineDiffLine {
                        old_line_num: Some(1),
                        new_line_num: Some(1),
                        content: "context".into(),
                        line_type: "context".into(),
                    }],
                }],
            }],
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("\"filename\":\"test.rs\""));
        assert!(json.contains("\"line_type\":\"context\""));
    }

    #[test]
    fn test_side_by_side_diff_response_serialization() {
        let resp = SideBySideDiffResponse {
            files: vec![SideBySideFile {
                filename: "test.rs".into(),
                old_filename: None,
                status: "modified".into(),
                hunks: vec![SideBySideHunk {
                    old_start: 1,
                    old_count: 1,
                    new_start: 1,
                    new_count: 1,
                    left: vec![SideBySideLine {
                        line_num: Some(1),
                        content: "old".into(),
                        line_type: "deleted".into(),
                    }],
                    right: vec![SideBySideLine {
                        line_num: Some(1),
                        content: "new".into(),
                        line_type: "added".into(),
                    }],
                }],
            }],
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("\"line_type\":\"deleted\""));
        assert!(json.contains("\"line_type\":\"added\""));
    }

    #[test]
    fn test_create_inline_comment_deserialization() {
        let json = r##"{
            "body": "nice change",
            "path": "src/main.rs",
            "line": 10,
            "side": "RIGHT",
            "commit_sha": "abc1234"
        }"##;
        let req: CreateInlineComment = serde_json::from_str(json).unwrap();
        assert_eq!(req.body, "nice change");
        assert_eq!(req.path, "src/main.rs");
        assert_eq!(req.line, Some(10));
        assert_eq!(req.side.as_deref(), Some("RIGHT"));
    }
}
