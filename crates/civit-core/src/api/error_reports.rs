#![forbid(unsafe_code)]

use axum::{
    Router,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::post,
};
use serde::{Deserialize, Serialize};

use crate::api::AppState;

#[derive(Debug, Serialize, Deserialize)]
pub struct ClientErrorReport {
    pub source: String,
    pub message: String,
    pub url: String,
    pub stack: Option<String>,
    pub component: Option<String>,
    pub user_agent: Option<String>,
    pub timestamp: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorReportResponse {
    pub id: String,
    pub status: String,
}

async fn submit_error_report(
    State(_state): State<AppState>,
    Json(report): Json<ClientErrorReport>,
) -> impl IntoResponse {
    tracing::error!(
        source = %report.source,
        url = %report.url,
        message = %report.message,
        component = ?report.component,
        user_agent = ?report.user_agent,
        timestamp = %report.timestamp,
        "CLIENT ERROR REPORT"
    );

    if let Some(stack) = &report.stack {
        tracing::error!(stack, "CLIENT ERROR STACK");
    }

    let id = uuid::Uuid::new_v4().to_string();
    (
        StatusCode::OK,
        Json(ErrorReportResponse {
            id,
            status: "received".to_string(),
        }),
    )
        .into_response()
}

pub fn error_reports_routes() -> Router<AppState> {
    Router::new().route("/debug/error-reports", post(submit_error_report))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_error_report_deserialization() {
        let json = r#"{
            "source": "console",
            "message": "Something went wrong",
            "url": "http://localhost:8080/dashboard",
            "stack": "Error at line 42",
            "component": "Dashboard",
            "user_agent": "Mozilla/5.0",
            "timestamp": "2025-01-01T00:00:00Z"
        }"#;
        let report: ClientErrorReport = serde_json::from_str(json).unwrap();
        assert_eq!(report.source, "console");
        assert_eq!(report.message, "Something went wrong");
        assert_eq!(report.url, "http://localhost:8080/dashboard");
        assert_eq!(report.stack, Some("Error at line 42".to_string()));
        assert_eq!(report.component, Some("Dashboard".to_string()));
        assert_eq!(report.user_agent, Some("Mozilla/5.0".to_string()));
        assert_eq!(report.timestamp, "2025-01-01T00:00:00Z");
    }

    #[test]
    fn test_client_error_report_optional_fields() {
        let json = r#"{
            "source": "unhandled",
            "message": "undefined is not a function",
            "url": "http://localhost:8080",
            "timestamp": "2025-01-01T00:00:00Z"
        }"#;
        let report: ClientErrorReport = serde_json::from_str(json).unwrap();
        assert!(report.stack.is_none());
        assert!(report.component.is_none());
        assert!(report.user_agent.is_none());
    }

    #[test]
    fn test_error_report_response_serialization() {
        let resp = ErrorReportResponse {
            id: "abc-123".into(),
            status: "received".into(),
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("abc-123"));
        assert!(json.contains("received"));
    }

    #[test]
    fn test_error_report_response_roundtrip() {
        let resp = ErrorReportResponse {
            id: uuid::Uuid::new_v4().to_string(),
            status: "received".to_string(),
        };
        let json = serde_json::to_string(&resp).unwrap();
        let de: ErrorReportResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(de.id, resp.id);
        assert_eq!(de.status, resp.status);
    }

    #[test]
    fn test_client_error_report_with_network_source() {
        let report = ClientErrorReport {
            source: "network".into(),
            message: "fetch failed".into(),
            url: "http://localhost:8080/api/v1/repos".into(),
            stack: None,
            component: None,
            user_agent: None,
            timestamp: "2025-06-01T12:00:00Z".into(),
        };
        let json = serde_json::to_string(&report).unwrap();
        let de: ClientErrorReport = serde_json::from_str(&json).unwrap();
        assert_eq!(de.source, "network");
    }

    #[test]
    fn test_client_error_report_with_leptos_source() {
        let report = ClientErrorReport {
            source: "leptos".into(),
            message: "hydration mismatch".into(),
            url: "/settings".into(),
            stack: Some("at Settings (settings.rs:10)".into()),
            component: Some("Settings".into()),
            user_agent: Some("curl/8.0".into()),
            timestamp: "2025-06-01T12:00:00Z".into(),
        };
        let json = serde_json::to_string(&report).unwrap();
        let de: ClientErrorReport = serde_json::from_str(&json).unwrap();
        assert_eq!(de.source, "leptos");
        assert_eq!(de.component, Some("Settings".to_string()));
    }
}
