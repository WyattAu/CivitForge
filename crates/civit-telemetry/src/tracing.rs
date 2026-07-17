#![forbid(unsafe_code)]

use std::collections::HashMap;
use parking_lot::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

pub struct TraceIdGenerator {
    counter: AtomicU64,
}

impl TraceIdGenerator {
    pub fn new() -> Self {
        Self {
            counter: AtomicU64::new(1),
        }
    }

    pub fn next_id(&self) -> u64 {
        self.counter.fetch_add(1, Ordering::Relaxed)
    }
}

impl Default for TraceIdGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct Span {
    pub trace_id: u64,
    pub span_id: u64,
    pub parent_id: Option<u64>,
    pub name: String,
    pub start: Instant,
    pub attributes: HashMap<String, String>,
}

impl Span {
    pub fn new(trace_id: u64, span_id: u64, name: &str) -> Self {
        Self {
            trace_id,
            span_id,
            parent_id: None,
            name: name.to_string(),
            start: Instant::now(),
            attributes: HashMap::new(),
        }
    }

    pub fn with_parent(mut self, parent_id: u64) -> Self {
        self.parent_id = Some(parent_id);
        self
    }

    pub fn elapsed_ms(&self) -> u64 {
        self.start.elapsed().as_millis() as u64
    }

    pub fn set_attribute(&mut self, key: &str, value: &str) {
        self.attributes.insert(key.to_string(), value.to_string());
    }
}

pub struct SpanRecorder {
    spans: Mutex<Vec<Span>>,
    max_spans: usize,
}

impl SpanRecorder {
    pub fn new(max_spans: usize) -> Self {
        Self {
            spans: Mutex::new(Vec::with_capacity(max_spans)),
            max_spans,
        }
    }

    pub fn record(&self, span: Span) {
        let mut spans = self.spans.lock();
        if spans.len() >= self.max_spans {
            spans.remove(0);
        }
        spans.push(span);
    }

    pub fn get_trace(&self, trace_id: u64) -> Vec<Span> {
        let spans = self.spans.lock();
        spans
            .iter()
            .filter(|s| s.trace_id == trace_id)
            .cloned()
            .collect()
    }

    pub fn is_empty(&self) -> bool {
        self.spans.lock().is_empty()
    }

    pub fn len(&self) -> usize {
        self.spans.lock().len()
    }

    pub fn clear(&self) {
        self.spans.lock().clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_trace_id_generator_sequential() {
        let idgen = TraceIdGenerator::new();
        let id1 = idgen.next_id();
        let id2 = idgen.next_id();
        let id3 = idgen.next_id();
        assert_eq!(id1, 1);
        assert_eq!(id2, 2);
        assert_eq!(id3, 3);
    }

    #[test]
    fn test_trace_id_generator_default() {
        let idgen = TraceIdGenerator::default();
        assert_eq!(idgen.next_id(), 1);
    }

    #[test]
    fn test_trace_id_generator_unique() {
        let idgen = TraceIdGenerator::new();
        let mut ids: Vec<u64> = (0..1000).map(|_| idgen.next_id()).collect();
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), 1000);
    }

    #[test]
    fn test_span_new() {
        let span = Span::new(1, 10, "root");
        assert_eq!(span.trace_id, 1);
        assert_eq!(span.span_id, 10);
        assert_eq!(span.name, "root");
        assert!(span.parent_id.is_none());
        assert!(span.attributes.is_empty());
    }

    #[test]
    fn test_span_with_parent() {
        let span = Span::new(1, 20, "child").with_parent(10);
        assert_eq!(span.parent_id, Some(10));
        assert_eq!(span.span_id, 20);
    }

    #[test]
    fn test_span_set_attribute() {
        let mut span = Span::new(1, 1, "test");
        span.set_attribute("http.method", "GET");
        span.set_attribute("http.status", "200");
        assert_eq!(span.attributes.get("http.method").unwrap(), "GET");
        assert_eq!(span.attributes.get("http.status").unwrap(), "200");
    }

    #[test]
    fn test_span_elapsed_ms() {
        let span = Span::new(1, 1, "timed");
        thread::sleep(Duration::from_millis(10));
        assert!(span.elapsed_ms() >= 10);
    }

    #[test]
    fn test_span_clone() {
        let span = Span::new(5, 50, "original");
        let cloned = span.clone();
        assert_eq!(cloned.trace_id, span.trace_id);
        assert_eq!(cloned.span_id, span.span_id);
        assert_eq!(cloned.name, span.name);
    }

    #[test]
    fn test_span_recorder_record() {
        let recorder = SpanRecorder::new(100);
        assert_eq!(recorder.len(), 0);
        recorder.record(Span::new(1, 1, "a"));
        assert_eq!(recorder.len(), 1);
        recorder.record(Span::new(1, 2, "b"));
        assert_eq!(recorder.len(), 2);
    }

    #[test]
    fn test_span_recorder_get_trace() {
        let recorder = SpanRecorder::new(100);
        recorder.record(Span::new(1, 1, "a"));
        recorder.record(Span::new(2, 2, "b"));
        recorder.record(Span::new(1, 3, "c"));
        let trace1 = recorder.get_trace(1);
        assert_eq!(trace1.len(), 2);
        let trace2 = recorder.get_trace(2);
        assert_eq!(trace2.len(), 1);
    }

    #[test]
    fn test_span_recorder_get_trace_empty() {
        let recorder = SpanRecorder::new(100);
        let trace = recorder.get_trace(999);
        assert!(trace.is_empty());
    }

    #[test]
    fn test_span_recorder_max_spans_eviction() {
        let recorder = SpanRecorder::new(3);
        recorder.record(Span::new(1, 1, "first"));
        recorder.record(Span::new(2, 2, "second"));
        recorder.record(Span::new(3, 3, "third"));
        recorder.record(Span::new(4, 4, "fourth"));
        assert_eq!(recorder.len(), 3);
        let trace = recorder.get_trace(1);
        assert!(trace.is_empty(), "first span should have been evicted");
    }

    #[test]
    fn test_span_recorder_clear() {
        let recorder = SpanRecorder::new(100);
        recorder.record(Span::new(1, 1, "a"));
        recorder.record(Span::new(2, 2, "b"));
        recorder.clear();
        assert_eq!(recorder.len(), 0);
    }

    #[test]
    fn test_span_recorder_attributes_preserved() {
        let recorder = SpanRecorder::new(100);
        let mut span = Span::new(1, 1, "attr_test");
        span.set_attribute("key", "value");
        recorder.record(span);
        let trace = recorder.get_trace(1);
        assert_eq!(trace[0].attributes.get("key").unwrap(), "value");
    }

    #[test]
    fn test_concurrent_trace_ids() {
        let idgen = std::sync::Arc::new(TraceIdGenerator::new());
        let handles: Vec<_> = (0..4)
            .map(|_| {
                let idgen = idgen.clone();
                thread::spawn(move || idgen.next_id())
            })
            .collect();
        let ids: Vec<u64> = handles.into_iter().map(|h| h.join().unwrap()).collect();
        let unique: std::collections::HashSet<u64> = ids.iter().copied().collect();
        assert_eq!(unique.len(), 4);
    }

    #[test]
    fn test_span_elapsed_zero_initially() {
        let span = Span::new(1, 1, "instant");
        assert!(span.elapsed_ms() < 5);
    }
}
