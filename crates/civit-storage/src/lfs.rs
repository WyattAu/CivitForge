//! Git LFS types and logic.

#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct LfsBatchRequest {
    pub operation: String,
    pub objects: Vec<LfsObjectRef>,
    #[serde(rename = "transfers")]
    pub transfers: Option<Vec<String>>,
    #[serde(rename = "ref")]
    pub ref_: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
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
    }
}
