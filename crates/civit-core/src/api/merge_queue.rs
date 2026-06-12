#![forbid(unsafe_code)]

use axum::{
    Json, Router,
    extract::{Path, State},
    response::IntoResponse,
    routing::{delete, get},
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::api::AppState;
use crate::api::auth::AuthUser;
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
