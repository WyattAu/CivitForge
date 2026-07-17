#![forbid(unsafe_code)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct SearchIndexSyncLogEntry {
    pub id: Uuid,
    pub repository_id: Uuid,
    pub commit_sha: String,
    pub files_indexed: i32,
    pub files_skipped: i32,
    pub duration_ms: Option<i32>,
    pub status: String,
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct SearchIndexQueueEntry {
    pub id: Uuid,
    pub repository_id: Uuid,
    pub commit_sha: String,
    pub priority: i32,
    pub status: String,
    pub attempts: i32,
    pub max_attempts: i32,
    pub created_at: DateTime<Utc>,
    pub processed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct QueueSummary {
    pub total: i64,
    pub pending: i64,
    pub processing: i64,
    pub completed: i64,
    pub failed: i64,
}

pub struct SearchIndexSyncService {
    pool: PgPool,
}

impl SearchIndexSyncService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn queue_indexing(
        &self,
        repository_id: Uuid,
        commit_sha: &str,
        priority: i32,
    ) -> Result<SearchIndexQueueEntry, sqlx::Error> {
        let entry = sqlx::query_as::<_, SearchIndexQueueEntry>(
            r#"
            INSERT INTO search_index_queue_v1 (repository_id, commit_sha, priority)
            VALUES ($1, $2, $3)
            ON CONFLICT (repository_id, commit_sha) DO UPDATE
                SET priority = GREATEST(search_index_queue_v1.priority, EXCLUDED.priority),
                    status = 'pending',
                    attempts = 0
            RETURNING id, repository_id, commit_sha, priority, status, attempts, max_attempts, created_at, processed_at
            "#,
        )
        .bind(repository_id)
        .bind(commit_sha)
        .bind(priority)
        .fetch_one(&self.pool)
        .await?;

        Ok(entry)
    }

    pub async fn process_queue(&self) -> Result<Option<SearchIndexQueueEntry>, sqlx::Error> {
        let entry = sqlx::query_as::<_, SearchIndexQueueEntry>(
            r#"
            UPDATE search_index_queue_v1
            SET status = 'processing', attempts = attempts + 1, processed_at = NOW()
            WHERE id = (
                SELECT id FROM search_index_queue_v1
                WHERE status = 'pending' AND attempts < max_attempts
                ORDER BY priority DESC, created_at ASC
                LIMIT 1
                FOR UPDATE SKIP LOCKED
            )
            RETURNING id, repository_id, commit_sha, priority, status, attempts, max_attempts, created_at, processed_at
            "#,
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(entry)
    }

    pub async fn index_commit(
        &self,
        repository_id: Uuid,
        commit_sha: &str,
    ) -> Result<(i32, i32), sqlx::Error> {
        let _ = repository_id;
        let _ = commit_sha;
        Ok((0, 0))
    }

    pub async fn record_sync(
        &self,
        repository_id: Uuid,
        commit_sha: &str,
        files_indexed: i32,
        files_skipped: i32,
        duration_ms: Option<i32>,
        status: &str,
        error_message: Option<&str>,
    ) -> Result<SearchIndexSyncLogEntry, sqlx::Error> {
        let entry = sqlx::query_as::<_, SearchIndexSyncLogEntry>(
            r#"
            INSERT INTO search_index_sync_log_v1
                (repository_id, commit_sha, files_indexed, files_skipped, duration_ms, status, error_message)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id, repository_id, commit_sha, files_indexed, files_skipped, duration_ms, status, error_message, created_at
            "#,
        )
        .bind(repository_id)
        .bind(commit_sha)
        .bind(files_indexed)
        .bind(files_skipped)
        .bind(duration_ms)
        .bind(status)
        .bind(error_message)
        .fetch_one(&self.pool)
        .await?;

        Ok(entry)
    }

    pub async fn get_sync_log(
        &self,
        repository_id: Uuid,
        limit: i64,
    ) -> Result<Vec<SearchIndexSyncLogEntry>, sqlx::Error> {
        let entries = sqlx::query_as::<_, SearchIndexSyncLogEntry>(
            r#"
            SELECT id, repository_id, commit_sha, files_indexed, files_skipped, duration_ms, status, error_message, created_at
            FROM search_index_sync_log_v1
            WHERE repository_id = $1
            ORDER BY created_at DESC
            LIMIT $2
            "#,
        )
        .bind(repository_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(entries)
    }

    pub async fn get_queue_status(&self) -> Result<QueueSummary, sqlx::Error> {
        let summary = sqlx::query_as::<_, QueueSummary>(
            r#"
            SELECT
                COUNT(*)::bigint AS total,
                COUNT(*) FILTER (WHERE status = 'pending')::bigint AS pending,
                COUNT(*) FILTER (WHERE status = 'processing')::bigint AS processing,
                COUNT(*) FILTER (WHERE status = 'completed')::bigint AS completed,
                COUNT(*) FILTER (WHERE status = 'failed')::bigint AS failed
            FROM search_index_queue_v1
            "#,
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(summary)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sync_log_serialization_roundtrip() {
        let entry = SearchIndexSyncLogEntry {
            id: Uuid::new_v4(),
            repository_id: Uuid::new_v4(),
            commit_sha: "abc123".into(),
            files_indexed: 5,
            files_skipped: 2,
            duration_ms: Some(150),
            status: "completed".into(),
            error_message: None,
            created_at: Utc::now(),
        };
        let json = serde_json::to_string(&entry).unwrap();
        let de: SearchIndexSyncLogEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(de.commit_sha, "abc123");
        assert_eq!(de.files_indexed, 5);
        assert_eq!(de.status, "completed");
    }

    #[test]
    fn test_queue_entry_serialization_roundtrip() {
        let entry = SearchIndexQueueEntry {
            id: Uuid::new_v4(),
            repository_id: Uuid::new_v4(),
            commit_sha: "def456".into(),
            priority: 10,
            status: "pending".into(),
            attempts: 0,
            max_attempts: 3,
            created_at: Utc::now(),
            processed_at: None,
        };
        let json = serde_json::to_string(&entry).unwrap();
        let de: SearchIndexQueueEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(de.commit_sha, "def456");
        assert_eq!(de.priority, 10);
        assert_eq!(de.max_attempts, 3);
    }

    #[test]
    fn test_queue_summary_defaults() {
        let summary = QueueSummary {
            total: 0,
            pending: 0,
            processing: 0,
            completed: 0,
            failed: 0,
        };
        assert_eq!(summary.total, 0);
        assert_eq!(summary.pending, 0);
    }
}
