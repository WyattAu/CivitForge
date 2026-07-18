#![forbid(unsafe_code)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ExportFormat {
    Json,
    Csv,
    GitArchive,
}

impl std::fmt::Display for ExportFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExportFormat::Json => write!(f, "json"),
            ExportFormat::Csv => write!(f, "csv"),
            ExportFormat::GitArchive => write!(f, "git_archive"),
        }
    }
}

impl std::str::FromStr for ExportFormat {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "json" => Ok(ExportFormat::Json),
            "csv" => Ok(ExportFormat::Csv),
            "git_archive" => Ok(ExportFormat::GitArchive),
            _ => Err(format!("unknown export format: {s}")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportRequest {
    pub organization_id: Uuid,
    pub format: ExportFormat,
    pub data_types: Vec<String>,
    pub repo_ids: Option<Vec<Uuid>>,
    pub date_from: Option<DateTime<Utc>>,
    pub date_to: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ExportJobRecord {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub user_id: Uuid,
    pub format: String,
    pub status: String,
    pub data_types: serde_json::Value,
    pub repo_ids: Option<serde_json::Value>,
    pub date_from: Option<DateTime<Utc>>,
    pub date_to: Option<DateTime<Utc>>,
    pub file_path: Option<String>,
    pub file_size_bytes: i64,
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

impl ExportJobRecord {
    pub fn data_types_list(&self) -> Vec<String> {
        self.data_types
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportRequest {
    pub source: String,
    pub source_url: String,
    pub api_token: Option<String>,
    pub organization_id: Uuid,
    pub repo_mapping: Option<serde_json::Value>,
    pub conflict_resolution: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ImportJobRecord {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub user_id: Uuid,
    pub source: String,
    pub status: String,
    pub items_imported: i64,
    pub items_skipped: i64,
    pub items_failed: i64,
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ExportSchedule {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub cron_expression: String,
    pub format: String,
    pub data_types: serde_json::Value,
    pub enabled: bool,
    pub last_run_at: Option<DateTime<Utc>>,
    pub next_run_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

impl ExportSchedule {
    pub fn data_types_list(&self) -> Vec<String> {
        self.data_types
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ExportNotification {
    pub id: Uuid,
    pub export_job_id: Uuid,
    pub notification_type: String,
    pub recipient: String,
    pub sent: bool,
    pub sent_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportValidationResult {
    pub valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub items_preview: Vec<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictResolutionResult {
    pub resolved_count: usize,
    pub skipped_count: usize,
    pub details: Vec<serde_json::Value>,
}

pub struct DataPortabilityService {
    pool: PgPool,
}

impl DataPortabilityService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create_export_job(
        &self,
        request: &ExportRequest,
        user_id: Uuid,
    ) -> Result<ExportJobRecord, sqlx::Error> {
        let id = Uuid::new_v4();
        let format_str = request.format.to_string();
        let data_types = serde_json::to_value(&request.data_types).unwrap_or_default();
        let repo_ids = request.repo_ids.as_ref().map(|ids| {
            serde_json::to_value(ids.iter().map(|id| id.to_string()).collect::<Vec<_>>())
                .unwrap_or_default()
        });

        let row = sqlx::query_as::<_, ExportJobRecord>(
            r#"INSERT INTO data_export_jobs
               (id, organization_id, user_id, format, status, data_types, repo_ids, date_from, date_to)
               VALUES ($1, $2, $3, $4, 'pending', $5, $6, $7, $8)
               RETURNING *"#,
        )
        .bind(id)
        .bind(request.organization_id)
        .bind(user_id)
        .bind(&format_str)
        .bind(&data_types)
        .bind(&repo_ids)
        .bind(request.date_from)
        .bind(request.date_to)
        .fetch_one(&self.pool)
        .await?;

        Ok(row)
    }

    pub async fn get_export_job(&self, id: Uuid) -> Result<Option<ExportJobRecord>, sqlx::Error> {
        let row = sqlx::query_as::<_, ExportJobRecord>(
            "SELECT * FROM data_export_jobs WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row)
    }

    pub async fn update_export_status(
        &self,
        id: Uuid,
        status: &str,
        file_path: Option<&str>,
        file_size: i64,
        error: Option<&str>,
    ) -> Result<ExportJobRecord, sqlx::Error> {
        let row = sqlx::query_as::<_, ExportJobRecord>(
            r#"UPDATE data_export_jobs
               SET status = $2, file_path = $3, file_size_bytes = $4, error_message = $5,
                   completed_at = CASE WHEN $2 IN ('completed', 'failed') THEN NOW() ELSE completed_at END
               WHERE id = $1
               RETURNING *"#,
        )
        .bind(id)
        .bind(status)
        .bind(file_path)
        .bind(file_size)
        .bind(error)
        .fetch_one(&self.pool)
        .await?;
        Ok(row)
    }

    pub async fn list_export_jobs(
        &self,
        organization_id: Uuid,
        limit: i64,
    ) -> Result<Vec<ExportJobRecord>, sqlx::Error> {
        let rows = sqlx::query_as::<_, ExportJobRecord>(
            r#"SELECT * FROM data_export_jobs
               WHERE organization_id = $1
               ORDER BY created_at DESC
               LIMIT $2"#,
        )
        .bind(organization_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows)
    }

    pub async fn create_import_job(
        &self,
        request: &ImportRequest,
        user_id: Uuid,
    ) -> Result<ImportJobRecord, sqlx::Error> {
        let id = Uuid::new_v4();

        let row = sqlx::query_as::<_, ImportJobRecord>(
            r#"INSERT INTO data_import_jobs
               (id, organization_id, user_id, source, status)
               VALUES ($1, $2, $3, $4, 'pending')
               RETURNING *"#,
        )
        .bind(id)
        .bind(request.organization_id)
        .bind(user_id)
        .bind(&request.source)
        .fetch_one(&self.pool)
        .await?;

        Ok(row)
    }

    pub async fn get_import_job(&self, id: Uuid) -> Result<Option<ImportJobRecord>, sqlx::Error> {
        let row = sqlx::query_as::<_, ImportJobRecord>(
            "SELECT * FROM data_import_jobs WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row)
    }

    pub async fn update_import_status(
        &self,
        id: Uuid,
        status: &str,
        imported: i64,
        skipped: i64,
        failed: i64,
        error: Option<&str>,
    ) -> Result<ImportJobRecord, sqlx::Error> {
        let row = sqlx::query_as::<_, ImportJobRecord>(
            r#"UPDATE data_import_jobs
               SET status = $2, items_imported = $3, items_skipped = $4, items_failed = $5, error_message = $6,
                   completed_at = CASE WHEN $2 IN ('completed', 'failed') THEN NOW() ELSE completed_at END
               WHERE id = $1
               RETURNING *"#,
        )
        .bind(id)
        .bind(status)
        .bind(imported)
        .bind(skipped)
        .bind(failed)
        .bind(error)
        .fetch_one(&self.pool)
        .await?;
        Ok(row)
    }

    pub async fn create_schedule(
        &self,
        organization_id: Uuid,
        cron: &str,
        format: &str,
        data_types: &[String],
    ) -> Result<ExportSchedule, sqlx::Error> {
        let id = Uuid::new_v4();
        let types_val = serde_json::to_value(data_types).unwrap_or_default();

        let row = sqlx::query_as::<_, ExportSchedule>(
            r#"INSERT INTO data_export_schedules
               (id, organization_id, cron_expression, format, data_types, enabled)
               VALUES ($1, $2, $3, $4, $5, true)
               RETURNING *"#,
        )
        .bind(id)
        .bind(organization_id)
        .bind(cron)
        .bind(format)
        .bind(&types_val)
        .fetch_one(&self.pool)
        .await?;

        Ok(row)
    }

    pub async fn list_schedules(
        &self,
        organization_id: Uuid,
    ) -> Result<Vec<ExportSchedule>, sqlx::Error> {
        let rows = sqlx::query_as::<_, ExportSchedule>(
            r#"SELECT * FROM data_export_schedules
               WHERE organization_id = $1
               ORDER BY created_at DESC"#,
        )
        .bind(organization_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows)
    }

    pub async fn create_notification(
        &self,
        export_job_id: Uuid,
        notification_type: &str,
        recipient: &str,
    ) -> Result<ExportNotification, sqlx::Error> {
        let id = Uuid::new_v4();

        let row = sqlx::query_as::<_, ExportNotification>(
            r#"INSERT INTO data_export_notifications
               (id, export_job_id, notification_type, recipient, sent)
               VALUES ($1, $2, $3, $4, false)
               RETURNING *"#,
        )
        .bind(id)
        .bind(export_job_id)
        .bind(notification_type)
        .bind(recipient)
        .fetch_one(&self.pool)
        .await?;

        Ok(row)
    }

    pub async fn validate_import_data(
        &self,
        data: &serde_json::Value,
        source: &str,
    ) -> ImportValidationResult {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();
        let mut items_preview = Vec::new();

        if data.is_null() {
            errors.push("import data is null".into());
            return ImportValidationResult {
                valid: false,
                errors,
                warnings,
                items_preview,
            };
        }

        let valid_sources = ["github", "gitlab", "gitea"];
        if !valid_sources.contains(&source) {
            errors.push(format!("unsupported source: {source}"));
        }

        if let Some(arr) = data.as_array() {
            for item in arr.iter().take(5) {
                items_preview.push(item.clone());
            }
            if arr.len() > 1000 {
                warnings.push(format!(
                    "large import: {} items may take a while",
                    arr.len()
                ));
            }
        } else {
            errors.push("import data must be a JSON array".into());
        }

        ImportValidationResult {
            valid: errors.is_empty(),
            errors,
            warnings,
            items_preview,
        }
    }

    pub async fn resolve_conflicts(
        &self,
        existing: &serde_json::Value,
        incoming: &serde_json::Value,
        strategy: &str,
    ) -> ConflictResolutionResult {
        let mut resolved = 0usize;
        let mut skipped = 0usize;
        let mut details = Vec::new();

        match strategy {
            "overwrite" => {
                resolved += 1;
                details.push(serde_json::json!({
                    "action": "overwrite",
                    "item": incoming,
                }));
            }
            "skip" => {
                skipped += 1;
                details.push(serde_json::json!({
                    "action": "skip",
                    "item": incoming,
                }));
            }
            "merge" => {
                if let (Some(e_obj), Some(i_obj)) = (existing.as_object(), incoming.as_object()) {
                    let mut merged = e_obj.clone();
                    for (k, v) in i_obj {
                        merged.insert(k.clone(), v.clone());
                    }
                    resolved += 1;
                    details.push(serde_json::json!({
                        "action": "merge",
                        "result": serde_json::Value::Object(merged),
                    }));
                } else {
                    skipped += 1;
                }
            }
            _ => {
                skipped += 1;
                details.push(serde_json::json!({
                    "action": "skip_unknown_strategy",
                    "strategy": strategy,
                }));
            }
        }

        ConflictResolutionResult {
            resolved_count: resolved,
            skipped_count: skipped,
            details,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_export_format_display() {
        assert_eq!(ExportFormat::Json.to_string(), "json");
        assert_eq!(ExportFormat::Csv.to_string(), "csv");
        assert_eq!(ExportFormat::GitArchive.to_string(), "git_archive");
    }

    #[test]
    fn test_export_format_parse() {
        assert_eq!("json".parse::<ExportFormat>().unwrap(), ExportFormat::Json);
        assert_eq!("csv".parse::<ExportFormat>().unwrap(), ExportFormat::Csv);
        assert_eq!("git_archive".parse::<ExportFormat>().unwrap(), ExportFormat::GitArchive);
        assert!("unknown".parse::<ExportFormat>().is_err());
    }

    #[test]
    fn test_export_job_record_serialization() {
        let job = ExportJobRecord {
            id: Uuid::new_v4(),
            organization_id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            format: "json".into(),
            status: "completed".into(),
            data_types: serde_json::json!(["repos", "issues"]),
            repo_ids: None,
            date_from: None,
            date_to: None,
            file_path: Some("/tmp/export.json".into()),
            file_size_bytes: 1024,
            error_message: None,
            created_at: Utc::now(),
            completed_at: Some(Utc::now()),
        };
        let json = serde_json::to_string(&job).unwrap();
        assert!(json.contains("\"format\":\"json\""));
        assert!(json.contains("\"status\":\"completed\""));
    }

    #[test]
    fn test_export_job_record_data_types_list() {
        let job = ExportJobRecord {
            id: Uuid::nil(),
            organization_id: Uuid::nil(),
            user_id: Uuid::nil(),
            format: "json".into(),
            status: "completed".into(),
            data_types: serde_json::json!(["repos", "issues"]),
            repo_ids: None,
            date_from: None,
            date_to: None,
            file_path: None,
            file_size_bytes: 0,
            error_message: None,
            created_at: Utc::now(),
            completed_at: None,
        };
        let types = job.data_types_list();
        assert_eq!(types.len(), 2);
        assert!(types.contains(&"repos".to_string()));
    }

    #[test]
    fn test_export_schedule_data_types_list() {
        let schedule = ExportSchedule {
            id: Uuid::nil(),
            organization_id: Uuid::nil(),
            cron_expression: "0 0 * * *".into(),
            format: "json".into(),
            data_types: serde_json::json!(["repos"]),
            enabled: true,
            last_run_at: None,
            next_run_at: None,
            created_at: Utc::now(),
        };
        let types = schedule.data_types_list();
        assert_eq!(types, vec!["repos"]);
    }

    #[test]
    fn test_import_request_serialization() {
        let req = ImportRequest {
            source: "github".into(),
            source_url: "https://github.com/org/repo".into(),
            api_token: Some("ghp_test".into()),
            organization_id: Uuid::new_v4(),
            repo_mapping: None,
            conflict_resolution: "skip".into(),
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("\"source\":\"github\""));
    }

    #[test]
    fn test_export_request_serialization() {
        let req = ExportRequest {
            organization_id: Uuid::new_v4(),
            format: ExportFormat::Json,
            data_types: vec!["repos".into()],
            repo_ids: None,
            date_from: None,
            date_to: None,
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("\"format\":\"json\""));
    }

    #[tokio::test]
    async fn test_validate_import_data_null() {
        let svc = DataPortabilityService {
            pool: PgPool::connect_lazy("postgres://localhost/test").unwrap(),
        };
        let result = svc.validate_import_data(&serde_json::Value::Null, "github").await;
        assert!(!result.valid);
        assert!(!result.errors.is_empty());
    }

    #[tokio::test]
    async fn test_validate_import_data_valid() {
        let svc = DataPortabilityService {
            pool: PgPool::connect_lazy("postgres://localhost/test").unwrap(),
        };
        let data = serde_json::json!([{"name": "repo1"}, {"name": "repo2"}]);
        let result = svc.validate_import_data(&data, "github").await;
        assert!(result.valid);
        assert!(result.errors.is_empty());
        assert_eq!(result.items_preview.len(), 2);
    }

    #[tokio::test]
    async fn test_validate_import_data_bad_source() {
        let svc = DataPortabilityService {
            pool: PgPool::connect_lazy("postgres://localhost/test").unwrap(),
        };
        let data = serde_json::json!([{"name": "repo1"}]);
        let result = svc.validate_import_data(&data, "bitbucket").await;
        assert!(!result.valid);
    }

    #[tokio::test]
    async fn test_resolve_conflicts_overwrite() {
        let svc = DataPortabilityService {
            pool: PgPool::connect_lazy("postgres://localhost/test").unwrap(),
        };
        let existing = serde_json::json!({"name": "old"});
        let incoming = serde_json::json!({"name": "new"});
        let result = svc.resolve_conflicts(&existing, &incoming, "overwrite").await;
        assert_eq!(result.resolved_count, 1);
        assert_eq!(result.skipped_count, 0);
    }

    #[tokio::test]
    async fn test_resolve_conflicts_skip() {
        let svc = DataPortabilityService {
            pool: PgPool::connect_lazy("postgres://localhost/test").unwrap(),
        };
        let existing = serde_json::json!({});
        let incoming = serde_json::json!({"name": "new"});
        let result = svc.resolve_conflicts(&existing, &incoming, "skip").await;
        assert_eq!(result.skipped_count, 1);
    }

    #[tokio::test]
    async fn test_resolve_conflicts_merge() {
        let svc = DataPortabilityService {
            pool: PgPool::connect_lazy("postgres://localhost/test").unwrap(),
        };
        let existing = serde_json::json!({"a": 1});
        let incoming = serde_json::json!({"b": 2});
        let result = svc.resolve_conflicts(&existing, &incoming, "merge").await;
        assert_eq!(result.resolved_count, 1);
    }
}
