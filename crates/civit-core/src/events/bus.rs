#![forbid(unsafe_code)]

use crate::events::model::Event;
use chrono::{DateTime, Utc};
use dashmap::DashMap;
use std::collections::VecDeque;
use std::sync::atomic::{AtomicU64, Ordering};

pub trait EventSubscriber: Send + Sync {
    fn on_event(&self, event: &Event);
}

pub struct EventBus {
    subscribers: DashMap<String, Vec<Box<dyn EventSubscriber>>>,
    event_log: std::sync::Mutex<VecDeque<Event>>,
    max_log_size: usize,
    publish_count: AtomicU64,
}

impl EventBus {
    pub fn new(max_log_size: usize) -> Self {
        Self {
            subscribers: DashMap::new(),
            event_log: std::sync::Mutex::new(VecDeque::with_capacity(max_log_size)),
            max_log_size,
            publish_count: AtomicU64::new(0),
        }
    }

    pub fn subscribe(&self, topic: &str, subscriber: Box<dyn EventSubscriber>) {
        self.subscribers
            .entry(topic.to_string())
            .or_default()
            .push(subscriber);
    }

    pub fn unsubscribe(&self, topic: &str) {
        self.subscribers.remove(topic);
    }

    pub fn publish(&self, topic: &str, event: Event) {
        self.publish_count.fetch_add(1, Ordering::Relaxed);

        {
            let mut log = self.event_log.lock().expect("event log lock poisoned");
            if log.len() >= self.max_log_size {
                log.pop_front();
            }
            log.push_back(event.clone());
        }

        if let Some(subs) = self.subscribers.get(topic) {
            for sub in subs.iter() {
                sub.on_event(&event);
            }
        }

        if topic != "global" {
            if let Some(global_subs) = self.subscribers.get("global") {
                for sub in global_subs.iter() {
                    sub.on_event(&event);
                }
            }
        }
    }

    pub fn replay(&self, topic: &str, since: DateTime<Utc>) -> Vec<Event> {
        let log = self.event_log.lock().expect("event log lock poisoned");
        log.iter()
            .filter(|e| e.timestamp >= since)
            .filter(|e| self.event_matches_topic(e, topic))
            .cloned()
            .collect()
    }

    pub fn recent(&self, count: usize) -> Vec<Event> {
        let log = self.event_log.lock().expect("event log lock poisoned");
        let len = log.len();
        let start = len.saturating_sub(count);
        log.iter().skip(start).cloned().collect()
    }

    pub fn publish_count(&self) -> u64 {
        self.publish_count.load(Ordering::Relaxed)
    }

    fn event_matches_topic(&self, event: &Event, topic: &str) -> bool {
        if topic == "global" {
            return true;
        }
        if let Some(repo_id) = self.extract_repo_id(event) {
            if topic == format!("repo:{repo_id}") {
                return true;
            }
        }
        false
    }

    fn extract_repo_id<'a>(&self, event: &'a Event) -> Option<&'a str> {
        use crate::events::model::EventPayload;
        match &event.payload {
            EventPayload::PushEvent { repo_id, .. } => Some(repo_id),
            EventPayload::PrEvent { repo_id, .. } => Some(repo_id),
            EventPayload::IssueEvent { repo_id, .. } => Some(repo_id),
            EventPayload::CiEvent { repo_id, .. } => Some(repo_id),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::model::{EventCategory, EventPayload};
    use std::sync::atomic::AtomicUsize;

    struct TestSubscriber {
        event_count: AtomicUsize,
    }

    impl TestSubscriber {
        fn new() -> Self {
            Self {
                event_count: AtomicUsize::new(0),
            }
        }
    }

    impl EventSubscriber for TestSubscriber {
        fn on_event(&self, _event: &Event) {
            self.event_count.fetch_add(1, Ordering::Relaxed);
        }
    }

    fn make_event(category: EventCategory, repo_id: &str) -> Event {
        Event::new(
            category,
            EventPayload::SystemEvent {
                level: crate::events::model::SystemLevel::Info,
                message: format!("test for {repo_id}"),
            },
            "test.local".to_string(),
        )
    }

    #[test]
    fn subscribe_and_publish() {
        let bus = EventBus::new(100);
        let sub = TestSubscriber::new();
        bus.subscribe("global", Box::new(sub));

        bus.publish("global", make_event(EventCategory::System, "none"));

        let subs = bus.subscribers.get("global").unwrap();
        subs[0].on_event(&make_event(EventCategory::System, "check"));
        assert_eq!(bus.publish_count.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn unsubscribe_removes_topic() {
        let bus = EventBus::new(100);
        bus.subscribe("repo:abc", Box::new(TestSubscriber::new()));
        bus.unsubscribe("repo:abc");
        assert!(!bus.subscribers.contains_key("repo:abc"));
    }

    #[test]
    fn publish_increments_count() {
        let bus = EventBus::new(100);
        bus.publish("global", make_event(EventCategory::System, "none"));
        bus.publish("global", make_event(EventCategory::System, "none"));
        assert_eq!(bus.publish_count(), 2);
    }

    #[test]
    fn recent_returns_latest() {
        let bus = EventBus::new(100);
        for i in 0..5 {
            bus.publish(
                "global",
                Event::new(
                    EventCategory::System,
                    EventPayload::SystemEvent {
                        level: crate::events::model::SystemLevel::Info,
                        message: format!("event-{i}"),
                    },
                    "test.local".to_string(),
                ),
            );
        }
        let recent = bus.recent(3);
        assert_eq!(recent.len(), 3);
    }

    #[test]
    fn replay_filters_by_time() {
        let bus = EventBus::new(100);
        let now = Utc::now();

        bus.publish(
            "global",
            Event::new(
                EventCategory::System,
                EventPayload::SystemEvent {
                    level: crate::events::model::SystemLevel::Info,
                    message: "old".to_string(),
                },
                "test.local".to_string(),
            ),
        );

        bus.publish(
            "global",
            Event::new(
                EventCategory::System,
                EventPayload::SystemEvent {
                    level: crate::events::model::SystemLevel::Info,
                    message: "new".to_string(),
                },
                "test.local".to_string(),
            ),
        );

        let results = bus.replay("global", now);
        assert!(!results.is_empty());
    }

    #[test]
    fn log_eviction_when_full() {
        let bus = EventBus::new(3);
        for i in 0..5 {
            bus.publish(
                "global",
                Event::new(
                    EventCategory::System,
                    EventPayload::SystemEvent {
                        level: crate::events::model::SystemLevel::Info,
                        message: format!("ev-{i}"),
                    },
                    "test.local".to_string(),
                ),
            );
        }
        assert_eq!(bus.recent(10).len(), 3);
    }
}
