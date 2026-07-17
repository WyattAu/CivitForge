#![forbid(unsafe_code)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashSet, VecDeque};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FederatedActivity {
    pub id: String,
    #[serde(rename = "type")]
    pub type_: String,
    pub actor: String,
    pub object: Option<String>,
    pub target: Option<String>,
    #[serde(default)]
    pub to: Vec<String>,
    #[serde(default)]
    pub cc: Vec<String>,
    pub published: Option<DateTime<Utc>>,
    pub raw_json: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DeliveryStatus {
    Pending,
    InFlight,
    Delivered,
    Failed { error: String },
    PermanentFailure { error: String },
}

#[derive(Debug, Clone, PartialEq)]
pub struct InboxEntry {
    pub activity: FederatedActivity,
    pub received_at: DateTime<Utc>,
    pub processed_at: Option<DateTime<Utc>>,
    pub processing_result: Option<ProcessingResult>,
    pub retry_count: u32,
    pub idempotency_key: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ProcessingResult {
    Success,
    Failed(String),
    Skipped(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct OutboxEntry {
    pub activity: FederatedActivity,
    pub target_instance: String,
    pub delivery_status: DeliveryStatus,
    pub attempts: u32,
    pub last_attempt: Option<DateTime<Utc>>,
    pub next_retry: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BackoffStrategy {
    Exponential { base_ms: u64, max_ms: u64 },
    Fixed { interval_ms: u64 },
}

impl Default for BackoffStrategy {
    fn default() -> Self {
        BackoffStrategy::Exponential {
            base_ms: 1000,
            max_ms: 300_000,
        }
    }
}

impl BackoffStrategy {
    pub fn compute_delay(&self, attempt: u32) -> chrono::Duration {
        match self {
            BackoffStrategy::Exponential { base_ms, max_ms } => {
                let delay_ms = (*base_ms as f64) * 2f64.powi(attempt as i32);
                let capped = delay_ms.min(*max_ms as f64);
                chrono::Duration::milliseconds(capped as i64)
            }
            BackoffStrategy::Fixed { interval_ms } => {
                chrono::Duration::milliseconds(*interval_ms as i64)
            }
        }
    }

    pub fn compute_delay_with_jitter(&self, attempt: u32) -> chrono::Duration {
        let base = self.compute_delay(attempt);
        let jitter_ms = (base.num_milliseconds() as f64 * 0.25) as i64;
        let base_ms = base.num_milliseconds();
        let rand_ms = if jitter_ms > 0 {
            ((attempt as i64 * 7919 + 104729) % (2 * jitter_ms + 1)) - jitter_ms
        } else {
            0
        };
        let final_ms = (base_ms + rand_ms).max(0);
        chrono::Duration::milliseconds(final_ms)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct InboxProcessor {
    inbox: VecDeque<InboxEntry>,
    processed: HashSet<String>,
    max_retries: u32,
    pending_idempotency: HashSet<String>,
}

impl Default for InboxProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl InboxProcessor {
    pub fn new() -> Self {
        Self {
            inbox: VecDeque::new(),
            processed: HashSet::new(),
            max_retries: 3,
            pending_idempotency: HashSet::new(),
        }
    }

    pub fn with_max_retries(max_retries: u32) -> Self {
        Self {
            inbox: VecDeque::new(),
            processed: HashSet::new(),
            max_retries,
            pending_idempotency: HashSet::new(),
        }
    }

    pub fn receive(&mut self, activity: FederatedActivity, idempotency_key: String) -> bool {
        if self.pending_idempotency.contains(&idempotency_key)
            || self.processed.contains(&idempotency_key)
        {
            return false;
        }

        self.pending_idempotency.insert(idempotency_key.clone());

        let entry = InboxEntry {
            activity,
            received_at: Utc::now(),
            processed_at: None,
            processing_result: None,
            retry_count: 0,
            idempotency_key,
        };

        self.inbox.push_back(entry);
        true
    }

    pub fn idempotency_check(&self, key: &str) -> bool {
        self.pending_idempotency.contains(key) || self.processed.contains(key)
    }

    pub fn process_next<F>(&mut self, mut handler: F) -> Option<&InboxEntry>
    where
        F: FnMut(&FederatedActivity) -> ProcessingResult,
    {
        let entry = self.inbox.front_mut()?;

        if entry.retry_count > self.max_retries {
            entry.processing_result = Some(ProcessingResult::Failed("max retries exceeded".into()));
            entry.processed_at = Some(Utc::now());
            let key = entry.idempotency_key.clone();
            self.processed.insert(key);
            let _ = self.inbox.pop_front();
            return None;
        }

        let result = handler(&entry.activity);
        entry.processing_result = Some(result.clone());
        entry.retry_count += 1;

        match result {
            ProcessingResult::Success => {
                entry.processed_at = Some(Utc::now());
                let key = entry.idempotency_key.clone();
                self.pending_idempotency.remove(&key);
                self.processed.insert(key);
                let _ = self.inbox.pop_front();
                None
            }
            ProcessingResult::Failed(_) => {
                if entry.retry_count >= self.max_retries {
                    entry.processed_at = Some(Utc::now());
                    let key = entry.idempotency_key.clone();
                    self.pending_idempotency.remove(&key);
                    self.processed.insert(key);
                    let _ = self.inbox.pop_front();
                }
                None
            }
            ProcessingResult::Skipped(_) => {
                entry.processed_at = Some(Utc::now());
                let key = entry.idempotency_key.clone();
                self.pending_idempotency.remove(&key);
                self.processed.insert(key);
                let _ = self.inbox.pop_front();
                None
            }
        }
    }

    pub fn retry_failed(&mut self) -> u32 {
        let mut count = 0u32;
        let len = self.inbox.len();
        for _ in 0..len {
            if let Some(mut entry) = self.inbox.pop_front() {
                let is_failed =
                    matches!(entry.processing_result, Some(ProcessingResult::Failed(_)));
                if is_failed && entry.retry_count < self.max_retries {
                    entry.processing_result = None;
                    count += 1;
                }
                self.inbox.push_back(entry);
            }
        }
        count
    }

    pub fn pending_count(&self) -> usize {
        self.inbox.len()
    }

    pub fn processed_count(&self) -> usize {
        self.processed.len()
    }

    pub fn is_empty(&self) -> bool {
        self.inbox.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OutboxProcessor {
    outbox: VecDeque<OutboxEntry>,
    delivered: HashSet<String>,
    backoff_strategy: BackoffStrategy,
}

impl Default for OutboxProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl OutboxProcessor {
    pub fn new() -> Self {
        Self {
            outbox: VecDeque::new(),
            delivered: HashSet::new(),
            backoff_strategy: BackoffStrategy::default(),
        }
    }

    pub fn with_backoff(backoff_strategy: BackoffStrategy) -> Self {
        Self {
            outbox: VecDeque::new(),
            delivered: HashSet::new(),
            backoff_strategy,
        }
    }

    pub fn enqueue(&mut self, activity: FederatedActivity, target_instance: String) -> String {
        let entry_id = format!("{}:{}", activity.id, target_instance);
        let entry = OutboxEntry {
            activity,
            target_instance,
            delivery_status: DeliveryStatus::Pending,
            attempts: 0,
            last_attempt: None,
            next_retry: None,
        };
        self.outbox.push_back(entry);
        entry_id
    }

    pub fn mark_delivered(&mut self, activity_id: &str, target: &str) -> bool {
        let entry_id = format!("{activity_id}:{target}");
        if self.delivered.contains(&entry_id) {
            return false;
        }
        self.delivered.insert(entry_id.clone());
        for entry in &mut self.outbox {
            if entry.activity.id == activity_id && entry.target_instance == target {
                entry.delivery_status = DeliveryStatus::Delivered;
                return true;
            }
        }
        false
    }

    pub fn mark_failed(&mut self, activity_id: &str, target: &str, permanent: bool) -> bool {
        for entry in &mut self.outbox {
            if entry.activity.id == activity_id && entry.target_instance == target {
                entry.attempts += 1;
                entry.last_attempt = Some(Utc::now());
                entry.delivery_status = if permanent {
                    DeliveryStatus::PermanentFailure {
                        error: "permanent failure".into(),
                    }
                } else {
                    let delay = self.backoff_strategy.compute_delay(entry.attempts);
                    entry.next_retry = Some(Utc::now() + delay);
                    DeliveryStatus::Failed {
                        error: "delivery failed".into(),
                    }
                };
                return true;
            }
        }
        false
    }

    pub fn mark_in_flight(&mut self, activity_id: &str, target: &str) -> bool {
        for entry in &mut self.outbox {
            if entry.activity.id == activity_id && entry.target_instance == target {
                entry.delivery_status = DeliveryStatus::InFlight;
                return true;
            }
        }
        false
    }

    pub fn retry_ready(&mut self) -> Vec<&OutboxEntry> {
        let now = Utc::now();
        self.outbox
            .iter()
            .filter(|e| {
                matches!(e.delivery_status, DeliveryStatus::Failed { .. })
                    && e.next_retry.is_some_and(|t| t <= now)
            })
            .collect()
    }

    pub fn pending_count(&self) -> usize {
        self.outbox
            .iter()
            .filter(|e| matches!(e.delivery_status, DeliveryStatus::Pending))
            .count()
    }

    pub fn in_flight_count(&self) -> usize {
        self.outbox
            .iter()
            .filter(|e| matches!(e.delivery_status, DeliveryStatus::InFlight))
            .count()
    }

    pub fn delivered_count(&self) -> usize {
        self.delivered.len()
    }

    pub fn is_empty(&self) -> bool {
        self.outbox.is_empty()
    }

    pub fn entry_count(&self) -> usize {
        self.outbox.len()
    }

    pub fn drain_pending(&mut self, limit: usize) -> Vec<(String, String)> {
        self.outbox
            .iter()
            .filter(|e| matches!(e.delivery_status, DeliveryStatus::Pending))
            .map(|e| (e.activity.id.clone(), e.target_instance.clone()))
            .take(limit)
            .collect()
    }

    pub fn drain_retry_ready(&mut self, limit: usize) -> Vec<(String, String)> {
        let ready: Vec<(String, String)> = self
            .retry_ready()
            .iter()
            .map(|e| (e.activity.id.clone(), e.target_instance.clone()))
            .take(limit)
            .collect();
        ready
    }

    pub fn get_activity_json(&self, activity_id: &str) -> Option<String> {
        self.outbox
            .iter()
            .find(|e| e.activity.id == activity_id)
            .map(|e| e.activity.raw_json.clone())
    }
}

#[cfg(test)]
fn make_activity(id: &str, type_: &str, actor: &str) -> FederatedActivity {
    FederatedActivity {
        id: id.to_string(),
        type_: type_.to_string(),
        actor: actor.to_string(),
        object: None,
        target: None,
        to: vec![],
        cc: vec![],
        published: None,
        raw_json: serde_json::to_string(
            &serde_json::json!({"id": id, "type": type_, "actor": actor}),
        )
        .unwrap(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_federated_activity_creation() {
        let act = make_activity("123", "Create", "actor1");
        assert_eq!(act.id, "123");
        assert_eq!(act.type_, "Create");
        assert_eq!(act.actor, "actor1");
        assert!(act.object.is_none());
        assert!(act.to.is_empty());
    }

    #[test]
    fn test_federated_activity_with_fields() {
        let act = FederatedActivity {
            id: "a1".into(),
            type_: "Like".into(),
            actor: "user1".into(),
            object: Some("https://example.com/note/1".into()),
            target: Some("https://other.example.com/inbox".into()),
            to: vec!["https://www.w3.org/ns/activitystreams#Public".into()],
            cc: vec!["https://example.com/followers".into()],
            published: Some(Utc::now()),
            raw_json: "{}".into(),
        };
        assert_eq!(act.object.as_deref(), Some("https://example.com/note/1"));
        assert_eq!(act.to.len(), 1);
        assert_eq!(act.cc.len(), 1);
    }

    #[test]
    fn test_inbox_receive() {
        let mut processor = InboxProcessor::new();
        let act = make_activity("1", "Create", "actor1");
        assert!(processor.receive(act, "key1".into()));
        assert_eq!(processor.pending_count(), 1);
    }

    #[test]
    fn test_inbox_receive_idempotent() {
        let mut processor = InboxProcessor::new();
        let act1 = make_activity("1", "Create", "actor1");
        let act2 = make_activity("1", "Create", "actor1");
        assert!(processor.receive(act1, "dup-key".into()));
        assert!(!processor.receive(act2, "dup-key".into()));
        assert_eq!(processor.pending_count(), 1);
    }

    #[test]
    fn test_inbox_idempotency_check() {
        let mut processor = InboxProcessor::new();
        let act = make_activity("1", "Create", "actor1");
        assert!(!processor.idempotency_check("key1"));
        processor.receive(act, "key1".into());
        assert!(processor.idempotency_check("key1"));
    }

    #[test]
    fn test_inbox_process_next_success() {
        let mut processor = InboxProcessor::new();
        processor.receive(make_activity("1", "Create", "actor1"), "k1".into());
        processor.process_next(|_| ProcessingResult::Success);
        assert!(processor.is_empty());
        assert_eq!(processor.processed_count(), 1);
    }

    #[test]
    fn test_inbox_process_next_failed() {
        let mut processor = InboxProcessor::with_max_retries(3);
        processor.receive(make_activity("1", "Create", "actor1"), "k1".into());
        processor.process_next(|_| ProcessingResult::Failed("err".into()));
        assert_eq!(processor.pending_count(), 1);
        assert_eq!(processor.processed_count(), 0);
    }

    #[test]
    fn test_inbox_process_max_retries() {
        let mut processor = InboxProcessor::with_max_retries(2);
        processor.receive(make_activity("1", "Create", "actor1"), "k1".into());
        processor.process_next(|_| ProcessingResult::Failed("err".into()));
        processor.process_next(|_| ProcessingResult::Failed("err".into()));
        assert!(processor.is_empty());
        assert_eq!(processor.processed_count(), 1);
    }

    #[test]
    fn test_inbox_retry_failed() {
        let mut processor = InboxProcessor::with_max_retries(3);
        processor.receive(make_activity("1", "Create", "actor1"), "k1".into());
        processor.receive(make_activity("2", "Create", "actor2"), "k2".into());
        processor.process_next(|_| ProcessingResult::Success);
        processor.process_next(|_| ProcessingResult::Failed("err".into()));
        let retried = processor.retry_failed();
        assert_eq!(retried, 1);
    }

    #[test]
    fn test_inbox_process_skipped() {
        let mut processor = InboxProcessor::new();
        processor.receive(make_activity("1", "Create", "actor1"), "k1".into());
        processor.process_next(|_| ProcessingResult::Skipped("already known".into()));
        assert!(processor.is_empty());
        assert_eq!(processor.processed_count(), 1);
    }

    #[test]
    fn test_inbox_process_empty() {
        let mut processor = InboxProcessor::new();
        let result = processor.process_next(|_| ProcessingResult::Success);
        assert!(result.is_none());
    }

    #[test]
    fn test_inbox_multiple_activities() {
        let mut processor = InboxProcessor::new();
        processor.receive(make_activity("1", "Create", "a1"), "k1".into());
        processor.receive(make_activity("2", "Delete", "a2"), "k2".into());
        processor.receive(make_activity("3", "Like", "a3"), "k3".into());
        assert_eq!(processor.pending_count(), 3);
        processor.process_next(|_| ProcessingResult::Success);
        assert_eq!(processor.pending_count(), 2);
        processor.process_next(|_| ProcessingResult::Success);
        assert_eq!(processor.pending_count(), 1);
        processor.process_next(|_| ProcessingResult::Success);
        assert!(processor.is_empty());
    }

    #[test]
    fn test_outbox_enqueue() {
        let mut processor = OutboxProcessor::new();
        let act = make_activity("1", "Create", "actor1");
        let id = processor.enqueue(act, "https://remote.example.com".into());
        assert_eq!(id, "1:https://remote.example.com");
        assert_eq!(processor.pending_count(), 1);
    }

    #[test]
    fn test_outbox_mark_delivered() {
        let mut processor = OutboxProcessor::new();
        processor.enqueue(make_activity("1", "Create", "actor1"), "remote1".into());
        processor.enqueue(make_activity("2", "Like", "actor2"), "remote2".into());
        assert!(processor.mark_delivered("1", "remote1"));
        assert_eq!(processor.delivered_count(), 1);
        assert!(!processor.mark_delivered("1", "remote1"));
        assert_eq!(processor.delivered_count(), 1);
    }

    #[test]
    fn test_outbox_mark_failed() {
        let mut processor = OutboxProcessor::new();
        processor.enqueue(make_activity("1", "Create", "actor1"), "remote1".into());
        assert!(processor.mark_failed("1", "remote1", false));
        assert_eq!(processor.pending_count(), 0);
    }

    #[test]
    fn test_outbox_mark_permanent_failure() {
        let mut processor = OutboxProcessor::new();
        processor.enqueue(make_activity("1", "Create", "actor1"), "remote1".into());
        assert!(processor.mark_failed("1", "remote1", true));
        let ready = processor.retry_ready();
        assert!(ready.is_empty());
    }

    #[test]
    fn test_outbox_mark_in_flight() {
        let mut processor = OutboxProcessor::new();
        processor.enqueue(make_activity("1", "Create", "actor1"), "remote1".into());
        assert!(processor.mark_in_flight("1", "remote1"));
        assert_eq!(processor.in_flight_count(), 1);
        assert!(!processor.mark_in_flight("nonexistent", "remote1"));
    }

    #[test]
    fn test_outbox_backoff_exponential() {
        let strategy = BackoffStrategy::Exponential {
            base_ms: 1000,
            max_ms: 60_000,
        };
        let d0 = strategy.compute_delay(0);
        let d1 = strategy.compute_delay(1);
        let d2 = strategy.compute_delay(2);
        let d10 = strategy.compute_delay(10);
        assert_eq!(d0.num_milliseconds(), 1000);
        assert_eq!(d1.num_milliseconds(), 2000);
        assert_eq!(d2.num_milliseconds(), 4000);
        assert_eq!(d10.num_milliseconds(), 60_000);
    }

    #[test]
    fn test_outbox_backoff_fixed() {
        let strategy = BackoffStrategy::Fixed { interval_ms: 5000 };
        let d0 = strategy.compute_delay(0);
        let d5 = strategy.compute_delay(5);
        assert_eq!(d0.num_milliseconds(), 5000);
        assert_eq!(d5.num_milliseconds(), 5000);
    }

    #[test]
    fn test_outbox_multiple_targets() {
        let mut processor = OutboxProcessor::new();
        let act = make_activity("1", "Announce", "actor1");
        processor.enqueue(act.clone(), "remote1".into());
        processor.enqueue(act, "remote2".into());
        processor.enqueue(make_activity("2", "Create", "actor1"), "remote1".into());
        assert_eq!(processor.pending_count(), 3);
        assert!(processor.mark_delivered("1", "remote1"));
        assert!(processor.mark_delivered("2", "remote1"));
        assert_eq!(processor.pending_count(), 1);
    }

    #[test]
    fn test_delivery_status_variants() {
        let statuses = [
            DeliveryStatus::Pending,
            DeliveryStatus::InFlight,
            DeliveryStatus::Delivered,
            DeliveryStatus::Failed {
                error: "timeout".into(),
            },
            DeliveryStatus::PermanentFailure {
                error: "404".into(),
            },
        ];
        assert_eq!(statuses.len(), 5);
        assert!(matches!(statuses[3], DeliveryStatus::Failed { .. }));
        assert!(matches!(
            statuses[4],
            DeliveryStatus::PermanentFailure { .. }
        ));
    }
}
