#![forbid(unsafe_code)]

use axum::{
    Json, Router,
    extract::{Path, State},
    response::IntoResponse,
    routing::{delete, get, patch, post},
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::api::AppState;
use crate::api::auth::{AuthUser, require_admin};
use crate::error::{CoreError, Result};

fn err_response(e: CoreError) -> axum::response::Response {
    let status = e.status_code();
    let body = e.error_response();
    (status, Json(body)).into_response()
}

#[derive(Debug, Deserialize)]
pub struct AddToMergeQueue {
    pub pr_number: i32,
}

#[derive(Debug, Deserialize)]
pub struct ReorderMergeQueueRequest {
    pub new_position: i32,
}

#[derive(Debug, Serialize)]
pub struct MergeQueueEntry {
    pub id: String,
    pub repo_id: String,
    pub pr_id: String,
    pub pr_number: i32,
    pub position: i32,
    pub status: String,
    pub ci_status: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize)]
pub struct MergeQueueResponse {
    pub items: Vec<MergeQueueEntry>,
    pub total: i64,
}

#[derive(Debug, Serialize)]
pub struct AdminMergeQueueEntry {
    pub id: String,
    pub pr_number: i64,
    pub pr_title: String,
    pub repo_full_name: String,
    pub branch: String,
    pub status: String,
    pub position: i32,
    pub enqueued_at: String,
}

#[derive(Debug, Serialize)]
pub struct AdminMergeQueueResponse {
    pub items: Vec<AdminMergeQueueEntry>,
    pub total: i64,
}

pub async fn get_repo_id(pool: &sqlx::PgPool, owner: &str, name: &str) -> Result<Uuid> {
    sqlx::query_scalar::<_, Uuid>(
        "SELECT r.id FROM repositories r JOIN users u ON r.owner_id = u.id WHERE u.username = $1 AND r.name = $2",
    )
    .bind(owner)
    .bind(name)
    .fetch_one(pool)
    .await
    .map_err(|_| CoreError::NotFound(format!("repo {owner}/{name}")))
}

/// POST /repos/{owner}/{name}/merge-queue
pub async fn add_to_merge_queue(
    State(state): State<AppState>,
    Path((owner, name)): Path<(String, String)>,
    _auth: AuthUser,
    Json(req): Json<AddToMergeQueue>,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let repo_id = match get_repo_id(pool, &owner, &name).await {
        Ok(id) => id,
        Err(e) => return err_response(e),
    };

    // Verify the PR exists and is open
    let pr = match state.db.get_pr_by_number(repo_id, req.pr_number).await {
        Ok(pr) => pr,
        Err(e) => return err_response(e),
    };

    if pr.status != "open" {
        return err_response(CoreError::BadRequest(format!(
            "PR #{} is {} (only open PRs can be added to the merge queue)",
            pr.number, pr.status
        )));
    }

    if pr.draft {
        return err_response(CoreError::BadRequest(format!(
            "PR #{} is a draft and cannot be added to the merge queue",
            pr.number,
        )));
    }

    // Get next position
    let max_pos: Option<(i32,)> = sqlx::query_as(
        "SELECT MAX(position) FROM merge_queue WHERE repo_id = $1 AND status = 'queued'",
    )
    .bind(repo_id)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten();

    let next_pos = max_pos.map(|(p,)| p + 1).unwrap_or(1);

    // Insert into queue (upsert: if already queued, keep existing position)
    let row = sqlx::query_as::<_, MergeQueueRow>(
        r#"INSERT INTO merge_queue (repo_id, pr_id, position, status, ci_status)
           VALUES ($1, $2, $3, 'queued', 'pending')
           ON CONFLICT (repo_id, pr_id) DO UPDATE SET updated_at = NOW()
           RETURNING *"#,
    )
    .bind(repo_id)
    .bind(pr.id)
    .bind(next_pos)
    .fetch_one(pool)
    .await
    .map_err(|e| CoreError::Database(format!("add_to_merge_queue: {e}")));

    match row {
        Ok(entry) => {
            let resp = MergeQueueEntry {
                id: entry.id.to_string(),
                repo_id: entry.repo_id.to_string(),
                pr_id: entry.pr_id.to_string(),
                pr_number: pr.number,
                position: entry.position,
                status: entry.status,
                ci_status: entry.ci_status,
                created_at: entry.created_at.to_rfc3339(),
                updated_at: entry.updated_at.to_rfc3339(),
            };
            (axum::http::StatusCode::CREATED, Json(resp)).into_response()
        }
        Err(e) => err_response(e),
    }
}

/// GET /repos/{owner}/{name}/merge-queue
pub async fn get_merge_queue(
    State(state): State<AppState>,
    Path((owner, name)): Path<(String, String)>,
    _auth: AuthUser,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let repo_id = match get_repo_id(pool, &owner, &name).await {
        Ok(id) => id,
        Err(e) => return err_response(e),
    };

    let entries_result: Result<Vec<MergeQueueRow>> = sqlx::query_as(
        "SELECT * FROM merge_queue WHERE repo_id = $1 AND status = 'queued' ORDER BY position ASC",
    )
    .bind(repo_id)
    .fetch_all(pool)
    .await
    .map_err(|e| CoreError::Database(format!("get_merge_queue: {e}")));

    match entries_result {
        Ok(rows) => {
            let total = rows.len() as i64;
            let mut items = Vec::new();
            for row in rows {
                let pr_number: Option<(i32,)> = sqlx::query_as(
                    "SELECT number FROM pull_requests WHERE id = $1",
                )
                .bind(row.pr_id)
                .fetch_optional(pool)
                .await
                .ok()
                .flatten();

                items.push(MergeQueueEntry {
                    id: row.id.to_string(),
                    repo_id: row.repo_id.to_string(),
                    pr_id: row.pr_id.to_string(),
                    pr_number: pr_number.map(|(n,)| n).unwrap_or(0),
                    position: row.position,
                    status: row.status,
                    ci_status: row.ci_status,
                    created_at: row.created_at.to_rfc3339(),
                    updated_at: row.updated_at.to_rfc3339(),
                });
            }

            (axum::http::StatusCode::OK, Json(MergeQueueResponse { items, total })).into_response()
        }
        Err(e) => err_response(e),
    }
}

/// POST /repos/{owner}/{name}/merge-queue/process
/// Process the merge queue: attempt to merge the next pending entry.
pub async fn process_merge_queue_handler(
    State(state): State<AppState>,
    Path((owner, name)): Path<(String, String)>,
    _auth: AuthUser,
) -> impl IntoResponse {
    match process_merge_queue(&state, &owner, &name).await {
        Ok(()) => (axum::http::StatusCode::OK, Json(serde_json::json!({"status": "processed"}))).into_response(),
        Err(e) => err_response(e),
    }
}

/// DELETE /repos/{owner}/{name}/merge-queue/{id}
pub async fn remove_from_merge_queue(
    State(state): State<AppState>,
    Path((owner, name, id)): Path<(String, String, String)>,
    _auth: AuthUser,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let repo_id = match get_repo_id(pool, &owner, &name).await {
        Ok(id) => id,
        Err(e) => return err_response(e),
    };

    let entry_id = match Uuid::parse_str(&id) {
        Ok(u) => u,
        Err(_) => return err_response(CoreError::BadRequest("invalid queue entry id".into())),
    };

    let result = sqlx::query(
        "DELETE FROM merge_queue WHERE id = $1 AND repo_id = $2",
    )
    .bind(entry_id)
    .bind(repo_id)
    .execute(pool)
    .await
    .map_err(|e| CoreError::Database(format!("remove_from_merge_queue: {e}")));

    match result {
        Ok(r) => {
            if r.rows_affected() == 0 {
                err_response(CoreError::NotFound("queue entry not found".into()))
            } else {
                // Re-sequence positions
                let _ = sqlx::query(
                    r#"UPDATE merge_queue SET position = sub.new_pos, updated_at = NOW()
                       FROM (
                           SELECT id, ROW_NUMBER() OVER (ORDER BY position) AS new_pos
                           FROM merge_queue WHERE repo_id = $1 AND status = 'queued'
                       ) sub
                       WHERE merge_queue.id = sub.id"#,
                )
                .bind(repo_id)
                .execute(pool)
                .await;

                (axum::http::StatusCode::NO_CONTENT, ()).into_response()
            }
        }
        Err(e) => err_response(e),
    }
}

/// GET /admin/merge-queue - list all merge queue entries across all repos
pub async fn admin_list_all_merge_queue(
    State(state): State<AppState>,
    auth: AuthUser,
) -> impl IntoResponse {
    if let Err(rejection) = require_admin(&auth) {
        return rejection.into_response();
    }

    let pool = state.db.pool();

    let rows_result: Result<Vec<MergeQueueRow>> = sqlx::query_as(
        "SELECT * FROM merge_queue WHERE status = 'queued' ORDER BY position ASC LIMIT 100",
    )
    .fetch_all(pool)
    .await
    .map_err(|e| CoreError::Database(format!("admin_list_all_merge_queue: {e}")));

    match rows_result {
        Ok(rows) => {
            let mut items = Vec::new();
            for row in rows {
                let pr_info: Option<(String, i64, String, String)> = sqlx::query_as(
                    r#"SELECT pr.title, pr.number::bigint, r.full_name, pr.source_branch
                       FROM pull_requests pr
                       JOIN repositories r ON pr.repo_id = r.id
                       WHERE pr.id = $1"#,
                )
                .bind(row.pr_id)
                .fetch_optional(pool)
                .await
                .ok()
                .flatten();

                let (pr_title, pr_number, repo_full_name, branch) = pr_info
                    .unwrap_or_default();

                items.push(AdminMergeQueueEntry {
                    id: row.id.to_string(),
                    pr_number,
                    pr_title,
                    repo_full_name,
                    branch,
                    status: row.status,
                    position: row.position,
                    enqueued_at: row.created_at.to_rfc3339(),
                });
            }
            let total = items.len() as i64;
            (axum::http::StatusCode::OK, Json(AdminMergeQueueResponse { items, total })).into_response()
        }
        Err(e) => err_response(e),
    }
}

/// DELETE /admin/merge-queue/{id}
pub async fn admin_remove_merge_queue_entry(
    State(state): State<AppState>,
    Path(id): Path<String>,
    auth: AuthUser,
) -> impl IntoResponse {
    if let Err(rejection) = require_admin(&auth) {
        return rejection.into_response();
    }

    let pool = state.db.pool();
    let entry_id = match Uuid::parse_str(&id) {
        Ok(u) => u,
        Err(_) => return err_response(CoreError::BadRequest("invalid queue entry id".into())),
    };

    let result = sqlx::query("DELETE FROM merge_queue WHERE id = $1")
        .bind(entry_id)
        .execute(pool)
        .await
        .map_err(|e| CoreError::Database(format!("admin_remove_merge_queue_entry: {e}")));

    match result {
        Ok(r) => {
            if r.rows_affected() == 0 {
                err_response(CoreError::NotFound("queue entry not found".into()))
            } else {
                (axum::http::StatusCode::NO_CONTENT, ()).into_response()
            }
        }
        Err(e) => err_response(e),
    }
}

/// PATCH /admin/merge-queue/{id}/reorder
pub async fn admin_reorder_merge_queue_entry(
    State(state): State<AppState>,
    Path(id): Path<String>,
    auth: AuthUser,
    Json(req): Json<ReorderMergeQueueRequest>,
) -> impl IntoResponse {
    if let Err(rejection) = require_admin(&auth) {
        return rejection.into_response();
    }

    let pool = state.db.pool();
    let entry_id = match Uuid::parse_str(&id) {
        Ok(u) => u,
        Err(_) => return err_response(CoreError::BadRequest("invalid queue entry id".into())),
    };

    // Get current entry to find repo_id
    let current: Option<MergeQueueRow> = sqlx::query_as(
        "SELECT * FROM merge_queue WHERE id = $1",
    )
    .bind(entry_id)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten();

    let current = match current {
        Some(c) => c,
        None => return err_response(CoreError::NotFound("queue entry not found".into())),
    };

    let new_pos = req.new_position.max(1);
    let old_pos = current.position;

    if old_pos == new_pos {
        return (axum::http::StatusCode::OK, ()).into_response();
    }

    if new_pos > old_pos {
        // Moving down: shift entries between old+1 and new up by 1
        let _ = sqlx::query(
            "UPDATE merge_queue SET position = position - 1, updated_at = NOW() WHERE repo_id = $1 AND status = 'queued' AND position > $2 AND position <= $3",
        )
        .bind(current.repo_id)
        .bind(old_pos)
        .bind(new_pos)
        .execute(pool)
        .await;
    } else {
        // Moving up: shift entries between new and old-1 down by 1
        let _ = sqlx::query(
            "UPDATE merge_queue SET position = position + 1, updated_at = NOW() WHERE repo_id = $1 AND status = 'queued' AND position >= $2 AND position < $3",
        )
        .bind(current.repo_id)
        .bind(new_pos)
        .bind(old_pos)
        .execute(pool)
        .await;
    }

    let _ = sqlx::query(
        "UPDATE merge_queue SET position = $1, updated_at = NOW() WHERE id = $2",
    )
    .bind(new_pos)
    .bind(entry_id)
    .execute(pool)
    .await;

    (axum::http::StatusCode::OK, ()).into_response()
}

/// Process the merge queue for a repository.
/// Takes the first pending entry, attempts to merge it, and either marks it as
/// merged or failed. Only one merge is attempted at a time.
pub async fn process_merge_queue(state: &AppState, owner: &str, name: &str) -> Result<()> {
    let pool = state.db.pool();
    let repo_id = get_repo_id(pool, owner, name).await?;

    // Get the first pending entry
    let entry: Option<MergeQueueRow> = sqlx::query_as(
        r#"SELECT * FROM merge_queue
           WHERE repo_id = $1 AND status = 'pending'
           ORDER BY position ASC LIMIT 1"#,
    )
    .bind(repo_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| CoreError::Database(format!("process_merge_queue: {e}")))?;

    let entry = match entry {
        Some(e) => e,
        None => return Ok(()),
    };

    // Mark as merging
    let _ = sqlx::query(
        "UPDATE merge_queue SET status = 'merging', updated_at = NOW() WHERE id = $1",
    )
    .bind(entry.id)
    .execute(pool)
    .await;

    // Get the PR
    let pr = match state.db.get_pr(entry.pr_id).await {
        Ok(pr) => pr,
        Err(e) => {
            mark_queue_failed(pool, entry.id).await;
            return Err(e);
        }
    };

    if pr.status != "open" || pr.draft {
        mark_queue_failed(pool, entry.id).await;
        return Ok(());
    }

    // Perform the merge
    let strategy: crate::git::MergeStrategy = pr
        .merge_strategy
        .parse()
        .unwrap_or(crate::git::MergeStrategy::Merge);

    let merge_result = match state.git_service.merge_branch(
        owner,
        name,
        &pr.source_branch,
        &pr.target_branch,
        strategy,
        "merge-queue (CivitForge)",
        "merge-queue@civitforge.local",
    ) {
        Ok(r) => r,
        Err(e) => {
            tracing::warn!(
                pr_number = pr.number,
                error = %e,
                "merge_queue: merge failed"
            );
            mark_queue_failed(pool, entry.id).await;
            return Ok(());
        }
    };

    // Record the merge in DB
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
        let _ = sqlx::query(
            "UPDATE merge_queue SET status = 'merged', merged_at = NOW(), updated_at = NOW() WHERE id = $1",
        )
        .bind(entry.id)
        .execute(pool)
        .await;

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
                    "merge_queue": true,
                }),
            )
            .await;

        tracing::info!(
            pr_number = pr.number,
            commit_sha = %merge_result.commit_sha,
            "merge_queue: PR merged successfully"
        );
    } else {
        mark_queue_failed(pool, entry.id).await;
    }

    Ok(())
}

async fn mark_queue_failed(pool: &sqlx::PgPool, entry_id: Uuid) {
    let _ = sqlx::query(
        "UPDATE merge_queue SET status = 'failed', updated_at = NOW() WHERE id = $1",
    )
    .bind(entry_id)
    .execute(pool)
    .await;
}

pub fn merge_queue_routes() -> Router<AppState> {
    Router::new()
        .route(
            "/api/v1/repos/{owner}/{name}/merge-queue",
            get(get_merge_queue).post(add_to_merge_queue),
        )
        .route(
            "/api/v1/repos/{owner}/{name}/merge-queue/{id}",
            delete(remove_from_merge_queue),
        )
        .route(
            "/api/v1/repos/{owner}/{name}/merge-queue/process",
            post(process_merge_queue_handler),
        )
        .route(
            "/api/v1/admin/merge-queue",
            get(admin_list_all_merge_queue),
        )
        .route(
            "/api/v1/admin/merge-queue/{id}",
            delete(admin_remove_merge_queue_entry),
        )
        .route(
            "/api/v1/admin/merge-queue/{id}/reorder",
            patch(admin_reorder_merge_queue_entry),
        )
}

#[derive(Debug, sqlx::FromRow)]
struct MergeQueueRow {
    id: Uuid,
    repo_id: Uuid,
    pr_id: Uuid,
    position: i32,
    status: String,
    ci_status: String,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merge_queue_routes_compiled() {
        let _ = merge_queue_routes();
    }
}
