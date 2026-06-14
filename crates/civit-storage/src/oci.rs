//! OCI Container Registry types and logic.

#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use sha2::Digest as _;

pub const OCI_API_VERSION: &str = "registry/2.0";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OciManifest {
    #[serde(rename = "schemaVersion")]
    pub schema_version: u32,
    pub media_type: String,
    pub config: OciDescriptor,
    pub layers: Vec<OciDescriptor>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub annotations: Option<std::collections::HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OciDescriptor {
    pub media_type: String,
    pub size: u64,
    pub digest: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub annotations: Option<std::collections::HashMap<String, String>>,
}

pub fn validate_tag(tag: &str) -> Result<(), String> {
    if tag.is_empty() {
        return Err("tag must not be empty".into());
    }
    if tag.len() > 128 {
        return Err("tag must not exceed 128 characters".into());
    }
    if tag.starts_with('.') || tag.starts_with('-') {
        return Err("tag must not start with '.' or '-'".into());
    }
    for ch in tag.chars() {
        if !ch.is_alphanumeric() && ch != '.' && ch != '-' && ch != '_' {
            return Err(format!("invalid character '{ch}' in tag"));
        }
    }
    Ok(())
}

pub fn create_oci_manifest(
    config_digest: &str,
    config_size: u64,
    layer_digest: &str,
    layer_size: u64,
) -> OciManifest {
    OciManifest {
        schema_version: 2,
        media_type: "application/vnd.oci.image.manifest.v1+json".into(),
        config: OciDescriptor {
            media_type: "application/vnd.oci.image.config.v1+json".into(),
            size: config_size,
            digest: config_digest.into(),
            annotations: None,
        },
        layers: vec![OciDescriptor {
            media_type: "application/vnd.oci.image.layer.v1.tar+gzip".into(),
            size: layer_size,
            digest: layer_digest.into(),
            annotations: None,
        }],
        annotations: None,
    }
}

#[derive(Debug, Deserialize, Default)]
pub struct CatalogParams {
    pub n: Option<usize>,
    pub last: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
pub struct TagsParams {
    pub n: Option<usize>,
    pub last: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
pub struct UploadChunkParams {
    pub range_start: Option<usize>,
    pub range_end: Option<usize>,
}

#[derive(Debug, Deserialize, Default)]
pub struct CompleteUploadParams {
    pub digest: Option<String>,
    pub content_type: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
pub struct ReferrersParams {
    pub artifact_type: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
pub struct RegistryListParams {
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

#[derive(Debug, Deserialize)]
pub struct CreatePolicy {
    pub role: String,
    pub entity_type: String,
    pub entity_id: Option<String>,
}

pub async fn resolve_repo(pool: &sqlx::PgPool, name: &str) -> Option<i64> {
    sqlx::query_scalar::<_, i64>("SELECT id FROM oci_repositories WHERE name = $1")
        .bind(name)
        .fetch_optional(pool)
        .await
        .unwrap_or(None)
}

pub async fn resolve_or_create_repo(pool: &sqlx::PgPool, name: &str) -> i64 {
    if let Some(id) = resolve_repo(pool, name).await {
        return id;
    }

    let (namespace_type, namespace_id) = if let Some((_ns, _rn)) = name.split_once('/') {
        let parts: Vec<&str> = name.splitn(2, '/').collect();
        (parts[0].to_string(), parts[0].to_string())
    } else {
        ("user".to_string(), "anonymous".to_string())
    };

    let result = sqlx::query_scalar::<_, i64>(
        "INSERT INTO oci_repositories (name, namespace_type, namespace_id) VALUES ($1, $2, $3) ON CONFLICT (name) DO NOTHING RETURNING id",
    )
    .bind(name)
    .bind(&namespace_type)
    .bind(&namespace_id)
    .fetch_optional(pool)
    .await
    .unwrap_or(None);

    match result {
        Some(id) => id,
        None => resolve_repo(pool, name).await.unwrap_or(0),
    }
}

pub fn compute_digest(data: &[u8]) -> String {
    let hash = sha2::Sha256::digest(data);
    format!("sha256:{hash:x}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_catalog_params_default() {
        let p = CatalogParams::default();
        assert!(p.n.is_none());
        assert!(p.last.is_none());
    }

    #[test]
    fn test_tags_params_default() {
        let p = TagsParams::default();
        assert!(p.n.is_none());
        assert!(p.last.is_none());
    }

    #[test]
    fn test_upload_chunk_params_default() {
        let p = UploadChunkParams::default();
        assert!(p.range_start.is_none());
        assert!(p.range_end.is_none());
    }

    #[test]
    fn test_complete_upload_params_default() {
        let p = CompleteUploadParams::default();
        assert!(p.digest.is_none());
        assert!(p.content_type.is_none());
    }

    #[test]
    fn test_referrers_params_default() {
        let p = ReferrersParams::default();
        assert!(p.artifact_type.is_none());
    }

    #[test]
    fn test_registry_list_params_default() {
        let p = RegistryListParams::default();
        assert!(p.limit.is_none());
        assert!(p.offset.is_none());
    }

    #[test]
    fn test_digest_computation() {
        let data = b"hello world";
        let digest = compute_digest(data);
        assert!(digest.starts_with("sha256:"));
        assert_eq!(digest.len(), 7 + 64);
    }

    #[test]
    fn test_digest_deterministic() {
        let data = b"test content";
        let d1 = compute_digest(data);
        let d2 = compute_digest(data);
        assert_eq!(d1, d2);
    }

    #[test]
    fn test_digest_different_inputs() {
        let d1 = compute_digest(b"alpha");
        let d2 = compute_digest(b"beta");
        assert_ne!(d1, d2);
    }

    #[test]
    fn test_digest_empty_input() {
        let digest = compute_digest(b"");
        assert!(digest.starts_with("sha256:"));
        assert_eq!(digest.len(), 7 + 64);
    }

    #[test]
    fn test_digest_known_vector() {
        let digest = compute_digest(b"");
        assert_eq!(
            digest,
            "sha256:e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    #[test]
    fn test_namespace_parsing() {
        let cases = vec![
            ("myorg/alpine", "myorg", "myorg"),
            ("user/nginx", "user", "user"),
            ("single", "user", "anonymous"),
        ];
        for (input, expected_type, expected_id) in cases {
            let (ns_type, ns_id) = if let Some((ns, _rn)) = input.split_once('/') {
                (ns.to_string(), ns.to_string())
            } else {
                ("user".to_string(), "anonymous".to_string())
            };
            assert_eq!(ns_type, expected_type, "input: {input}");
            assert_eq!(ns_id, expected_id, "input: {input}");
        }
    }

    #[test]
    fn test_oci_manifest_roundtrip() {
        let manifest = create_oci_manifest("sha256:abc123", 512, "sha256:def456", 1024);
        let json = serde_json::to_string(&manifest).unwrap();
        let parsed: OciManifest = serde_json::from_str(&json).unwrap();
        assert_eq!(manifest, parsed);
    }

    #[test]
    fn test_oci_manifest_serialization() {
        let manifest = create_oci_manifest("sha256:aaa", 100, "sha256:bbb", 200);
        let json = serde_json::to_string_pretty(&manifest).unwrap();
        assert!(json.contains("schemaVersion"));
        assert!(json.contains("application/vnd.oci.image.manifest.v1+json"));
        assert!(json.contains("sha256:aaa"));
        assert!(json.contains("sha256:bbb"));
    }

    #[test]
    fn test_oci_manifest_with_annotations() {
        let mut annotations = HashMap::new();
        annotations.insert("org.opencontainers.image.title".into(), "test".into());
        let manifest = OciManifest {
            schema_version: 2,
            media_type: "application/vnd.oci.image.manifest.v1+json".into(),
            config: OciDescriptor {
                media_type: "application/vnd.oci.image.config.v1+json".into(),
                size: 128,
                digest: "sha256:cfg".into(),
                annotations: None,
            },
            layers: vec![],
            annotations: Some(annotations),
        };
        let json = serde_json::to_string(&manifest).unwrap();
        assert!(json.contains("org.opencontainers.image.title"));
        let parsed: OciManifest = serde_json::from_str(&json).unwrap();
        assert_eq!(manifest, parsed);
    }

    #[test]
    fn test_validate_tag_valid() {
        assert!(validate_tag("latest").is_ok());
        assert!(validate_tag("v1.0").is_ok());
        assert!(validate_tag("my_tag").is_ok());
        assert!(validate_tag("abc123").is_ok());
        assert!(validate_tag("a").is_ok());
    }

    #[test]
    fn test_validate_tag_empty() {
        assert!(validate_tag("").is_err());
    }

    #[test]
    fn test_validate_tag_too_long() {
        let long_tag = "a".repeat(129);
        assert!(validate_tag(&long_tag).is_err());
    }

    #[test]
    fn test_validate_tag_max_length() {
        let tag = "a".repeat(128);
        assert!(validate_tag(&tag).is_ok());
    }

    #[test]
    fn test_validate_tag_starts_with_dot() {
        assert!(validate_tag(".hidden").is_err());
    }

    #[test]
    fn test_validate_tag_starts_with_dash() {
        assert!(validate_tag("-beta").is_err());
    }

    #[test]
    fn test_validate_tag_invalid_chars() {
        assert!(validate_tag("tag with space").is_err());
        assert!(validate_tag("tag/slash").is_err());
        assert!(validate_tag("tag@at").is_err());
        assert!(validate_tag("tag:colon").is_err());
    }

    #[test]
    fn test_create_oci_manifest_structure() {
        let manifest = create_oci_manifest("sha256:config", 256, "sha256:layer", 512);
        assert_eq!(manifest.schema_version, 2);
        assert_eq!(manifest.layers.len(), 1);
        assert_eq!(manifest.config.size, 256);
        assert_eq!(manifest.layers[0].size, 512);
        assert!(manifest.annotations.is_none());
    }
}
