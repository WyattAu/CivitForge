#![forbid(unsafe_code)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogStreamEvent {
    pub pipeline_id: String,
    pub step_index: usize,
    pub step_name: String,
    pub log_line: String,
    pub timestamp: DateTime<Utc>,
    pub is_error: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineStatusEvent {
    pub pipeline_id: String,
    pub status: String,
    pub step_index: Option<usize>,
    pub step_name: Option<String>,
    pub message: String,
    pub timestamp: DateTime<Utc>,
}

pub struct LogBroadcaster {
    log_tx: broadcast::Sender<LogStreamEvent>,
    status_tx: broadcast::Sender<PipelineStatusEvent>,
}

impl LogBroadcaster {
    pub fn new(capacity: usize) -> Self {
        let (log_tx, _) = broadcast::channel(capacity);
        let (status_tx, _) = broadcast::channel(capacity);
        Self { log_tx, status_tx }
    }

    pub fn subscribe_logs(&self) -> broadcast::Receiver<LogStreamEvent> {
        self.log_tx.subscribe()
    }

    pub fn subscribe_status(&self) -> broadcast::Receiver<PipelineStatusEvent> {
        self.status_tx.subscribe()
    }

    pub fn emit_log(&self, event: LogStreamEvent) {
        let _ = self.log_tx.send(event);
    }

    pub fn emit_status(&self, event: PipelineStatusEvent) {
        let _ = self.status_tx.send(event);
    }

    pub fn log_receiver_count(&self) -> usize {
        self.log_tx.receiver_count()
    }

    pub fn status_receiver_count(&self) -> usize {
        self.status_tx.receiver_count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_log_event(pipeline_id: &str, step_index: usize, log_line: &str) -> LogStreamEvent {
        LogStreamEvent {
            pipeline_id: pipeline_id.to_string(),
            step_index,
            step_name: format!("step-{step_index}"),
            log_line: log_line.to_string(),
            timestamp: Utc::now(),
            is_error: false,
        }
    }

    fn make_status_event(pipeline_id: &str, status: &str) -> PipelineStatusEvent {
        PipelineStatusEvent {
            pipeline_id: pipeline_id.to_string(),
            status: status.to_string(),
            step_index: None,
            step_name: None,
            message: format!("{status} pipeline {pipeline_id}"),
            timestamp: Utc::now(),
        }
    }

    #[test]
    fn new_broadcaster_has_no_receivers() {
        let b = LogBroadcaster::new(16);
        assert_eq!(b.log_receiver_count(), 0);
        assert_eq!(b.status_receiver_count(), 0);
    }

    #[test]
    fn subscribe_logs_returns_receiver() {
        let b = LogBroadcaster::new(16);
        let _rx = b.subscribe_logs();
        assert_eq!(b.log_receiver_count(), 1);
    }

    #[test]
    fn subscribe_status_returns_receiver() {
        let b = LogBroadcaster::new(16);
        let _rx = b.subscribe_status();
        assert_eq!(b.status_receiver_count(), 1);
    }

    #[test]
    fn emit_log_delivers_to_subscriber() {
        let b = LogBroadcaster::new(16);
        let mut rx = b.subscribe_logs();
        b.emit_log(make_log_event("p1", 0, "hello world"));
        let received = rx.try_recv().unwrap();
        assert_eq!(received.pipeline_id, "p1");
        assert_eq!(received.log_line, "hello world");
        assert_eq!(received.step_index, 0);
    }

    #[test]
    fn emit_status_delivers_to_subscriber() {
        let b = LogBroadcaster::new(16);
        let mut rx = b.subscribe_status();
        b.emit_status(make_status_event("p1", "running"));
        let received = rx.try_recv().unwrap();
        assert_eq!(received.pipeline_id, "p1");
        assert_eq!(received.status, "running");
    }

    #[test]
    fn multiple_log_receivers_all_get_event() {
        let b = LogBroadcaster::new(16);
        let mut rx1 = b.subscribe_logs();
        let mut rx2 = b.subscribe_logs();
        let mut rx3 = b.subscribe_logs();
        b.emit_log(make_log_event("p2", 1, "multi-receiver test"));
        assert_eq!(rx1.try_recv().unwrap().log_line, "multi-receiver test");
        assert_eq!(rx2.try_recv().unwrap().log_line, "multi-receiver test");
        assert_eq!(rx3.try_recv().unwrap().log_line, "multi-receiver test");
    }

    #[test]
    fn multiple_status_receivers_all_get_event() {
        let b = LogBroadcaster::new(16);
        let mut rx1 = b.subscribe_status();
        let mut rx2 = b.subscribe_status();
        b.emit_status(make_status_event("p2", "success"));
        assert_eq!(rx1.try_recv().unwrap().status, "success");
        assert_eq!(rx2.try_recv().unwrap().status, "success");
    }

    #[test]
    fn emit_without_subscribers_does_not_panic() {
        let b = LogBroadcaster::new(16);
        b.emit_log(make_log_event("p3", 0, "no one listening"));
        b.emit_status(make_status_event("p3", "failed"));
    }

    #[test]
    fn channel_capacity_overflow_reports_lagged() {
        let b = LogBroadcaster::new(2);
        let mut rx = b.subscribe_logs();
        b.emit_log(make_log_event("p4", 0, "line-0"));
        b.emit_log(make_log_event("p4", 0, "line-1"));
        b.emit_log(make_log_event("p4", 0, "line-2"));
        let mut lagged = false;
        loop {
            match rx.try_recv() {
                Ok(_) => {}
                Err(tokio::sync::broadcast::error::TryRecvError::Lagged(n)) => {
                    lagged = true;
                    assert!(n > 0, "should lag by at least 1 message");
                    break;
                }
                Err(tokio::sync::broadcast::error::TryRecvError::Closed) => break,
                Err(tokio::sync::broadcast::error::TryRecvError::Empty) => break,
            }
        }
        assert!(lagged, "should receive Lagged error when capacity exceeded");
    }

    #[test]
    fn log_stream_event_serialization() {
        let event = make_log_event("pipe-123", 2, "compiling src/main.rs");
        let json = serde_json::to_string(&event).unwrap();
        let deserialized: LogStreamEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.pipeline_id, "pipe-123");
        assert_eq!(deserialized.step_index, 2);
        assert_eq!(deserialized.step_name, "step-2");
        assert_eq!(deserialized.log_line, "compiling src/main.rs");
        assert!(!deserialized.is_error);
    }

    #[test]
    fn log_stream_event_error_flag() {
        let event = LogStreamEvent {
            pipeline_id: "p".to_string(),
            step_index: 0,
            step_name: "build".to_string(),
            log_line: "error: undefined variable".to_string(),
            timestamp: Utc::now(),
            is_error: true,
        };
        let json = serde_json::to_string(&event).unwrap();
        let deserialized: LogStreamEvent = serde_json::from_str(&json).unwrap();
        assert!(deserialized.is_error);
    }

    #[test]
    fn pipeline_status_event_serialization() {
        let event = PipelineStatusEvent {
            pipeline_id: "pipe-456".to_string(),
            status: "failed".to_string(),
            step_index: Some(3),
            step_name: Some("test".to_string()),
            message: "tests failed with exit code 1".to_string(),
            timestamp: Utc::now(),
        };
        let json = serde_json::to_string(&event).unwrap();
        let deserialized: PipelineStatusEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.pipeline_id, "pipe-456");
        assert_eq!(deserialized.status, "failed");
        assert_eq!(deserialized.step_index, Some(3));
        assert_eq!(deserialized.step_name, Some("test".to_string()));
    }

    #[test]
    fn pipeline_status_event_without_step() {
        let event = PipelineStatusEvent {
            pipeline_id: "p7".to_string(),
            status: "cancelled".to_string(),
            step_index: None,
            step_name: None,
            message: "pipeline cancelled".to_string(),
            timestamp: Utc::now(),
        };
        let json = serde_json::to_string(&event).unwrap();
        let deserialized: PipelineStatusEvent = serde_json::from_str(&json).unwrap();
        assert!(deserialized.step_index.is_none());
        assert!(deserialized.step_name.is_none());
    }

    #[test]
    fn receiver_count_decreases_on_drop() {
        let b = LogBroadcaster::new(16);
        assert_eq!(b.log_receiver_count(), 0);
        {
            let _rx = b.subscribe_logs();
            assert_eq!(b.log_receiver_count(), 1);
        }
        assert_eq!(b.log_receiver_count(), 0);
    }

    #[test]
    fn sequential_log_events_delivered_in_order() {
        let b = LogBroadcaster::new(16);
        let mut rx = b.subscribe_logs();
        for i in 0..5 {
            b.emit_log(make_log_event("p-order", i, &format!("line-{i}")));
        }
        for i in 0..5 {
            let received = rx.try_recv().unwrap();
            assert_eq!(received.step_index, i);
            assert_eq!(received.log_line, format!("line-{i}"));
        }
    }
}
