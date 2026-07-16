#![forbid(unsafe_code)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct EventQueue {
    pub id: Uuid,
    pub queue_name: String,
    pub message_count: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct EventQueueMessage {
    pub id: Uuid,
    pub queue_id: Uuid,
    pub payload: serde_json::Value,
    pub status: String,
    pub attempts: i32,
    pub max_attempts: i32,
    pub created_at: DateTime<Utc>,
    pub processed_at: Option<DateTime<Utc>>,
}

pub struct EventQueueService {
    max_retries: i32,
}

impl Default for EventQueueService {
    fn default() -> Self {
        Self::new()
    }
}

impl EventQueueService {
    pub fn new() -> Self {
        Self { max_retries: 3 }
    }

    pub async fn create_queue(
        &self,
        pool: &sqlx::PgPool,
        queue_name: &str,
    ) -> Result<EventQueue, sqlx::Error> {
        let id = Uuid::new_v4();

        sqlx::query_as::<_, EventQueue>(
            "INSERT INTO event_queues (id, queue_name) \
             VALUES ($1, $2) \
             RETURNING id, queue_name, message_count, created_at",
        )
        .bind(id)
        .bind(queue_name)
        .fetch_one(pool)
        .await
    }

    pub async fn get_queue(
        &self,
        pool: &sqlx::PgPool,
        queue_name: &str,
    ) -> Result<Option<EventQueue>, sqlx::Error> {
        sqlx::query_as::<_, EventQueue>(
            "SELECT id, queue_name, message_count, created_at \
             FROM event_queues \
             WHERE queue_name = $1",
        )
        .bind(queue_name)
        .fetch_optional(pool)
        .await
    }

    pub async fn enqueue_message(
        &self,
        pool: &sqlx::PgPool,
        queue_name: &str,
        payload: serde_json::Value,
    ) -> Result<EventQueueMessage, sqlx::Error> {
        let queue = match self.get_queue(pool, queue_name).await? {
            Some(q) => q,
            None => self.create_queue(pool, queue_name).await?,
        };

        let message_id = Uuid::new_v4();

        sqlx::query_as::<_, EventQueueMessage>(
            "INSERT INTO event_queue_messages (id, queue_id, payload, status, attempts, max_attempts) \
             VALUES ($1, $2, $3, 'pending', 0, $4) \
             RETURNING id, queue_id, payload, status, attempts, max_attempts, created_at, processed_at",
        )
        .bind(message_id)
        .bind(queue.id)
        .bind(&payload)
        .bind(self.max_retries)
        .fetch_one(pool)
        .await
    }

    pub async fn dequeue_messages(
        &self,
        pool: &sqlx::PgPool,
        queue_name: &str,
        limit: i64,
    ) -> Result<Vec<EventQueueMessage>, sqlx::Error> {
        let queue = match self.get_queue(pool, queue_name).await? {
            Some(q) => q,
            None => return Ok(Vec::new()),
        };

        sqlx::query_as::<_, EventQueueMessage>(
            "SELECT id, queue_id, payload, status, attempts, max_attempts, created_at, processed_at \
             FROM event_queue_messages \
             WHERE queue_id = $1 AND status = 'pending' \
             ORDER BY created_at ASC \
             LIMIT $2",
        )
        .bind(queue.id)
        .bind(limit)
        .fetch_all(pool)
        .await
    }

    pub async fn complete_message(
        &self,
        pool: &sqlx::PgPool,
        message_id: &str,
    ) -> Result<bool, sqlx::Error> {
        let message_uuid = Uuid::parse_str(message_id).unwrap_or_default();

        let result = sqlx::query(
            "UPDATE event_queue_messages \
             SET status = 'completed', processed_at = NOW() \
             WHERE id = $1 AND status = 'pending'",
        )
        .bind(message_uuid)
        .execute(pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn fail_message(
        &self,
        pool: &sqlx::PgPool,
        message_id: &str,
        _error: &str,
    ) -> Result<bool, sqlx::Error> {
        let message_uuid = Uuid::parse_str(message_id).unwrap_or_default();

        let message = sqlx::query_as::<_, EventQueueMessage>(
            "SELECT id, queue_id, payload, status, attempts, max_attempts, created_at, processed_at \
             FROM event_queue_messages \
             WHERE id = $1",
        )
        .bind(message_uuid)
        .fetch_optional(pool)
        .await?;

        match message {
            Some(msg) => {
                let new_attempts = msg.attempts + 1;
                let new_status = if new_attempts >= msg.max_attempts {
                    "dead_letter"
                } else {
                    "retrying"
                };

                sqlx::query(
                    "UPDATE event_queue_messages \
                     SET status = $2, attempts = $3 \
                     WHERE id = $1",
                )
                .bind(message_uuid)
                .bind(new_status)
                .bind(new_attempts)
                .execute(pool)
                .await?;

                Ok(true)
            }
            None => Ok(false),
        }
    }

    pub async fn retry_messages(
        &self,
        pool: &sqlx::PgPool,
        queue_name: &str,
    ) -> Result<u32, sqlx::Error> {
        let queue = match self.get_queue(pool, queue_name).await? {
            Some(q) => q,
            None => return Ok(0),
        };

        let result = sqlx::query(
            "UPDATE event_queue_messages \
             SET status = 'pending' \
             WHERE queue_id = $1 AND status = 'retrying' \
             AND attempts < max_attempts",
        )
        .bind(queue.id)
        .execute(pool)
        .await?;

        Ok(result.rows_affected() as u32)
    }

    pub async fn get_queue_stats(
        &self,
        pool: &sqlx::PgPool,
        queue_name: &str,
    ) -> Result<QueueStats, sqlx::Error> {
        let queue = match self.get_queue(pool, queue_name).await? {
            Some(q) => q,
            None => {
                return Ok(QueueStats {
                    queue_name: queue_name.to_string(),
                    total_messages: 0,
                    pending_messages: 0,
                    completed_messages: 0,
                    failed_messages: 0,
                    dead_letter_messages: 0,
                });
            }
        };

        let total: (i64,) = sqlx::query_as(
            "SELECT COUNT(*)::bigint FROM event_queue_messages WHERE queue_id = $1",
        )
        .bind(queue.id)
        .fetch_one(pool)
        .await?;

        let pending: (i64,) = sqlx::query_as(
            "SELECT COUNT(*)::bigint FROM event_queue_messages WHERE queue_id = $1 AND status = 'pending'",
        )
        .bind(queue.id)
        .fetch_one(pool)
        .await?;

        let completed: (i64,) = sqlx::query_as(
            "SELECT COUNT(*)::bigint FROM event_queue_messages WHERE queue_id = $1 AND status = 'completed'",
        )
        .bind(queue.id)
        .fetch_one(pool)
        .await?;

        let failed: (i64,) = sqlx::query_as(
            "SELECT COUNT(*)::bigint FROM event_queue_messages WHERE queue_id = $1 AND status = 'failed'",
        )
        .bind(queue.id)
        .fetch_one(pool)
        .await?;

        let dead_letter: (i64,) = sqlx::query_as(
            "SELECT COUNT(*)::bigint FROM event_queue_messages WHERE queue_id = $1 AND status = 'dead_letter'",
        )
        .bind(queue.id)
        .fetch_one(pool)
        .await?;

        Ok(QueueStats {
            queue_name: queue_name.to_string(),
            total_messages: total.0,
            pending_messages: pending.0,
            completed_messages: completed.0,
            failed_messages: failed.0,
            dead_letter_messages: dead_letter.0,
        })
    }

    pub async fn start_worker(
        &self,
        pool: sqlx::PgPool,
        queue_name: String,
        handler: impl Fn(EventQueueMessage) -> std::pin::Pin<Box<dyn std::future::Future<Output = bool> + Send>> + Send + Sync + 'static,
    ) {
        let _max_retries = self.max_retries;

        tokio::spawn(async move {
            tracing::info!("Event queue worker started for queue: {}", queue_name);
            loop {
                tokio::time::sleep(Duration::from_secs(1)).await;

                let messages = match sqlx::query_as::<_, EventQueueMessage>(
                    "SELECT id, queue_id, payload, status, attempts, max_attempts, created_at, processed_at \
                     FROM event_queue_messages \
                     WHERE queue_id = (SELECT id FROM event_queues WHERE queue_name = $1) \
                     AND status = 'pending' \
                     ORDER BY created_at ASC \
                     LIMIT 10",
                )
                .bind(&queue_name)
                .fetch_all(&pool)
                .await
                {
                    Ok(msgs) => msgs,
                    Err(e) => {
                        tracing::error!("Failed to fetch messages from queue {}: {e}", queue_name);
                        continue;
                    }
                };

                for message in messages {
                    let success = handler(message.clone()).await;

                    if success {
                let _ = sqlx::query(
                    "UPDATE event_queue_messages \
                     SET status = 'completed', processed_at = NOW() \
                     WHERE id = $1",
                )
                .bind(message.id)
                .execute(&pool)
                .await;
                    } else {
                        let new_attempts = message.attempts + 1;
                        let new_status = if new_attempts >= message.max_attempts {
                            "dead_letter"
                        } else {
                            "retrying"
                        };

                        let _ = sqlx::query(
                            "UPDATE event_queue_messages \
                             SET status = $2, attempts = $3 \
                             WHERE id = $1",
                        )
                        .bind(message.id)
                        .bind(new_status)
                        .bind(new_attempts)
                        .execute(&pool)
                        .await;
                    }
                }
            }
        });
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueStats {
    pub queue_name: String,
    pub total_messages: i64,
    pub pending_messages: i64,
    pub completed_messages: i64,
    pub failed_messages: i64,
    pub dead_letter_messages: i64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_queue_creation() {
        let queue = EventQueue {
            id: Uuid::new_v4().to_string(),
            queue_name: "test_queue".to_string(),
            message_count: 0,
            created_at: Utc::now(),
        };

        assert_eq!(queue.queue_name, "test_queue");
        assert_eq!(queue.message_count, 0);
    }

    #[test]
    fn test_event_queue_message_creation() {
        let message = EventQueueMessage {
            id: Uuid::new_v4().to_string(),
            queue_id: Uuid::new_v4().to_string(),
            payload: serde_json::json!({"data": "test"}),
            status: "pending".to_string(),
            attempts: 0,
            max_attempts: 3,
            created_at: Utc::now(),
            processed_at: None,
        };

        assert_eq!(message.status, "pending");
        assert_eq!(message.attempts, 0);
        assert_eq!(message.max_attempts, 3);
        assert!(message.processed_at.is_none());
    }

    #[test]
    fn test_queue_service_new() {
        let service = EventQueueService::new();
        assert_eq!(service.max_retries, 3);
    }

    #[test]
    fn test_queue_stats_creation() {
        let stats = QueueStats {
            queue_name: "test".to_string(),
            total_messages: 100,
            pending_messages: 10,
            completed_messages: 80,
            failed_messages: 5,
            dead_letter_messages: 5,
        };

        assert_eq!(stats.total_messages, 100);
        assert_eq!(stats.pending_messages, 10);
        assert_eq!(stats.completed_messages, 80);
    }

    #[test]
    fn test_event_queue_serialization_roundtrip() {
        let queue = EventQueue {
            id: Uuid::new_v4().to_string(),
            queue_name: "test_queue".to_string(),
            message_count: 42,
            created_at: Utc::now(),
        };

        let json = serde_json::to_string(&queue).unwrap();
        let deserialized: EventQueue = serde_json::from_str(&json).unwrap();
        assert_eq!(queue.id, deserialized.id);
        assert_eq!(queue.queue_name, deserialized.queue_name);
        assert_eq!(queue.message_count, deserialized.message_count);
    }

    #[test]
    fn test_event_queue_message_serialization_roundtrip() {
        let message = EventQueueMessage {
            id: Uuid::new_v4().to_string(),
            queue_id: Uuid::new_v4().to_string(),
            payload: serde_json::json!({"key": "value"}),
            status: "pending".to_string(),
            attempts: 1,
            max_attempts: 3,
            created_at: Utc::now(),
            processed_at: Some(Utc::now()),
        };

        let json = serde_json::to_string(&message).unwrap();
        let deserialized: EventQueueMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(message.id, deserialized.id);
        assert_eq!(message.status, deserialized.status);
        assert!(deserialized.processed_at.is_some());
    }

    #[test]
    fn test_queue_stats_serialization_roundtrip() {
        let stats = QueueStats {
            queue_name: "test".to_string(),
            total_messages: 100,
            pending_messages: 10,
            completed_messages: 80,
            failed_messages: 5,
            dead_letter_messages: 5,
        };

        let json = serde_json::to_string(&stats).unwrap();
        let deserialized: QueueStats = serde_json::from_str(&json).unwrap();
        assert_eq!(stats.queue_name, deserialized.queue_name);
        assert_eq!(stats.total_messages, deserialized.total_messages);
    }
}