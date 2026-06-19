//! Pipeline schedule types (cron-triggered runs).

#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct CreateScheduleRequest {
    pub cron: String,
    pub name: Option<String>,
    pub ref_name: Option<String>,
    #[serde(default = "default_yaml_path")]
    pub yaml_path: String,
    #[serde(default = "default_true")]
    pub enabled: bool,
}

pub fn default_yaml_path() -> String {
    ".civit/pipeline.yaml".to_string()
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Deserialize)]
pub struct UpdateScheduleRequest {
    pub cron: Option<String>,
    pub name: Option<Option<String>>,
    pub ref_name: Option<Option<String>>,
    pub yaml_path: Option<String>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleResponse {
    pub id: String,
    pub repo_id: String,
    pub cron: String,
    pub name: Option<String>,
    pub ref_name: Option<String>,
    pub yaml_path: String,
    pub enabled: bool,
    pub last_run_at: Option<String>,
    pub next_run_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManualRunResponse {
    pub schedule_id: String,
    pub run_id: String,
    pub status: String,
    pub triggered_at: String,
}

#[derive(Debug, sqlx::FromRow)]
pub struct ScheduleRow {
    pub id: Uuid,
    pub repo_id: Uuid,
    pub cron: String,
    pub name: Option<String>,
    pub ref_name: Option<String>,
    pub yaml_path: String,
    pub enabled: bool,
    pub last_run_at: Option<chrono::DateTime<chrono::Utc>>,
    pub next_run_at: Option<chrono::DateTime<chrono::Utc>>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl From<ScheduleRow> for ScheduleResponse {
    fn from(r: ScheduleRow) -> Self {
        Self {
            id: r.id.to_string(),
            repo_id: r.repo_id.to_string(),
            cron: r.cron,
            name: r.name,
            ref_name: r.ref_name,
            yaml_path: r.yaml_path,
            enabled: r.enabled,
            last_run_at: r.last_run_at.map(|t| t.to_rfc3339()),
            next_run_at: r.next_run_at.map(|t| t.to_rfc3339()),
            created_at: r.created_at.to_rfc3339(),
            updated_at: r.updated_at.to_rfc3339(),
        }
    }
}

pub async fn list_schedules_db(
    pool: &sqlx::PgPool,
    repo_id: Uuid,
) -> std::result::Result<Vec<ScheduleResponse>, sqlx::Error> {
    let rows: Vec<ScheduleRow> = sqlx::query_as(
        "SELECT id, repo_id, cron, name, ref_name, yaml_path, enabled, last_run_at, next_run_at, created_at, updated_at FROM pipeline_schedules WHERE repo_id = $1 ORDER BY created_at",
    )
    .bind(repo_id)
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(|r| r.into()).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_schedule_request_deserialize() {
        let json = r#"{"cron": "0 6 * * 1", "name": "weekly"}"#;
        let req: CreateScheduleRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.cron, "0 6 * * 1");
        assert_eq!(req.name, Some("weekly".to_string()));
        assert_eq!(req.yaml_path, ".civit/pipeline.yaml");
        assert!(req.enabled);
    }

    #[test]
    fn test_schedule_response_serialize() {
        let resp = ScheduleResponse {
            id: "00000000-0000-0000-0000-000000000001".to_string(),
            repo_id: "00000000-0000-0000-0000-000000000002".to_string(),
            cron: "0 6 * * 1".to_string(),
            name: Some("weekly".to_string()),
            ref_name: Some("main".to_string()),
            yaml_path: ".civit/pipeline.yaml".to_string(),
            enabled: true,
            last_run_at: None,
            next_run_at: Some("2025-06-02T06:00:00+00:00".to_string()),
            created_at: "2025-06-01T00:00:00+00:00".to_string(),
            updated_at: "2025-06-01T00:00:00+00:00".to_string(),
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("0 6 * * 1"));
    }

    #[test]
    fn test_cron_expressions() {
        let crons = [
            "0 6 * * 1",
            "*/15 * * * *",
            "0 0 1 * *",
            "30 4 * * 0",
            "0 22 * * 1-5",
            "0 0 * * 1,4",
        ];
        for cron in crons {
            let req = CreateScheduleRequest {
                cron: cron.to_string(),
                name: None,
                ref_name: None,
                yaml_path: default_yaml_path(),
                enabled: true,
            };
            assert_eq!(req.cron, cron);
        }
    }

    #[test]
    fn test_create_schedule_request_defaults() {
        let json = r#"{"cron": "0 * * * *"}"#;
        let req: CreateScheduleRequest = serde_json::from_str(json).unwrap();
        assert!(req.enabled);
        assert_eq!(req.yaml_path, ".civit/pipeline.yaml");
        assert!(req.name.is_none());
        assert!(req.ref_name.is_none());
    }

    #[test]
    fn test_create_schedule_request_disabled() {
        let json = r#"{"cron": "0 0 * * *", "enabled": false}"#;
        let req: CreateScheduleRequest = serde_json::from_str(json).unwrap();
        assert!(!req.enabled);
    }

    #[test]
    fn test_create_schedule_request_with_ref() {
        let json = r#"{"cron": "0 6 * * *", "ref_name": "develop", "name": "nightly"}"#;
        let req: CreateScheduleRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.ref_name.as_deref(), Some("develop"));
        assert_eq!(req.name.as_deref(), Some("nightly"));
    }

    #[test]
    fn test_update_schedule_request_partial() {
        let json = r#"{"enabled": false}"#;
        let req: UpdateScheduleRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.enabled, Some(false));
        assert!(req.cron.is_none());
    }

    #[test]
    fn test_update_schedule_request_name_to_none() {
        let json = r#"{"name": null}"#;
        let req: UpdateScheduleRequest = serde_json::from_str(json).unwrap();
        // name field is present but null — may deserialize as None or Some(None)
        // depending on serde version; just ensure it's not a string
        assert!(req.name.as_ref().and_then(|o| o.as_ref()).is_none());
    }

    #[test]
    fn test_update_schedule_request_cron_update() {
        let json = r#"{"cron": "0 8 * * 1-5"}"#;
        let req: UpdateScheduleRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.cron.as_deref(), Some("0 8 * * 1-5"));
    }

    #[test]
    fn test_manual_run_response_serialize() {
        let resp = ManualRunResponse {
            schedule_id: "sch-1".into(),
            run_id: "run-1".into(),
            status: "pending".into(),
            triggered_at: "2025-06-01T06:00:00Z".into(),
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("sch-1"));
        assert!(json.contains("pending"));
    }

    #[test]
    fn test_schedule_response_no_next_run() {
        let resp = ScheduleResponse {
            id: "s1".into(),
            repo_id: "r1".into(),
            cron: "0 0 30 2 *".into(),
            name: None,
            ref_name: None,
            yaml_path: ".civit/pipeline.yaml".into(),
            enabled: false,
            last_run_at: None,
            next_run_at: None,
            created_at: "2025-01-01T00:00:00Z".into(),
            updated_at: "2025-01-01T00:00:00Z".into(),
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("false"));
    }

    #[test]
    fn test_schedule_response_with_last_run() {
        let resp = ScheduleResponse {
            id: "s1".into(),
            repo_id: "r1".into(),
            cron: "0 0 * * *".into(),
            name: Some("daily".into()),
            ref_name: None,
            yaml_path: ".civit/pipeline.yaml".into(),
            enabled: true,
            last_run_at: Some("2025-06-01T00:00:00Z".into()),
            next_run_at: Some("2025-06-02T00:00:00Z".into()),
            created_at: "2025-01-01T00:00:00Z".into(),
            updated_at: "2025-06-01T00:00:00Z".into(),
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("daily"));
        assert!(json.contains("2025-06-01T00:00:00Z"));
    }

    #[test]
    fn test_create_schedule_request_empty_cron() {
        let json = r#"{"cron": ""}"#;
        let req: CreateScheduleRequest = serde_json::from_str(json).unwrap();
        assert!(req.cron.is_empty());
    }

    #[test]
    fn test_update_schedule_request_all_fields() {
        let json = r#"{"cron": "0 12 * * *", "name": "new-name", "ref_name": "develop", "yaml_path": ".civit/new.yaml", "enabled": false}"#;
        let req: UpdateScheduleRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.cron.as_deref(), Some("0 12 * * *"));
        assert_eq!(req.name, Some(Some("new-name".into())));
        assert_eq!(req.ref_name, Some(Some("develop".into())));
        assert_eq!(req.yaml_path.as_deref(), Some(".civit/new.yaml"));
        assert_eq!(req.enabled, Some(false));
    }

    #[test]
    fn test_manual_run_response_empty_strings() {
        let resp = ManualRunResponse {
            schedule_id: "".into(),
            run_id: "".into(),
            status: "".into(),
            triggered_at: "".into(),
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("\"schedule_id\":\"\""));
    }

    #[test]
    fn test_schedule_response_special_cron_chars() {
        let resp = ScheduleResponse {
            id: "s1".into(),
            repo_id: "r1".into(),
            cron: "0 0 1,15 * *".into(),
            name: None,
            ref_name: None,
            yaml_path: ".civit/pipeline.yaml".into(),
            enabled: true,
            last_run_at: None,
            next_run_at: None,
            created_at: "2025-01-01T00:00:00Z".into(),
            updated_at: "2025-01-01T00:00:00Z".into(),
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("0 0 1,15 * *"));
    }

    #[test]
    fn test_update_schedule_request_name_to_some() {
        let json = r#"{"name": "renamed"}"#;
        let req: UpdateScheduleRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.name, Some(Some("renamed".into())));
    }

    #[test]
    fn test_schedule_response_long_name() {
        let resp = ScheduleResponse {
            id: "s1".into(),
            repo_id: "r1".into(),
            cron: "0 0 * * *".into(),
            name: Some("a".repeat(256)),
            ref_name: None,
            yaml_path: ".civit/pipeline.yaml".into(),
            enabled: true,
            last_run_at: None,
            next_run_at: None,
            created_at: "2025-01-01T00:00:00Z".into(),
            updated_at: "2025-01-01T00:00:00Z".into(),
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains(&"a".repeat(256)));
    }
}
