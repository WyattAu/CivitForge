use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardShareV2 {
    pub id: Uuid,
    pub dashboard_id: Uuid,
    pub user_id: Uuid,
    pub permission: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateDashboardShareV2 {
    pub dashboard_id: Uuid,
    pub user_id: Uuid,
    pub permission: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportScheduleV3 {
    pub id: Uuid,
    pub report_id: Uuid,
    pub cron_expression: String,
    pub enabled: bool,
    pub last_run_at: Option<DateTime<Utc>>,
    pub next_run_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateReportScheduleV3 {
    pub report_id: Uuid,
    pub cron_expression: String,
    pub enabled: Option<bool>,
    pub next_run_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateReportScheduleV3 {
    pub cron_expression: Option<String>,
    pub enabled: Option<bool>,
    pub next_run_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardAnalytics {
    pub dashboard_id: Uuid,
    pub view_count: i64,
    pub unique_viewers: i64,
    pub last_viewed_at: Option<DateTime<Utc>>,
    pub widget_interactions: i64,
    pub avg_view_duration_seconds: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportAnalytics {
    pub report_id: Uuid,
    pub generation_count: i64,
    pub avg_generation_time_ms: f64,
    pub last_generated_at: Option<DateTime<Utc>>,
    pub export_count: i64,
    pub popular_format: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardStatsV5 {
    pub total_dashboards: i64,
    pub public_dashboards: i64,
    pub total_reports: i64,
    pub scheduled_reports: i64,
    pub total_shares: i64,
    pub total_schedules: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardShareV3 {
    pub id: Uuid,
    pub dashboard_id: Uuid,
    pub user_id: Uuid,
    pub permission: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateDashboardShareV3 {
    pub dashboard_id: Uuid,
    pub user_id: Uuid,
    pub permission: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportScheduleV4 {
    pub id: Uuid,
    pub report_id: Uuid,
    pub cron_expression: String,
    pub enabled: bool,
    pub last_run_at: Option<DateTime<Utc>>,
    pub next_run_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateReportScheduleV4 {
    pub report_id: Uuid,
    pub cron_expression: String,
    pub enabled: Option<bool>,
    pub next_run_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateReportScheduleV4 {
    pub cron_expression: Option<String>,
    pub enabled: Option<bool>,
    pub next_run_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardAnalyticsV2 {
    pub dashboard_id: Uuid,
    pub view_count: i64,
    pub unique_viewers: i64,
    pub last_viewed_at: Option<DateTime<Utc>>,
    pub widget_interactions: i64,
    pub avg_view_duration_seconds: f64,
    pub share_count: i64,
    pub last_shared_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportAnalyticsV2 {
    pub report_id: Uuid,
    pub generation_count: i64,
    pub avg_generation_time_ms: f64,
    pub last_generated_at: Option<DateTime<Utc>>,
    pub export_count: i64,
    pub popular_format: String,
    pub schedule_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardStatsV6 {
    pub total_dashboards: i64,
    pub public_dashboards: i64,
    pub total_reports: i64,
    pub scheduled_reports: i64,
    pub total_shares: i64,
    pub total_schedules: i64,
    pub avg_shares_per_dashboard: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardShareV4 {
    pub id: Uuid,
    pub dashboard_id: Uuid,
    pub user_id: Uuid,
    pub permission: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateDashboardShareV4 {
    pub dashboard_id: Uuid,
    pub user_id: Uuid,
    pub permission: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportScheduleV5 {
    pub id: Uuid,
    pub report_id: Uuid,
    pub cron_expression: String,
    pub enabled: bool,
    pub last_run_at: Option<DateTime<Utc>>,
    pub next_run_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateReportScheduleV5 {
    pub report_id: Uuid,
    pub cron_expression: String,
    pub enabled: Option<bool>,
    pub next_run_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateReportScheduleV5 {
    pub cron_expression: Option<String>,
    pub enabled: Option<bool>,
    pub next_run_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardAnalyticsV3 {
    pub dashboard_id: Uuid,
    pub view_count: i64,
    pub unique_viewers: i64,
    pub last_viewed_at: Option<DateTime<Utc>>,
    pub widget_interactions: i64,
    pub avg_view_duration_seconds: f64,
    pub share_count: i64,
    pub last_shared_at: Option<DateTime<Utc>>,
    pub export_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportAnalyticsV3 {
    pub report_id: Uuid,
    pub generation_count: i64,
    pub avg_generation_time_ms: f64,
    pub last_generated_at: Option<DateTime<Utc>>,
    pub export_count: i64,
    pub popular_format: String,
    pub schedule_count: i64,
    pub success_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardStatsV7 {
    pub total_dashboards: i64,
    pub public_dashboards: i64,
    pub total_reports: i64,
    pub scheduled_reports: i64,
    pub total_shares: i64,
    pub total_schedules: i64,
    pub avg_shares_per_dashboard: f64,
    pub total_views: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardTemplateV2 {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub template_type: String,
    pub config: serde_json::Value,
    pub is_public: bool,
    pub author_id: Option<Uuid>,
    pub usage_count: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateDashboardTemplateV2 {
    pub name: String,
    pub description: Option<String>,
    pub template_type: String,
    pub config: Option<serde_json::Value>,
    pub is_public: Option<bool>,
    pub author_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportTemplateV2 {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub report_type: String,
    pub config: serde_json::Value,
    pub is_public: bool,
    pub author_id: Option<Uuid>,
    pub usage_count: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateReportTemplateV2 {
    pub name: String,
    pub description: Option<String>,
    pub report_type: String,
    pub config: Option<serde_json::Value>,
    pub is_public: Option<bool>,
    pub author_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateMarketplaceItemV2 {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub template_type: String,
    pub author_id: Option<Uuid>,
    pub usage_count: i64,
    pub rating: Option<f64>,
    pub tags: Vec<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateAnalyticsV2 {
    pub template_id: Uuid,
    pub usage_count: i64,
    pub last_used_at: Option<DateTime<Utc>>,
    pub popular_configs: Vec<serde_json::Value>,
    pub avg_rating: Option<f64>,
    pub total_ratings: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardShareV5 {
    pub id: Uuid,
    pub dashboard_id: Uuid,
    pub user_id: Uuid,
    pub permission: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateDashboardShareV5 {
    pub dashboard_id: Uuid,
    pub user_id: Uuid,
    pub permission: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportScheduleV6 {
    pub id: Uuid,
    pub report_id: Uuid,
    pub cron_expression: String,
    pub enabled: bool,
    pub last_run_at: Option<DateTime<Utc>>,
    pub next_run_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateReportScheduleV6 {
    pub report_id: Uuid,
    pub cron_expression: String,
    pub enabled: Option<bool>,
    pub next_run_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateReportScheduleV6 {
    pub cron_expression: Option<String>,
    pub enabled: Option<bool>,
    pub next_run_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardAnalyticsV4 {
    pub dashboard_id: Uuid,
    pub view_count: i64,
    pub unique_viewers: i64,
    pub last_viewed_at: Option<DateTime<Utc>>,
    pub widget_interactions: i64,
    pub avg_view_duration_seconds: f64,
    pub share_count: i64,
    pub last_shared_at: Option<DateTime<Utc>>,
    pub export_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportAnalyticsV4 {
    pub report_id: Uuid,
    pub generation_count: i64,
    pub avg_generation_time_ms: f64,
    pub last_generated_at: Option<DateTime<Utc>>,
    pub export_count: i64,
    pub popular_format: String,
    pub schedule_count: i64,
    pub success_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardStatsV8 {
    pub total_dashboards: i64,
    pub public_dashboards: i64,
    pub total_reports: i64,
    pub scheduled_reports: i64,
    pub total_shares: i64,
    pub total_schedules: i64,
    pub avg_shares_per_dashboard: f64,
    pub total_views: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardTemplateV3 {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub template_type: String,
    pub config: serde_json::Value,
    pub is_public: bool,
    pub author_id: Option<Uuid>,
    pub usage_count: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateDashboardTemplateV3 {
    pub name: String,
    pub description: Option<String>,
    pub template_type: String,
    pub config: Option<serde_json::Value>,
    pub is_public: Option<bool>,
    pub author_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportTemplateV3 {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub report_type: String,
    pub config: serde_json::Value,
    pub is_public: bool,
    pub author_id: Option<Uuid>,
    pub usage_count: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateReportTemplateV3 {
    pub name: String,
    pub description: Option<String>,
    pub report_type: String,
    pub config: Option<serde_json::Value>,
    pub is_public: Option<bool>,
    pub author_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateMarketplaceItemV3 {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub template_type: String,
    pub author_id: Option<Uuid>,
    pub usage_count: i64,
    pub rating: Option<f64>,
    pub tags: Vec<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateAnalyticsV3 {
    pub template_id: Uuid,
    pub usage_count: i64,
    pub last_used_at: Option<DateTime<Utc>>,
    pub popular_configs: Vec<serde_json::Value>,
    pub avg_rating: Option<f64>,
    pub total_ratings: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardShareV6 {
    pub id: Uuid,
    pub dashboard_id: Uuid,
    pub user_id: Uuid,
    pub permission: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateDashboardShareV6 {
    pub dashboard_id: Uuid,
    pub user_id: Uuid,
    pub permission: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportScheduleV7 {
    pub id: Uuid,
    pub report_id: Uuid,
    pub cron_expression: String,
    pub enabled: bool,
    pub last_run_at: Option<DateTime<Utc>>,
    pub next_run_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateReportScheduleV7 {
    pub report_id: Uuid,
    pub cron_expression: String,
    pub enabled: Option<bool>,
    pub next_run_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateReportScheduleV7 {
    pub cron_expression: Option<String>,
    pub enabled: Option<bool>,
    pub next_run_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardAnalyticsV5 {
    pub dashboard_id: Uuid,
    pub view_count: i64,
    pub unique_viewers: i64,
    pub last_viewed_at: Option<DateTime<Utc>>,
    pub widget_interactions: i64,
    pub avg_view_duration_seconds: f64,
    pub share_count: i64,
    pub last_shared_at: Option<DateTime<Utc>>,
    pub export_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportAnalyticsV5 {
    pub report_id: Uuid,
    pub generation_count: i64,
    pub avg_generation_time_ms: f64,
    pub last_generated_at: Option<DateTime<Utc>>,
    pub export_count: i64,
    pub popular_format: String,
    pub schedule_count: i64,
    pub success_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardStatsV9 {
    pub total_dashboards: i64,
    pub public_dashboards: i64,
    pub total_reports: i64,
    pub scheduled_reports: i64,
    pub total_shares: i64,
    pub total_schedules: i64,
    pub avg_shares_per_dashboard: f64,
    pub total_views: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardShareV7 {
    pub id: uuid::Uuid,
    pub dashboard_id: uuid::Uuid,
    pub user_id: uuid::Uuid,
    pub permission: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateDashboardShareV7 {
    pub dashboard_id: uuid::Uuid,
    pub user_id: uuid::Uuid,
    pub permission: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportScheduleV8 {
    pub id: uuid::Uuid,
    pub report_id: uuid::Uuid,
    pub cron_expression: String,
    pub enabled: bool,
    pub last_run_at: Option<chrono::DateTime<chrono::Utc>>,
    pub next_run_at: chrono::DateTime<chrono::Utc>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateReportScheduleV8 {
    pub report_id: uuid::Uuid,
    pub cron_expression: String,
    pub enabled: Option<bool>,
    pub next_run_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateReportScheduleV8 {
    pub cron_expression: Option<String>,
    pub enabled: Option<bool>,
    pub next_run_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardAnalyticsV6 {
    pub dashboard_id: uuid::Uuid,
    pub view_count: i64,
    pub unique_viewers: i64,
    pub last_viewed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub widget_interactions: i64,
    pub avg_view_duration_seconds: f64,
    pub share_count: i64,
    pub last_shared_at: Option<chrono::DateTime<chrono::Utc>>,
    pub export_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportAnalyticsV6 {
    pub report_id: uuid::Uuid,
    pub generation_count: i64,
    pub avg_generation_time_ms: f64,
    pub last_generated_at: Option<chrono::DateTime<chrono::Utc>>,
    pub export_count: i64,
    pub popular_format: String,
    pub schedule_count: i64,
    pub success_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardStatsV10 {
    pub total_dashboards: i64,
    pub public_dashboards: i64,
    pub total_reports: i64,
    pub scheduled_reports: i64,
    pub total_shares: i64,
    pub total_schedules: i64,
    pub avg_shares_per_dashboard: f64,
    pub total_views: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardShareV8 {
    pub id: uuid::Uuid,
    pub dashboard_id: uuid::Uuid,
    pub user_id: uuid::Uuid,
    pub permission: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateDashboardShareV8 {
    pub dashboard_id: uuid::Uuid,
    pub user_id: uuid::Uuid,
    pub permission: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportScheduleV9 {
    pub id: uuid::Uuid,
    pub report_id: uuid::Uuid,
    pub cron_expression: String,
    pub enabled: bool,
    pub last_run_at: Option<chrono::DateTime<chrono::Utc>>,
    pub next_run_at: chrono::DateTime<chrono::Utc>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateReportScheduleV9 {
    pub report_id: uuid::Uuid,
    pub cron_expression: String,
    pub enabled: Option<bool>,
    pub next_run_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateReportScheduleV9 {
    pub cron_expression: Option<String>,
    pub enabled: Option<bool>,
    pub next_run_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardAnalyticsV7 {
    pub dashboard_id: uuid::Uuid,
    pub view_count: i64,
    pub unique_viewers: i64,
    pub last_viewed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub widget_interactions: i64,
    pub avg_view_duration_seconds: f64,
    pub share_count: i64,
    pub last_shared_at: Option<chrono::DateTime<chrono::Utc>>,
    pub export_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportAnalyticsV7 {
    pub report_id: uuid::Uuid,
    pub generation_count: i64,
    pub avg_generation_time_ms: f64,
    pub last_generated_at: Option<chrono::DateTime<chrono::Utc>>,
    pub export_count: i64,
    pub popular_format: String,
    pub schedule_count: i64,
    pub success_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardStatsV11 {
    pub total_dashboards: i64,
    pub public_dashboards: i64,
    pub total_reports: i64,
    pub scheduled_reports: i64,
    pub total_shares: i64,
    pub total_schedules: i64,
    pub avg_shares_per_dashboard: f64,
    pub total_views: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardTemplateV4 {
    pub id: uuid::Uuid,
    pub name: String,
    pub description: String,
    pub template_type: String,
    pub config: serde_json::Value,
    pub is_public: bool,
    pub author_id: Option<uuid::Uuid>,
    pub usage_count: i64,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateDashboardTemplateV4 {
    pub name: String,
    pub description: Option<String>,
    pub template_type: String,
    pub config: Option<serde_json::Value>,
    pub is_public: Option<bool>,
    pub author_id: Option<uuid::Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportTemplateV4 {
    pub id: uuid::Uuid,
    pub name: String,
    pub description: String,
    pub report_type: String,
    pub config: serde_json::Value,
    pub is_public: bool,
    pub author_id: Option<uuid::Uuid>,
    pub usage_count: i64,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateReportTemplateV4 {
    pub name: String,
    pub description: Option<String>,
    pub report_type: String,
    pub config: Option<serde_json::Value>,
    pub is_public: Option<bool>,
    pub author_id: Option<uuid::Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateMarketplaceItemV4 {
    pub id: uuid::Uuid,
    pub name: String,
    pub description: String,
    pub template_type: String,
    pub author_id: Option<uuid::Uuid>,
    pub usage_count: i64,
    pub rating: Option<f64>,
    pub tags: Vec<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateAnalyticsV4 {
    pub template_id: uuid::Uuid,
    pub usage_count: i64,
    pub last_used_at: Option<chrono::DateTime<chrono::Utc>>,
    pub popular_configs: Vec<serde_json::Value>,
    pub avg_rating: Option<f64>,
    pub total_ratings: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardShareV9 {
    pub id: uuid::Uuid,
    pub dashboard_id: uuid::Uuid,
    pub user_id: uuid::Uuid,
    pub permission: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateDashboardShareV9 {
    pub dashboard_id: uuid::Uuid,
    pub user_id: uuid::Uuid,
    pub permission: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportScheduleV10 {
    pub id: uuid::Uuid,
    pub report_id: uuid::Uuid,
    pub cron_expression: String,
    pub enabled: bool,
    pub last_run_at: Option<chrono::DateTime<chrono::Utc>>,
    pub next_run_at: chrono::DateTime<chrono::Utc>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateReportScheduleV10 {
    pub report_id: uuid::Uuid,
    pub cron_expression: String,
    pub enabled: Option<bool>,
    pub next_run_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateReportScheduleV10 {
    pub cron_expression: Option<String>,
    pub enabled: Option<bool>,
    pub next_run_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardAnalyticsV8 {
    pub dashboard_id: uuid::Uuid,
    pub view_count: i64,
    pub unique_viewers: i64,
    pub last_viewed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub widget_interactions: i64,
    pub avg_view_duration_seconds: f64,
    pub share_count: i64,
    pub last_shared_at: Option<chrono::DateTime<chrono::Utc>>,
    pub export_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportAnalyticsV8 {
    pub report_id: uuid::Uuid,
    pub generation_count: i64,
    pub avg_generation_time_ms: f64,
    pub last_generated_at: Option<chrono::DateTime<chrono::Utc>>,
    pub export_count: i64,
    pub popular_format: String,
    pub schedule_count: i64,
    pub success_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardStatsV12 {
    pub total_dashboards: i64,
    pub public_dashboards: i64,
    pub total_reports: i64,
    pub scheduled_reports: i64,
    pub total_shares: i64,
    pub total_schedules: i64,
    pub avg_shares_per_dashboard: f64,
    pub total_views: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardShareV10 {
    pub id: uuid::Uuid,
    pub dashboard_id: uuid::Uuid,
    pub user_id: uuid::Uuid,
    pub permission: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateDashboardShareV10 {
    pub dashboard_id: uuid::Uuid,
    pub user_id: uuid::Uuid,
    pub permission: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportScheduleV11 {
    pub id: uuid::Uuid,
    pub report_id: uuid::Uuid,
    pub cron_expression: String,
    pub enabled: bool,
    pub last_run_at: Option<chrono::DateTime<chrono::Utc>>,
    pub next_run_at: chrono::DateTime<chrono::Utc>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateReportScheduleV11 {
    pub report_id: uuid::Uuid,
    pub cron_expression: String,
    pub enabled: Option<bool>,
    pub next_run_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateReportScheduleV11 {
    pub cron_expression: Option<String>,
    pub enabled: Option<bool>,
    pub next_run_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardAnalyticsV9 {
    pub dashboard_id: uuid::Uuid,
    pub view_count: i64,
    pub unique_viewers: i64,
    pub last_viewed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub widget_interactions: i64,
    pub avg_view_duration_seconds: f64,
    pub share_count: i64,
    pub last_shared_at: Option<chrono::DateTime<chrono::Utc>>,
    pub export_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportAnalyticsV9 {
    pub report_id: uuid::Uuid,
    pub generation_count: i64,
    pub avg_generation_time_ms: f64,
    pub last_generated_at: Option<chrono::DateTime<chrono::Utc>>,
    pub export_count: i64,
    pub popular_format: String,
    pub schedule_count: i64,
    pub success_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardStatsV13 {
    pub total_dashboards: i64,
    pub public_dashboards: i64,
    pub total_reports: i64,
    pub scheduled_reports: i64,
    pub total_shares: i64,
    pub total_schedules: i64,
    pub avg_shares_per_dashboard: f64,
    pub total_views: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardTemplateV5 {
    pub id: uuid::Uuid,
    pub name: String,
    pub description: String,
    pub template_type: String,
    pub config: serde_json::Value,
    pub is_public: bool,
    pub author_id: Option<uuid::Uuid>,
    pub usage_count: i64,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateDashboardTemplateV5 {
    pub name: String,
    pub description: Option<String>,
    pub template_type: String,
    pub config: Option<serde_json::Value>,
    pub is_public: Option<bool>,
    pub author_id: Option<uuid::Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportTemplateV5 {
    pub id: uuid::Uuid,
    pub name: String,
    pub description: String,
    pub report_type: String,
    pub config: serde_json::Value,
    pub is_public: bool,
    pub author_id: Option<uuid::Uuid>,
    pub usage_count: i64,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateReportTemplateV5 {
    pub name: String,
    pub description: Option<String>,
    pub report_type: String,
    pub config: Option<serde_json::Value>,
    pub is_public: Option<bool>,
    pub author_id: Option<uuid::Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateMarketplaceItemV5 {
    pub id: uuid::Uuid,
    pub name: String,
    pub description: String,
    pub template_type: String,
    pub author_id: Option<uuid::Uuid>,
    pub usage_count: i64,
    pub rating: Option<f64>,
    pub tags: Vec<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateAnalyticsV5 {
    pub template_id: uuid::Uuid,
    pub usage_count: i64,
    pub last_used_at: Option<chrono::DateTime<chrono::Utc>>,
    pub popular_configs: Vec<serde_json::Value>,
    pub avg_rating: Option<f64>,
    pub total_ratings: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardShareV11 {
    pub id: uuid::Uuid,
    pub dashboard_id: uuid::Uuid,
    pub user_id: uuid::Uuid,
    pub permission: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateDashboardShareV11 {
    pub dashboard_id: uuid::Uuid,
    pub user_id: uuid::Uuid,
    pub permission: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportScheduleV12 {
    pub id: uuid::Uuid,
    pub report_id: uuid::Uuid,
    pub cron_expression: String,
    pub enabled: bool,
    pub last_run_at: Option<chrono::DateTime<chrono::Utc>>,
    pub next_run_at: chrono::DateTime<chrono::Utc>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateReportScheduleV12 {
    pub report_id: uuid::Uuid,
    pub cron_expression: String,
    pub enabled: Option<bool>,
    pub next_run_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateReportScheduleV12 {
    pub cron_expression: Option<String>,
    pub enabled: Option<bool>,
    pub next_run_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardStatsV14 {
    pub total_dashboards: i64,
    pub public_dashboards: i64,
    pub total_reports: i64,
    pub scheduled_reports: i64,
    pub total_shares: i64,
    pub total_schedules: i64,
    pub avg_shares_per_dashboard: f64,
    pub total_views: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardAnalyticsV10 {
    pub dashboard_id: uuid::Uuid,
    pub view_count: i64,
    pub unique_viewers: i64,
    pub last_viewed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub widget_interactions: i64,
    pub avg_view_duration_seconds: f64,
    pub share_count: i64,
    pub last_shared_at: Option<chrono::DateTime<chrono::Utc>>,
    pub export_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportAnalyticsV10 {
    pub report_id: uuid::Uuid,
    pub generation_count: i64,
    pub avg_generation_time_ms: f64,
    pub last_generated_at: Option<chrono::DateTime<chrono::Utc>>,
    pub export_count: i64,
    pub popular_format: String,
    pub schedule_count: i64,
    pub success_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardTemplateV6 {
    pub id: uuid::Uuid,
    pub name: String,
    pub description: String,
    pub template_type: String,
    pub config: serde_json::Value,
    pub is_public: bool,
    pub author_id: Option<uuid::Uuid>,
    pub usage_count: i64,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateDashboardTemplateV6 {
    pub name: String,
    pub description: Option<String>,
    pub template_type: String,
    pub config: Option<serde_json::Value>,
    pub is_public: Option<bool>,
    pub author_id: Option<uuid::Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportTemplateV6 {
    pub id: uuid::Uuid,
    pub name: String,
    pub description: String,
    pub report_type: String,
    pub config: serde_json::Value,
    pub is_public: bool,
    pub author_id: Option<uuid::Uuid>,
    pub usage_count: i64,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateReportTemplateV6 {
    pub name: String,
    pub description: Option<String>,
    pub report_type: String,
    pub config: Option<serde_json::Value>,
    pub is_public: Option<bool>,
    pub author_id: Option<uuid::Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardShareV12 {
    pub id: uuid::Uuid,
    pub dashboard_id: uuid::Uuid,
    pub user_id: uuid::Uuid,
    pub permission: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateDashboardShareV12 {
    pub dashboard_id: uuid::Uuid,
    pub user_id: uuid::Uuid,
    pub permission: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportScheduleV13 {
    pub id: uuid::Uuid,
    pub report_id: uuid::Uuid,
    pub cron_expression: String,
    pub enabled: bool,
    pub last_run_at: Option<chrono::DateTime<chrono::Utc>>,
    pub next_run_at: chrono::DateTime<chrono::Utc>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateReportScheduleV13 {
    pub report_id: uuid::Uuid,
    pub cron_expression: String,
    pub enabled: Option<bool>,
    pub next_run_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateReportScheduleV13 {
    pub cron_expression: Option<String>,
    pub enabled: Option<bool>,
    pub next_run_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardAnalyticsV11 {
    pub dashboard_id: uuid::Uuid,
    pub view_count: i64,
    pub unique_viewers: i64,
    pub last_viewed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub widget_interactions: i64,
    pub avg_view_duration_seconds: f64,
    pub share_count: i64,
    pub last_shared_at: Option<chrono::DateTime<chrono::Utc>>,
    pub export_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportAnalyticsV11 {
    pub report_id: uuid::Uuid,
    pub generation_count: i64,
    pub avg_generation_time_ms: f64,
    pub last_generated_at: Option<chrono::DateTime<chrono::Utc>>,
    pub export_count: i64,
    pub popular_format: String,
    pub schedule_count: i64,
    pub success_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardStatsV15 {
    pub total_dashboards: i64,
    pub public_dashboards: i64,
    pub total_reports: i64,
    pub scheduled_reports: i64,
    pub total_shares: i64,
    pub total_schedules: i64,
    pub avg_shares_per_dashboard: f64,
    pub total_views: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardTemplateV7 {
    pub id: uuid::Uuid,
    pub name: String,
    pub description: String,
    pub template_type: String,
    pub config: serde_json::Value,
    pub is_public: bool,
    pub author_id: Option<uuid::Uuid>,
    pub usage_count: i64,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateDashboardTemplateV7 {
    pub name: String,
    pub description: Option<String>,
    pub template_type: String,
    pub config: Option<serde_json::Value>,
    pub is_public: Option<bool>,
    pub author_id: Option<uuid::Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportTemplateV7 {
    pub id: uuid::Uuid,
    pub name: String,
    pub description: String,
    pub report_type: String,
    pub config: serde_json::Value,
    pub is_public: bool,
    pub author_id: Option<uuid::Uuid>,
    pub usage_count: i64,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateReportTemplateV7 {
    pub name: String,
    pub description: Option<String>,
    pub report_type: String,
    pub config: Option<serde_json::Value>,
    pub is_public: Option<bool>,
    pub author_id: Option<uuid::Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardShareV13 {
    pub id: uuid::Uuid,
    pub dashboard_id: uuid::Uuid,
    pub user_id: uuid::Uuid,
    pub permission: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateDashboardShareV13 {
    pub dashboard_id: uuid::Uuid,
    pub user_id: uuid::Uuid,
    pub permission: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportScheduleV14 {
    pub id: uuid::Uuid,
    pub report_id: uuid::Uuid,
    pub cron_expression: String,
    pub enabled: bool,
    pub last_run_at: Option<chrono::DateTime<chrono::Utc>>,
    pub next_run_at: chrono::DateTime<chrono::Utc>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateReportScheduleV14 {
    pub report_id: uuid::Uuid,
    pub cron_expression: String,
    pub enabled: Option<bool>,
    pub next_run_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateReportScheduleV14 {
    pub cron_expression: Option<String>,
    pub enabled: Option<bool>,
    pub next_run_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardAnalyticsV12 {
    pub dashboard_id: uuid::Uuid,
    pub view_count: i64,
    pub unique_viewers: i64,
    pub last_viewed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub widget_interactions: i64,
    pub avg_view_duration_seconds: f64,
    pub share_count: i64,
    pub last_shared_at: Option<chrono::DateTime<chrono::Utc>>,
    pub export_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportAnalyticsV12 {
    pub report_id: uuid::Uuid,
    pub generation_count: i64,
    pub avg_generation_time_ms: f64,
    pub last_generated_at: Option<chrono::DateTime<chrono::Utc>>,
    pub export_count: i64,
    pub popular_format: String,
    pub schedule_count: i64,
    pub success_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardStatsV16 {
    pub total_dashboards: i64,
    pub public_dashboards: i64,
    pub total_reports: i64,
    pub scheduled_reports: i64,
    pub total_shares: i64,
    pub total_schedules: i64,
    pub avg_shares_per_dashboard: f64,
    pub total_views: i64,
}

// V17: Dashboard sharing v14 and report scheduling v15

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardShareV14 {
    pub id: uuid::Uuid,
    pub dashboard_id: uuid::Uuid,
    pub user_id: uuid::Uuid,
    pub permission: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateDashboardShareV14 {
    pub dashboard_id: uuid::Uuid,
    pub user_id: uuid::Uuid,
    pub permission: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportScheduleV15 {
    pub id: uuid::Uuid,
    pub report_id: uuid::Uuid,
    pub cron_expression: String,
    pub enabled: bool,
    pub last_run_at: Option<chrono::DateTime<chrono::Utc>>,
    pub next_run_at: chrono::DateTime<chrono::Utc>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateReportScheduleV15 {
    pub report_id: uuid::Uuid,
    pub cron_expression: String,
    pub enabled: Option<bool>,
    pub next_run_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateReportScheduleV15 {
    pub cron_expression: Option<String>,
    pub enabled: Option<bool>,
    pub next_run_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardShareListV14 {
    pub shares: Vec<DashboardShareV14>,
    pub total_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportScheduleListV15 {
    pub schedules: Vec<ReportScheduleV15>,
    pub total_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardAnalyticsV13 {
    pub dashboard_id: uuid::Uuid,
    pub view_count: i64,
    pub unique_viewers: i64,
    pub last_viewed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub widget_interactions: i64,
    pub avg_view_duration_seconds: f64,
    pub share_count: i64,
    pub last_shared_at: Option<chrono::DateTime<chrono::Utc>>,
    pub export_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportAnalyticsV13 {
    pub report_id: uuid::Uuid,
    pub generation_count: i64,
    pub avg_generation_time_ms: f64,
    pub last_generated_at: Option<chrono::DateTime<chrono::Utc>>,
    pub export_count: i64,
    pub popular_format: String,
    pub schedule_count: i64,
    pub success_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardStatsV17 {
    pub total_dashboards: i64,
    pub public_dashboards: i64,
    pub total_reports: i64,
    pub scheduled_reports: i64,
    pub total_shares: i64,
    pub total_schedules: i64,
    pub avg_shares_per_dashboard: f64,
    pub total_views: i64,
}

// V18: Enhanced dashboard sharing and report scheduling

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardShareV15 {
    pub id: uuid::Uuid,
    pub dashboard_id: uuid::Uuid,
    pub user_id: uuid::Uuid,
    pub permission: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateDashboardShareV15 {
    pub dashboard_id: uuid::Uuid,
    pub user_id: uuid::Uuid,
    pub permission: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardShareListV15 {
    pub shares: Vec<DashboardShareV15>,
    pub total_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportScheduleV16 {
    pub id: uuid::Uuid,
    pub report_id: uuid::Uuid,
    pub cron_expression: String,
    pub enabled: bool,
    pub last_run_at: Option<chrono::DateTime<chrono::Utc>>,
    pub next_run_at: chrono::DateTime<chrono::Utc>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateReportScheduleV16 {
    pub report_id: uuid::Uuid,
    pub cron_expression: String,
    pub enabled: Option<bool>,
    pub next_run_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateReportScheduleV16 {
    pub cron_expression: Option<String>,
    pub enabled: Option<bool>,
    pub next_run_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportScheduleListV16 {
    pub schedules: Vec<ReportScheduleV16>,
    pub total_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardTemplateV18 {
    pub id: uuid::Uuid,
    pub name: String,
    pub description: String,
    pub template_type: String,
    pub config: serde_json::Value,
    pub is_public: bool,
    pub author_id: Option<uuid::Uuid>,
    pub usage_count: i64,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateDashboardTemplateV18 {
    pub name: String,
    pub description: Option<String>,
    pub template_type: String,
    pub config: Option<serde_json::Value>,
    pub is_public: Option<bool>,
    pub author_id: Option<uuid::Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportTemplateV18 {
    pub id: uuid::Uuid,
    pub name: String,
    pub description: String,
    pub report_type: String,
    pub config: serde_json::Value,
    pub is_public: bool,
    pub author_id: Option<uuid::Uuid>,
    pub usage_count: i64,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateReportTemplateV18 {
    pub name: String,
    pub description: Option<String>,
    pub report_type: String,
    pub config: Option<serde_json::Value>,
    pub is_public: Option<bool>,
    pub author_id: Option<uuid::Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardAnalyticsV14 {
    pub dashboard_id: uuid::Uuid,
    pub view_count: i64,
    pub unique_viewers: i64,
    pub last_viewed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub widget_interactions: i64,
    pub avg_view_duration_seconds: f64,
    pub share_count: i64,
    pub last_shared_at: Option<chrono::DateTime<chrono::Utc>>,
    pub export_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportAnalyticsV14 {
    pub report_id: uuid::Uuid,
    pub generation_count: i64,
    pub avg_generation_time_ms: f64,
    pub last_generated_at: Option<chrono::DateTime<chrono::Utc>>,
    pub export_count: i64,
    pub popular_format: String,
    pub schedule_count: i64,
    pub success_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardStatsV18 {
    pub total_dashboards: i64,
    pub public_dashboards: i64,
    pub total_reports: i64,
    pub scheduled_reports: i64,
    pub total_shares: i64,
    pub total_schedules: i64,
    pub avg_shares_per_dashboard: f64,
    pub total_views: i64,
}

// V19: Enhanced dashboard sharing and report scheduling

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardShareV16 {
    pub id: uuid::Uuid,
    pub dashboard_id: uuid::Uuid,
    pub user_id: uuid::Uuid,
    pub permission: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateDashboardShareV16 {
    pub dashboard_id: uuid::Uuid,
    pub user_id: uuid::Uuid,
    pub permission: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardShareListV16 {
    pub shares: Vec<DashboardShareV16>,
    pub total_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportScheduleV17 {
    pub id: uuid::Uuid,
    pub report_id: uuid::Uuid,
    pub cron_expression: String,
    pub enabled: bool,
    pub last_run_at: Option<chrono::DateTime<chrono::Utc>>,
    pub next_run_at: chrono::DateTime<chrono::Utc>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateReportScheduleV17 {
    pub report_id: uuid::Uuid,
    pub cron_expression: String,
    pub enabled: Option<bool>,
    pub next_run_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateReportScheduleV17 {
    pub cron_expression: Option<String>,
    pub enabled: Option<bool>,
    pub next_run_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportScheduleListV17 {
    pub schedules: Vec<ReportScheduleV17>,
    pub total_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardTemplateV19 {
    pub id: uuid::Uuid,
    pub name: String,
    pub description: String,
    pub template_type: String,
    pub config: serde_json::Value,
    pub is_public: bool,
    pub author_id: Option<uuid::Uuid>,
    pub usage_count: i64,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateDashboardTemplateV19 {
    pub name: String,
    pub description: Option<String>,
    pub template_type: String,
    pub config: Option<serde_json::Value>,
    pub is_public: Option<bool>,
    pub author_id: Option<uuid::Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportTemplateV19 {
    pub id: uuid::Uuid,
    pub name: String,
    pub description: String,
    pub report_type: String,
    pub config: serde_json::Value,
    pub is_public: bool,
    pub author_id: Option<uuid::Uuid>,
    pub usage_count: i64,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateReportTemplateV19 {
    pub name: String,
    pub description: Option<String>,
    pub report_type: String,
    pub config: Option<serde_json::Value>,
    pub is_public: Option<bool>,
    pub author_id: Option<uuid::Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardAnalyticsV15 {
    pub dashboard_id: uuid::Uuid,
    pub view_count: i64,
    pub unique_viewers: i64,
    pub last_viewed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub widget_interactions: i64,
    pub avg_view_duration_seconds: f64,
    pub share_count: i64,
    pub last_shared_at: Option<chrono::DateTime<chrono::Utc>>,
    pub export_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportAnalyticsV15 {
    pub report_id: uuid::Uuid,
    pub generation_count: i64,
    pub avg_generation_time_ms: f64,
    pub last_generated_at: Option<chrono::DateTime<chrono::Utc>>,
    pub export_count: i64,
    pub popular_format: String,
    pub schedule_count: i64,
    pub success_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardStatsV19 {
    pub total_dashboards: i64,
    pub public_dashboards: i64,
    pub total_reports: i64,
    pub scheduled_reports: i64,
    pub total_shares: i64,
    pub total_schedules: i64,
    pub avg_shares_per_dashboard: f64,
    pub total_views: i64,
}

// V20: Enhanced dashboard sharing v17 and report scheduling v18

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardShareV17 {
    pub id: uuid::Uuid,
    pub dashboard_id: uuid::Uuid,
    pub user_id: uuid::Uuid,
    pub permission: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateDashboardShareV17 {
    pub dashboard_id: uuid::Uuid,
    pub user_id: uuid::Uuid,
    pub permission: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardShareListV17 {
    pub shares: Vec<DashboardShareV17>,
    pub total_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportScheduleV18 {
    pub id: uuid::Uuid,
    pub report_id: uuid::Uuid,
    pub cron_expression: String,
    pub enabled: bool,
    pub last_run_at: Option<chrono::DateTime<chrono::Utc>>,
    pub next_run_at: chrono::DateTime<chrono::Utc>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateReportScheduleV18 {
    pub report_id: uuid::Uuid,
    pub cron_expression: String,
    pub enabled: Option<bool>,
    pub next_run_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateReportScheduleV18 {
    pub cron_expression: Option<String>,
    pub enabled: Option<bool>,
    pub next_run_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportScheduleListV18 {
    pub schedules: Vec<ReportScheduleV18>,
    pub total_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardAnalyticsV16 {
    pub dashboard_id: uuid::Uuid,
    pub view_count: i64,
    pub unique_viewers: i64,
    pub last_viewed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub widget_interactions: i64,
    pub avg_view_duration_seconds: f64,
    pub share_count: i64,
    pub last_shared_at: Option<chrono::DateTime<chrono::Utc>>,
    pub export_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportAnalyticsV16 {
    pub report_id: uuid::Uuid,
    pub generation_count: i64,
    pub avg_generation_time_ms: f64,
    pub last_generated_at: Option<chrono::DateTime<chrono::Utc>>,
    pub export_count: i64,
    pub popular_format: String,
    pub schedule_count: i64,
    pub success_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardStatsV20 {
    pub total_dashboards: i64,
    pub public_dashboards: i64,
    pub total_reports: i64,
    pub scheduled_reports: i64,
    pub total_shares: i64,
    pub total_schedules: i64,
    pub avg_shares_per_dashboard: f64,
    pub total_views: i64,
}

// V21: Enhanced dashboard sharing v18 and report scheduling v19

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardShareV18 {
    pub id: uuid::Uuid,
    pub dashboard_id: uuid::Uuid,
    pub user_id: uuid::Uuid,
    pub permission: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateDashboardShareV18 {
    pub dashboard_id: uuid::Uuid,
    pub user_id: uuid::Uuid,
    pub permission: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardShareListV18 {
    pub shares: Vec<DashboardShareV18>,
    pub total_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportScheduleV19 {
    pub id: uuid::Uuid,
    pub report_id: uuid::Uuid,
    pub cron_expression: String,
    pub enabled: bool,
    pub last_run_at: Option<chrono::DateTime<chrono::Utc>>,
    pub next_run_at: chrono::DateTime<chrono::Utc>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateReportScheduleV19 {
    pub report_id: uuid::Uuid,
    pub cron_expression: String,
    pub enabled: Option<bool>,
    pub next_run_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateReportScheduleV19 {
    pub cron_expression: Option<String>,
    pub enabled: Option<bool>,
    pub next_run_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportScheduleListV19 {
    pub schedules: Vec<ReportScheduleV19>,
    pub total_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardTemplateV20 {
    pub id: uuid::Uuid,
    pub name: String,
    pub description: String,
    pub template_type: String,
    pub config: serde_json::Value,
    pub is_public: bool,
    pub author_id: Option<uuid::Uuid>,
    pub usage_count: i64,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateDashboardTemplateV20 {
    pub name: String,
    pub description: Option<String>,
    pub template_type: String,
    pub config: Option<serde_json::Value>,
    pub is_public: Option<bool>,
    pub author_id: Option<uuid::Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportTemplateV20 {
    pub id: uuid::Uuid,
    pub name: String,
    pub description: String,
    pub report_type: String,
    pub config: serde_json::Value,
    pub is_public: bool,
    pub author_id: Option<uuid::Uuid>,
    pub usage_count: i64,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateReportTemplateV20 {
    pub name: String,
    pub description: Option<String>,
    pub report_type: String,
    pub config: Option<serde_json::Value>,
    pub is_public: Option<bool>,
    pub author_id: Option<uuid::Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardAnalyticsV17 {
    pub dashboard_id: uuid::Uuid,
    pub view_count: i64,
    pub unique_viewers: i64,
    pub last_viewed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub widget_interactions: i64,
    pub avg_view_duration_seconds: f64,
    pub share_count: i64,
    pub last_shared_at: Option<chrono::DateTime<chrono::Utc>>,
    pub export_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportAnalyticsV17 {
    pub report_id: uuid::Uuid,
    pub generation_count: i64,
    pub avg_generation_time_ms: f64,
    pub last_generated_at: Option<chrono::DateTime<chrono::Utc>>,
    pub export_count: i64,
    pub popular_format: String,
    pub schedule_count: i64,
    pub success_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardStatsV21 {
    pub total_dashboards: i64,
    pub public_dashboards: i64,
    pub total_reports: i64,
    pub scheduled_reports: i64,
    pub total_shares: i64,
    pub total_schedules: i64,
    pub avg_shares_per_dashboard: f64,
    pub total_views: i64,
}

// V22 types (from dashboard_reporting_v22.rs)

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardVersionV19 {
    pub id: Uuid,
    pub dashboard_id: Uuid,
    pub version: i32,
    pub definition: serde_json::Value,
    pub change_description: String,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateDashboardVersionV19 {
    pub dashboard_id: Uuid,
    pub definition: serde_json::Value,
    pub change_description: Option<String>,
    pub created_by: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportTemplateRatingV19 {
    pub id: Uuid,
    pub template_id: Uuid,
    pub user_id: Uuid,
    pub rating: i32,
    pub review: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateReportTemplateRatingV19 {
    pub template_id: Uuid,
    pub user_id: Uuid,
    pub rating: i32,
    pub review: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateReportTemplateRatingV19 {
    pub rating: Option<i32>,
    pub review: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardSharingAnalyticsV22 {
    pub dashboard_id: Uuid,
    pub total_shares: i64,
    pub unique_viewers: i64,
    pub share_count_last_24h: i64,
    pub share_count_last_7d: i64,
    pub top_sharers: Vec<DashboardSharerV22>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardSharerV22 {
    pub user_id: Uuid,
    pub username: String,
    pub share_count: i64,
    pub last_shared_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardPerformanceV22 {
    pub dashboard_id: Uuid,
    pub avg_load_time_ms: f64,
    pub p95_load_time_ms: f64,
    pub total_loads: i64,
    pub cache_hit_rate: f64,
    pub last_measured_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardVersionStatsV19 {
    pub total_versions: i64,
    pub latest_version: i32,
    pub total_changes: i64,
    pub contributors: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportTemplateRatingStatsV19 {
    pub template_id: Uuid,
    pub total_ratings: i64,
    pub average_rating: f64,
    pub rating_distribution: HashMap<i32, i64>,
    pub total_reviews: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardVersionDiffV19 {
    pub version_a: DashboardVersionV19,
    pub version_b: DashboardVersionV19,
    pub changes: serde_json::Value,
    pub added_widgets: Vec<String>,
    pub removed_widgets: Vec<String>,
    pub modified_widgets: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardSharingRequestV22 {
    pub dashboard_id: Uuid,
    pub user_id: Uuid,
    pub permission: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardPerformanceRequestV22 {
    pub dashboard_id: Uuid,
    pub load_time_ms: f64,
    pub cache_hit: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardAnalyticsSummaryV22 {
    pub total_dashboards: i64,
    pub total_shares: i64,
    pub total_views: i64,
    pub avg_load_time_ms: f64,
    pub most_viewed_dashboards: Vec<DashboardViewCountV22>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardViewCountV22 {
    pub dashboard_id: Uuid,
    pub dashboard_name: String,
    pub view_count: i64,
    pub last_viewed_at: DateTime<Utc>,
}

// V23 types (from dashboard_reporting_v23.rs)

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardWidgetLibraryV20 {
    pub id: Uuid,
    pub name: String,
    pub r#type: String,
    pub category: String,
    pub config: serde_json::Value,
    pub preview_url: Option<String>,
    pub usage_count: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateDashboardWidgetLibraryV20 {
    pub name: String,
    pub r#type: String,
    pub category: Option<String>,
    pub config: Option<serde_json::Value>,
    pub preview_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateDashboardWidgetLibraryV20 {
    pub name: Option<String>,
    pub r#type: Option<String>,
    pub category: Option<String>,
    pub config: Option<serde_json::Value>,
    pub preview_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportGenerationQueueV20 {
    pub id: Uuid,
    pub report_id: Uuid,
    pub status: String,
    pub priority: i32,
    pub scheduled_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateReportGenerationQueueV20 {
    pub report_id: Uuid,
    pub status: Option<String>,
    pub priority: Option<i32>,
    pub scheduled_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardSharingAnalyticsV23 {
    pub dashboard_id: Uuid,
    pub total_shares: i64,
    pub unique_viewers: i64,
    pub share_count_last_24h: i64,
    pub share_count_last_7d: i64,
    pub top_sharers: Vec<DashboardSharerV23>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardSharerV23 {
    pub user_id: Uuid,
    pub username: String,
    pub share_count: i64,
    pub last_shared_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardPerformanceV23 {
    pub dashboard_id: Uuid,
    pub avg_load_time_ms: f64,
    pub p95_load_time_ms: f64,
    pub total_loads: i64,
    pub cache_hit_rate: f64,
    pub last_measured_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardWidgetLibrarySummaryV20 {
    pub total_widgets: i64,
    pub total_categories: i64,
    pub most_used_widgets: Vec<DashboardWidgetLibraryV20>,
    pub category_counts: HashMap<String, i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportGenerationQueueSummaryV20 {
    pub total_queued: i64,
    pub pending_count: i64,
    pub processing_count: i64,
    pub completed_count: i64,
    pub failed_count: i64,
    pub avg_generation_time_ms: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardAnalyticsSummaryV23 {
    pub total_dashboards: i64,
    pub total_shares: i64,
    pub total_views: i64,
    pub avg_load_time_ms: f64,
    pub most_viewed_dashboards: Vec<DashboardViewCountV23>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardViewCountV23 {
    pub dashboard_id: Uuid,
    pub dashboard_name: String,
    pub view_count: i64,
    pub last_viewed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardWidgetLibraryRequestV20 {
    pub category: Option<String>,
    pub r#type: Option<String>,
    pub search: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportGenerationQueueRequestV20 {
    pub status: Option<String>,
    pub since: Option<DateTime<Utc>>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}
