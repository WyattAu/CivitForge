#![forbid(unsafe_code)]

//! Import job persistence — Migration API v1 (ADR-0006).
//!
//! Jobs track forge migrations end-to-end: queued → cloning → verifying →
//! completed/failed, with post-clone verification artifacts.

use crate::error::{DbError, Result};
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ImportJob {
    pub id: Uuid,
    pub user_id: Uuid,
    pub forge: String,
    pub source_url: String,
    pub dest_owner: String,
    pub dest_name: String,
    pub status: String,
    pub error: Option<String>,
    pub commit_count: Option<i64>,
    pub verified_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl super::DbRepository {
    pub async fn create_import_job(
        &self,
        user_id: Uuid,
        forge: &str,
        source_url: &str,
        dest_owner: &str,
        dest_name: &str,
    ) -> Result<ImportJob> {
        sqlx::query_as(
            "INSERT INTO import_jobs (user_id, forge, source_url, dest_owner, dest_name)
             VALUES ($1, $2, $3, $4, $5)
             RETURNING *",
        )
        .bind(user_id)
        .bind(forge)
        .bind(source_url)
        .bind(dest_owner)
        .bind(dest_name)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_import_job: {e}")))
    }

    pub async fn get_import_job(&self, id: Uuid) -> Result<ImportJob> {
        sqlx::query_as("SELECT * FROM import_jobs WHERE id = $1")
            .bind(id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("get_import_job: {e}")))
    }

    pub async fn list_import_jobs(&self, user_id: Uuid, limit: i64) -> Result<Vec<ImportJob>> {
        sqlx::query_as(
            "SELECT * FROM import_jobs WHERE user_id = $1 ORDER BY created_at DESC LIMIT $2",
        )
        .bind(user_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("list_import_jobs: {e}")))
    }

    pub async fn update_import_job_status(&self, id: Uuid, status: &str) -> Result<()> {
        sqlx::query("UPDATE import_jobs SET status = $2, updated_at = NOW() WHERE id = $1")
            .bind(id)
            .bind(status)
            .execute(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("update_import_job_status: {e}")))
            .map(|_| ())
    }

    pub async fn fail_import_job(&self, id: Uuid, error: &str) -> Result<()> {
        sqlx::query(
            "UPDATE import_jobs SET status = 'failed', error = $2, updated_at = NOW() WHERE id = $1",
        )
        .bind(id)
        .bind(error)
        .execute(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("fail_import_job: {e}")))
        .map(|_| ())
    }

    /// Post-clone verification: record commit count + mark completed.
    pub async fn complete_import_job(&self, id: Uuid, commit_count: i64) -> Result<()> {
        sqlx::query(
            "UPDATE import_jobs
             SET status = 'completed', commit_count = $2, verified_at = NOW(), updated_at = NOW()
             WHERE id = $1",
        )
        .bind(id)
        .bind(commit_count)
        .execute(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("complete_import_job: {e}")))
        .map(|_| ())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // SQL shape guard: statuses stay aligned with the CHECK constraint
    // in 639_add_import_jobs.sql.
    #[test]
    fn job_status_values_match_migration_constraint() {
        let valid = ["queued", "cloning", "verifying", "completed", "failed"];
        let sql = include_str!("../migrations/639_add_import_jobs.sql");
        for status in valid {
            assert!(sql.contains(status), "status {status} missing from CHECK");
        }
    }
}
