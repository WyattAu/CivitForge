#![forbid(unsafe_code)]

use chrono::{DateTime, Utc};
use lettre::{
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor, message::header::ContentType,
    transport::smtp::authentication::Credentials,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NotificationChannel {
    Email,
    Webhook,
    InApp,
    Slack,
    Mattermost,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum NotificationPriority {
    Low,
    Normal,
    High,
    Urgent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    pub id: String,
    pub channel: NotificationChannel,
    pub recipient: String,
    pub subject: String,
    pub body: String,
    pub priority: NotificationPriority,
    pub created_at: DateTime<Utc>,
    pub read: bool,
    pub read_at: Option<DateTime<Utc>>,
    pub metadata: Option<serde_json::Value>,
    pub dispatched: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationPreferences {
    pub user_id: String,
    pub enabled_channels: Vec<NotificationChannel>,
    pub muted_categories: Vec<String>,
    pub min_priority: NotificationPriority,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmtpConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub from_address: String,
    pub use_tls: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlackConfig {
    /// Bot OAuth token (starts with "xoxb-")
    pub bot_token: String,
    /// Default channel (e.g. "#alerts")
    pub default_channel: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NotificationChannelConfig {
    pub slack_webhook_url: Option<String>,
    pub slack: Option<SlackConfig>,
    pub mattermost_webhook_url: Option<String>,
    pub default_webhook_url: Option<String>,
    pub smtp: Option<SmtpConfig>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DispatchResult {
    pub notification_id: String,
    pub channel: NotificationChannel,
    pub success: bool,
    pub error: Option<String>,
}

pub struct NotificationService {
    notifications: std::sync::Mutex<Vec<Notification>>,
    preferences: std::sync::Mutex<Vec<NotificationPreferences>>,
    config: std::sync::Mutex<NotificationChannelConfig>,
}

impl NotificationService {
    pub fn new() -> Self {
        Self::with_config(NotificationChannelConfig::default())
    }

    pub fn with_config(config: NotificationChannelConfig) -> Self {
        Self {
            notifications: std::sync::Mutex::new(Vec::new()),
            preferences: std::sync::Mutex::new(Vec::new()),
            config: std::sync::Mutex::new(config),
        }
    }

    pub fn send(&self, notification: Notification) -> String {
        let id = notification.id.clone();
        self.notifications.lock().unwrap().push(notification);
        id
    }

    pub fn get_for_user(&self, user_id: &str, limit: usize) -> Vec<Notification> {
        let notifications = self.notifications.lock().unwrap();
        notifications
            .iter()
            .filter(|n| n.recipient == user_id)
            .rev()
            .take(limit)
            .cloned()
            .collect()
    }

    pub fn mark_read(&self, id: &str) -> bool {
        let mut notifications = self.notifications.lock().unwrap();
        if let Some(n) = notifications.iter_mut().find(|n| n.id == id) {
            n.read = true;
            n.read_at = Some(Utc::now());
            return true;
        }
        false
    }

    pub fn unread_count(&self, user_id: &str) -> usize {
        let notifications = self.notifications.lock().unwrap();
        notifications
            .iter()
            .filter(|n| n.recipient == user_id && !n.read)
            .count()
    }

    pub fn set_preferences(&self, prefs: NotificationPreferences) {
        let mut preferences = self.preferences.lock().unwrap();
        preferences.retain(|p| p.user_id != prefs.user_id);
        preferences.push(prefs);
    }

    pub fn get_preferences(&self, user_id: &str) -> Option<NotificationPreferences> {
        self.preferences
            .lock()
            .unwrap()
            .iter()
            .find(|p| p.user_id == user_id)
            .cloned()
    }

    pub fn count(&self) -> usize {
        self.notifications.lock().unwrap().len()
    }

    pub fn delete(&self, id: &str) -> bool {
        let mut notifications = self.notifications.lock().unwrap();
        let before = notifications.len();
        notifications.retain(|n| n.id != id);
        notifications.len() < before
    }

    pub fn set_config(&self, config: NotificationChannelConfig) {
        *self.config.lock().unwrap() = config;
    }

    pub fn get_config(&self) -> NotificationChannelConfig {
        self.config.lock().unwrap().clone()
    }

    pub fn pending_count(&self) -> usize {
        self.notifications
            .lock()
            .unwrap()
            .iter()
            .filter(|n| !n.dispatched)
            .count()
    }

    fn mark_dispatched(&self, id: &str) {
        if let Some(n) = self
            .notifications
            .lock()
            .unwrap()
            .iter_mut()
            .find(|n| n.id == id)
        {
            n.dispatched = true;
        }
    }

    fn resolve_webhook_url(
        notification: &Notification,
        config_url: Option<&String>,
        fallback_metadata_key: &str,
    ) -> Option<String> {
        config_url.cloned().or_else(|| {
            notification
                .metadata
                .as_ref()?
                .get(fallback_metadata_key)
                .and_then(|v| v.as_str().map(String::from))
        })
    }

    async fn dispatch_email(
        smtp_config: Option<SmtpConfig>,
        notification: &Notification,
    ) -> Option<String> {
        let Some(smtp) = smtp_config else {
            // No SMTP configured — log-only mode (backward compatible with stub behavior)
            tracing::info!(
                notification_id = %notification.id,
                recipient = %notification.recipient,
                subject = %notification.subject,
                "email dispatched (log-only, no SMTP configured)"
            );
            return None;
        };

        let from_addr = smtp
            .from_address
            .parse::<lettre::Address>()
            .ok()
            .unwrap_or_else(|| {
                lettre::Address::new(String::from("noreply"), String::from("localhost")).unwrap()
            });
        let to_addr = notification
            .recipient
            .parse::<lettre::Address>()
            .ok()
            .unwrap_or_else(|| {
                lettre::Address::new(String::from("unknown"), String::from("localhost")).unwrap()
            });

        let email = match Message::builder()
            .from(lettre::message::Mailbox::new(None, from_addr))
            .to(lettre::message::Mailbox::new(None, to_addr))
            .subject(&notification.subject)
            .header(ContentType::TEXT_PLAIN)
            .body(notification.body.clone())
        {
            Ok(m) => m,
            Err(e) => return Some(format!("email build error: {e}")),
        };

        let creds = Credentials::new(smtp.username.clone(), smtp.password.clone());

        let transport = if smtp.use_tls {
            match AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&smtp.host) {
                Ok(t) => t.port(smtp.port).credentials(creds).build(),
                Err(e) => return Some(format!("SMTP TLS relay error: {e}")),
            }
        } else {
            AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(&smtp.host)
                .port(smtp.port)
                .credentials(creds)
                .build()
        };

        match transport.send(email).await {
            Ok(_) => {
                tracing::info!(
                    notification_id = %notification.id,
                    recipient = %notification.recipient,
                    subject = %notification.subject,
                    "email dispatched via SMTP"
                );
                None
            }
            Err(e) => Some(format!(
                "SMTP send to {} failed: {e}",
                notification.recipient
            )),
        }
    }

    async fn post_slack(
        client: &reqwest::Client,
        slack_config: SlackConfig,
        notification: &Notification,
    ) -> Result<(), String> {
        let channel = notification
            .metadata
            .as_ref()
            .and_then(|m| m.get("slack_channel"))
            .and_then(|v| v.as_str())
            .unwrap_or(&slack_config.default_channel);

        let payload = serde_json::json!({
            "channel": channel,
            "text": format!("{}: {}", notification.subject, notification.body),
            "attachments": [{
                "title": notification.subject.clone(),
                "text": notification.body.clone(),
                "footer": format!("id: {}", notification.id),
                "color": match notification.priority {
                    NotificationPriority::Low => "#36a64f",
                    NotificationPriority::Normal => "#36a64f",
                    NotificationPriority::High => "#f2c744",
                    NotificationPriority::Urgent => "#e01e5a",
                },
            }]
        });

        let resp = client
            .post("https://slack.com/api/chat.postMessage")
            .header(
                "Authorization",
                format!("Bearer {}", slack_config.bot_token),
            )
            .json(&payload)
            .send()
            .await
            .map_err(|e| format!("Slack API request failed: {e}"))?;

        let status = resp.status();
        let body: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| format!("Slack API response parse failed: {e}"))?;

        if !status.is_success() || body["ok"].as_bool() != Some(true) {
            let err_msg = body["error"].as_str().unwrap_or("unknown error");
            return Err(format!("Slack API error: {err_msg}"));
        }
        Ok(())
    }

    async fn post_webhook(
        client: &reqwest::Client,
        url: &str,
        notification: &Notification,
    ) -> Result<(), String> {
        let payload = serde_json::json!({
            "text": format!("{}: {}", notification.subject, notification.body),
            "notification_id": notification.id,
        });

        client
            .post(url)
            .json(&payload)
            .send()
            .await
            .map_err(|e| format!("webhook POST to {url} failed: {e}"))
            .and_then(|resp| {
                if resp.status().is_success() {
                    Ok(())
                } else {
                    Err(format!(
                        "webhook at {url} returned status {}",
                        resp.status()
                    ))
                }
            })
    }

    pub async fn dispatch_single(&self, id: &str, client: &reqwest::Client) -> DispatchResult {
        let notification = {
            let notifications = self.notifications.lock().unwrap();
            notifications.iter().find(|n| n.id == id).cloned()
        };

        let Some(notification) = notification else {
            return DispatchResult {
                notification_id: id.to_string(),
                channel: NotificationChannel::InApp,
                success: false,
                error: Some("not found".to_string()),
            };
        };

        if notification.dispatched {
            return DispatchResult {
                notification_id: id.to_string(),
                channel: notification.channel,
                success: false,
                error: Some("already dispatched".to_string()),
            };
        }

        // Resolve all config before any async work to avoid MutexGuard across await
        let (smtp_config, slack_config, slack_webhook_url, mattermost_url, webhook_url) = {
            let config = self.config.lock().unwrap();
            let slack = Self::resolve_webhook_url(
                &notification,
                config.slack_webhook_url.as_ref(),
                "webhook_url",
            );
            let mattermost = Self::resolve_webhook_url(
                &notification,
                config.mattermost_webhook_url.as_ref(),
                "webhook_url",
            );
            let webhook = notification
                .metadata
                .as_ref()
                .and_then(|m| {
                    m.get("webhook_url")
                        .and_then(|v| v.as_str().map(String::from))
                })
                .or(config.default_webhook_url.clone());
            (
                config.smtp.clone(),
                config.slack.clone(),
                slack,
                mattermost,
                webhook,
            )
        };

        let error = match notification.channel {
            NotificationChannel::Email => Self::dispatch_email(smtp_config, &notification).await,
            NotificationChannel::InApp => {
                tracing::debug!(
                    notification_id = %id,
                    recipient = %notification.recipient,
                    "in-app notification considered dispatched"
                );
                None
            }
            NotificationChannel::Slack => {
                // Prefer Slack Web API (bot token) over legacy webhook URL
                if let Some(slack_cfg) = slack_config {
                    Self::post_slack(client, slack_cfg, &notification)
                        .await
                        .err()
                } else if let Some(url) = slack_webhook_url {
                    Self::post_webhook(client, &url, &notification).await.err()
                } else {
                    Some("no slack configuration (bot_token or webhook_url) available".to_string())
                }
            }
            NotificationChannel::Mattermost => match mattermost_url {
                Some(url) => Self::post_webhook(client, &url, &notification).await.err(),
                None => Some("no mattermost webhook URL configured".to_string()),
            },
            NotificationChannel::Webhook => match webhook_url {
                Some(url) => Self::post_webhook(client, &url, &notification).await.err(),
                None => Some("no webhook URL configured".to_string()),
            },
        };

        let success = error.is_none();
        if success {
            self.mark_dispatched(id);
        }

        DispatchResult {
            notification_id: id.to_string(),
            channel: notification.channel,
            success,
            error,
        }
    }

    pub async fn dispatch_pending(&self, client: &reqwest::Client) -> Vec<DispatchResult> {
        let ids: Vec<String> = {
            let notifications = self.notifications.lock().unwrap();
            notifications
                .iter()
                .filter(|n| !n.dispatched)
                .map(|n| n.id.clone())
                .collect()
        };

        let mut results = Vec::with_capacity(ids.len());
        for id in ids {
            results.push(self.dispatch_single(&id, client).await);
        }
        results
    }
}

impl Default for NotificationService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_notification(id: &str, recipient: &str, channel: NotificationChannel) -> Notification {
        Notification {
            id: id.to_string(),
            channel,
            recipient: recipient.to_string(),
            subject: format!("Subject {id}"),
            body: format!("Body {id}"),
            priority: NotificationPriority::Normal,
            created_at: Utc::now(),
            read: false,
            read_at: None,
            metadata: None,
            dispatched: false,
        }
    }

    #[test]
    fn test_send_notification() {
        let svc = NotificationService::new();
        let id = svc.send(make_notification(
            "n1",
            "user-1",
            NotificationChannel::InApp,
        ));
        assert_eq!(id, "n1");
        assert_eq!(svc.count(), 1);
    }

    #[test]
    fn test_get_for_user() {
        let svc = NotificationService::new();
        svc.send(make_notification(
            "n1",
            "user-1",
            NotificationChannel::Email,
        ));
        svc.send(make_notification(
            "n2",
            "user-1",
            NotificationChannel::InApp,
        ));
        svc.send(make_notification(
            "n3",
            "user-2",
            NotificationChannel::InApp,
        ));
        let results = svc.get_for_user("user-1", 10);
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_get_for_user_limit() {
        let svc = NotificationService::new();
        svc.send(make_notification(
            "n1",
            "user-1",
            NotificationChannel::InApp,
        ));
        svc.send(make_notification(
            "n2",
            "user-1",
            NotificationChannel::InApp,
        ));
        svc.send(make_notification(
            "n3",
            "user-1",
            NotificationChannel::InApp,
        ));
        let results = svc.get_for_user("user-1", 2);
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_get_for_user_returns_most_recent() {
        let svc = NotificationService::new();
        svc.send(make_notification(
            "n1",
            "user-1",
            NotificationChannel::InApp,
        ));
        svc.send(make_notification(
            "n2",
            "user-1",
            NotificationChannel::InApp,
        ));
        let results = svc.get_for_user("user-1", 1);
        assert_eq!(results[0].id, "n2");
    }

    #[test]
    fn test_mark_read() {
        let svc = NotificationService::new();
        svc.send(make_notification(
            "n1",
            "user-1",
            NotificationChannel::InApp,
        ));
        assert!(svc.mark_read("n1"));
        let notifications = svc.get_for_user("user-1", 10);
        assert!(notifications[0].read);
        assert!(notifications[0].read_at.is_some());
    }

    #[test]
    fn test_mark_read_nonexistent() {
        let svc = NotificationService::new();
        assert!(!svc.mark_read("nonexistent"));
    }

    #[test]
    fn test_unread_count() {
        let svc = NotificationService::new();
        svc.send(make_notification(
            "n1",
            "user-1",
            NotificationChannel::InApp,
        ));
        svc.send(make_notification(
            "n2",
            "user-1",
            NotificationChannel::InApp,
        ));
        assert_eq!(svc.unread_count("user-1"), 2);
        svc.mark_read("n1");
        assert_eq!(svc.unread_count("user-1"), 1);
    }

    #[test]
    fn test_unread_count_different_user() {
        let svc = NotificationService::new();
        svc.send(make_notification(
            "n1",
            "user-1",
            NotificationChannel::InApp,
        ));
        assert_eq!(svc.unread_count("user-2"), 0);
    }

    #[test]
    fn test_set_and_get_preferences() {
        let svc = NotificationService::new();
        let prefs = NotificationPreferences {
            user_id: "user-1".to_string(),
            enabled_channels: vec![NotificationChannel::Email, NotificationChannel::Slack],
            muted_categories: vec!["marketing".to_string()],
            min_priority: NotificationPriority::Normal,
        };
        svc.set_preferences(prefs);
        let retrieved = svc.get_preferences("user-1").unwrap();
        assert_eq!(retrieved.enabled_channels.len(), 2);
        assert_eq!(retrieved.min_priority, NotificationPriority::Normal);
    }

    #[test]
    fn test_set_preferences_overwrites() {
        let svc = NotificationService::new();
        svc.set_preferences(NotificationPreferences {
            user_id: "user-1".to_string(),
            enabled_channels: vec![NotificationChannel::Email],
            muted_categories: Vec::new(),
            min_priority: NotificationPriority::Low,
        });
        svc.set_preferences(NotificationPreferences {
            user_id: "user-1".to_string(),
            enabled_channels: vec![NotificationChannel::Slack],
            muted_categories: vec!["all".to_string()],
            min_priority: NotificationPriority::Urgent,
        });
        let prefs = svc.get_preferences("user-1").unwrap();
        assert_eq!(prefs.enabled_channels.len(), 1);
        assert_eq!(prefs.min_priority, NotificationPriority::Urgent);
    }

    #[test]
    fn test_get_preferences_nonexistent() {
        let svc = NotificationService::new();
        assert!(svc.get_preferences("nonexistent").is_none());
    }

    #[test]
    fn test_delete_notification() {
        let svc = NotificationService::new();
        svc.send(make_notification(
            "n1",
            "user-1",
            NotificationChannel::InApp,
        ));
        assert!(svc.delete("n1"));
        assert_eq!(svc.count(), 0);
    }

    #[test]
    fn test_delete_nonexistent() {
        let svc = NotificationService::new();
        assert!(!svc.delete("nonexistent"));
    }

    #[test]
    fn test_notification_serialization_roundtrip() {
        let notification = make_notification("n1", "user-1", NotificationChannel::Slack);
        let json = serde_json::to_string(&notification).unwrap();
        let de: Notification = serde_json::from_str(&json).unwrap();
        assert_eq!(de.id, "n1");
        assert_eq!(de.recipient, "user-1");
        assert_eq!(de.channel, NotificationChannel::Slack);
        assert!(!de.dispatched);
    }

    #[test]
    fn test_channel_serialization() {
        assert_eq!(
            serde_json::to_string(&NotificationChannel::Email).unwrap(),
            "\"Email\""
        );
        assert_eq!(
            serde_json::to_string(&NotificationChannel::Webhook).unwrap(),
            "\"Webhook\""
        );
        assert_eq!(
            serde_json::to_string(&NotificationChannel::InApp).unwrap(),
            "\"InApp\""
        );
        assert_eq!(
            serde_json::to_string(&NotificationChannel::Slack).unwrap(),
            "\"Slack\""
        );
        assert_eq!(
            serde_json::to_string(&NotificationChannel::Mattermost).unwrap(),
            "\"Mattermost\""
        );
    }

    #[test]
    fn test_priority_serialization() {
        assert_eq!(
            serde_json::to_string(&NotificationPriority::Low).unwrap(),
            "\"Low\""
        );
        assert_eq!(
            serde_json::to_string(&NotificationPriority::Normal).unwrap(),
            "\"Normal\""
        );
        assert_eq!(
            serde_json::to_string(&NotificationPriority::High).unwrap(),
            "\"High\""
        );
        assert_eq!(
            serde_json::to_string(&NotificationPriority::Urgent).unwrap(),
            "\"Urgent\""
        );
    }

    #[test]
    fn test_priority_ordering() {
        assert!(NotificationPriority::Low < NotificationPriority::Normal);
        assert!(NotificationPriority::Normal < NotificationPriority::High);
        assert!(NotificationPriority::High < NotificationPriority::Urgent);
    }

    #[test]
    fn test_preferences_serialization_roundtrip() {
        let prefs = NotificationPreferences {
            user_id: "user-1".to_string(),
            enabled_channels: vec![NotificationChannel::Email],
            muted_categories: vec!["noise".to_string()],
            min_priority: NotificationPriority::High,
        };
        let json = serde_json::to_string(&prefs).unwrap();
        let de: NotificationPreferences = serde_json::from_str(&json).unwrap();
        assert_eq!(de.user_id, "user-1");
        assert_eq!(de.min_priority, NotificationPriority::High);
    }

    #[test]
    fn test_notification_with_metadata() {
        let svc = NotificationService::new();
        let mut notif = make_notification("n1", "user-1", NotificationChannel::Webhook);
        notif.metadata = Some(serde_json::json!({"pr": 42, "action": "merged"}));
        svc.send(notif);
        let results = svc.get_for_user("user-1", 1);
        assert_eq!(results[0].metadata.as_ref().unwrap()["pr"], 42);
    }

    #[test]
    fn test_default_is_empty() {
        let svc = NotificationService::default();
        assert_eq!(svc.count(), 0);
    }

    #[tokio::test]
    async fn test_dispatch_single_email() {
        let svc = NotificationService::new();
        svc.send(make_notification(
            "e1",
            "user-1",
            NotificationChannel::Email,
        ));
        let client = reqwest::Client::new();
        let result = svc.dispatch_single("e1", &client).await;
        assert!(result.success);
        assert!(result.error.is_none());
        assert_eq!(result.notification_id, "e1");
        assert_eq!(result.channel, NotificationChannel::Email);
        assert_eq!(svc.pending_count(), 0);
        let notifs = svc.get_for_user("user-1", 10);
        assert!(notifs[0].dispatched);
    }

    #[tokio::test]
    async fn test_dispatch_single_inapp() {
        let svc = NotificationService::new();
        svc.send(make_notification(
            "i1",
            "user-1",
            NotificationChannel::InApp,
        ));
        let client = reqwest::Client::new();
        let result = svc.dispatch_single("i1", &client).await;
        assert!(result.success);
        assert!(result.error.is_none());
        assert_eq!(svc.pending_count(), 0);
    }

    #[tokio::test]
    async fn test_dispatch_single_slack_no_url() {
        let svc = NotificationService::new();
        svc.send(make_notification(
            "s1",
            "user-1",
            NotificationChannel::Slack,
        ));
        let client = reqwest::Client::new();
        let result = svc.dispatch_single("s1", &client).await;
        assert!(!result.success);
        assert_eq!(
            result.error.as_deref(),
            Some("no slack configuration (bot_token or webhook_url) available")
        );
        assert!(!svc.get_for_user("user-1", 10)[0].dispatched);
    }

    #[tokio::test]
    async fn test_dispatch_single_mattermost_no_url() {
        let svc = NotificationService::new();
        svc.send(make_notification(
            "m1",
            "user-1",
            NotificationChannel::Mattermost,
        ));
        let client = reqwest::Client::new();
        let result = svc.dispatch_single("m1", &client).await;
        assert!(!result.success);
        assert_eq!(
            result.error.as_deref(),
            Some("no mattermost webhook URL configured")
        );
    }

    #[tokio::test]
    async fn test_dispatch_single_webhook_no_url() {
        let svc = NotificationService::new();
        svc.send(make_notification(
            "w1",
            "user-1",
            NotificationChannel::Webhook,
        ));
        let client = reqwest::Client::new();
        let result = svc.dispatch_single("w1", &client).await;
        assert!(!result.success);
        assert_eq!(result.error.as_deref(), Some("no webhook URL configured"));
    }

    #[tokio::test]
    async fn test_dispatch_single_not_found() {
        let svc = NotificationService::new();
        let client = reqwest::Client::new();
        let result = svc.dispatch_single("missing", &client).await;
        assert!(!result.success);
        assert_eq!(result.error.as_deref(), Some("not found"));
    }

    #[tokio::test]
    async fn test_dispatch_single_already_dispatched() {
        let svc = NotificationService::new();
        svc.send(make_notification(
            "d1",
            "user-1",
            NotificationChannel::Email,
        ));
        let client = reqwest::Client::new();
        let r1 = svc.dispatch_single("d1", &client).await;
        assert!(r1.success);
        let r2 = svc.dispatch_single("d1", &client).await;
        assert!(!r2.success);
        assert_eq!(r2.error.as_deref(), Some("already dispatched"));
        assert_eq!(r2.channel, NotificationChannel::Email);
    }

    #[tokio::test]
    async fn test_dispatch_pending_multiple_channels() {
        let svc = NotificationService::new();
        svc.send(make_notification(
            "e1",
            "user-1",
            NotificationChannel::Email,
        ));
        svc.send(make_notification(
            "i1",
            "user-1",
            NotificationChannel::InApp,
        ));
        svc.send(make_notification(
            "s1",
            "user-1",
            NotificationChannel::Slack,
        ));
        assert_eq!(svc.pending_count(), 3);
        let client = reqwest::Client::new();
        let results = svc.dispatch_pending(&client).await;
        assert_eq!(results.len(), 3);
        let email = results.iter().find(|r| r.notification_id == "e1").unwrap();
        let inapp = results.iter().find(|r| r.notification_id == "i1").unwrap();
        let slack = results.iter().find(|r| r.notification_id == "s1").unwrap();
        assert!(email.success);
        assert!(inapp.success);
        assert!(!slack.success);
        assert_eq!(
            slack.error.as_deref(),
            Some("no slack configuration (bot_token or webhook_url) available")
        );
        assert_eq!(svc.pending_count(), 1);
    }

    #[tokio::test]
    async fn test_dispatch_pending_skips_already_dispatched() {
        let svc = NotificationService::new();
        svc.send(make_notification(
            "e1",
            "user-1",
            NotificationChannel::Email,
        ));
        let client = reqwest::Client::new();
        let r1 = svc.dispatch_pending(&client).await;
        assert_eq!(r1.len(), 1);
        assert!(r1[0].success);
        assert_eq!(svc.pending_count(), 0);
        let r2 = svc.dispatch_pending(&client).await;
        assert_eq!(r2.len(), 0);
    }

    #[tokio::test]
    async fn test_dispatch_single_webhook_url_from_metadata() {
        let svc = NotificationService::new();
        let mut notif = make_notification("w1", "user-1", NotificationChannel::Webhook);
        notif.metadata = Some(serde_json::json!({
            "webhook_url": "http://127.0.0.1:1/impossible"
        }));
        svc.send(notif);
        let client = reqwest::Client::new();
        let result = svc.dispatch_single("w1", &client).await;
        assert!(!result.success);
        assert!(result.error.as_ref().unwrap().contains("webhook POST"));
        assert!(
            result
                .error
                .as_ref()
                .unwrap()
                .contains("http://127.0.0.1:1/impossible")
        );
    }

    #[tokio::test]
    async fn test_dispatch_single_webhook_url_from_config() {
        let svc = NotificationService::with_config(NotificationChannelConfig {
            default_webhook_url: Some("http://127.0.0.1:1/fallback".to_string()),
            ..Default::default()
        });
        svc.send(make_notification(
            "w1",
            "user-1",
            NotificationChannel::Webhook,
        ));
        let client = reqwest::Client::new();
        let result = svc.dispatch_single("w1", &client).await;
        assert!(!result.success);
        assert!(
            result
                .error
                .as_ref()
                .unwrap()
                .contains("http://127.0.0.1:1/fallback")
        );
    }

    #[tokio::test]
    async fn test_dispatch_single_slack_url_from_config() {
        let svc = NotificationService::with_config(NotificationChannelConfig {
            slack_webhook_url: Some("http://127.0.0.1:1/slack".to_string()),
            ..Default::default()
        });
        svc.send(make_notification(
            "s1",
            "user-1",
            NotificationChannel::Slack,
        ));
        let client = reqwest::Client::new();
        let result = svc.dispatch_single("s1", &client).await;
        assert!(!result.success);
        assert!(
            result
                .error
                .as_ref()
                .unwrap()
                .contains("http://127.0.0.1:1/slack")
        );
    }

    #[tokio::test]
    async fn test_dispatch_single_mattermost_url_from_config() {
        let svc = NotificationService::with_config(NotificationChannelConfig {
            mattermost_webhook_url: Some("http://127.0.0.1:1/mm".to_string()),
            ..Default::default()
        });
        svc.send(make_notification(
            "m1",
            "user-1",
            NotificationChannel::Mattermost,
        ));
        let client = reqwest::Client::new();
        let result = svc.dispatch_single("m1", &client).await;
        assert!(!result.success);
        assert!(
            result
                .error
                .as_ref()
                .unwrap()
                .contains("http://127.0.0.1:1/mm")
        );
    }

    #[test]
    fn test_notification_dispatched_field_default_false() {
        let n = make_notification("n1", "user-1", NotificationChannel::Email);
        assert!(!n.dispatched);
    }

    #[test]
    fn test_pending_count_tracks_undispatched() {
        let svc = NotificationService::new();
        assert_eq!(svc.pending_count(), 0);
        svc.send(make_notification(
            "n1",
            "user-1",
            NotificationChannel::Email,
        ));
        assert_eq!(svc.pending_count(), 1);
    }

    #[test]
    fn test_channel_config_default() {
        let config = NotificationChannelConfig::default();
        assert!(config.slack_webhook_url.is_none());
        assert!(config.mattermost_webhook_url.is_none());
        assert!(config.default_webhook_url.is_none());
        assert!(config.smtp.is_none());
    }

    #[test]
    fn test_channel_config_roundtrip() {
        let config = NotificationChannelConfig {
            slack_webhook_url: Some("https://hooks.slack.com/xxx".to_string()),
            slack: None,
            mattermost_webhook_url: Some("https://mattermost.example.com/hooks/xxx".to_string()),
            default_webhook_url: Some("https://example.com/webhook".to_string()),
            smtp: Some(SmtpConfig {
                host: "smtp.example.com".to_string(),
                port: 587,
                username: "user".to_string(),
                password: "pass".to_string(),
                from_address: "noreply@example.com".to_string(),
                use_tls: true,
            }),
        };
        let json = serde_json::to_string(&config).unwrap();
        let de: NotificationChannelConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(
            de.slack_webhook_url.as_deref(),
            Some("https://hooks.slack.com/xxx")
        );
        assert_eq!(de.smtp.as_ref().unwrap().port, 587);
        assert!(de.smtp.as_ref().unwrap().use_tls);
    }

    #[test]
    fn test_slack_config_serialization() {
        let config = SlackConfig {
            bot_token: "xoxb-test-token".to_string(),
            default_channel: "#alerts".to_string(),
        };
        let json = serde_json::to_string(&config).unwrap();
        let de: SlackConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(de.bot_token, "xoxb-test-token");
        assert_eq!(de.default_channel, "#alerts");
    }

    #[test]
    fn test_smtp_config_serialization() {
        let config = SmtpConfig {
            host: "smtp.gmail.com".to_string(),
            port: 587,
            username: "user@example.com".to_string(),
            password: "secret".to_string(),
            from_address: "bot@example.com".to_string(),
            use_tls: true,
        };
        let json = serde_json::to_string(&config).unwrap();
        let de: SmtpConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(de.host, "smtp.gmail.com");
        assert_eq!(de.port, 587);
        assert!(de.use_tls);
    }

    #[test]
    fn test_channel_config_with_slack_api() {
        let config = NotificationChannelConfig {
            slack: Some(SlackConfig {
                bot_token: "xoxb-123".to_string(),
                default_channel: "#ops".to_string(),
            }),
            slack_webhook_url: Some("https://hooks.slack.com/old".to_string()),
            ..Default::default()
        };
        assert!(config.slack.is_some());
        assert_eq!(config.slack.as_ref().unwrap().default_channel, "#ops");
    }

    #[tokio::test]
    async fn test_dispatch_email_with_smtp_config_rejects_invalid_host() {
        let svc = NotificationService::with_config(NotificationChannelConfig {
            smtp: Some(SmtpConfig {
                host: "invalid-smtp-host-that-does-not-exist.invalid".to_string(),
                port: 587,
                username: "user".to_string(),
                password: "pass".to_string(),
                from_address: "noreply@example.com".to_string(),
                use_tls: true,
            }),
            ..Default::default()
        });
        let mut notif = make_notification("e-smtp", "user@example.com", NotificationChannel::Email);
        notif.subject = "Test SMTP".to_string();
        notif.body = "Test body".to_string();
        svc.send(notif);
        let client = reqwest::Client::new();
        let result = svc.dispatch_single("e-smtp", &client).await;
        // Should fail because the SMTP host doesn't exist
        assert!(!result.success);
        assert!(result.error.is_some());
        assert!(result.error.as_ref().unwrap().contains("SMTP"));
    }

    #[tokio::test]
    async fn test_dispatch_email_log_only_when_no_smtp_config() {
        let svc = NotificationService::new(); // no SMTP config
        svc.send(make_notification(
            "e-log",
            "user@example.com",
            NotificationChannel::Email,
        ));
        let client = reqwest::Client::new();
        let result = svc.dispatch_single("e-log", &client).await;
        // Without SMTP config, email dispatch succeeds in log-only mode
        assert!(result.success);
        assert!(result.error.is_none());
        assert_eq!(svc.pending_count(), 0);
    }

    #[tokio::test]
    async fn test_dispatch_slack_no_config() {
        let svc = NotificationService::new();
        svc.send(make_notification(
            "s-no-config",
            "user-1",
            NotificationChannel::Slack,
        ));
        let client = reqwest::Client::new();
        let result = svc.dispatch_single("s-no-config", &client).await;
        assert!(!result.success);
        assert!(
            result
                .error
                .as_ref()
                .unwrap()
                .contains("no slack configuration")
        );
    }

    #[tokio::test]
    async fn test_dispatch_result_equality() {
        let r1 = DispatchResult {
            notification_id: "n1".to_string(),
            channel: NotificationChannel::Email,
            success: true,
            error: None,
        };
        let r2 = DispatchResult {
            notification_id: "n1".to_string(),
            channel: NotificationChannel::Email,
            success: true,
            error: None,
        };
        assert_eq!(r1, r2);
    }

    #[test]
    fn test_notification_channel_hash() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(NotificationChannel::Email);
        set.insert(NotificationChannel::Slack);
        set.insert(NotificationChannel::Webhook);
        assert_eq!(set.len(), 3);
        // Inserting duplicate should not increase size
        set.insert(NotificationChannel::Email);
        assert_eq!(set.len(), 3);
    }

    #[test]
    fn test_multiple_smtp_configs_independent() {
        let svc1 = NotificationService::with_config(NotificationChannelConfig {
            smtp: Some(SmtpConfig {
                host: "smtp1.example.com".to_string(),
                port: 25,
                username: "u1".to_string(),
                password: "p1".to_string(),
                from_address: "from1@example.com".to_string(),
                use_tls: false,
            }),
            ..Default::default()
        });
        let svc2 = NotificationService::with_config(NotificationChannelConfig {
            smtp: Some(SmtpConfig {
                host: "smtp2.example.com".to_string(),
                port: 465,
                username: "u2".to_string(),
                password: "p2".to_string(),
                from_address: "from2@example.com".to_string(),
                use_tls: true,
            }),
            ..Default::default()
        });
        let cfg1 = svc1.get_config();
        let cfg2 = svc2.get_config();
        assert_eq!(cfg1.smtp.as_ref().unwrap().host, "smtp1.example.com");
        assert_eq!(cfg2.smtp.as_ref().unwrap().host, "smtp2.example.com");
        assert_eq!(cfg1.smtp.as_ref().unwrap().port, 25);
        assert_eq!(cfg2.smtp.as_ref().unwrap().port, 465);
    }
}
