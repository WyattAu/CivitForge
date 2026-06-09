#![forbid(unsafe_code)]

use crate::api::AppState;
use crate::error::CoreError;
use axum::{
    Router,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::get,
};
use regex::Regex;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct DetectedSecret {
    pub file: String,
    pub line: usize,
    pub secret_type: String,
    pub masked_value: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct SecretScanResponse {
    pub total_secrets: usize,
    pub secrets: Vec<DetectedSecret>,
    pub scanned_files: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct RotationReportEntry {
    pub secret_type: String,
    pub locations: Vec<String>,
    pub recommendation: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct RotationReport {
    pub total_types: usize,
    pub entries: Vec<RotationReportEntry>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ScanHistoryEntry {
    pub scan_id: String,
    pub repo_owner: String,
    pub repo_name: String,
    pub total_secrets: usize,
    pub scanned_files: usize,
    pub scanned_at: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ScanHistoryResponse {
    pub scans: Vec<ScanHistoryEntry>,
}

struct SecretPattern {
    name: &'static str,
    pattern: Regex,
}

fn secret_patterns() -> Vec<SecretPattern> {
    vec![
        SecretPattern {
            name: "AWS Access Key",
            pattern: Regex::new(r"AKIA[0-9A-Z]{16}").unwrap(),
        },
        SecretPattern {
            name: "GitHub Token",
            pattern: Regex::new(r"ghp_[A-Za-z0-9]{36}").unwrap(),
        },
        SecretPattern {
            name: "GitHub OAuth Token",
            pattern: Regex::new(r"gho_[A-Za-z0-9]{36}").unwrap(),
        },
        SecretPattern {
            name: "GitHub App Token",
            pattern: Regex::new(r"(ghu|ghs)_[A-Za-z0-9]{36}").unwrap(),
        },
        SecretPattern {
            name: "Generic Password",
            pattern: Regex::new(r#"(?i)(password|passwd|pwd)\s*[:=]\s*['\"]([^'\"]{8,})['\"]"#)
                .unwrap(),
        },
        SecretPattern {
            name: "Generic API Key",
            pattern: Regex::new(r#"(?i)(api[_-]?key|apikey)\s*[:=]\s*['\"]([^'\"]{8,})['\"]"#)
                .unwrap(),
        },
        SecretPattern {
            name: "Private Key (PEM)",
            pattern: Regex::new(r"-----BEGIN (RSA |EC |DSA |OPENSSH )?PRIVATE KEY-----").unwrap(),
        },
        SecretPattern {
            name: "JWT Token",
            pattern: Regex::new(r"eyJ[A-Za-z0-9_-]{10,}\.eyJ[A-Za-z0-9_-]{10,}\.[A-Za-z0-9_-]+")
                .unwrap(),
        },
        SecretPattern {
            name: "Database Connection String",
            pattern: Regex::new(r"(?i)(mysql|postgres|postgresql|mongodb|redis|amqp)://[^\s]{10,}")
                .unwrap(),
        },
    ]
}

fn mask_value(value: &str) -> String {
    if value.len() <= 8 {
        "*".repeat(value.len())
    } else {
        let first = &value[..4];
        let last = &value[value.len() - 4..];
        format!("{first}...{last}")
    }
}

fn scan_content(content: &str, file_path: &str, patterns: &[SecretPattern]) -> Vec<DetectedSecret> {
    let mut secrets = Vec::new();
    for (line_idx, line) in content.lines().enumerate() {
        for pat in patterns {
            for mat in pat.pattern.find_iter(line) {
                secrets.push(DetectedSecret {
                    file: file_path.to_string(),
                    line: line_idx + 1,
                    secret_type: pat.name.to_string(),
                    masked_value: mask_value(mat.as_str()),
                });
            }
            // Also check capture groups for password/key patterns
            for cap in pat.pattern.captures_iter(line) {
                if let Some(m) = cap.get(2) {
                    secrets.push(DetectedSecret {
                        file: file_path.to_string(),
                        line: line_idx + 1,
                        secret_type: pat.name.to_string(),
                        masked_value: mask_value(m.as_str()),
                    });
                }
            }
        }
    }
    secrets
}

async fn scan_repo_files(
    state: &AppState,
    owner: &str,
    name: &str,
) -> Result<(Vec<DetectedSecret>, usize), CoreError> {
    let repo_path = state.git_service.repo_path(owner, name);
    if !repo_path.join("HEAD").exists() {
        return Err(CoreError::NotFound("repository not found".into()));
    }

    let repo = gix::open(&repo_path).map_err(|e| CoreError::Git(e.to_string()))?;

    let head_id = repo.head_id().map_err(|e| CoreError::Git(e.to_string()))?;
    let commit = head_id
        .object()
        .map_err(|e| CoreError::Git(e.to_string()))?
        .try_into_commit()
        .map_err(|e| CoreError::Git(e.to_string()))?;

    let tree = commit
        .tree_id()
        .map_err(|e| CoreError::Git(e.to_string()))?
        .object()
        .map_err(|e| CoreError::Git(e.to_string()))?
        .try_into_tree()
        .map_err(|e| CoreError::Git(e.to_string()))?;

    let patterns = secret_patterns();
    let mut all_secrets = Vec::new();
    let mut file_count = 0usize;

    fn walk_tree(
        tree: &gix::Tree<'_>,
        prefix: &str,
        patterns: &[SecretPattern],
        secrets: &mut Vec<DetectedSecret>,
        count: &mut usize,
    ) {
        for entry_result in tree.iter() {
            let entry = match entry_result {
                Ok(e) => e,
                Err(_) => continue,
            };
            let mode = entry.mode();
            if mode.is_tree() {
                if let Some(subtree) = entry.object().ok().and_then(|o| o.try_into_tree().ok()) {
                    let sub_prefix = if prefix.is_empty() {
                        entry.filename().to_string()
                    } else {
                        format!("{}/{}", prefix, entry.filename())
                    };
                    walk_tree(&subtree, &sub_prefix, patterns, secrets, count);
                }
            } else if mode.is_blob() {
                let full_path = if prefix.is_empty() {
                    entry.filename().to_string()
                } else {
                    format!("{}/{}", prefix, entry.filename())
                };
                // Skip binary-looking files by extension
                let ext = full_path.rsplit('.').next().unwrap_or("");
                let skip_exts = [
                    "png", "jpg", "jpeg", "gif", "bmp", "ico", "svg", "webp", "mp3", "mp4", "avi",
                    "mov", "zip", "tar", "gz", "bz2", "7z", "rar", "exe", "dll", "so", "dylib",
                    "o", "a", "bin",
                ];
                if skip_exts.contains(&ext) {
                    continue;
                }

                if let Some(blob) = entry.object().ok().and_then(|o| o.try_into_blob().ok()) {
                    if let Ok(content) = std::str::from_utf8(blob.data.as_ref()) {
                        *count += 1;
                        let found = scan_content(content, &full_path, patterns);
                        secrets.extend(found);
                    }
                }
            }
        }
    }

    walk_tree(&tree, "", &patterns, &mut all_secrets, &mut file_count);

    Ok((all_secrets, file_count))
}

pub fn secret_scanning_routes() -> Router<AppState> {
    Router::new()
        .route(
            "/api/v1/repos/{owner}/{name}/secret-scanning",
            get(scan_secrets),
        )
        .route(
            "/api/v1/repos/{owner}/{name}/secret-scanning/rotate",
            get(rotation_report),
        )
        .route(
            "/api/v1/repos/{owner}/{name}/secret-scanning/history",
            get(scan_history),
        )
}

async fn scan_secrets(
    State(state): State<AppState>,
    Path((owner, name)): Path<(String, String)>,
) -> impl IntoResponse {
    match scan_repo_files(&state, &owner, &name).await {
        Ok((secrets, scanned_files)) => {
            let resp = SecretScanResponse {
                total_secrets: secrets.len(),
                scanned_files,
                secrets,
            };
            (StatusCode::OK, Json(resp)).into_response()
        }
        Err(CoreError::NotFound(msg)) => (
            StatusCode::NOT_FOUND,
            Json(CoreError::NotFound(msg).error_response()),
        )
            .into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(e.error_response())).into_response(),
    }
}

async fn rotation_report(
    State(state): State<AppState>,
    Path((owner, name)): Path<(String, String)>,
) -> impl IntoResponse {
    match scan_repo_files(&state, &owner, &name).await {
        Ok((secrets, _)) => {
            let mut type_map: std::collections::HashMap<String, Vec<String>> =
                std::collections::HashMap::new();
            for secret in &secrets {
                type_map
                    .entry(secret.secret_type.clone())
                    .or_default()
                    .push(format!("{}:{}", secret.file, secret.line));
            }

            let entries: Vec<RotationReportEntry> = type_map
                .into_iter()
                .map(|(secret_type, locations)| {
                    let recommendation = match secret_type.as_str() {
                        "AWS Access Key" => "Rotate the IAM access key immediately via AWS Console. Delete the old key after verifying the new key works.".to_string(),
                        "GitHub Token" | "GitHub OAuth Token" | "GitHub App Token" => "Revoke the token in GitHub Settings > Developer settings > Personal access tokens. Generate a new token with minimal required scopes.".to_string(),
                        "Private Key (PEM)" => "Generate a new key pair. Update all systems using the old public key. Remove the old private key from version control.".to_string(),
                        "JWT Token" => "Invalidate the token by updating the signing secret. Revoke all active sessions using this token.".to_string(),
                        "Database Connection String" => "Change the database password. Update connection strings in all environments. Deploy updated configuration.".to_string(),
                        _ => "Review the detected secret and rotate according to the service provider's documentation.".to_string(),
                    };
                    RotationReportEntry {
                        secret_type,
                        locations,
                        recommendation,
                    }
                })
                .collect();

            let resp = RotationReport {
                total_types: entries.len(),
                entries,
            };
            (StatusCode::OK, Json(resp)).into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(e.error_response())).into_response(),
    }
}

async fn scan_history(
    State(state): State<AppState>,
    Path((owner, name)): Path<(String, String)>,
) -> impl IntoResponse {
    let result = sqlx::query_as::<_, (String, String, String, i64, i64, String)>(
        "SELECT scan_id, repo_owner, repo_name, total_secrets, scanned_files, scanned_at::text FROM secret_scan_results WHERE repo_owner = $1 AND repo_name = $2 ORDER BY scanned_at DESC LIMIT 50",
    )
    .bind(&owner)
    .bind(&name)
    .fetch_all(state.db.pool())
    .await;

    match result {
        Ok(rows) => {
            let scans: Vec<ScanHistoryEntry> = rows
                .into_iter()
                .map(|r| ScanHistoryEntry {
                    scan_id: r.0,
                    repo_owner: r.1,
                    repo_name: r.2,
                    total_secrets: r.3 as usize,
                    scanned_files: r.4 as usize,
                    scanned_at: r.5,
                })
                .collect();
            (StatusCode::OK, Json(ScanHistoryResponse { scans })).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Internal(format!("database error: {e}")).error_response()),
        )
            .into_response(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use regex::Regex;

    #[test]
    fn test_aws_key_pattern() {
        let re = Regex::new(r"AKIA[0-9A-Z]{16}").unwrap();
        assert!(re.is_match("AKIAIOSFODNN7EXAMPLE"));
        assert!(!re.is_match("AKIAIOSFODNN7EXAMPL"));
        assert!(!re.is_match("akia123456789012345"));
    }

    #[test]
    fn test_github_token_pattern() {
        let re = Regex::new(r"ghp_[A-Za-z0-9]{36}").unwrap();
        assert!(re.is_match("ghp_ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghij"));
        assert!(!re.is_match("ghp_short"));
    }

    #[test]
    fn test_private_key_pattern() {
        let re = Regex::new(r"-----BEGIN (RSA |EC |DSA |OPENSSH )?PRIVATE KEY-----").unwrap();
        assert!(re.is_match("-----BEGIN RSA PRIVATE KEY-----"));
        assert!(re.is_match("-----BEGIN PRIVATE KEY-----"));
        assert!(re.is_match("-----BEGIN EC PRIVATE KEY-----"));
        assert!(!re.is_match("-----BEGIN PUBLIC KEY-----"));
    }

    #[test]
    fn test_jwt_pattern() {
        let re =
            Regex::new(r"eyJ[A-Za-z0-9_-]{10,}\.eyJ[A-Za-z0-9_-]{10,}\.[A-Za-z0-9_-]+").unwrap();
        assert!(re.is_match("eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0.abc123signature"));
    }

    #[test]
    fn test_connection_string_pattern() {
        let re =
            Regex::new(r"(?i)(mysql|postgres|postgresql|mongodb|redis|amqp)://[^\s]{10,}").unwrap();
        assert!(re.is_match("postgres://user:pass@localhost:5432/mydb"));
        assert!(re.is_match("mysql://admin:secret@db.example.com/mydb"));
    }

    #[test]
    fn test_mask_value() {
        assert_eq!(mask_value("AKIAIOSFODNN7EXAMPLE"), "AKIA...MPLE");
        assert_eq!(mask_value("short"), "*****");
        assert_eq!(mask_value("12345678"), "********");
    }

    #[test]
    fn test_scan_content_finds_aws_key() {
        let patterns = secret_patterns();
        let content = "AWS_KEY = \"AKIAIOSFODNN7EXAMPLE\"";
        let secrets = scan_content(content, "config.txt", &patterns);
        assert!(!secrets.is_empty());
        assert_eq!(secrets[0].secret_type, "AWS Access Key");
    }

    #[test]
    fn test_scan_content_finds_private_key() {
        let patterns = secret_patterns();
        let content = "-----BEGIN RSA PRIVATE KEY-----\nMIIEowIBAAK...";
        let secrets = scan_content(content, "key.pem", &patterns);
        assert!(!secrets.is_empty());
        assert_eq!(secrets[0].secret_type, "Private Key (PEM)");
    }

    #[test]
    fn test_scan_content_no_false_positive() {
        let patterns = secret_patterns();
        let content = "This is a normal file with no secrets.";
        let secrets = scan_content(content, "readme.md", &patterns);
        assert!(secrets.is_empty());
    }

    #[test]
    fn test_secret_patterns_count() {
        let patterns = secret_patterns();
        assert_eq!(patterns.len(), 9);
    }

    #[test]
    fn test_rotation_report_entry_serialization() {
        let entry = RotationReportEntry {
            secret_type: "AWS Access Key".into(),
            locations: vec!["config.txt:5".into()],
            recommendation: "Rotate immediately".into(),
        };
        let json = serde_json::to_string(&entry).unwrap();
        assert!(json.contains("AWS Access Key"));
        assert!(json.contains("config.txt:5"));
    }

    #[test]
    fn test_scan_response_serialization() {
        let resp = SecretScanResponse {
            total_secrets: 2,
            scanned_files: 10,
            secrets: vec![DetectedSecret {
                file: "a.txt".into(),
                line: 1,
                secret_type: "test".into(),
                masked_value: "****".into(),
            }],
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("\"total_secrets\":2"));
        assert!(json.contains("\"scanned_files\":10"));
    }

    #[test]
    fn test_scan_history_entry_serialization() {
        let entry = ScanHistoryEntry {
            scan_id: "abc123".into(),
            repo_owner: "user".into(),
            repo_name: "repo".into(),
            total_secrets: 0,
            scanned_files: 5,
            scanned_at: "2025-01-01T00:00:00Z".into(),
        };
        let json = serde_json::to_string(&entry).unwrap();
        assert!(json.contains("\"scan_id\":\"abc123\""));
    }

    #[test]
    fn test_mask_value_short() {
        assert_eq!(mask_value("ab"), "**");
    }

    #[test]
    fn test_mask_value_exact_8() {
        assert_eq!(mask_value("12345678"), "********");
    }

    #[test]
    fn test_jwt_pattern_no_match() {
        let re =
            Regex::new(r"eyJ[A-Za-z0-9_-]{10,}\.eyJ[A-Za-z0-9_-]{10,}\.[A-Za-z0-9_-]+").unwrap();
        assert!(!re.is_match("not.a.jwt"));
    }

    #[test]
    fn test_generic_password_pattern() {
        let re =
            Regex::new(r#"(?i)(password|passwd|pwd)\s*[:=]\s*['\"]([^'\"]{8,})['\"]"#).unwrap();
        assert!(re.is_match("password = \"SuperSecret123\""));
        assert!(re.is_match("PASSWD: 'mysecretpassword'"));
    }

    #[test]
    fn test_generic_api_key_pattern() {
        let re = Regex::new(r#"(?i)(api[_-]?key|apikey)\s*[:=]\s*['\"]([^'\"]{8,})['\"]"#).unwrap();
        assert!(re.is_match("api_key = \"abcdef1234567890\""));
        assert!(re.is_match("APIKEY: 'sk-1234567890abcdef'"));
    }

    #[test]
    fn test_rotation_report_serialization() {
        let report = RotationReport {
            total_types: 1,
            entries: vec![RotationReportEntry {
                secret_type: "test".into(),
                locations: vec!["file.txt:1".into()],
                recommendation: "rotate".into(),
            }],
        };
        let json = serde_json::to_string(&report).unwrap();
        assert!(json.contains("\"total_types\":1"));
    }
}
