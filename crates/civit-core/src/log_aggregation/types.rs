use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
    Fatal,
}

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogLevel::Trace => write!(f, "trace"),
            LogLevel::Debug => write!(f, "debug"),
            LogLevel::Info => write!(f, "info"),
            LogLevel::Warn => write!(f, "warn"),
            LogLevel::Error => write!(f, "error"),
            LogLevel::Fatal => write!(f, "fatal"),
        }
    }
}

impl std::str::FromStr for LogLevel {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "trace" => Ok(LogLevel::Trace),
            "debug" => Ok(LogLevel::Debug),
            "info" => Ok(LogLevel::Info),
            "warn" => Ok(LogLevel::Warn),
            "error" => Ok(LogLevel::Error),
            "fatal" => Ok(LogLevel::Fatal),
            _ => Err(format!("unknown log level: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub id: Uuid,
    pub level: LogLevel,
    pub message: String,
    pub source: String,
    pub service: String,
    pub trace_id: Option<String>,
    pub span_id: Option<String>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateLogEntry {
    pub level: LogLevel,
    pub message: String,
    pub source: String,
    pub service: Option<String>,
    pub trace_id: Option<String>,
    pub span_id: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogSearchFilter {
    pub level: Option<LogLevel>,
    pub source: Option<String>,
    pub service: Option<String>,
    pub trace_id: Option<String>,
    pub search: Option<String>,
    pub since: Option<DateTime<Utc>>,
    pub until: Option<DateTime<Utc>>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogSearchResult {
    pub entries: Vec<LogEntry>,
    pub total_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogRetentionConfig {
    pub max_age_days: i32,
    pub min_level: LogLevel,
    pub batch_size: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogExportRequest {
    pub format: LogExportFormat,
    pub filter: LogSearchFilter,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogExportFormat {
    Json,
    Csv,
    Text,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogExportResult {
    pub entries: Vec<LogEntry>,
    pub format: LogExportFormat,
    pub exported_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogRetentionPolicy {
    pub id: Uuid,
    pub service: String,
    pub level: LogLevel,
    pub retention_days: i32,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateLogRetentionPolicy {
    pub service: String,
    pub level: LogLevel,
    pub retention_days: Option<i32>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateLogRetentionPolicy {
    pub service: Option<String>,
    pub level: Option<LogLevel>,
    pub retention_days: Option<i32>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogServiceStats {
    pub total_entries: i64,
    pub level_counts: std::collections::HashMap<String, i64>,
    pub service_counts: std::collections::HashMap<String, i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntryV3 {
    pub id: uuid::Uuid,
    pub level: LogLevel,
    pub message: String,
    pub source: String,
    pub service: String,
    pub trace_id: Option<String>,
    pub span_id: Option<String>,
    pub metadata: serde_json::Value,
    pub retention_days: i32,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateLogEntryV3 {
    pub level: LogLevel,
    pub message: String,
    pub source: String,
    pub service: Option<String>,
    pub trace_id: Option<String>,
    pub span_id: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub retention_days: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogSearchFilterV3 {
    pub level: Option<LogLevel>,
    pub source: Option<String>,
    pub service: Option<String>,
    pub trace_id: Option<String>,
    pub search: Option<String>,
    pub full_text_search: Option<String>,
    pub since: Option<chrono::DateTime<chrono::Utc>>,
    pub until: Option<chrono::DateTime<chrono::Utc>>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogSearchResultV3 {
    pub entries: Vec<LogEntryV3>,
    pub total_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogCorrelation {
    pub trace_id: String,
    pub entries: Vec<LogEntryV3>,
    pub service_count: i64,
    pub entry_count: i64,
}
