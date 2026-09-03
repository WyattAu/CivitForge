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

pub const VALID_DIRECTIONS: &[&str] = &["push", "pull", "both"];

pub fn validate_direction(direction: &str) -> bool {
    VALID_DIRECTIONS.contains(&direction)
}

pub fn validate_mirror_url(url: &str) -> Result<(), String> {
    if url.is_empty() {
        return Err("URL must not be empty".into());
    }
    // git@ URLs are SSH-style, not HTTP — allow directly
    if url.starts_with("git@") {
        return Ok(());
    }
    // Migrated to validkit: HttpsUrl::try_from validates https URL with ValidError, mapping to String.
    // For http URLs, fallback to permissive check to keep backwards compat (but prefer https).
    if url.starts_with("https://") {
        validkit::HttpsUrl::try_from(url)
            .map(|_| ())
            .map_err(|e| e.to_string())?;
        if !url.contains('.') && !url.contains("localhost") {
            return Err("URL must contain a valid hostname".into());
        }
        return Ok(());
    }
    if url.starts_with("http://") {
        // Keep http allowance via legacy check, but also ensure validkit-like host check
        if !url.contains('.') && !url.contains("localhost") {
            return Err("URL must contain a valid hostname".into());
        }
        // Optionally validate via url::Url if available, but for now accept http as valid
        return Ok(());
    }
    Err("URL must start with http://, https://, or git@".into())
}

pub fn compute_next_sync(
    last_sync: Option<chrono::DateTime<chrono::Utc>>,
    interval_minutes: i32,
) -> chrono::DateTime<chrono::Utc> {
    match last_sync {
        Some(last) => last + chrono::Duration::minutes(interval_minutes as i64),
        None => chrono::Utc::now(),
    }
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

    #[test]
    fn test_create_mirror_request_both_direction() {
        let json = r#"{"url":"https://example.com/repo.git","direction":"both"}"#;
        let req: CreateMirrorRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.direction, "both");
    }

    #[test]
    fn test_validate_direction_valid() {
        assert!(validate_direction("push"));
        assert!(validate_direction("pull"));
        assert!(validate_direction("both"));
    }

    #[test]
    fn test_validate_direction_invalid() {
        assert!(!validate_direction("up"));
        assert!(!validate_direction("down"));
        assert!(!validate_direction(""));
        assert!(!validate_direction("PUSH"));
    }

    #[test]
    fn test_validate_mirror_url_https() {
        assert!(validate_mirror_url("https://github.com/user/repo.git").is_ok());
    }

    #[test]
    fn test_validate_mirror_url_http() {
        assert!(validate_mirror_url("http://gitlab.com/user/repo.git").is_ok());
    }

    #[test]
    fn test_validate_mirror_url_git_ssh() {
        assert!(validate_mirror_url("git@github.com:user/repo.git").is_ok());
    }

    #[test]
    fn test_validate_mirror_url_localhost() {
        assert!(validate_mirror_url("https://localhost/repo.git").is_ok());
    }

    #[test]
    fn test_validate_mirror_url_empty() {
        assert!(validate_mirror_url("").is_err());
    }

    #[test]
    fn test_validate_mirror_url_no_protocol() {
        assert!(validate_mirror_url("github.com/repo.git").is_err());
    }

    #[test]
    fn test_validate_mirror_url_ftp() {
        assert!(validate_mirror_url("ftp://example.com/repo.git").is_err());
    }

    #[test]
    fn test_compute_next_sync_with_last_sync() {
        let last = chrono::DateTime::parse_from_rfc3339("2024-01-01T00:00:00Z")
            .unwrap()
            .with_timezone(&chrono::Utc);
        let next = compute_next_sync(Some(last), 30);
        let expected = last + chrono::Duration::minutes(30);
        assert_eq!(next, expected);
    }

    #[test]
    fn test_compute_next_sync_no_last_sync() {
        let before = chrono::Utc::now();
        let next = compute_next_sync(None, 60);
        let after = chrono::Utc::now();
        assert!(next >= before);
        assert!(next <= after);
    }

    #[test]
    fn test_update_mirror_request_parse() {
        let json = r#"{"url":"https://new-url.com/repo.git","direction":"pull"}"#;
        let req: UpdateMirrorRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.url, Some("https://new-url.com/repo.git".into()));
        assert_eq!(req.direction, Some("pull".into()));
        assert!(req.enabled.is_none());
        assert!(req.sync_interval_minutes.is_none());
    }

    #[test]
    fn test_update_mirror_request_empty() {
        let json = r#"{}"#;
        let req: UpdateMirrorRequest = serde_json::from_str(json).unwrap();
        assert!(req.url.is_none());
        assert!(req.direction.is_none());
        assert!(req.enabled.is_none());
        assert!(req.sync_interval_minutes.is_none());
    }

    #[test]
    fn test_valid_directions_constant() {
        assert_eq!(VALID_DIRECTIONS, &["push", "pull", "both"]);
    }

    #[tokio::test]
    async fn test_handle_sync_output_success() {
        use std::os::unix::process::ExitStatusExt;
        let output = Ok(std::process::Output {
            status: std::process::ExitStatus::from_raw(0),
            stdout: b"done".to_vec(),
            stderr: b"warn: something".to_vec(),
        });
        let result = handle_sync_output(output).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "warn: something");
    }

    #[tokio::test]
    async fn test_handle_sync_output_failure() {
        use std::os::unix::process::ExitStatusExt;
        let output = Ok(std::process::Output {
            status: std::process::ExitStatus::from_raw(1),
            stdout: vec![],
            stderr: b"error: failed".to_vec(),
        });
        let result = handle_sync_output(output).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "error: failed");
    }

    #[tokio::test]
    async fn test_handle_sync_output_io_error() {
        let output = Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "git not found",
        ));
        let result = handle_sync_output(output).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("git not found"));
    }

    #[test]
    fn test_validate_mirror_url_with_port() {
        assert!(validate_mirror_url("https://example.com:8080/repo.git").is_ok());
    }

    #[test]
    fn test_validate_mirror_url_with_path_params() {
        assert!(validate_mirror_url("https://example.com/repo.git?token=abc").is_ok());
    }

    #[test]
    fn test_validate_mirror_url_git_at_with_port() {
        // git@ URLs don't have the dot check
        assert!(validate_mirror_url("git@server:repo.git").is_ok());
    }

    #[test]
    fn test_validate_mirror_url_http_no_dot() {
        // http without dot and not localhost should fail
        assert!(validate_mirror_url("http://server").is_err());
    }

    #[test]
    fn test_validate_mirror_url_ftp_protocol() {
        assert!(validate_mirror_url("ftp://example.com/repo.git").is_err());
    }

    #[test]
    fn test_validate_mirror_url_ssh_protocol() {
        assert!(validate_mirror_url("ssh://git@example.com/repo.git").is_err());
    }

    #[test]
    fn test_compute_next_sync_large_interval() {
        let last = chrono::DateTime::parse_from_rfc3339("2024-01-01T00:00:00Z")
            .unwrap()
            .with_timezone(&chrono::Utc);
        let next = compute_next_sync(Some(last), i32::MAX);
        assert!(next > last);
    }

    #[test]
    fn test_compute_next_sync_zero_interval() {
        let last = chrono::DateTime::parse_from_rfc3339("2024-01-01T00:00:00Z")
            .unwrap()
            .with_timezone(&chrono::Utc);
        let next = compute_next_sync(Some(last), 0);
        assert_eq!(next, last);
    }

    #[test]
    fn test_compute_next_sync_negative_interval() {
        let last = chrono::DateTime::parse_from_rfc3339("2024-01-01T00:00:00Z")
            .unwrap()
            .with_timezone(&chrono::Utc);
        let next = compute_next_sync(Some(last), -30);
        assert!(next < last);
    }

    #[test]
    fn test_create_mirror_request_long_url() {
        let url = format!("https://{}.com/repo.git", "a".repeat(1000));
        let json = format!(r#"{{"url":"{url}","direction":"push"}}"#);
        let req: CreateMirrorRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(req.url.len(), url.len());
    }

    #[test]
    fn test_update_mirror_request_disable() {
        let json = r#"{"enabled": false}"#;
        let req: UpdateMirrorRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.enabled, Some(false));
    }

    #[test]
    fn test_update_mirror_request_change_interval() {
        let json = r#"{"sync_interval_minutes": 30}"#;
        let req: UpdateMirrorRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.sync_interval_minutes, Some(30));
    }

    #[test]
    fn test_create_mirror_request_zero_interval() {
        let json = r#"{"url":"https://example.com/repo.git","direction":"push","sync_interval_minutes":0}"#;
        let req: CreateMirrorRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.sync_interval_minutes, 0);
    }

    #[test]
    fn test_valid_directions_all_checked() {
        for dir in &["push", "pull", "both"] {
            assert!(validate_direction(dir));
        }
        for dir in &["Push", "PUSH", "pull ", " both"] {
            assert!(!validate_direction(dir));
        }
    }
}
