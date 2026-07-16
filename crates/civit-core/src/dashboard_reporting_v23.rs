#![forbid(unsafe_code)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

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
    pub category_counts: std::collections::HashMap<String, i64>,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dashboard_widget_library_v20_serialization() {
        let widget = DashboardWidgetLibraryV20 {
            id: Uuid::new_v4(),
            name: "CPU Usage Chart".to_string(),
            r#type: "line_chart".to_string(),
            category: "monitoring".to_string(),
            config: serde_json::json!({"refresh_interval": 30}),
            preview_url: Some("/previews/cpu_chart.png".to_string()),
            usage_count: 42,
            created_at: Utc::now(),
        };
        let json = serde_json::to_string(&widget).unwrap();
        assert!(json.contains("CPU Usage Chart"));
        assert!(json.contains("line_chart"));
    }

    #[test]
    fn test_report_generation_queue_v20_serialization() {
        let queue = ReportGenerationQueueV20 {
            id: Uuid::new_v4(),
            report_id: Uuid::new_v4(),
            status: "processing".to_string(),
            priority: 1,
            scheduled_at: Utc::now(),
            started_at: Some(Utc::now()),
            completed_at: None,
            error_message: None,
        };
        let json = serde_json::to_string(&queue).unwrap();
        assert!(json.contains("processing"));
        assert!(json.contains("priority"));
    }

    #[test]
    fn test_create_dashboard_widget_library_v20() {
        let req = CreateDashboardWidgetLibraryV20 {
            name: "Memory Usage Gauge".to_string(),
            r#type: "gauge".to_string(),
            category: Some("monitoring".to_string()),
            config: Some(serde_json::json!({"max_value": 100})),
            preview_url: None,
        };
        assert_eq!(req.name, "Memory Usage Gauge");
        assert_eq!(req.r#type, "gauge");
    }

    #[test]
    fn test_dashboard_sharing_analytics_v23() {
        let analytics = DashboardSharingAnalyticsV23 {
            dashboard_id: Uuid::new_v4(),
            total_shares: 30,
            unique_viewers: 20,
            share_count_last_24h: 8,
            share_count_last_7d: 25,
            top_sharers: vec![],
        };
        assert_eq!(analytics.total_shares, 30);
        assert_eq!(analytics.unique_viewers, 20);
    }

    #[test]
    fn test_report_generation_queue_summary_v20() {
        let summary = ReportGenerationQueueSummaryV20 {
            total_queued: 100,
            pending_count: 10,
            processing_count: 2,
            completed_count: 85,
            failed_count: 3,
            avg_generation_time_ms: 2500.0,
        };
        assert_eq!(summary.total_queued, 100);
        assert_eq!(summary.completed_count, 85);
    }
}
