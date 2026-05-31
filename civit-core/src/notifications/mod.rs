#![forbid(unsafe_code)]

use chrono::{DateTime, Utc};
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationPreferences {
    pub user_id: String,
    pub enabled_channels: Vec<NotificationChannel>,
    pub muted_categories: Vec<String>,
    pub min_priority: NotificationPriority,
}

pub struct NotificationService {
    notifications: std::sync::Mutex<Vec<Notification>>,
    preferences: std::sync::Mutex<Vec<NotificationPreferences>>,
}

impl NotificationService {
    pub fn new() -> Self {
        Self {
            notifications: std::sync::Mutex::new(Vec::new()),
            preferences: std::sync::Mutex::new(Vec::new()),
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
}
