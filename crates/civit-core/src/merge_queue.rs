#![forbid(unsafe_code)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::collections::VecDeque;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergeQueueEntry {
    pub id: String,
    pub pr_number: u32,
    pub head_sha: String,
    pub base_branch: String,
    pub status: MergeStatus,
    pub position: u32,
    pub enqueued_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct DbQueueEntry {
    pub id: Uuid,
    pub repository_id: Uuid,
    pub pull_request_id: Uuid,
    pub position: i32,
    pub status: String,
    pub merge_sha: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct DbQueueCheck {
    pub id: Uuid,
    pub queue_entry_id: Uuid,
    pub check_name: String,
    pub status: String,
    pub output: serde_json::Value,
    pub completed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct QueueStatusSummary {
    pub total_entries: i64,
    pub waiting: i64,
    pub running: i64,
    pub passed: i64,
    pub failed: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MergeStatus {
    Queued,
    Testing,
    Merging,
    Completed,
    Failed,
    Cancelled,
}

pub struct MergeQueueStore {
    pool: PgPool,
}

impl MergeQueueStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn enqueue_pr(
        &self,
        repository_id: Uuid,
        pull_request_id: Uuid,
    ) -> Result<DbQueueEntry, sqlx::Error> {
        let max_pos: Option<i32> = sqlx::query_scalar(
            "SELECT MAX(position) FROM merge_queue_entries_v1 WHERE repository_id = $1",
        )
        .bind(repository_id)
        .fetch_optional(&self.pool)
        .await?;

        let next_pos = max_pos.map(|p| p + 1).unwrap_or(0);

        let entry = sqlx::query_as::<_, DbQueueEntry>(
            r#"
            INSERT INTO merge_queue_entries_v1 (repository_id, pull_request_id, position, status)
            VALUES ($1, $2, $3, 'waiting')
            ON CONFLICT (repository_id, pull_request_id) DO UPDATE
                SET position = EXCLUDED.position, status = 'waiting', updated_at = NOW()
            RETURNING id, repository_id, pull_request_id, position, status, merge_sha, created_at, updated_at
            "#,
        )
        .bind(repository_id)
        .bind(pull_request_id)
        .bind(next_pos)
        .fetch_one(&self.pool)
        .await?;

        Ok(entry)
    }

    pub async fn dequeue_pr(
        &self,
        repository_id: Uuid,
    ) -> Result<Option<DbQueueEntry>, sqlx::Error> {
        let entry = sqlx::query_as::<_, DbQueueEntry>(
            r#"
            UPDATE merge_queue_entries_v1
            SET status = 'running', updated_at = NOW()
            WHERE id = (
                SELECT id FROM merge_queue_entries_v1
                WHERE repository_id = $1 AND status = 'waiting'
                ORDER BY position ASC
                LIMIT 1
                FOR UPDATE SKIP LOCKED
            )
            RETURNING id, repository_id, pull_request_id, position, status, merge_sha, created_at, updated_at
            "#,
        )
        .bind(repository_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(entry)
    }

    pub async fn get_queue(
        &self,
        repository_id: Uuid,
    ) -> Result<Vec<DbQueueEntry>, sqlx::Error> {
        let entries = sqlx::query_as::<_, DbQueueEntry>(
            r#"
            SELECT id, repository_id, pull_request_id, position, status, merge_sha, created_at, updated_at
            FROM merge_queue_entries_v1
            WHERE repository_id = $1 AND status IN ('waiting', 'running')
            ORDER BY position ASC
            "#,
        )
        .bind(repository_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(entries)
    }

    pub async fn update_check_status(
        &self,
        queue_entry_id: Uuid,
        check_name: &str,
        status: &str,
        output: serde_json::Value,
    ) -> Result<DbQueueCheck, sqlx::Error> {
        let check = sqlx::query_as::<_, DbQueueCheck>(
            r#"
            INSERT INTO merge_queue_checks_v1 (queue_entry_id, check_name, status, output, completed_at)
            VALUES ($1, $2, $3, $4, CASE WHEN $3 IN ('passed', 'failed') THEN NOW() ELSE NULL END)
            ON CONFLICT (queue_entry_id, check_name) DO UPDATE
                SET status = EXCLUDED.status, output = EXCLUDED.output,
                    completed_at = EXCLUDED.completed_at
            RETURNING id, queue_entry_id, check_name, status, output, completed_at, created_at
            "#,
        )
        .bind(queue_entry_id)
        .bind(check_name)
        .bind(status)
        .bind(output)
        .fetch_one(&self.pool)
        .await?;

        Ok(check)
    }

    pub async fn process_queue(
        &self,
        repository_id: Uuid,
    ) -> Result<Option<DbQueueEntry>, sqlx::Error> {
        self.dequeue_pr(repository_id).await
    }

    pub async fn get_queue_status(
        &self,
        repository_id: Uuid,
    ) -> Result<QueueStatusSummary, sqlx::Error> {
        let summary = sqlx::query_as::<_, QueueStatusSummary>(
            r#"
            SELECT
                COUNT(*)::bigint AS total_entries,
                COUNT(*) FILTER (WHERE status = 'waiting')::bigint AS waiting,
                COUNT(*) FILTER (WHERE status = 'running')::bigint AS running,
                COUNT(*) FILTER (WHERE status = 'passed')::bigint AS passed,
                COUNT(*) FILTER (WHERE status = 'failed')::bigint AS failed
            FROM merge_queue_entries_v1
            WHERE repository_id = $1
            "#,
        )
        .bind(repository_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(summary)
    }
}

pub struct MergeQueue {
    queue: std::sync::Mutex<VecDeque<MergeQueueEntry>>,
    max_size: usize,
}

impl MergeQueue {
    pub fn new(max_size: usize) -> Self {
        Self {
            queue: std::sync::Mutex::new(VecDeque::with_capacity(max_size)),
            max_size,
        }
    }

    pub fn enqueue(&self, entry: MergeQueueEntry) -> Result<u32, String> {
        let mut queue = self.queue.lock().unwrap();
        if queue.len() >= self.max_size {
            return Err("merge queue is full".into());
        }
        let position = queue.len() as u32;
        queue.push_back(entry);
        Ok(position)
    }

    pub fn dequeue_next(&self) -> Option<MergeQueueEntry> {
        let mut queue = self.queue.lock().unwrap();
        queue.pop_front()
    }

    pub fn peek(&self) -> Option<MergeQueueEntry> {
        let queue = self.queue.lock().unwrap();
        queue.front().cloned()
    }

    pub fn cancel(&self, pr_number: u32) -> bool {
        let mut queue = self.queue.lock().unwrap();
        if let Some(entry) = queue.iter_mut().find(|e| e.pr_number == pr_number)
            && (entry.status == MergeStatus::Queued || entry.status == MergeStatus::Testing)
        {
            entry.status = MergeStatus::Cancelled;
            return true;
        }
        false
    }

    pub fn update_status(&self, pr_number: u32, status: MergeStatus) -> bool {
        let mut queue = self.queue.lock().unwrap();
        if let Some(entry) = queue.iter_mut().find(|e| e.pr_number == pr_number) {
            entry.status = status;
            match status {
                MergeStatus::Testing => entry.started_at = Some(Utc::now()),
                MergeStatus::Completed | MergeStatus::Failed => {
                    entry.completed_at = Some(Utc::now())
                }
                _ => {}
            }
            return true;
        }
        false
    }

    pub fn len(&self) -> usize {
        self.queue.lock().unwrap().len()
    }

    pub fn is_empty(&self) -> bool {
        self.queue.lock().unwrap().is_empty()
    }

    pub fn position_of(&self, pr_number: u32) -> Option<u32> {
        let queue = self.queue.lock().unwrap();
        queue
            .iter()
            .position(|e| e.pr_number == pr_number)
            .map(|p| p as u32)
    }

    pub fn clear_completed(&self) -> usize {
        let mut queue = self.queue.lock().unwrap();
        let before = queue.len();
        queue.retain(|e| {
            e.status != MergeStatus::Completed
                && e.status != MergeStatus::Failed
                && e.status != MergeStatus::Cancelled
        });
        before - queue.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_entry(pr: u32) -> MergeQueueEntry {
        MergeQueueEntry {
            id: format!("mq-{pr}"),
            pr_number: pr,
            head_sha: format!("sha{pr}"),
            base_branch: "main".to_string(),
            status: MergeStatus::Queued,
            position: 0,
            enqueued_at: Utc::now(),
            started_at: None,
            completed_at: None,
            error: None,
        }
    }

    #[test]
    fn test_enqueue_and_len() {
        let mq = MergeQueue::new(10);
        assert_eq!(mq.len(), 0);
        assert!(mq.is_empty());
        let pos = mq.enqueue(make_entry(1)).unwrap();
        assert_eq!(pos, 0);
        assert_eq!(mq.len(), 1);
        assert!(!mq.is_empty());
    }

    #[test]
    fn test_enqueue_full() {
        let mq = MergeQueue::new(1);
        mq.enqueue(make_entry(1)).unwrap();
        assert!(mq.enqueue(make_entry(2)).is_err());
    }

    #[test]
    fn test_dequeue_next() {
        let mq = MergeQueue::new(10);
        mq.enqueue(make_entry(1)).unwrap();
        mq.enqueue(make_entry(2)).unwrap();
        let first = mq.dequeue_next().unwrap();
        assert_eq!(first.pr_number, 1);
        assert_eq!(mq.len(), 1);
    }

    #[test]
    fn test_dequeue_empty() {
        let mq = MergeQueue::new(10);
        assert!(mq.dequeue_next().is_none());
    }

    #[test]
    fn test_peek() {
        let mq = MergeQueue::new(10);
        mq.enqueue(make_entry(1)).unwrap();
        mq.enqueue(make_entry(2)).unwrap();
        let peeked = mq.peek().unwrap();
        assert_eq!(peeked.pr_number, 1);
        assert_eq!(mq.len(), 2);
    }

    #[test]
    fn test_peek_empty() {
        let mq = MergeQueue::new(10);
        assert!(mq.peek().is_none());
    }

    #[test]
    fn test_cancel_queued() {
        let mq = MergeQueue::new(10);
        mq.enqueue(make_entry(1)).unwrap();
        assert!(mq.cancel(1));
        let entry = mq.position_of(1);
        assert!(entry.is_some());
    }

    #[test]
    fn test_cancel_testing() {
        let mq = MergeQueue::new(10);
        mq.enqueue(make_entry(1)).unwrap();
        mq.update_status(1, MergeStatus::Testing);
        assert!(mq.cancel(1));
    }

    #[test]
    fn test_cancel_completed_fails() {
        let mq = MergeQueue::new(10);
        mq.enqueue(make_entry(1)).unwrap();
        mq.update_status(1, MergeStatus::Completed);
        assert!(!mq.cancel(1));
    }

    #[test]
    fn test_cancel_nonexistent() {
        let mq = MergeQueue::new(10);
        assert!(!mq.cancel(999));
    }

    #[test]
    fn test_update_status_to_testing() {
        let mq = MergeQueue::new(10);
        mq.enqueue(make_entry(1)).unwrap();
        assert!(mq.update_status(1, MergeStatus::Testing));
        let entry = mq.peek().unwrap();
        assert_eq!(entry.status, MergeStatus::Testing);
        assert!(entry.started_at.is_some());
    }

    #[test]
    fn test_update_status_to_completed() {
        let mq = MergeQueue::new(10);
        mq.enqueue(make_entry(1)).unwrap();
        mq.update_status(1, MergeStatus::Testing);
        assert!(mq.update_status(1, MergeStatus::Completed));
        let entry = mq.peek().unwrap();
        assert_eq!(entry.status, MergeStatus::Completed);
        assert!(entry.completed_at.is_some());
    }

    #[test]
    fn test_update_status_to_failed() {
        let mq = MergeQueue::new(10);
        mq.enqueue(make_entry(1)).unwrap();
        assert!(mq.update_status(1, MergeStatus::Failed));
        let entry = mq.peek().unwrap();
        assert_eq!(entry.status, MergeStatus::Failed);
        assert!(entry.completed_at.is_some());
    }

    #[test]
    fn test_update_status_nonexistent() {
        let mq = MergeQueue::new(10);
        assert!(!mq.update_status(999, MergeStatus::Testing));
    }

    #[test]
    fn test_position_of() {
        let mq = MergeQueue::new(10);
        mq.enqueue(make_entry(1)).unwrap();
        mq.enqueue(make_entry(2)).unwrap();
        mq.enqueue(make_entry(3)).unwrap();
        assert_eq!(mq.position_of(1), Some(0));
        assert_eq!(mq.position_of(2), Some(1));
        assert_eq!(mq.position_of(3), Some(2));
        assert_eq!(mq.position_of(999), None);
    }

    #[test]
    fn test_clear_completed() {
        let mq = MergeQueue::new(10);
        mq.enqueue(make_entry(1)).unwrap();
        mq.enqueue(make_entry(2)).unwrap();
        mq.enqueue(make_entry(3)).unwrap();
        mq.update_status(1, MergeStatus::Completed);
        mq.update_status(2, MergeStatus::Failed);
        let cleared = mq.clear_completed();
        assert_eq!(cleared, 2);
        assert_eq!(mq.len(), 1);
    }

    #[test]
    fn test_clear_completed_includes_cancelled() {
        let mq = MergeQueue::new(10);
        mq.enqueue(make_entry(1)).unwrap();
        mq.enqueue(make_entry(2)).unwrap();
        mq.cancel(1);
        let cleared = mq.clear_completed();
        assert_eq!(cleared, 1);
        assert_eq!(mq.len(), 1);
    }

    #[test]
    fn test_entry_serialization_roundtrip() {
        let entry = make_entry(42);
        let json = serde_json::to_string(&entry).unwrap();
        let de: MergeQueueEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(de.pr_number, 42);
        assert_eq!(de.head_sha, "sha42");
        assert_eq!(de.base_branch, "main");
    }

    #[test]
    fn test_status_serialization_roundtrip() {
        assert_eq!(
            serde_json::to_string(&MergeStatus::Queued).unwrap(),
            "\"Queued\""
        );
        assert_eq!(
            serde_json::to_string(&MergeStatus::Completed).unwrap(),
            "\"Completed\""
        );
        let de: MergeStatus = serde_json::from_str("\"Failed\"").unwrap();
        assert_eq!(de, MergeStatus::Failed);
    }
}
