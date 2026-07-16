#![forbid(unsafe_code)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

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
    pub rating_distribution: std::collections::HashMap<i32, i64>,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dashboard_version_v19_serialization() {
        let version = DashboardVersionV19 {
            id: Uuid::new_v4(),
            dashboard_id: Uuid::new_v4(),
            version: 1,
            definition: serde_json::json!({"widgets": []}),
            change_description: "Initial dashboard".to_string(),
            created_by: Uuid::new_v4(),
            created_at: Utc::now(),
        };
        let json = serde_json::to_string(&version).unwrap();
        assert!(json.contains("Initial dashboard"));
        assert!(json.contains("1"));
    }

    #[test]
    fn test_report_template_rating_v19_serialization() {
        let rating = ReportTemplateRatingV19 {
            id: Uuid::new_v4(),
            template_id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            rating: 5,
            review: "Excellent template".to_string(),
            created_at: Utc::now(),
        };
        let json = serde_json::to_string(&rating).unwrap();
        assert!(json.contains("Excellent template"));
        assert!(json.contains("5"));
    }

    #[test]
    fn test_dashboard_sharing_analytics_v22() {
        let analytics = DashboardSharingAnalyticsV22 {
            dashboard_id: Uuid::new_v4(),
            total_shares: 25,
            unique_viewers: 15,
            share_count_last_24h: 5,
            share_count_last_7d: 20,
            top_sharers: vec![],
        };
        assert_eq!(analytics.total_shares, 25);
        assert_eq!(analytics.unique_viewers, 15);
    }

    #[test]
    fn test_dashboard_performance_v22() {
        let perf = DashboardPerformanceV22 {
            dashboard_id: Uuid::new_v4(),
            avg_load_time_ms: 150.0,
            p95_load_time_ms: 300.0,
            total_loads: 1000,
            cache_hit_rate: 0.85,
            last_measured_at: Utc::now(),
        };
        assert_eq!(perf.total_loads, 1000);
        assert!(perf.cache_hit_rate > 0.8);
    }

    #[test]
    fn test_report_template_rating_stats_v19() {
        let mut rating_distribution = std::collections::HashMap::new();
        rating_distribution.insert(1, 2);
        rating_distribution.insert(2, 3);
        rating_distribution.insert(3, 5);
        rating_distribution.insert(4, 10);
        rating_distribution.insert(5, 20);
        
        let stats = ReportTemplateRatingStatsV19 {
            template_id: Uuid::new_v4(),
            total_ratings: 40,
            average_rating: 4.25,
            rating_distribution,
            total_reviews: 35,
        };
        assert_eq!(stats.total_ratings, 40);
        assert!(stats.average_rating > 4.0);
    }

    #[test]
    fn test_dashboard_version_diff_v19() {
        let diff = DashboardVersionDiffV19 {
            version_a: DashboardVersionV19 {
                id: Uuid::new_v4(),
                dashboard_id: Uuid::new_v4(),
                version: 1,
                definition: serde_json::json!({"widgets": ["widget1"]}),
                change_description: "v1".to_string(),
                created_by: Uuid::new_v4(),
                created_at: Utc::now(),
            },
            version_b: DashboardVersionV19 {
                id: Uuid::new_v4(),
                dashboard_id: Uuid::new_v4(),
                version: 2,
                definition: serde_json::json!({"widgets": ["widget1", "widget2"]}),
                change_description: "v2".to_string(),
                created_by: Uuid::new_v4(),
                created_at: Utc::now(),
            },
            changes: serde_json::json!({"added": ["widget2"]}),
            added_widgets: vec!["widget2".to_string()],
            removed_widgets: vec![],
            modified_widgets: vec![],
        };
        assert_eq!(diff.added_widgets.len(), 1);
        assert!(diff.removed_widgets.is_empty());
    }
}
