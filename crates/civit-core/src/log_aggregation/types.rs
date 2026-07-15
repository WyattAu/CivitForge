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
