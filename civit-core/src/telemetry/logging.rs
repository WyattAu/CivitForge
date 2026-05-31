#![forbid(unsafe_code)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogLevel::Trace => write!(f, "TRACE"),
            LogLevel::Debug => write!(f, "DEBUG"),
            LogLevel::Info => write!(f, "INFO"),
            LogLevel::Warn => write!(f, "WARN"),
            LogLevel::Error => write!(f, "ERROR"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: DateTime<Utc>,
    pub level: LogLevel,
    pub message: String,
    pub target: String,
    pub module: String,
    pub fields: serde_json::Value,
    pub span_id: Option<String>,
    pub trace_id: Option<String>,
    pub request_id: Option<String>,
}

impl LogEntry {
    pub fn new(level: LogLevel, target: &str, message: &str) -> Self {
        Self {
            timestamp: Utc::now(),
            level,
            target: target.to_string(),
            module: target.to_string(),
            message: message.to_string(),
            fields: serde_json::Value::Object(serde_json::Map::new()),
            span_id: None,
            trace_id: None,
            request_id: None,
        }
    }

    pub fn with_field(mut self, key: &str, value: serde_json::Value) -> Self {
        if let serde_json::Value::Object(ref mut map) = self.fields {
            map.insert(key.to_string(), value);
        }
        self
    }

    pub fn with_request_id(mut self, id: &str) -> Self {
        self.request_id = Some(id.to_string());
        self
    }

    pub fn with_trace_context(mut self, trace_id: &str, span_id: &str) -> Self {
        self.trace_id = Some(trace_id.to_string());
        self.span_id = Some(span_id.to_string());
        self
    }
}

pub trait LogFormatter: Send + Sync {
    fn format(&self, entry: &LogEntry) -> String;
}

pub struct JsonLogFormatter;

impl LogFormatter for JsonLogFormatter {
    fn format(&self, entry: &LogEntry) -> String {
        serde_json::to_string(entry).unwrap_or_else(|_| entry.message.clone())
    }
}

pub struct TextLogFormatter;

impl LogFormatter for TextLogFormatter {
    fn format(&self, entry: &LogEntry) -> String {
        let mut parts = vec![
            entry.timestamp.to_rfc3339(),
            entry.level.to_string(),
            entry.target.clone(),
            entry.message.clone(),
        ];
        if let Some(ref id) = entry.request_id {
            parts.push(format!("req_id={id}"));
        }
        parts.join(" | ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_level_display() {
        assert_eq!(LogLevel::Trace.to_string(), "TRACE");
        assert_eq!(LogLevel::Debug.to_string(), "DEBUG");
        assert_eq!(LogLevel::Info.to_string(), "INFO");
        assert_eq!(LogLevel::Warn.to_string(), "WARN");
        assert_eq!(LogLevel::Error.to_string(), "ERROR");
    }

    #[test]
    fn test_log_level_ordering() {
        assert!(LogLevel::Error > LogLevel::Warn);
        assert!(LogLevel::Warn > LogLevel::Info);
        assert!(LogLevel::Info > LogLevel::Debug);
        assert!(LogLevel::Debug > LogLevel::Trace);
    }

    #[test]
    fn test_log_entry_new() {
        let entry = LogEntry::new(LogLevel::Info, "my_module", "hello");
        assert_eq!(entry.level, LogLevel::Info);
        assert_eq!(entry.target, "my_module");
        assert_eq!(entry.message, "hello");
        assert!(entry.span_id.is_none());
        assert!(entry.trace_id.is_none());
        assert!(entry.request_id.is_none());
    }

    #[test]
    fn test_log_entry_with_field() {
        let entry = LogEntry::new(LogLevel::Info, "mod", "msg")
            .with_field("user_id", serde_json::Value::Number(42.into()))
            .with_field("action", serde_json::Value::String("login".to_string()));
        assert_eq!(
            entry.fields["user_id"],
            serde_json::Value::Number(42.into())
        );
        assert_eq!(
            entry.fields["action"],
            serde_json::Value::String("login".to_string())
        );
    }

    #[test]
    fn test_log_entry_with_request_id() {
        let entry = LogEntry::new(LogLevel::Error, "mod", "fail").with_request_id("req-123");
        assert_eq!(entry.request_id.as_deref(), Some("req-123"));
    }

    #[test]
    fn test_log_entry_with_trace_context() {
        let entry = LogEntry::new(LogLevel::Info, "mod", "trace")
            .with_trace_context("trace-abc", "span-xyz");
        assert_eq!(entry.trace_id.as_deref(), Some("trace-abc"));
        assert_eq!(entry.span_id.as_deref(), Some("span-xyz"));
    }

    #[test]
    fn test_log_entry_chaining() {
        let entry = LogEntry::new(LogLevel::Debug, "mod", "chain")
            .with_request_id("r1")
            .with_trace_context("t1", "s1")
            .with_field("key", serde_json::json!("val"));
        assert_eq!(entry.request_id.as_deref(), Some("r1"));
        assert_eq!(entry.trace_id.as_deref(), Some("t1"));
        assert_eq!(entry.span_id.as_deref(), Some("s1"));
        assert_eq!(entry.fields["key"], serde_json::json!("val"));
    }

    #[test]
    fn test_json_formatter() {
        let formatter = JsonLogFormatter;
        let entry = LogEntry::new(LogLevel::Info, "test", "hello");
        let output = formatter.format(&entry);
        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert_eq!(parsed["level"], "Info");
        assert_eq!(parsed["message"], "hello");
    }

    #[test]
    fn test_text_formatter() {
        let formatter = TextLogFormatter;
        let entry = LogEntry::new(LogLevel::Warn, "my_mod", "warning msg");
        let output = formatter.format(&entry);
        assert!(output.contains("WARN"));
        assert!(output.contains("my_mod"));
        assert!(output.contains("warning msg"));
    }

    #[test]
    fn test_text_formatter_with_request_id() {
        let formatter = TextLogFormatter;
        let entry = LogEntry::new(LogLevel::Info, "mod", "msg").with_request_id("req-999");
        let output = formatter.format(&entry);
        assert!(output.contains("req_id=req-999"));
    }

    #[test]
    fn test_text_formatter_without_request_id() {
        let formatter = TextLogFormatter;
        let entry = LogEntry::new(LogLevel::Info, "mod", "msg");
        let output = formatter.format(&entry);
        assert!(!output.contains("req_id="));
    }

    #[test]
    fn test_log_entry_clone() {
        let entry = LogEntry::new(LogLevel::Error, "mod", "err").with_request_id("r1");
        let cloned = entry.clone();
        assert_eq!(cloned.request_id, entry.request_id);
        assert_eq!(cloned.level, entry.level);
    }

    #[test]
    fn test_log_entry_serialization_roundtrip() {
        let entry = LogEntry::new(LogLevel::Info, "mod", "test")
            .with_request_id("r1")
            .with_trace_context("t1", "s1")
            .with_field("k", serde_json::json!("v"));
        let json = serde_json::to_string(&entry).unwrap();
        let deserialized: LogEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.level, LogLevel::Info);
        assert_eq!(deserialized.request_id.as_deref(), Some("r1"));
    }

    #[test]
    fn test_log_level_serialization() {
        let levels = vec![
            LogLevel::Trace,
            LogLevel::Debug,
            LogLevel::Info,
            LogLevel::Warn,
            LogLevel::Error,
        ];
        for level in &levels {
            let json = serde_json::to_string(level).unwrap();
            let back: LogLevel = serde_json::from_str(&json).unwrap();
            assert_eq!(*level, back);
        }
    }

    #[test]
    fn test_log_entry_timestamp_is_recent() {
        let before = Utc::now();
        let entry = LogEntry::new(LogLevel::Info, "mod", "now");
        let after = Utc::now();
        assert!(entry.timestamp >= before);
        assert!(entry.timestamp <= after);
    }

    #[test]
    fn test_json_formatter_output_valid_json() {
        let formatter = JsonLogFormatter;
        let entry =
            LogEntry::new(LogLevel::Error, "mod", "err").with_field("code", serde_json::json!(500));
        let output = formatter.format(&entry);
        assert!(serde_json::from_str::<serde_json::Value>(&output).is_ok());
    }

    #[test]
    fn test_text_formatter_parts_count() {
        let formatter = TextLogFormatter;
        let entry = LogEntry::new(LogLevel::Info, "mod", "msg").with_request_id("r1");
        let output = formatter.format(&entry);
        let parts: Vec<&str> = output.split(" | ").collect();
        assert_eq!(parts.len(), 5);
    }
}
