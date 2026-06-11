//! Git LFS types and logic.

#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use sha2::Digest as _;

pub fn compute_oid(data: &[u8]) -> String {
    let hash = sha2::Sha256::digest(data);
    format!("{hash:x}")
}

pub fn parse_lfs_pointer(content: &str) -> Option<(String, u64)> {
    let mut version = None;
    let mut oid = None;
    let mut size = None;

    for line in content.lines() {
        let line = line.trim();
        if let Some(v) = line.strip_prefix("version ") {
            version = Some(v.trim().to_string());
        } else if let Some(o) = line.strip_prefix("oid sha256:") {
            oid = Some(o.trim().to_string());
        } else if let Some(s) = line.strip_prefix("size ") {
            size = s.trim().parse::<u64>().ok();
        }
    }

    if version.as_deref() == Some("https://git-lfs.github.com/spec/v1") {
        if let (Some(o), Some(s)) = (oid, size) {
            return Some((o, s));
        }
    }
    None
}

pub const LARGE_FILE_THRESHOLD: u64 = 100 * 1024 * 1024;

pub fn is_large_file(size: u64) -> bool {
    size >= LARGE_FILE_THRESHOLD
}

#[derive(Debug, Deserialize)]
pub struct LfsBatchRequest {
    pub operation: String,
    pub objects: Vec<LfsObjectRef>,
    #[serde(rename = "transfers")]
    pub transfers: Option<Vec<String>>,
    #[serde(rename = "ref")]
    pub ref_: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct LfsObjectRef {
    pub oid: String,
    pub size: u64,
}

#[derive(Debug, Serialize)]
pub struct LfsBatchResponse {
    pub transfer: String,
    pub objects: Vec<LfsObjectResponse>,
    pub hash_algo: String,
}

#[derive(Debug, Serialize)]
pub struct LfsObjectResponse {
    pub oid: String,
    pub size: u64,
    pub actions: Option<LfsActions>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<LfsError>,
}

#[derive(Debug, Serialize)]
pub struct LfsActions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub download: Option<LfsAction>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub upload: Option<LfsAction>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verify: Option<LfsAction>,
}

#[derive(Debug, Serialize)]
pub struct LfsAction {
    pub href: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub header: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_in: Option<u64>,
}

#[derive(Debug, Serialize)]
pub struct LfsError {
    pub code: u32,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct LfsVerifyResponse {
    pub oid: String,
    pub size: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

pub async fn ensure_lfs_table(pool: &sqlx::postgres::PgPool) {
    let _ = sqlx::query(
        "CREATE TABLE IF NOT EXISTS lfs_objects (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            repo_id UUID NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
            oid TEXT NOT NULL,
            size BIGINT NOT NULL,
            storage_path TEXT NOT NULL,
            verified BOOLEAN NOT NULL DEFAULT false,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            UNIQUE (repo_id, oid)
        )",
    )
    .execute(pool)
    .await;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lfs_batch_request_parse() {
        let json = r#"{
            "operation": "upload",
            "objects": [
                {"oid": "abc123def456", "size": 1024}
            ]
        }"#;
        let req: LfsBatchRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.operation, "upload");
        assert_eq!(req.objects.len(), 1);
        assert_eq!(req.objects[0].oid, "abc123def456");
        assert_eq!(req.objects[0].size, 1024);
    }

    #[test]
    fn test_lfs_batch_request_with_optional_fields() {
        let json = r#"{
            "operation": "download",
            "objects": [
                {"oid": "aaa", "size": 100},
                {"oid": "bbb", "size": 200}
            ],
            "transfers": ["basic"],
            "ref": "refs/heads/main"
        }"#;
        let req: LfsBatchRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.operation, "download");
        assert_eq!(req.objects.len(), 2);
        assert_eq!(req.transfers, Some(vec!["basic".to_string()]));
        assert_eq!(req.ref_, Some("refs/heads/main".to_string()));
    }

    #[test]
    fn test_lfs_batch_request_minimal() {
        let json = r#"{"operation":"upload","objects":[]}"#;
        let req: LfsBatchRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.operation, "upload");
        assert!(req.objects.is_empty());
        assert!(req.transfers.is_none());
        assert!(req.ref_.is_none());
    }

    #[test]
    fn test_lfs_batch_response_serialization() {
        let resp = LfsBatchResponse {
            transfer: "basic".into(),
            objects: vec![LfsObjectResponse {
                oid: "abc123".into(),
                size: 1024,
                actions: Some(LfsActions {
                    download: Some(LfsAction {
                        href: "https://example.com/lfs/abc123".into(),
                        header: None,
                        expires_in: Some(86400),
                    }),
                    upload: None,
                    verify: None,
                }),
                error: None,
            }],
            hash_algo: "sha256".into(),
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("\"transfer\":\"basic\""));
        assert!(json.contains("\"oid\":\"abc123\""));
        assert!(json.contains("\"hash_algo\":\"sha256\""));
    }

    #[test]
    fn test_lfs_batch_response_with_error() {
        let resp = LfsBatchResponse {
            transfer: "basic".into(),
            objects: vec![LfsObjectResponse {
                oid: "bad".into(),
                size: 0,
                actions: None,
                error: Some(LfsError {
                    code: 404,
                    message: "Object not found".into(),
                }),
            }],
            hash_algo: "sha256".into(),
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("\"code\":404"));
        assert!(json.contains("Object not found"));
    }

    #[test]
    fn test_lfs_object_ref_serialization_roundtrip() {
        let obj = LfsObjectRef {
            oid: "deadbeef".into(),
            size: 4096,
        };
        let json = serde_json::to_string(&obj).unwrap();
        let parsed: LfsObjectRef = serde_json::from_str(&json).unwrap();
        assert_eq!(obj, parsed);
    }

    #[test]
    fn test_lfs_verify_response() {
        let resp = LfsVerifyResponse {
            oid: "abc123".into(),
            size: 512,
            error: None,
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("\"oid\":\"abc123\""));
        assert!(!json.contains("error"));
    }

    #[test]
    fn test_lfs_verify_response_with_error() {
        let resp = LfsVerifyResponse {
            oid: "bad".into(),
            size: 0,
            error: Some("hash mismatch".into()),
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("hash mismatch"));
    }

    #[test]
    fn test_compute_oid_deterministic() {
        let data = b"test file content";
        let oid1 = compute_oid(data);
        let oid2 = compute_oid(data);
        assert_eq!(oid1, oid2);
    }

    #[test]
    fn test_compute_oid_format() {
        let oid = compute_oid(b"hello");
        assert_eq!(oid.len(), 64);
        assert!(oid.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_compute_oid_empty() {
        let oid = compute_oid(b"");
        assert_eq!(oid.len(), 64);
    }

    #[test]
    fn test_parse_lfs_pointer_valid() {
        let content = "version https://git-lfs.github.com/spec/v1\noid sha256:abc123def456\nsize 1024\n";
        let result = parse_lfs_pointer(content);
        assert_eq!(result, Some(("abc123def456".into(), 1024)));
    }

    #[test]
    fn test_parse_lfs_pointer_with_whitespace() {
        let content = "version  https://git-lfs.github.com/spec/v1  \noid sha256:abc  \nsize 2048  \n";
        let result = parse_lfs_pointer(content);
        assert_eq!(result, Some(("abc".into(), 2048)));
    }

    #[test]
    fn test_parse_lfs_pointer_wrong_version() {
        let content = "version https://wrong.spec/v1\noid sha256:abc\nsize 100\n";
        let result = parse_lfs_pointer(content);
        assert!(result.is_none());
    }

    #[test]
    fn test_parse_lfs_pointer_missing_oid() {
        let content = "version https://git-lfs.github.com/spec/v1\nsize 100\n";
        let result = parse_lfs_pointer(content);
        assert!(result.is_none());
    }

    #[test]
    fn test_parse_lfs_pointer_missing_size() {
        let content = "version https://git-lfs.github.com/spec/v1\noid sha256:abc\n";
        let result = parse_lfs_pointer(content);
        assert!(result.is_none());
    }

    #[test]
    fn test_parse_lfs_pointer_invalid_size() {
        let content = "version https://git-lfs.github.com/spec/v1\noid sha256:abc\nsize notanumber\n";
        let result = parse_lfs_pointer(content);
        assert!(result.is_none());
    }

    #[test]
    fn test_parse_lfs_pointer_empty() {
        assert!(parse_lfs_pointer("").is_none());
    }

    #[test]
    fn test_is_large_file() {
        assert!(!is_large_file(0));
        assert!(!is_large_file(1024));
        assert!(!is_large_file(LARGE_FILE_THRESHOLD - 1));
        assert!(is_large_file(LARGE_FILE_THRESHOLD));
        assert!(is_large_file(LARGE_FILE_THRESHOLD + 1));
    }

    #[test]
    fn test_large_file_threshold_value() {
        assert_eq!(LARGE_FILE_THRESHOLD, 100 * 1024 * 1024);
    }
}
