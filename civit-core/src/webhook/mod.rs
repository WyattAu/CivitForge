#![forbid(unsafe_code)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Duration;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WebhookEvent {
    Push,
    PullRequest,
    PullRequestReview,
    Issue,
    IssueComment,
    Pipeline,
    PipelineRun,
    Repository,
    Release,
    Star,
    Fork,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeliveryStatus {
    Pending,
    Success,
    Failed,
    Retrying,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookEndpoint {
    pub id: String,
    pub url: String,
    pub secret: Option<String>,
    pub events: Vec<WebhookEvent>,
    pub active: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookDelivery {
    pub id: String,
    pub endpoint_id: String,
    pub event: WebhookEvent,
    pub payload: serde_json::Value,
    pub status: DeliveryStatus,
    pub response_code: Option<u16>,
    pub attempts: u32,
    pub max_retries: u32,
    pub next_retry: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub last_attempt: Option<DateTime<Utc>>,
}

pub struct WebhookService {
    endpoints: Mutex<HashMap<String, WebhookEndpoint>>,
    deliveries: Mutex<Vec<WebhookDelivery>>,
    max_retries: u32,
    retry_delay: Duration,
}

impl WebhookService {
    pub fn new(max_retries: u32, retry_delay: Duration) -> Self {
        Self {
            endpoints: Mutex::new(HashMap::new()),
            deliveries: Mutex::new(Vec::new()),
            max_retries,
            retry_delay,
        }
    }

    pub fn register_endpoint(&self, endpoint: WebhookEndpoint) {
        let mut endpoints = self.endpoints.lock().unwrap();
        endpoints.insert(endpoint.id.clone(), endpoint);
    }

    pub fn remove_endpoint(&self, id: &str) -> bool {
        let mut endpoints = self.endpoints.lock().unwrap();
        endpoints.remove(id).is_some()
    }

    pub fn trigger(&self, event: WebhookEvent, payload: serde_json::Value) -> Vec<WebhookDelivery> {
        let endpoints = self.endpoints.lock().unwrap();
        let matching: Vec<&WebhookEndpoint> = endpoints
            .values()
            .filter(|ep| ep.active && ep.events.contains(&event))
            .collect();

        let mut deliveries = Vec::new();
        for ep in matching {
            let delivery = WebhookDelivery {
                id: uuid::Uuid::new_v4().to_string(),
                endpoint_id: ep.id.clone(),
                event: event.clone(),
                payload: payload.clone(),
                status: DeliveryStatus::Pending,
                response_code: None,
                attempts: 0,
                max_retries: self.max_retries,
                next_retry: None,
                created_at: Utc::now(),
                last_attempt: None,
            };
            deliveries.push(delivery);
        }

        let mut stored = self.deliveries.lock().unwrap();
        for d in &deliveries {
            stored.push(d.clone());
        }

        deliveries
    }

    pub fn get_deliveries(&self, endpoint_id: &str) -> Vec<WebhookDelivery> {
        let deliveries = self.deliveries.lock().unwrap();
        deliveries
            .iter()
            .filter(|d| d.endpoint_id == endpoint_id)
            .cloned()
            .collect()
    }

    pub fn get_endpoint(&self, id: &str) -> Option<WebhookEndpoint> {
        self.endpoints.lock().unwrap().get(id).cloned()
    }

    pub fn endpoint_count(&self) -> usize {
        self.endpoints.lock().unwrap().len()
    }

    pub fn delivery_count(&self) -> usize {
        self.deliveries.lock().unwrap().len()
    }

    pub fn mark_success(&self, delivery_id: &str) -> bool {
        let mut deliveries = self.deliveries.lock().unwrap();
        if let Some(d) = deliveries.iter_mut().find(|d| d.id == delivery_id) {
            d.status = DeliveryStatus::Success;
            return true;
        }
        false
    }

    pub fn mark_failed(&self, delivery_id: &str) -> bool {
        let mut deliveries = self.deliveries.lock().unwrap();
        if let Some(d) = deliveries.iter_mut().find(|d| d.id == delivery_id) {
            d.status = DeliveryStatus::Failed;
            d.last_attempt = Some(Utc::now());
            d.attempts += 1;
            if d.attempts < d.max_retries {
                d.status = DeliveryStatus::Retrying;
                d.next_retry = Some(
                    Utc::now() + chrono::Duration::from_std(self.retry_delay).unwrap_or_default(),
                );
            }
            return true;
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_endpoint(id: &str, events: Vec<WebhookEvent>) -> WebhookEndpoint {
        WebhookEndpoint {
            id: id.to_string(),
            url: format!("https://example.com/webhook/{id}"),
            secret: None,
            events,
            active: true,
            created_at: Utc::now(),
        }
    }

    #[test]
    fn test_register_endpoint() {
        let svc = WebhookService::new(3, Duration::from_secs(60));
        svc.register_endpoint(make_endpoint("ep1", vec![WebhookEvent::Push]));
        assert_eq!(svc.endpoint_count(), 1);
        assert!(svc.get_endpoint("ep1").is_some());
    }

    #[test]
    fn test_remove_endpoint() {
        let svc = WebhookService::new(3, Duration::from_secs(60));
        svc.register_endpoint(make_endpoint("ep1", vec![WebhookEvent::Push]));
        assert!(svc.remove_endpoint("ep1"));
        assert!(!svc.remove_endpoint("ep1"));
        assert_eq!(svc.endpoint_count(), 0);
    }

    #[test]
    fn test_trigger_matching_event() {
        let svc = WebhookService::new(3, Duration::from_secs(60));
        svc.register_endpoint(make_endpoint("ep1", vec![WebhookEvent::Push]));
        let deliveries = svc.trigger(WebhookEvent::Push, serde_json::json!({"ref": "main"}));
        assert_eq!(deliveries.len(), 1);
        assert_eq!(deliveries[0].endpoint_id, "ep1");
        assert_eq!(deliveries[0].status, DeliveryStatus::Pending);
    }

    #[test]
    fn test_trigger_no_matching_endpoint() {
        let svc = WebhookService::new(3, Duration::from_secs(60));
        svc.register_endpoint(make_endpoint("ep1", vec![WebhookEvent::Push]));
        let deliveries = svc.trigger(WebhookEvent::Issue, serde_json::json!({}));
        assert!(deliveries.is_empty());
    }

    #[test]
    fn test_trigger_inactive_endpoint() {
        let svc = WebhookService::new(3, Duration::from_secs(60));
        let mut ep = make_endpoint("ep1", vec![WebhookEvent::Push]);
        ep.active = false;
        svc.register_endpoint(ep);
        let deliveries = svc.trigger(WebhookEvent::Push, serde_json::json!({}));
        assert!(deliveries.is_empty());
    }

    #[test]
    fn test_trigger_multiple_endpoints() {
        let svc = WebhookService::new(3, Duration::from_secs(60));
        svc.register_endpoint(make_endpoint(
            "ep1",
            vec![WebhookEvent::Push, WebhookEvent::Release],
        ));
        svc.register_endpoint(make_endpoint("ep2", vec![WebhookEvent::Push]));
        let deliveries = svc.trigger(WebhookEvent::Push, serde_json::json!({}));
        assert_eq!(deliveries.len(), 2);
    }

    #[test]
    fn test_delivery_counts() {
        let svc = WebhookService::new(3, Duration::from_secs(60));
        svc.register_endpoint(make_endpoint("ep1", vec![WebhookEvent::Push]));
        svc.trigger(WebhookEvent::Push, serde_json::json!({}));
        svc.trigger(WebhookEvent::Push, serde_json::json!({}));
        assert_eq!(svc.delivery_count(), 2);
    }

    #[test]
    fn test_get_deliveries_by_endpoint() {
        let svc = WebhookService::new(3, Duration::from_secs(60));
        svc.register_endpoint(make_endpoint("ep1", vec![WebhookEvent::Push]));
        svc.register_endpoint(make_endpoint("ep2", vec![WebhookEvent::Push]));
        svc.trigger(WebhookEvent::Push, serde_json::json!({}));
        assert_eq!(svc.get_deliveries("ep1").len(), 1);
        assert_eq!(svc.get_deliveries("ep2").len(), 1);
    }

    #[test]
    fn test_mark_success() {
        let svc = WebhookService::new(3, Duration::from_secs(60));
        svc.register_endpoint(make_endpoint("ep1", vec![WebhookEvent::Push]));
        let deliveries = svc.trigger(WebhookEvent::Push, serde_json::json!({}));
        let id = &deliveries[0].id;
        assert!(svc.mark_success(id));
        assert_eq!(svc.get_deliveries("ep1")[0].status, DeliveryStatus::Success);
    }

    #[test]
    fn test_mark_success_nonexistent() {
        let svc = WebhookService::new(3, Duration::from_secs(60));
        assert!(!svc.mark_success("nope"));
    }

    #[test]
    fn test_mark_failed_triggers_retry() {
        let svc = WebhookService::new(3, Duration::from_secs(60));
        svc.register_endpoint(make_endpoint("ep1", vec![WebhookEvent::Push]));
        let deliveries = svc.trigger(WebhookEvent::Push, serde_json::json!({}));
        let id = &deliveries[0].id;
        assert!(svc.mark_failed(id));
        let del = &svc.get_deliveries("ep1")[0];
        assert_eq!(del.status, DeliveryStatus::Retrying);
        assert_eq!(del.attempts, 1);
        assert!(del.next_retry.is_some());
    }

    #[test]
    fn test_mark_failed_exhausts_retries() {
        let svc = WebhookService::new(3, Duration::from_secs(60));
        svc.register_endpoint(make_endpoint("ep1", vec![WebhookEvent::Push]));
        let deliveries = svc.trigger(WebhookEvent::Push, serde_json::json!({}));
        let id = &deliveries[0].id;
        svc.mark_failed(id);
        svc.mark_failed(id);
        svc.mark_failed(id);
        let del = &svc.get_deliveries("ep1")[0];
        assert_eq!(del.status, DeliveryStatus::Failed);
    }

    #[test]
    fn test_webhook_event_hash() {
        let mut set = std::collections::HashSet::new();
        set.insert(WebhookEvent::Push);
        set.insert(WebhookEvent::Push);
        assert_eq!(set.len(), 1);
    }

    #[test]
    fn test_endpoint_with_secret() {
        let svc = WebhookService::new(3, Duration::from_secs(60));
        let ep = WebhookEndpoint {
            id: "ep1".to_string(),
            url: "https://example.com/hook".to_string(),
            secret: Some("s3cret".to_string()),
            events: vec![WebhookEvent::Push],
            active: true,
            created_at: Utc::now(),
        };
        svc.register_endpoint(ep);
        let retrieved = svc.get_endpoint("ep1").unwrap();
        assert_eq!(retrieved.secret.as_deref(), Some("s3cret"));
    }

    #[test]
    fn test_delivery_id_unique() {
        let svc = WebhookService::new(3, Duration::from_secs(60));
        svc.register_endpoint(make_endpoint("ep1", vec![WebhookEvent::Push]));
        let d1 = svc.trigger(WebhookEvent::Push, serde_json::json!({}));
        let d2 = svc.trigger(WebhookEvent::Push, serde_json::json!({}));
        assert_ne!(d1[0].id, d2[0].id);
    }

    #[test]
    fn test_all_webhook_events_serializable() {
        let events = vec![
            WebhookEvent::Push,
            WebhookEvent::PullRequest,
            WebhookEvent::PullRequestReview,
            WebhookEvent::Issue,
            WebhookEvent::IssueComment,
            WebhookEvent::Pipeline,
            WebhookEvent::PipelineRun,
            WebhookEvent::Repository,
            WebhookEvent::Release,
            WebhookEvent::Star,
            WebhookEvent::Fork,
        ];
        for event in &events {
            let json = serde_json::to_string(event).unwrap();
            let back: WebhookEvent = serde_json::from_str(&json).unwrap();
            assert_eq!(*event, back);
        }
    }

    #[test]
    fn test_endpoint_count_empty() {
        let svc = WebhookService::new(3, Duration::from_secs(60));
        assert_eq!(svc.endpoint_count(), 0);
    }
}
