#![forbid(unsafe_code)]

use crate::api::AppState;
use crate::events::log_stream::{LogStreamEvent, PipelineStatusEvent};
use axum::extract::Path;
use axum::extract::State;
use axum::response::sse::{Event as SseEvent, KeepAlive, Sse};
use axum::routing::get;
use futures::stream::Stream;
use serde::Serialize;
use std::convert::Infallible;
use std::sync::Arc;
use tokio::sync::broadcast;

#[derive(Debug, Serialize)]
struct SseLogData {
    #[serde(rename = "type")]
    event_type: String,
    #[serde(flatten)]
    payload: serde_json::Value,
}

fn log_rx_to_sse(
    rx: broadcast::Receiver<LogStreamEvent>,
    pipeline_id: Arc<String>,
) -> impl Stream<Item = Result<SseEvent, Infallible>> + Send + 'static {
    futures::stream::unfold(rx, move |mut rx| {
        let pid = pipeline_id.clone();
        async move {
            loop {
                match rx.recv().await {
                    Ok(val) if val.pipeline_id == *pid => {
                        let data = SseLogData {
                            event_type: "log".to_string(),
                            payload: serde_json::to_value(&val).unwrap_or_default(),
                        };
                        let sse = SseEvent::default()
                            .data(serde_json::to_string(&data).unwrap_or_default());
                        return Some((Ok(sse), rx));
                    }
                    Ok(_) => continue,
                    Err(broadcast::error::RecvError::Lagged(_)) => continue,
                    Err(broadcast::error::RecvError::Closed) => return None,
                }
            }
        }
    })
}

fn status_rx_to_sse(
    rx: broadcast::Receiver<PipelineStatusEvent>,
    pipeline_id: Arc<String>,
) -> impl Stream<Item = Result<SseEvent, Infallible>> + Send + 'static {
    futures::stream::unfold(rx, move |mut rx| {
        let pid = pipeline_id.clone();
        async move {
            loop {
                match rx.recv().await {
                    Ok(val) if val.pipeline_id == *pid => {
                        let data = SseLogData {
                            event_type: "pipeline_status".to_string(),
                            payload: serde_json::to_value(&val).unwrap_or_default(),
                        };
                        let sse = SseEvent::default()
                            .data(serde_json::to_string(&data).unwrap_or_default());
                        return Some((Ok(sse), rx));
                    }
                    Ok(_) => continue,
                    Err(broadcast::error::RecvError::Lagged(_)) => continue,
                    Err(broadcast::error::RecvError::Closed) => return None,
                }
            }
        }
    })
}

pub async fn stream_pipeline_logs(
    Path(pipeline_id): Path<String>,
    State(state): State<AppState>,
) -> Sse<impl Stream<Item = Result<SseEvent, Infallible>>> {
    tracing::info!(pipeline_id = %pipeline_id, "new SSE log stream connection");

    let log_rx = state.log_broadcaster.subscribe_logs();
    let status_rx = state.log_broadcaster.subscribe_status();
    let pid = Arc::new(pipeline_id);

    let log_stream = log_rx_to_sse(log_rx, pid.clone());
    let status_stream = status_rx_to_sse(status_rx, pid);

    let stream = futures::stream::select(status_stream, log_stream);
    Sse::new(stream).keep_alive(KeepAlive::default())
}

pub fn log_stream_routes() -> axum::Router<AppState> {
    axum::Router::new().route(
        "/api/v1/pipelines/{pipeline_id}/logs/stream",
        get(stream_pipeline_logs),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

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
    fn sse_log_data_serialization() {
        let event = make_log_event("p1", 0, "hello");
        let data = SseLogData {
            event_type: "log".to_string(),
            payload: serde_json::to_value(&event).unwrap(),
        };
        let json = serde_json::to_string(&data).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["type"], "log");
        assert_eq!(parsed["pipeline_id"], "p1");
        assert_eq!(parsed["log_line"], "hello");
    }

    #[test]
    fn sse_status_data_serialization() {
        let event = make_status_event("p1", "running");
        let data = SseLogData {
            event_type: "pipeline_status".to_string(),
            payload: serde_json::to_value(&event).unwrap(),
        };
        let json = serde_json::to_string(&data).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["type"], "pipeline_status");
        assert_eq!(parsed["status"], "running");
    }

    #[test]
    fn sse_event_constructed_from_data() {
        let event = make_log_event("p1", 0, "test line");
        let data = SseLogData {
            event_type: "log".to_string(),
            payload: serde_json::to_value(&event).unwrap(),
        };
        let sse = SseEvent::default().data(serde_json::to_string(&data).unwrap());
        let _ = sse;
    }

    #[test]
    fn sse_event_json_data_roundtrip() {
        let event = make_log_event("p1", 0, "test");
        let data = SseLogData {
            event_type: "log".to_string(),
            payload: serde_json::to_value(&event).unwrap(),
        };
        let json = serde_json::to_string(&data).unwrap();
        let roundtrip: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(roundtrip["type"], "log");
        assert_eq!(roundtrip["pipeline_id"], "p1");
    }

    #[tokio::test]
    async fn log_broadcaster_delivers_event_via_receiver() {
        let b = Arc::new(crate::events::LogBroadcaster::new(16));
        let mut rx = b.subscribe_logs();
        b.emit_log(make_log_event("p1", 0, "stream test"));
        let received = rx.recv().await.unwrap();
        assert_eq!(received.pipeline_id, "p1");
        assert_eq!(received.log_line, "stream test");
    }

    #[tokio::test]
    async fn status_broadcaster_delivers_event_via_receiver() {
        let b = Arc::new(crate::events::LogBroadcaster::new(16));
        let mut rx = b.subscribe_status();
        b.emit_status(make_status_event("p1", "success"));
        let received = rx.recv().await.unwrap();
        assert_eq!(received.pipeline_id, "p1");
        assert_eq!(received.status, "success");
    }

    #[tokio::test]
    async fn broadcast_receiver_skips_lagged_messages() {
        let b = Arc::new(crate::events::LogBroadcaster::new(2));
        let mut rx = b.subscribe_logs();
        b.emit_log(make_log_event("p1", 0, "line-0"));
        b.emit_log(make_log_event("p1", 0, "line-1"));
        b.emit_log(make_log_event("p1", 0, "line-2"));
        loop {
            match rx.recv().await {
                Ok(_) => continue,
                Err(broadcast::error::RecvError::Lagged(_)) => break,
                Err(broadcast::error::RecvError::Closed) => break,
            }
        }
    }

    #[test]
    fn sse_log_data_contains_pipeline_id() {
        let event = make_log_event("pipe-abc", 3, "error in test");
        let data = SseLogData {
            event_type: "log".to_string(),
            payload: serde_json::to_value(&event).unwrap(),
        };
        let json = serde_json::to_string(&data).unwrap();
        assert!(json.contains("pipe-abc"));
        assert!(json.contains("error in test"));
    }
}
