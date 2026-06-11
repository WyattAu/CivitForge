//! Push/pull mirror types and logic.

#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct CreateMirrorRequest {
    pub url: String,
    #[serde(rename = "direction")]
    pub direction: String,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_sync_interval")]
    pub sync_interval_minutes: u32,
}

fn default_true() -> bool {
    true
}

fn default_sync_interval() -> u32 {
    60
}

#[derive(Debug, Deserialize)]
pub struct UpdateMirrorRequest {
    pub url: Option<String>,
    pub direction: Option<String>,
    pub enabled: Option<bool>,
    pub sync_interval_minutes: Option<u32>,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct MirrorRecord {
    pub id: Uuid,
    pub repo_id: Uuid,
    pub url: String,
    pub direction: String,
    pub enabled: bool,
    pub sync_interval_minutes: i32,
    pub last_sync_at: Option<chrono::DateTime<chrono::Utc>>,
    pub last_sync_status: Option<String>,
    pub last_sync_error: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl MirrorRecord {
    pub fn sync_interval(&self) -> i32 {
        self.sync_interval_minutes
    }

    pub fn last_sync(&self) -> Option<&chrono::DateTime<chrono::Utc>> {
        self.last_sync_at.as_ref()
    }

    pub fn repository_id(&self) -> Uuid {
        self.repo_id
    }
}

pub async fn ensure_mirrors_table(pool: &sqlx::postgres::PgPool) {
    let _ = sqlx::query(
        "CREATE TABLE IF NOT EXISTS repo_mirrors (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            repo_id UUID NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
            url TEXT NOT NULL,
            direction TEXT NOT NULL CHECK (direction IN ('push', 'pull', 'both')),
            enabled BOOLEAN NOT NULL DEFAULT true,
            sync_interval_minutes INT NOT NULL DEFAULT 60,
            last_sync_at TIMESTAMPTZ,
            last_sync_status TEXT,
            last_sync_error TEXT,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",
    )
    .execute(pool)
    .await;
}

pub async fn handle_sync_output(
    output: std::result::Result<std::process::Output, std::io::Error>,
) -> Result<String, String> {
    match output {
        Ok(out) => {
            if out.status.success() {
                let stderr = String::from_utf8_lossy(&out.stderr);
                Ok(stderr.to_string())
            } else {
                let stderr = String::from_utf8_lossy(&out.stderr);
                Err(stderr.to_string())
            }
        }
        Err(e) => Err(format!("failed to execute git: {e}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_mirror_request_parse() {
        let json = r#"{"url":"https://github.com/example/repo.git","direction":"push","enabled":true,"sync_interval_minutes":30}"#;
        let req: CreateMirrorRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.url, "https://github.com/example/repo.git");
        assert_eq!(req.direction, "push");
        assert!(req.enabled);
        assert_eq!(req.sync_interval_minutes, 30);
    }

    #[test]
    fn test_create_mirror_request_defaults() {
        let json = r#"{"url":"https://example.com/repo.git","direction":"pull"}"#;
        let req: CreateMirrorRequest = serde_json::from_str(json).unwrap();
        assert!(req.enabled);
        assert_eq!(req.sync_interval_minutes, 60);
    }
}
