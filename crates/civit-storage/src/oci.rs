//! OCI Container Registry types and logic.

#![forbid(unsafe_code)]

use serde::Deserialize;
use sha2::Digest as _;

pub const OCI_API_VERSION: &str = "registry/2.0";

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

    #[test]
    fn test_catalog_params_default() {
        let p = CatalogParams::default();
        assert!(p.n.is_none());
        assert!(p.last.is_none());
    }

    #[test]
    fn test_digest_computation() {
        let data = b"hello world";
        let digest = compute_digest(data);
        assert!(digest.starts_with("sha256:"));
        assert!(digest.len() > 7);
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
}
