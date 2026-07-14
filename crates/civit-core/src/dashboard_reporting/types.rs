use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dashboard {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub widgets: serde_json::Value,
    pub layout: serde_json::Value,
    pub is_public: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateDashboard {
    pub name: String,
    pub description: Option<String>,
    pub widgets: Option<serde_json::Value>,
    pub layout: Option<serde_json::Value>,
    pub is_public: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateDashboard {
    pub name: Option<String>,
    pub description: Option<String>,
    pub widgets: Option<serde_json::Value>,
    pub layout: Option<serde_json::Value>,
    pub is_public: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Widget {
    pub widget_type: String,
    pub title: String,
    pub config: serde_json::Value,
    pub position: WidgetPosition,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WidgetPosition {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Report {
    pub id: Uuid,
    pub name: String,
    pub report_type: String,
    pub config: serde_json::Value,
    pub schedule: Option<String>,
    pub last_generated_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateReport {
    pub name: String,
    pub report_type: String,
    pub config: Option<serde_json::Value>,
    pub schedule: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateReport {
    pub name: Option<String>,
    pub report_type: Option<String>,
    pub config: Option<serde_json::Value>,
    pub schedule: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportData {
    pub report_id: Uuid,
    pub name: String,
    pub report_type: String,
    pub data: serde_json::Value,
    pub generated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportSchedule {
    pub report_id: Uuid,
    pub cron_expression: String,
    pub enabled: bool,
    pub next_run_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardStats {
    pub total_dashboards: i64,
    pub public_dashboards: i64,
    pub total_reports: i64,
    pub scheduled_reports: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardWidgetV2 {
    pub id: Uuid,
    pub dashboard_id: Uuid,
    pub widget_type: String,
    pub config: serde_json::Value,
    pub position: serde_json::Value,
    pub size: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateDashboardWidgetV2 {
    pub dashboard_id: Uuid,
    pub widget_type: String,
    pub config: Option<serde_json::Value>,
    pub position: Option<serde_json::Value>,
    pub size: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateDashboardWidgetV2 {
    pub widget_type: Option<String>,
    pub config: Option<serde_json::Value>,
    pub position: Option<serde_json::Value>,
    pub size: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportScheduleV2 {
    pub id: Uuid,
    pub report_id: Uuid,
    pub cron_expression: String,
    pub enabled: bool,
    pub last_run_at: Option<DateTime<Utc>>,
    pub next_run_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateReportScheduleV2 {
    pub report_id: Uuid,
    pub cron_expression: String,
    pub enabled: Option<bool>,
    pub next_run_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateReportScheduleV2 {
    pub cron_expression: Option<String>,
    pub enabled: Option<bool>,
    pub next_run_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardShareRequest {
    pub dashboard_id: Uuid,
    pub user_id: Uuid,
    pub permission: DashboardPermission,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum DashboardPermission {
    View,
    Edit,
    Admin,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardShare {
    pub id: Uuid,
    pub dashboard_id: Uuid,
    pub user_id: Uuid,
    pub permission: DashboardPermission,
    pub shared_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportExportRequest {
    pub report_id: Uuid,
    pub format: ReportExportFormat,
    pub since: Option<DateTime<Utc>>,
    pub until: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ReportExportFormat {
    Json,
    Csv,
    Pdf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportExportResult {
    pub report_id: Uuid,
    pub format: ReportExportFormat,
    pub data: serde_json::Value,
    pub exported_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardTemplate {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub template_type: String,
    pub config: serde_json::Value,
    pub is_public: bool,
    pub author_id: Option<Uuid>,
    pub usage_count: i64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateDashboardTemplate {
    pub name: String,
    pub description: Option<String>,
    pub template_type: String,
    pub config: Option<serde_json::Value>,
    pub is_public: Option<bool>,
    pub author_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateDashboardTemplate {
    pub name: Option<String>,
    pub description: Option<String>,
    pub template_type: Option<String>,
    pub config: Option<serde_json::Value>,
    pub is_public: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportTemplate {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub report_type: String,
    pub config: serde_json::Value,
    pub is_public: bool,
    pub author_id: Option<Uuid>,
    pub usage_count: i64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateReportTemplate {
    pub name: String,
    pub description: Option<String>,
    pub report_type: String,
    pub config: Option<serde_json::Value>,
    pub is_public: Option<bool>,
    pub author_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateReportTemplate {
    pub name: Option<String>,
    pub description: Option<String>,
    pub report_type: Option<String>,
    pub config: Option<serde_json::Value>,
    pub is_public: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateMarketplaceItem {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub template_type: String,
    pub author_id: Option<Uuid>,
    pub usage_count: i64,
    pub rating: Option<f64>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateAnalytics {
    pub template_id: Uuid,
    pub usage_count: i64,
    pub last_used_at: Option<DateTime<Utc>>,
    pub popular_configs: Vec<serde_json::Value>,
}
