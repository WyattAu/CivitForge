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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntryV4 {
    pub id: uuid::Uuid,
    pub level: LogLevel,
    pub message: String,
    pub source: String,
    pub service: String,
    pub trace_id: Option<String>,
    pub span_id: Option<String>,
    pub metadata: serde_json::Value,
    pub retention_days: i32,
    pub indexed: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateLogEntryV4 {
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
pub struct LogSearchFilterV4 {
    pub level: Option<LogLevel>,
    pub source: Option<String>,
    pub service: Option<String>,
    pub trace_id: Option<String>,
    pub search: Option<String>,
    pub indexed: Option<bool>,
    pub since: Option<chrono::DateTime<chrono::Utc>>,
    pub until: Option<chrono::DateTime<chrono::Utc>>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogSearchResultV4 {
    pub entries: Vec<LogEntryV4>,
    pub total_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogAlertRule {
    pub id: uuid::Uuid,
    pub name: String,
    pub level: LogLevel,
    pub pattern: String,
    pub threshold: i32,
    pub window_seconds: i32,
    pub enabled: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateLogAlertRule {
    pub name: String,
    pub level: LogLevel,
    pub pattern: String,
    pub threshold: Option<i32>,
    pub window_seconds: Option<i32>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateLogAlertRule {
    pub name: Option<String>,
    pub level: Option<LogLevel>,
    pub pattern: Option<String>,
    pub threshold: Option<i32>,
    pub window_seconds: Option<i32>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntryV5 {
    pub id: uuid::Uuid,
    pub level: LogLevel,
    pub message: String,
    pub source: String,
    pub service: String,
    pub trace_id: Option<String>,
    pub span_id: Option<String>,
    pub metadata: serde_json::Value,
    pub retention_days: i32,
    pub indexed: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateLogEntryV5 {
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
pub struct LogSearchFilterV5 {
    pub level: Option<LogLevel>,
    pub source: Option<String>,
    pub service: Option<String>,
    pub trace_id: Option<String>,
    pub search: Option<String>,
    pub indexed: Option<bool>,
    pub since: Option<chrono::DateTime<chrono::Utc>>,
    pub until: Option<chrono::DateTime<chrono::Utc>>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogSearchResultV5 {
    pub entries: Vec<LogEntryV5>,
    pub total_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogAlertRuleV2 {
    pub id: uuid::Uuid,
    pub name: String,
    pub level: LogLevel,
    pub pattern: String,
    pub threshold: i32,
    pub window_seconds: i32,
    pub enabled: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateLogAlertRuleV2 {
    pub name: String,
    pub level: LogLevel,
    pub pattern: String,
    pub threshold: Option<i32>,
    pub window_seconds: Option<i32>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateLogAlertRuleV2 {
    pub name: Option<String>,
    pub level: Option<LogLevel>,
    pub pattern: Option<String>,
    pub threshold: Option<i32>,
    pub window_seconds: Option<i32>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogNotificationConfig {
    pub id: uuid::Uuid,
    pub alert_rule_id: uuid::Uuid,
    pub notification_type: String,
    pub endpoint: String,
    pub enabled: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateLogNotificationConfig {
    pub alert_rule_id: uuid::Uuid,
    pub notification_type: String,
    pub endpoint: String,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntryV6 {
    pub id: uuid::Uuid,
    pub level: LogLevel,
    pub message: String,
    pub source: String,
    pub service: String,
    pub trace_id: Option<String>,
    pub span_id: Option<String>,
    pub metadata: serde_json::Value,
    pub retention_days: i32,
    pub indexed: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateLogEntryV6 {
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
pub struct LogSearchFilterV6 {
    pub level: Option<LogLevel>,
    pub source: Option<String>,
    pub service: Option<String>,
    pub trace_id: Option<String>,
    pub search: Option<String>,
    pub indexed: Option<bool>,
    pub since: Option<chrono::DateTime<chrono::Utc>>,
    pub until: Option<chrono::DateTime<chrono::Utc>>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogSearchResultV6 {
    pub entries: Vec<LogEntryV6>,
    pub total_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogAlertRuleV3 {
    pub id: uuid::Uuid,
    pub name: String,
    pub level: LogLevel,
    pub pattern: String,
    pub threshold: i32,
    pub window_seconds: i32,
    pub enabled: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateLogAlertRuleV3 {
    pub name: String,
    pub level: LogLevel,
    pub pattern: String,
    pub threshold: Option<i32>,
    pub window_seconds: Option<i32>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateLogAlertRuleV3 {
    pub name: Option<String>,
    pub level: Option<LogLevel>,
    pub pattern: Option<String>,
    pub threshold: Option<i32>,
    pub window_seconds: Option<i32>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntryV7 {
    pub id: uuid::Uuid,
    pub level: LogLevel,
    pub message: String,
    pub source: String,
    pub service: String,
    pub trace_id: Option<String>,
    pub span_id: Option<String>,
    pub metadata: serde_json::Value,
    pub retention_days: i32,
    pub indexed: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateLogEntryV7 {
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
pub struct LogSearchFilterV7 {
    pub level: Option<LogLevel>,
    pub source: Option<String>,
    pub service: Option<String>,
    pub trace_id: Option<String>,
    pub search: Option<String>,
    pub indexed: Option<bool>,
    pub since: Option<chrono::DateTime<chrono::Utc>>,
    pub until: Option<chrono::DateTime<chrono::Utc>>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogSearchResultV7 {
    pub entries: Vec<LogEntryV7>,
    pub total_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogAlertRuleV4 {
    pub id: uuid::Uuid,
    pub name: String,
    pub level: LogLevel,
    pub pattern: String,
    pub threshold: i32,
    pub window_seconds: i32,
    pub enabled: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateLogAlertRuleV4 {
    pub name: String,
    pub level: LogLevel,
    pub pattern: String,
    pub threshold: Option<i32>,
    pub window_seconds: Option<i32>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateLogAlertRuleV4 {
    pub name: Option<String>,
    pub level: Option<LogLevel>,
    pub pattern: Option<String>,
    pub threshold: Option<i32>,
    pub window_seconds: Option<i32>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogNotificationConfigV2 {
    pub id: uuid::Uuid,
    pub alert_rule_id: uuid::Uuid,
    pub notification_type: String,
    pub endpoint: String,
    pub enabled: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateLogNotificationConfigV2 {
    pub alert_rule_id: uuid::Uuid,
    pub notification_type: String,
    pub endpoint: String,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogPatternMatch {
    pub pattern: String,
    pub match_count: i64,
    pub first_match_at: chrono::DateTime<chrono::Utc>,
    pub last_match_at: chrono::DateTime<chrono::Utc>,
    pub affected_services: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogThresholdAlert {
    pub alert_rule_id: uuid::Uuid,
    pub rule_name: String,
    pub current_count: i64,
    pub threshold: i32,
    pub window_seconds: i32,
    pub triggered_at: chrono::DateTime<chrono::Utc>,
    pub level: LogLevel,
    pub pattern: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntryV8 {
    pub id: uuid::Uuid,
    pub level: LogLevel,
    pub message: String,
    pub source: String,
    pub service: String,
    pub trace_id: Option<String>,
    pub span_id: Option<String>,
    pub metadata: serde_json::Value,
    pub retention_days: i32,
    pub indexed: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateLogEntryV8 {
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
pub struct LogSearchFilterV8 {
    pub level: Option<LogLevel>,
    pub source: Option<String>,
    pub service: Option<String>,
    pub trace_id: Option<String>,
    pub search: Option<String>,
    pub indexed: Option<bool>,
    pub since: Option<chrono::DateTime<chrono::Utc>>,
    pub until: Option<chrono::DateTime<chrono::Utc>>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogSearchResultV8 {
    pub entries: Vec<LogEntryV8>,
    pub total_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogAlertRuleV5 {
    pub id: uuid::Uuid,
    pub name: String,
    pub level: LogLevel,
    pub pattern: String,
    pub threshold: i32,
    pub window_seconds: i32,
    pub enabled: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateLogAlertRuleV5 {
    pub name: String,
    pub level: LogLevel,
    pub pattern: String,
    pub threshold: Option<i32>,
    pub window_seconds: Option<i32>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateLogAlertRuleV5 {
    pub name: Option<String>,
    pub level: Option<LogLevel>,
    pub pattern: Option<String>,
    pub threshold: Option<i32>,
    pub window_seconds: Option<i32>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogPatternMatchV8 {
    pub pattern: String,
    pub match_count: i64,
    pub first_match_at: chrono::DateTime<chrono::Utc>,
    pub last_match_at: chrono::DateTime<chrono::Utc>,
    pub affected_services: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogThresholdAlertV8 {
    pub alert_rule_id: uuid::Uuid,
    pub rule_name: String,
    pub current_count: i64,
    pub threshold: i32,
    pub window_seconds: i32,
    pub triggered_at: chrono::DateTime<chrono::Utc>,
    pub level: LogLevel,
    pub pattern: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogNotificationConfigV3 {
    pub id: uuid::Uuid,
    pub alert_rule_id: uuid::Uuid,
    pub notification_type: String,
    pub endpoint: String,
    pub enabled: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateLogNotificationConfigV3 {
    pub alert_rule_id: uuid::Uuid,
    pub notification_type: String,
    pub endpoint: String,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntryV9 {
    pub id: uuid::Uuid,
    pub level: LogLevel,
    pub message: String,
    pub source: String,
    pub service: String,
    pub trace_id: Option<String>,
    pub span_id: Option<String>,
    pub metadata: serde_json::Value,
    pub retention_days: i32,
    pub indexed: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateLogEntryV9 {
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
pub struct LogSearchFilterV9 {
    pub level: Option<LogLevel>,
    pub source: Option<String>,
    pub service: Option<String>,
    pub trace_id: Option<String>,
    pub search: Option<String>,
    pub indexed: Option<bool>,
    pub since: Option<chrono::DateTime<chrono::Utc>>,
    pub until: Option<chrono::DateTime<chrono::Utc>>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogSearchResultV9 {
    pub entries: Vec<LogEntryV9>,
    pub total_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogAlertRuleV6 {
    pub id: uuid::Uuid,
    pub name: String,
    pub level: LogLevel,
    pub pattern: String,
    pub threshold: i32,
    pub window_seconds: i32,
    pub enabled: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateLogAlertRuleV6 {
    pub name: String,
    pub level: LogLevel,
    pub pattern: String,
    pub threshold: Option<i32>,
    pub window_seconds: Option<i32>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateLogAlertRuleV6 {
    pub name: Option<String>,
    pub level: Option<LogLevel>,
    pub pattern: Option<String>,
    pub threshold: Option<i32>,
    pub window_seconds: Option<i32>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogPatternMatchV9 {
    pub pattern: String,
    pub match_count: i64,
    pub first_match_at: chrono::DateTime<chrono::Utc>,
    pub last_match_at: chrono::DateTime<chrono::Utc>,
    pub affected_services: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogThresholdAlertV9 {
    pub alert_rule_id: uuid::Uuid,
    pub rule_name: String,
    pub current_count: i64,
    pub threshold: i32,
    pub window_seconds: i32,
    pub triggered_at: chrono::DateTime<chrono::Utc>,
    pub level: LogLevel,
    pub pattern: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntryV10 {
    pub id: uuid::Uuid,
    pub level: LogLevel,
    pub message: String,
    pub source: String,
    pub service: String,
    pub trace_id: Option<String>,
    pub span_id: Option<String>,
    pub metadata: serde_json::Value,
    pub retention_days: i32,
    pub indexed: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateLogEntryV10 {
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
pub struct LogSearchFilterV10 {
    pub level: Option<LogLevel>,
    pub source: Option<String>,
    pub service: Option<String>,
    pub trace_id: Option<String>,
    pub search: Option<String>,
    pub indexed: Option<bool>,
    pub since: Option<chrono::DateTime<chrono::Utc>>,
    pub until: Option<chrono::DateTime<chrono::Utc>>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogSearchResultV10 {
    pub entries: Vec<LogEntryV10>,
    pub total_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogAlertRuleV7 {
    pub id: uuid::Uuid,
    pub name: String,
    pub level: LogLevel,
    pub pattern: String,
    pub threshold: i32,
    pub window_seconds: i32,
    pub enabled: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateLogAlertRuleV7 {
    pub name: String,
    pub level: LogLevel,
    pub pattern: String,
    pub threshold: Option<i32>,
    pub window_seconds: Option<i32>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateLogAlertRuleV7 {
    pub name: Option<String>,
    pub level: Option<LogLevel>,
    pub pattern: Option<String>,
    pub threshold: Option<i32>,
    pub window_seconds: Option<i32>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogPatternMatchV10 {
    pub pattern: String,
    pub match_count: i64,
    pub first_match_at: chrono::DateTime<chrono::Utc>,
    pub last_match_at: chrono::DateTime<chrono::Utc>,
    pub affected_services: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogThresholdAlertV10 {
    pub alert_rule_id: uuid::Uuid,
    pub rule_name: String,
    pub current_count: i64,
    pub threshold: i32,
    pub window_seconds: i32,
    pub triggered_at: chrono::DateTime<chrono::Utc>,
    pub level: LogLevel,
    pub pattern: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogNotificationConfigV4 {
    pub id: uuid::Uuid,
    pub alert_rule_id: uuid::Uuid,
    pub notification_type: String,
    pub endpoint: String,
    pub enabled: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateLogNotificationConfigV4 {
    pub alert_rule_id: uuid::Uuid,
    pub notification_type: String,
    pub endpoint: String,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntryV11 {
    pub id: uuid::Uuid,
    pub level: LogLevel,
    pub message: String,
    pub source: String,
    pub service: String,
    pub trace_id: Option<String>,
    pub span_id: Option<String>,
    pub metadata: serde_json::Value,
    pub retention_days: i32,
    pub indexed: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateLogEntryV11 {
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
pub struct LogSearchFilterV11 {
    pub level: Option<LogLevel>,
    pub source: Option<String>,
    pub service: Option<String>,
    pub trace_id: Option<String>,
    pub search: Option<String>,
    pub indexed: Option<bool>,
    pub since: Option<chrono::DateTime<chrono::Utc>>,
    pub until: Option<chrono::DateTime<chrono::Utc>>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogSearchResultV11 {
    pub entries: Vec<LogEntryV11>,
    pub total_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogAlertRuleV8 {
    pub id: uuid::Uuid,
    pub name: String,
    pub level: LogLevel,
    pub pattern: String,
    pub threshold: i32,
    pub window_seconds: i32,
    pub enabled: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateLogAlertRuleV8 {
    pub name: String,
    pub level: LogLevel,
    pub pattern: String,
    pub threshold: Option<i32>,
    pub window_seconds: Option<i32>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateLogAlertRuleV8 {
    pub name: Option<String>,
    pub level: Option<LogLevel>,
    pub pattern: Option<String>,
    pub threshold: Option<i32>,
    pub window_seconds: Option<i32>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogPatternMatchV11 {
    pub pattern: String,
    pub match_count: i64,
    pub first_match_at: chrono::DateTime<chrono::Utc>,
    pub last_match_at: chrono::DateTime<chrono::Utc>,
    pub affected_services: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogThresholdAlertV11 {
    pub alert_rule_id: uuid::Uuid,
    pub rule_name: String,
    pub current_count: i64,
    pub threshold: i32,
    pub window_seconds: i32,
    pub triggered_at: chrono::DateTime<chrono::Utc>,
    pub level: LogLevel,
    pub pattern: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogNotificationConfigV5 {
    pub id: uuid::Uuid,
    pub alert_rule_id: uuid::Uuid,
    pub notification_type: String,
    pub endpoint: String,
    pub enabled: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateLogNotificationConfigV5 {
    pub alert_rule_id: uuid::Uuid,
    pub notification_type: String,
    pub endpoint: String,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntryV12 {
    pub id: uuid::Uuid,
    pub level: LogLevel,
    pub message: String,
    pub source: String,
    pub service: String,
    pub trace_id: Option<String>,
    pub span_id: Option<String>,
    pub metadata: serde_json::Value,
    pub retention_days: i32,
    pub indexed: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateLogEntryV12 {
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
pub struct LogSearchFilterV12 {
    pub level: Option<LogLevel>,
    pub source: Option<String>,
    pub service: Option<String>,
    pub trace_id: Option<String>,
    pub search: Option<String>,
    pub indexed: Option<bool>,
    pub since: Option<chrono::DateTime<chrono::Utc>>,
    pub until: Option<chrono::DateTime<chrono::Utc>>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogSearchResultV12 {
    pub entries: Vec<LogEntryV12>,
    pub total_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogAlertRuleV9 {
    pub id: uuid::Uuid,
    pub name: String,
    pub level: LogLevel,
    pub pattern: String,
    pub threshold: i32,
    pub window_seconds: i32,
    pub enabled: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateLogAlertRuleV9 {
    pub name: String,
    pub level: LogLevel,
    pub pattern: String,
    pub threshold: Option<i32>,
    pub window_seconds: Option<i32>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateLogAlertRuleV9 {
    pub name: Option<String>,
    pub level: Option<LogLevel>,
    pub pattern: Option<String>,
    pub threshold: Option<i32>,
    pub window_seconds: Option<i32>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntryV13 {
    pub id: uuid::Uuid,
    pub level: LogLevel,
    pub message: String,
    pub source: String,
    pub service: String,
    pub trace_id: Option<String>,
    pub span_id: Option<String>,
    pub metadata: serde_json::Value,
    pub retention_days: i32,
    pub indexed: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateLogEntryV13 {
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
pub struct LogSearchFilterV13 {
    pub level: Option<LogLevel>,
    pub source: Option<String>,
    pub service: Option<String>,
    pub trace_id: Option<String>,
    pub search: Option<String>,
    pub indexed: Option<bool>,
    pub since: Option<chrono::DateTime<chrono::Utc>>,
    pub until: Option<chrono::DateTime<chrono::Utc>>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogSearchResultV13 {
    pub entries: Vec<LogEntryV13>,
    pub total_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogAlertRuleV10 {
    pub id: uuid::Uuid,
    pub name: String,
    pub level: LogLevel,
    pub pattern: String,
    pub threshold: i32,
    pub window_seconds: i32,
    pub enabled: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateLogAlertRuleV10 {
    pub name: String,
    pub level: LogLevel,
    pub pattern: String,
    pub threshold: Option<i32>,
    pub window_seconds: Option<i32>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateLogAlertRuleV10 {
    pub name: Option<String>,
    pub level: Option<LogLevel>,
    pub pattern: Option<String>,
    pub threshold: Option<i32>,
    pub window_seconds: Option<i32>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogPatternMatchV13 {
    pub pattern: String,
    pub match_count: i64,
    pub first_match_at: chrono::DateTime<chrono::Utc>,
    pub last_match_at: chrono::DateTime<chrono::Utc>,
    pub affected_services: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogThresholdAlertV13 {
    pub alert_rule_id: uuid::Uuid,
    pub rule_name: String,
    pub current_count: i64,
    pub threshold: i32,
    pub window_seconds: i32,
    pub triggered_at: chrono::DateTime<chrono::Utc>,
    pub level: LogLevel,
    pub pattern: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntryV14 {
    pub id: uuid::Uuid,
    pub level: LogLevel,
    pub message: String,
    pub source: String,
    pub service: String,
    pub trace_id: Option<String>,
    pub span_id: Option<String>,
    pub metadata: serde_json::Value,
    pub retention_days: i32,
    pub indexed: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateLogEntryV14 {
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
pub struct LogSearchFilterV14 {
    pub level: Option<LogLevel>,
    pub source: Option<String>,
    pub service: Option<String>,
    pub trace_id: Option<String>,
    pub search: Option<String>,
    pub indexed: Option<bool>,
    pub since: Option<chrono::DateTime<chrono::Utc>>,
    pub until: Option<chrono::DateTime<chrono::Utc>>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogSearchResultV14 {
    pub entries: Vec<LogEntryV14>,
    pub total_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogAlertRuleV11 {
    pub id: uuid::Uuid,
    pub name: String,
    pub level: LogLevel,
    pub pattern: String,
    pub threshold: i32,
    pub window_seconds: i32,
    pub enabled: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateLogAlertRuleV11 {
    pub name: String,
    pub level: LogLevel,
    pub pattern: String,
    pub threshold: Option<i32>,
    pub window_seconds: Option<i32>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateLogAlertRuleV11 {
    pub name: Option<String>,
    pub level: Option<LogLevel>,
    pub pattern: Option<String>,
    pub threshold: Option<i32>,
    pub window_seconds: Option<i32>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogNotificationConfigV6 {
    pub id: uuid::Uuid,
    pub alert_rule_id: uuid::Uuid,
    pub notification_type: String,
    pub endpoint: String,
    pub enabled: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateLogNotificationConfigV6 {
    pub alert_rule_id: uuid::Uuid,
    pub notification_type: String,
    pub endpoint: String,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogPatternMatchV14 {
    pub pattern: String,
    pub match_count: i64,
    pub first_match_at: chrono::DateTime<chrono::Utc>,
    pub last_match_at: chrono::DateTime<chrono::Utc>,
    pub affected_services: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogThresholdAlertV14 {
    pub alert_rule_id: uuid::Uuid,
    pub rule_name: String,
    pub current_count: i64,
    pub threshold: i32,
    pub window_seconds: i32,
    pub triggered_at: chrono::DateTime<chrono::Utc>,
    pub level: LogLevel,
    pub pattern: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntryV15 {
    pub id: uuid::Uuid,
    pub level: LogLevel,
    pub message: String,
    pub source: String,
    pub service: String,
    pub trace_id: Option<String>,
    pub span_id: Option<String>,
    pub metadata: serde_json::Value,
    pub retention_days: i32,
    pub indexed: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateLogEntryV15 {
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
pub struct LogSearchFilterV15 {
    pub level: Option<LogLevel>,
    pub source: Option<String>,
    pub service: Option<String>,
    pub trace_id: Option<String>,
    pub search: Option<String>,
    pub indexed: Option<bool>,
    pub since: Option<chrono::DateTime<chrono::Utc>>,
    pub until: Option<chrono::DateTime<chrono::Utc>>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogSearchResultV15 {
    pub entries: Vec<LogEntryV15>,
    pub total_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogAlertRuleV12 {
    pub id: uuid::Uuid,
    pub name: String,
    pub level: LogLevel,
    pub pattern: String,
    pub threshold: i32,
    pub window_seconds: i32,
    pub enabled: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateLogAlertRuleV12 {
    pub name: String,
    pub level: LogLevel,
    pub pattern: String,
    pub threshold: Option<i32>,
    pub window_seconds: Option<i32>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateLogAlertRuleV12 {
    pub name: Option<String>,
    pub level: Option<LogLevel>,
    pub pattern: Option<String>,
    pub threshold: Option<i32>,
    pub window_seconds: Option<i32>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogPatternMatchV15 {
    pub pattern: String,
    pub match_count: i64,
    pub first_match_at: chrono::DateTime<chrono::Utc>,
    pub last_match_at: chrono::DateTime<chrono::Utc>,
    pub affected_services: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogThresholdAlertV15 {
    pub alert_rule_id: uuid::Uuid,
    pub rule_name: String,
    pub current_count: i64,
    pub threshold: i32,
    pub window_seconds: i32,
    pub triggered_at: chrono::DateTime<chrono::Utc>,
    pub level: LogLevel,
    pub pattern: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntryV16 {
    pub id: uuid::Uuid,
    pub level: LogLevel,
    pub message: String,
    pub source: String,
    pub service: String,
    pub trace_id: Option<String>,
    pub span_id: Option<String>,
    pub metadata: serde_json::Value,
    pub retention_days: i32,
    pub indexed: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateLogEntryV16 {
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
pub struct LogSearchFilterV16 {
    pub level: Option<LogLevel>,
    pub source: Option<String>,
    pub service: Option<String>,
    pub trace_id: Option<String>,
    pub search: Option<String>,
    pub indexed: Option<bool>,
    pub since: Option<chrono::DateTime<chrono::Utc>>,
    pub until: Option<chrono::DateTime<chrono::Utc>>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogSearchResultV16 {
    pub entries: Vec<LogEntryV16>,
    pub total_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogAlertRuleV13 {
    pub id: uuid::Uuid,
    pub name: String,
    pub level: LogLevel,
    pub pattern: String,
    pub threshold: i32,
    pub window_seconds: i32,
    pub enabled: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateLogAlertRuleV13 {
    pub name: String,
    pub level: LogLevel,
    pub pattern: String,
    pub threshold: Option<i32>,
    pub window_seconds: Option<i32>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateLogAlertRuleV13 {
    pub name: Option<String>,
    pub level: Option<LogLevel>,
    pub pattern: Option<String>,
    pub threshold: Option<i32>,
    pub window_seconds: Option<i32>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogPatternMatchV16 {
    pub pattern: String,
    pub match_count: i64,
    pub first_match_at: chrono::DateTime<chrono::Utc>,
    pub last_match_at: chrono::DateTime<chrono::Utc>,
    pub affected_services: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogThresholdAlertV16 {
    pub alert_rule_id: uuid::Uuid,
    pub rule_name: String,
    pub current_count: i64,
    pub threshold: i32,
    pub window_seconds: i32,
    pub triggered_at: chrono::DateTime<chrono::Utc>,
    pub level: LogLevel,
    pub pattern: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogNotificationConfigV7 {
    pub id: uuid::Uuid,
    pub alert_rule_id: uuid::Uuid,
    pub notification_type: String,
    pub endpoint: String,
    pub enabled: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateLogNotificationConfigV7 {
    pub alert_rule_id: uuid::Uuid,
    pub notification_type: String,
    pub endpoint: String,
    pub enabled: Option<bool>,
}

// V17: Log entries with enhanced indexing and alert rules v14

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntryV17 {
    pub id: uuid::Uuid,
    pub level: LogLevel,
    pub message: String,
    pub source: String,
    pub service: String,
    pub trace_id: Option<String>,
    pub span_id: Option<String>,
    pub metadata: serde_json::Value,
    pub retention_days: i32,
    pub indexed: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateLogEntryV17 {
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
pub struct LogSearchFilterV17 {
    pub level: Option<LogLevel>,
    pub source: Option<String>,
    pub service: Option<String>,
    pub trace_id: Option<String>,
    pub search: Option<String>,
    pub indexed: Option<bool>,
    pub since: Option<chrono::DateTime<chrono::Utc>>,
    pub until: Option<chrono::DateTime<chrono::Utc>>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogSearchResultV17 {
    pub entries: Vec<LogEntryV17>,
    pub total_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogAlertRuleV14 {
    pub id: uuid::Uuid,
    pub name: String,
    pub level: LogLevel,
    pub pattern: String,
    pub threshold: i32,
    pub window_seconds: i32,
    pub enabled: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateLogAlertRuleV14 {
    pub name: String,
    pub level: LogLevel,
    pub pattern: String,
    pub threshold: Option<i32>,
    pub window_seconds: Option<i32>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateLogAlertRuleV14 {
    pub name: Option<String>,
    pub level: Option<LogLevel>,
    pub pattern: Option<String>,
    pub threshold: Option<i32>,
    pub window_seconds: Option<i32>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogPatternMatchV17 {
    pub pattern: String,
    pub match_count: i64,
    pub first_match_at: chrono::DateTime<chrono::Utc>,
    pub last_match_at: chrono::DateTime<chrono::Utc>,
    pub affected_services: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogThresholdAlertV17 {
    pub alert_rule_id: uuid::Uuid,
    pub rule_name: String,
    pub current_count: i64,
    pub threshold: i32,
    pub window_seconds: i32,
    pub triggered_at: chrono::DateTime<chrono::Utc>,
    pub level: LogLevel,
    pub pattern: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogNotificationConfigV8 {
    pub id: uuid::Uuid,
    pub alert_rule_id: uuid::Uuid,
    pub notification_type: String,
    pub endpoint: String,
    pub enabled: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateLogNotificationConfigV8 {
    pub alert_rule_id: uuid::Uuid,
    pub notification_type: String,
    pub endpoint: String,
    pub enabled: Option<bool>,
}

// V18: Enhanced log aggregation with pattern matching and threshold monitoring

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntryV18 {
    pub id: uuid::Uuid,
    pub level: LogLevel,
    pub message: String,
    pub source: String,
    pub service: String,
    pub trace_id: Option<String>,
    pub span_id: Option<String>,
    pub metadata: serde_json::Value,
    pub retention_days: i32,
    pub indexed: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateLogEntryV18 {
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
pub struct LogSearchFilterV18 {
    pub level: Option<LogLevel>,
    pub source: Option<String>,
    pub service: Option<String>,
    pub trace_id: Option<String>,
    pub search: Option<String>,
    pub indexed: Option<bool>,
    pub since: Option<chrono::DateTime<chrono::Utc>>,
    pub until: Option<chrono::DateTime<chrono::Utc>>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogSearchResultV18 {
    pub entries: Vec<LogEntryV18>,
    pub total_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogAlertRuleV15 {
    pub id: uuid::Uuid,
    pub name: String,
    pub level: LogLevel,
    pub pattern: String,
    pub threshold: i32,
    pub window_seconds: i32,
    pub enabled: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateLogAlertRuleV15 {
    pub name: String,
    pub level: LogLevel,
    pub pattern: String,
    pub threshold: Option<i32>,
    pub window_seconds: Option<i32>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateLogAlertRuleV15 {
    pub name: Option<String>,
    pub level: Option<LogLevel>,
    pub pattern: Option<String>,
    pub threshold: Option<i32>,
    pub window_seconds: Option<i32>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogPatternMatchV18 {
    pub pattern: String,
    pub match_count: i64,
    pub first_match_at: chrono::DateTime<chrono::Utc>,
    pub last_match_at: chrono::DateTime<chrono::Utc>,
    pub affected_services: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogThresholdAlertV18 {
    pub alert_rule_id: uuid::Uuid,
    pub rule_name: String,
    pub current_count: i64,
    pub threshold: i32,
    pub window_seconds: i32,
    pub triggered_at: chrono::DateTime<chrono::Utc>,
    pub level: LogLevel,
    pub pattern: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogNotificationConfigV9 {
    pub id: uuid::Uuid,
    pub alert_rule_id: uuid::Uuid,
    pub notification_type: String,
    pub endpoint: String,
    pub enabled: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateLogNotificationConfigV9 {
    pub alert_rule_id: uuid::Uuid,
    pub notification_type: String,
    pub endpoint: String,
    pub enabled: Option<bool>,
}

// V19: Enhanced log aggregation with advanced pattern matching and threshold monitoring

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntryV19 {
    pub id: uuid::Uuid,
    pub level: LogLevel,
    pub message: String,
    pub source: String,
    pub service: String,
    pub trace_id: Option<String>,
    pub span_id: Option<String>,
    pub metadata: serde_json::Value,
    pub retention_days: i32,
    pub indexed: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateLogEntryV19 {
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
pub struct LogSearchFilterV19 {
    pub level: Option<LogLevel>,
    pub source: Option<String>,
    pub service: Option<String>,
    pub trace_id: Option<String>,
    pub search: Option<String>,
    pub indexed: Option<bool>,
    pub since: Option<chrono::DateTime<chrono::Utc>>,
    pub until: Option<chrono::DateTime<chrono::Utc>>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogSearchResultV19 {
    pub entries: Vec<LogEntryV19>,
    pub total_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogAlertRuleV16 {
    pub id: uuid::Uuid,
    pub name: String,
    pub level: LogLevel,
    pub pattern: String,
    pub threshold: i32,
    pub window_seconds: i32,
    pub enabled: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateLogAlertRuleV16 {
    pub name: String,
    pub level: LogLevel,
    pub pattern: String,
    pub threshold: Option<i32>,
    pub window_seconds: Option<i32>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateLogAlertRuleV16 {
    pub name: Option<String>,
    pub level: Option<LogLevel>,
    pub pattern: Option<String>,
    pub threshold: Option<i32>,
    pub window_seconds: Option<i32>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogPatternMatchV19 {
    pub pattern: String,
    pub match_count: i64,
    pub first_match_at: chrono::DateTime<chrono::Utc>,
    pub last_match_at: chrono::DateTime<chrono::Utc>,
    pub affected_services: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogThresholdAlertV19 {
    pub alert_rule_id: uuid::Uuid,
    pub rule_name: String,
    pub current_count: i64,
    pub threshold: i32,
    pub window_seconds: i32,
    pub triggered_at: chrono::DateTime<chrono::Utc>,
    pub level: LogLevel,
    pub pattern: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogNotificationConfigV10 {
    pub id: uuid::Uuid,
    pub alert_rule_id: uuid::Uuid,
    pub notification_type: String,
    pub endpoint: String,
    pub enabled: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateLogNotificationConfigV10 {
    pub alert_rule_id: uuid::Uuid,
    pub notification_type: String,
    pub endpoint: String,
    pub enabled: Option<bool>,
}

// V20: Log entries v20 and alert rules v17

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntryV20 {
    pub id: uuid::Uuid,
    pub level: LogLevel,
    pub message: String,
    pub source: String,
    pub service: String,
    pub trace_id: Option<String>,
    pub span_id: Option<String>,
    pub metadata: serde_json::Value,
    pub retention_days: i32,
    pub indexed: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateLogEntryV20 {
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
pub struct LogSearchFilterV20 {
    pub level: Option<LogLevel>,
    pub source: Option<String>,
    pub service: Option<String>,
    pub trace_id: Option<String>,
    pub search: Option<String>,
    pub indexed: Option<bool>,
    pub since: Option<chrono::DateTime<chrono::Utc>>,
    pub until: Option<chrono::DateTime<chrono::Utc>>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogSearchResultV20 {
    pub entries: Vec<LogEntryV20>,
    pub total_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogAlertRuleV17 {
    pub id: uuid::Uuid,
    pub name: String,
    pub level: LogLevel,
    pub pattern: String,
    pub threshold: i32,
    pub window_seconds: i32,
    pub enabled: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateLogAlertRuleV17 {
    pub name: String,
    pub level: LogLevel,
    pub pattern: String,
    pub threshold: Option<i32>,
    pub window_seconds: Option<i32>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateLogAlertRuleV17 {
    pub name: Option<String>,
    pub level: Option<LogLevel>,
    pub pattern: Option<String>,
    pub threshold: Option<i32>,
    pub window_seconds: Option<i32>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogPatternMatchV20 {
    pub pattern: String,
    pub match_count: i64,
    pub first_match_at: chrono::DateTime<chrono::Utc>,
    pub last_match_at: chrono::DateTime<chrono::Utc>,
    pub affected_services: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogThresholdAlertV20 {
    pub alert_rule_id: uuid::Uuid,
    pub rule_name: String,
    pub current_count: i64,
    pub threshold: i32,
    pub window_seconds: i32,
    pub triggered_at: chrono::DateTime<chrono::Utc>,
    pub level: LogLevel,
    pub pattern: String,
}

// V21: Log entries v21 and alert rules v18

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntryV21 {
    pub id: uuid::Uuid,
    pub level: LogLevel,
    pub message: String,
    pub source: String,
    pub service: String,
    pub trace_id: Option<String>,
    pub span_id: Option<String>,
    pub metadata: serde_json::Value,
    pub retention_days: i32,
    pub indexed: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateLogEntryV21 {
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
pub struct LogSearchFilterV21 {
    pub level: Option<LogLevel>,
    pub source: Option<String>,
    pub service: Option<String>,
    pub trace_id: Option<String>,
    pub search: Option<String>,
    pub indexed: Option<bool>,
    pub since: Option<chrono::DateTime<chrono::Utc>>,
    pub until: Option<chrono::DateTime<chrono::Utc>>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogSearchResultV21 {
    pub entries: Vec<LogEntryV21>,
    pub total_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogAlertRuleV18 {
    pub id: uuid::Uuid,
    pub name: String,
    pub level: LogLevel,
    pub pattern: String,
    pub threshold: i32,
    pub window_seconds: i32,
    pub enabled: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateLogAlertRuleV18 {
    pub name: String,
    pub level: LogLevel,
    pub pattern: String,
    pub threshold: Option<i32>,
    pub window_seconds: Option<i32>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateLogAlertRuleV18 {
    pub name: Option<String>,
    pub level: Option<LogLevel>,
    pub pattern: Option<String>,
    pub threshold: Option<i32>,
    pub window_seconds: Option<i32>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogPatternMatchV21 {
    pub pattern: String,
    pub match_count: i64,
    pub first_match_at: chrono::DateTime<chrono::Utc>,
    pub last_match_at: chrono::DateTime<chrono::Utc>,
    pub affected_services: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogThresholdAlertV21 {
    pub alert_rule_id: uuid::Uuid,
    pub rule_name: String,
    pub current_count: i64,
    pub threshold: i32,
    pub window_seconds: i32,
    pub triggered_at: chrono::DateTime<chrono::Utc>,
    pub level: LogLevel,
    pub pattern: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogNotificationConfigV11 {
    pub id: uuid::Uuid,
    pub alert_rule_id: uuid::Uuid,
    pub notification_type: String,
    pub endpoint: String,
    pub enabled: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateLogNotificationConfigV11 {
    pub alert_rule_id: uuid::Uuid,
    pub notification_type: String,
    pub endpoint: String,
    pub enabled: Option<bool>,
}

// V22 types (from log_aggregation_v22.rs)

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogRetentionPolicyV19 {
    pub id: Uuid,
    pub service: String,
    pub level: String,
    pub retention_days: i32,
    pub archive_after_days: Option<i32>,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateLogRetentionPolicyV19 {
    pub service: String,
    pub level: String,
    pub retention_days: Option<i32>,
    pub archive_after_days: Option<i32>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateLogRetentionPolicyV19 {
    pub service: Option<String>,
    pub level: Option<String>,
    pub retention_days: Option<i32>,
    pub archive_after_days: Option<i32>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogArchiveV19 {
    pub id: Uuid,
    pub service: String,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub entry_count: i64,
    pub size_bytes: i64,
    pub archive_path: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateLogArchiveV19 {
    pub service: String,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub entry_count: i64,
    pub size_bytes: i64,
    pub archive_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogArchiveStatsV19 {
    pub total_archives: i64,
    pub total_size_bytes: i64,
    pub total_entries: i64,
    pub service_counts: std::collections::HashMap<String, i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogLifecycleEventV19 {
    pub id: Uuid,
    pub log_id: Uuid,
    pub event_type: String,
    pub details: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogSearchOptimizationV22 {
    pub index_name: String,
    pub index_size_bytes: i64,
    pub query_count: i64,
    pub avg_query_time_ms: f64,
    pub last_optimized_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogSearchPerformanceV22 {
    pub total_queries: i64,
    pub avg_response_time_ms: f64,
    pub p95_response_time_ms: f64,
    pub cache_hit_rate: f64,
    pub index_efficiency: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogRetentionStatsV19 {
    pub active_policies: i64,
    pub total_entries_managed: i64,
    pub entries_archived: i64,
    pub entries_deleted: i64,
    pub last_cleanup_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogArchiveRequestV19 {
    pub service: Option<String>,
    pub before_date: DateTime<Utc>,
    pub dry_run: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogArchiveResultV19 {
    pub entries_archived: i64,
    pub size_bytes: i64,
    pub archive_path: String,
    pub duration_ms: i64,
}

// V23 types (from log_aggregation_v23.rs)

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogIndexOptimizationV20 {
    pub id: Uuid,
    pub index_name: String,
    pub table_name: String,
    pub columns: Vec<String>,
    pub query_pattern: Option<String>,
    pub improvement_percent: Option<f64>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateLogIndexOptimizationV20 {
    pub index_name: String,
    pub table_name: String,
    pub columns: Vec<String>,
    pub query_pattern: Option<String>,
    pub improvement_percent: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogCompressionStatsV20 {
    pub id: Uuid,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub original_bytes: i64,
    pub compressed_bytes: i64,
    pub compression_ratio: f64,
    pub entry_count: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateLogCompressionStatsV20 {
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub original_bytes: i64,
    pub compressed_bytes: i64,
    pub compression_ratio: f64,
    pub entry_count: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogIndexOptimizationSummaryV20 {
    pub total_optimizations: i64,
    pub avg_improvement_percent: f64,
    pub tables_optimized: Vec<String>,
    pub last_optimized_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogCompressionSummaryV20 {
    pub total_periods: i64,
    pub total_original_bytes: i64,
    pub total_compressed_bytes: i64,
    pub avg_compression_ratio: f64,
    pub total_entries_compressed: i64,
    pub last_compressed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogQueryPerformanceV23 {
    pub query_pattern: String,
    pub avg_execution_time_ms: f64,
    pub p95_execution_time_ms: f64,
    pub total_executions: i64,
    pub suggested_index: Option<String>,
    pub estimated_improvement_percent: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogStorageOptimizationV23 {
    pub table_name: String,
    pub current_size_bytes: i64,
    pub estimated_optimizable_bytes: i64,
    pub optimization_suggestions: Vec<String>,
    pub last_analyzed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogIndexOptimizationRequestV20 {
    pub table_name: Option<String>,
    pub limit: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogCompressionRequestV20 {
    pub since: Option<DateTime<Utc>>,
    pub until: Option<DateTime<Utc>>,
    pub limit: Option<i64>,
}
