//! Environment Webhooks and Notifications types and logic.

#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentWebhookResponse {
    pub id: String,
    pub environment_id: String,
    pub url: String,
    pub events: Vec<String>,
    pub enabled: bool,
    pub created_at: String,
}

#[derive(Debug, sqlx::FromRow)]
pub struct EnvironmentWebhookRow {
    pub id: Uuid,
    pub environment_id: Uuid,
    pub url: String,
    pub events: Vec<String>,
    pub enabled: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl From<EnvironmentWebhookRow> for EnvironmentWebhookResponse {
    fn from(r: EnvironmentWebhookRow) -> Self {
        Self {
            id: r.id.to_string(),
            environment_id: r.environment_id.to_string(),
            url: r.url,
            events: r.events,
            enabled: r.enabled,
            created_at: r.created_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateWebhookRequest {
    pub url: String,
    #[serde(default)]
    pub events: Vec<String>,
    #[serde(default = "default_true")]
    pub enabled: bool,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Deserialize)]
pub struct UpdateWebhookRequest {
    pub url: Option<String>,
    pub events: Option<Vec<String>>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentNotificationResponse {
    pub id: String,
    pub environment_id: String,
    pub notification_type: String,
    pub config: serde_json::Value,
    pub enabled: bool,
    pub created_at: String,
}

#[derive(Debug, sqlx::FromRow)]
pub struct EnvironmentNotificationRow {
    pub id: Uuid,
    pub environment_id: Uuid,
    pub notification_type: String,
    pub config: serde_json::Value,
    pub enabled: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl From<EnvironmentNotificationRow> for EnvironmentNotificationResponse {
    fn from(r: EnvironmentNotificationRow) -> Self {
        Self {
            id: r.id.to_string(),
            environment_id: r.environment_id.to_string(),
            notification_type: r.notification_type,
            config: r.config,
            enabled: r.enabled,
            created_at: r.created_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateNotificationRequest {
    pub notification_type: String,
    #[serde(default)]
    pub config: Option<serde_json::Value>,
    #[serde(default = "default_true")]
    pub enabled: bool,
}

#[derive(Debug, Deserialize)]
pub struct UpdateNotificationRequest {
    pub notification_type: Option<String>,
    pub config: Option<serde_json::Value>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookDeliveryResponse {
    pub id: String,
    pub webhook_id: String,
    pub event: String,
    pub payload: serde_json::Value,
    pub status: String,
    pub response_status: Option<i32>,
    pub response_body: Option<String>,
    pub attempts: i32,
    pub max_attempts: i32,
    pub next_retry_at: Option<String>,
    pub created_at: String,
}

#[derive(Debug, sqlx::FromRow)]
pub struct WebhookDeliveryRow {
    pub id: Uuid,
    pub webhook_id: Uuid,
    pub event: String,
    pub payload: serde_json::Value,
    pub status: String,
    pub response_status: Option<i32>,
    pub response_body: Option<String>,
    pub attempts: i32,
    pub max_attempts: i32,
    pub next_retry_at: Option<chrono::DateTime<chrono::Utc>>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl From<WebhookDeliveryRow> for WebhookDeliveryResponse {
    fn from(r: WebhookDeliveryRow) -> Self {
        Self {
            id: r.id.to_string(),
            webhook_id: r.webhook_id.to_string(),
            event: r.event,
            payload: r.payload,
            status: r.status,
            response_status: r.response_status,
            response_body: r.response_body,
            attempts: r.attempts,
            max_attempts: r.max_attempts,
            next_retry_at: r.next_retry_at.map(|dt| dt.to_rfc3339()),
            created_at: r.created_at.to_rfc3339(),
        }
    }
}

// ---------------------------------------------------------------------------
// Webhook DB operations
// ---------------------------------------------------------------------------

/// Create a new environment webhook.
pub async fn create_webhook(
    pool: &sqlx::PgPool,
    environment_id: Uuid,
    url: &str,
    events: &[String],
    enabled: bool,
) -> std::result::Result<EnvironmentWebhookResponse, sqlx::Error> {
    sqlx::query_as::<_, EnvironmentWebhookRow>(
        "INSERT INTO environment_webhooks (environment_id, url, events, enabled) \
         VALUES ($1, $2, $3, $4) \
         RETURNING *",
    )
    .bind(environment_id)
    .bind(url)
    .bind(events)
    .bind(enabled)
    .fetch_one(pool)
    .await
    .map(|r| r.into())
}

/// Get a webhook by ID.
pub async fn get_webhook(
    pool: &sqlx::PgPool,
    webhook_id: Uuid,
) -> std::result::Result<Option<EnvironmentWebhookResponse>, sqlx::Error> {
    sqlx::query_as::<_, EnvironmentWebhookRow>(
        "SELECT * FROM environment_webhooks WHERE id = $1",
    )
    .bind(webhook_id)
    .fetch_optional(pool)
    .await
    .map(|r| r.map(|r| r.into()))
}

/// Update a webhook.
pub async fn update_webhook(
    pool: &sqlx::PgPool,
    webhook_id: Uuid,
    url: Option<&str>,
    events: Option<&[String]>,
    enabled: Option<bool>,
) -> std::result::Result<EnvironmentWebhookResponse, sqlx::Error> {
    sqlx::query_as::<_, EnvironmentWebhookRow>(
        "UPDATE environment_webhooks \
         SET url = COALESCE($2, url), \
             events = COALESCE($3, events), \
             enabled = COALESCE($4, enabled) \
         WHERE id = $1 \
         RETURNING *",
    )
    .bind(webhook_id)
    .bind(url)
    .bind(events)
    .bind(enabled)
    .fetch_one(pool)
    .await
    .map(|r| r.into())
}

/// Delete a webhook.
pub async fn delete_webhook(
    pool: &sqlx::PgPool,
    webhook_id: Uuid,
) -> std::result::Result<bool, sqlx::Error> {
    let result = sqlx::query("DELETE FROM environment_webhooks WHERE id = $1")
        .bind(webhook_id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

/// List webhooks for an environment.
pub async fn list_webhooks(
    pool: &sqlx::PgPool,
    environment_id: Uuid,
) -> std::result::Result<Vec<EnvironmentWebhookResponse>, sqlx::Error> {
    sqlx::query_as::<_, EnvironmentWebhookRow>(
        "SELECT * FROM environment_webhooks WHERE environment_id = $1 ORDER BY created_at DESC",
    )
    .bind(environment_id)
    .fetch_all(pool)
    .await
    .map(|rows| rows.into_iter().map(|r| r.into()).collect())
}

// ---------------------------------------------------------------------------
// Notification DB operations
// ---------------------------------------------------------------------------

/// Create a new notification config.
pub async fn create_notification(
    pool: &sqlx::PgPool,
    environment_id: Uuid,
    notification_type: &str,
    config: &serde_json::Value,
    enabled: bool,
) -> std::result::Result<EnvironmentNotificationResponse, sqlx::Error> {
    sqlx::query_as::<_, EnvironmentNotificationRow>(
        "INSERT INTO environment_notifications (environment_id, notification_type, config, enabled) \
         VALUES ($1, $2, $3, $4) \
         RETURNING *",
    )
    .bind(environment_id)
    .bind(notification_type)
    .bind(config)
    .bind(enabled)
    .fetch_one(pool)
    .await
    .map(|r| r.into())
}

/// Get a notification by ID.
pub async fn get_notification(
    pool: &sqlx::PgPool,
    notification_id: Uuid,
) -> std::result::Result<Option<EnvironmentNotificationResponse>, sqlx::Error> {
    sqlx::query_as::<_, EnvironmentNotificationRow>(
        "SELECT * FROM environment_notifications WHERE id = $1",
    )
    .bind(notification_id)
    .fetch_optional(pool)
    .await
    .map(|r| r.map(|r| r.into()))
}

/// Update a notification config.
pub async fn update_notification(
    pool: &sqlx::PgPool,
    notification_id: Uuid,
    notification_type: Option<&str>,
    config: Option<&serde_json::Value>,
    enabled: Option<bool>,
) -> std::result::Result<EnvironmentNotificationResponse, sqlx::Error> {
    sqlx::query_as::<_, EnvironmentNotificationRow>(
        "UPDATE environment_notifications \
         SET notification_type = COALESCE($2, notification_type), \
             config = COALESCE($3, config), \
             enabled = COALESCE($4, enabled) \
         WHERE id = $1 \
         RETURNING *",
    )
    .bind(notification_id)
    .bind(notification_type)
    .bind(config)
    .bind(enabled)
    .fetch_one(pool)
    .await
    .map(|r| r.into())
}

/// Delete a notification config.
pub async fn delete_notification(
    pool: &sqlx::PgPool,
    notification_id: Uuid,
) -> std::result::Result<bool, sqlx::Error> {
    let result = sqlx::query("DELETE FROM environment_notifications WHERE id = $1")
        .bind(notification_id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

/// List notifications for an environment.
pub async fn list_notifications(
    pool: &sqlx::PgPool,
    environment_id: Uuid,
) -> std::result::Result<Vec<EnvironmentNotificationResponse>, sqlx::Error> {
    sqlx::query_as::<_, EnvironmentNotificationRow>(
        "SELECT * FROM environment_notifications WHERE environment_id = $1 ORDER BY created_at DESC",
    )
    .bind(environment_id)
    .fetch_all(pool)
    .await
    .map(|rows| rows.into_iter().map(|r| r.into()).collect())
}

// ---------------------------------------------------------------------------
// Webhook Delivery operations
// ---------------------------------------------------------------------------

/// Record a webhook delivery attempt.
pub async fn record_delivery(
    pool: &sqlx::PgPool,
    webhook_id: Uuid,
    event: &str,
    payload: &serde_json::Value,
    status: &str,
    response_status: Option<i32>,
    response_body: Option<&str>,
) -> std::result::Result<WebhookDeliveryResponse, sqlx::Error> {
    sqlx::query_as::<_, WebhookDeliveryRow>(
        "INSERT INTO environment_webhook_deliveries \
         (webhook_id, event, payload, status, response_status, response_body) \
         VALUES ($1, $2, $3, $4, $5, $6) \
         RETURNING *",
    )
    .bind(webhook_id)
    .bind(event)
    .bind(payload)
    .bind(status)
    .bind(response_status)
    .bind(response_body)
    .fetch_one(pool)
    .await
    .map(|r| r.into())
}

/// List deliveries for a webhook.
pub async fn list_deliveries(
    pool: &sqlx::PgPool,
    webhook_id: Uuid,
    limit: i64,
    offset: i64,
) -> std::result::Result<Vec<WebhookDeliveryResponse>, sqlx::Error> {
    sqlx::query_as::<_, WebhookDeliveryRow>(
        "SELECT * FROM environment_webhook_deliveries \
         WHERE webhook_id = $1 \
         ORDER BY created_at DESC \
         LIMIT $2 OFFSET $3",
    )
    .bind(webhook_id)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await
    .map(|rows| rows.into_iter().map(|r| r.into()).collect())
}

/// Filter webhooks by event type.
pub async fn get_webhooks_for_event(
    pool: &sqlx::PgPool,
    environment_id: Uuid,
    event: &str,
) -> std::result::Result<Vec<EnvironmentWebhookResponse>, sqlx::Error> {
    sqlx::query_as::<_, EnvironmentWebhookRow>(
        "SELECT * FROM environment_webhooks \
         WHERE environment_id = $1 AND enabled = true AND $2 = ANY(events)",
    )
    .bind(environment_id)
    .bind(event)
    .fetch_all(pool)
    .await
    .map(|rows| rows.into_iter().map(|r| r.into()).collect())
}

/// Get delivery statistics for a webhook.
pub async fn get_delivery_stats(
    pool: &sqlx::PgPool,
    webhook_id: Uuid,
) -> std::result::Result<serde_json::Value, sqlx::Error> {
    let row: (i64, i64, i64) = sqlx::query_as(
        "SELECT \
            COUNT(*), \
            COUNT(*) FILTER (WHERE status = 'success'), \
            COUNT(*) FILTER (WHERE status = 'failed') \
         FROM environment_webhook_deliveries WHERE webhook_id = $1",
    )
    .bind(webhook_id)
    .fetch_one(pool)
    .await?;

    Ok(serde_json::json!({
        "webhook_id": webhook_id.to_string(),
        "total_deliveries": row.0,
        "successful": row.1,
        "failed": row.2
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_webhook_response_serialize() {
        let resp = EnvironmentWebhookResponse {
            id: "00000000-0000-0000-0000-000000000001".to_string(),
            environment_id: "00000000-0000-0000-0000-000000000002".to_string(),
            url: "https://example.com/webhook".to_string(),
            events: vec!["deployment.created".to_string()],
            enabled: true,
            created_at: "2025-01-01T00:00:00+00:00".to_string(),
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("example.com"));
    }

    #[test]
    fn test_notification_response_serialize() {
        let resp = EnvironmentNotificationResponse {
            id: "00000000-0000-0000-0000-000000000001".to_string(),
            environment_id: "00000000-0000-0000-0000-000000000002".to_string(),
            notification_type: "slack".to_string(),
            config: serde_json::json!({"channel": "#deployments"}),
            enabled: true,
            created_at: "2025-01-01T00:00:00+00:00".to_string(),
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("slack"));
    }

    #[test]
    fn test_create_webhook_request() {
        let json = r#"{"url": "https://example.com/hook", "events": ["deploy"], "enabled": true}"#;
        let req: CreateWebhookRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.url, "https://example.com/hook");
        assert!(req.enabled);
    }

    #[test]
    fn test_create_notification_request() {
        let json = r#"{"notification_type": "email", "config": {"to": "admin@example.com"}}"#;
        let req: CreateNotificationRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.notification_type, "email");
    }

    #[test]
    fn test_delivery_response_serialize() {
        let resp = WebhookDeliveryResponse {
            id: "00000000-0000-0000-0000-000000000001".to_string(),
            webhook_id: "00000000-0000-0000-0000-000000000002".to_string(),
            event: "deployment.created".to_string(),
            payload: serde_json::json!({"status": "success"}),
            status: "success".to_string(),
            response_status: Some(200),
            response_body: None,
            attempts: 1,
            max_attempts: 3,
            next_retry_at: None,
            created_at: "2025-01-01T00:00:00+00:00".to_string(),
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("deployment.created"));
    }
}
