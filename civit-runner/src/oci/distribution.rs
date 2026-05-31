#![forbid(unsafe_code)]

use crate::oci::manifest::{OciDescriptor, OciIndex, OciManifest};
use anyhow::Result;
use base64::Engine as _;
use reqwest::Client;
use serde::Deserialize;
use sha2::{Digest, Sha256};

#[derive(Debug, Clone)]
pub struct RegistryConfig {
    pub host: String,
    pub scheme: String,
    pub username: Option<String>,
    pub password: Option<String>,
}

impl Default for RegistryConfig {
    fn default() -> Self {
        Self {
            host: "localhost:5000".to_string(),
            scheme: "http".to_string(),
            username: None,
            password: None,
        }
    }
}

pub struct OciRegistry {
    config: RegistryConfig,
    client: Client,
}

impl Default for OciRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl OciRegistry {
    pub fn new() -> Self {
        Self {
            config: RegistryConfig::default(),
            client: Client::new(),
        }
    }

    pub fn with_config(config: RegistryConfig) -> Self {
        Self {
            config,
            client: Client::new(),
        }
    }

    fn base_url(&self) -> String {
        format!("{}://{}", self.config.scheme, self.config.host)
    }

    fn auth_headers(&self) -> reqwest::header::HeaderMap {
        let mut headers = reqwest::header::HeaderMap::new();
        if let (Some(user), Some(pass)) = (&self.config.username, &self.config.password) {
            let creds = format!("{user}:{pass}");
            let encoded = base64::engine::general_purpose::STANDARD.encode(creds);
            headers.insert(
                reqwest::header::AUTHORIZATION,
                reqwest::header::HeaderValue::from_str(&format!("Basic {encoded}")).unwrap(),
            );
        }
        headers
    }

    pub async fn push_manifest(
        &self,
        name: &str,
        reference: &str,
        manifest: &OciManifest,
    ) -> Result<()> {
        let url = format!("{}/v2/{}/manifests/{}", self.base_url(), name, reference);
        let body = serde_json::to_vec(manifest)?;
        self.client
            .put(&url)
            .headers(self.auth_headers())
            .header("Content-Type", "application/vnd.oci.image.manifest.v1+json")
            .body(body)
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }

    pub async fn pull_manifest(&self, name: &str, reference: &str) -> Result<OciManifest> {
        let url = format!("{}/v2/{}/manifests/{}", self.base_url(), name, reference);
        let resp = self
            .client
            .get(&url)
            .headers(self.auth_headers())
            .header(
                "Accept",
                "application/vnd.oci.image.manifest.v1+json, application/vnd.oci.image.index.v1+json",
            )
            .send()
            .await?
            .error_for_status()?
            .text()
            .await?;
        Ok(serde_json::from_str(&resp)?)
    }

    pub async fn push_blob(&self, name: &str, data: &[u8]) -> Result<OciDescriptor> {
        let digest = blob_digest(data);
        let url = format!(
            "{}/v2/{}/blobs/uploads/?digest={}",
            self.base_url(),
            name,
            digest
        );
        self.client
            .post(&url)
            .headers(self.auth_headers())
            .header("Content-Type", "application/octet-stream")
            .body(data.to_vec())
            .send()
            .await?
            .error_for_status()?;

        Ok(OciDescriptor {
            media_type: "application/octet-stream".to_string(),
            digest,
            size: data.len() as u64,
            annotations: None,
        })
    }

    pub async fn pull_blob(&self, name: &str, digest: &str) -> Result<Vec<u8>> {
        let url = format!("{}/v2/{}/blobs/{}", self.base_url(), name, digest);
        let resp = self
            .client
            .get(&url)
            .headers(self.auth_headers())
            .send()
            .await?
            .error_for_status()?
            .bytes()
            .await?;
        Ok(resp.to_vec())
    }

    pub async fn push_index(&self, name: &str, index: &OciIndex) -> Result<()> {
        let url = format!("{}/v2/{}/manifests/index", self.base_url(), name);
        let body = serde_json::to_vec(index)?;
        self.client
            .put(&url)
            .headers(self.auth_headers())
            .header("Content-Type", "application/vnd.oci.image.index.v1+json")
            .body(body)
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }

    pub async fn pull_index(&self, name: &str) -> Result<OciIndex> {
        let url = format!("{}/v2/{}/manifests/index", self.base_url(), name);
        let resp = self
            .client
            .get(&url)
            .headers(self.auth_headers())
            .header("Accept", "application/vnd.oci.image.index.v1+json")
            .send()
            .await?
            .error_for_status()?
            .text()
            .await?;
        Ok(serde_json::from_str(&resp)?)
    }

    pub async fn tag(&self, name: &str, source_ref: &str, target_ref: &str) -> Result<()> {
        let manifest = self.pull_manifest(name, source_ref).await?;
        self.push_manifest(name, target_ref, &manifest).await
    }

    pub async fn list_tags(&self, name: &str) -> Result<Vec<String>> {
        let url = format!("{}/v2/{}/tags/list", self.base_url(), name);
        let resp = self
            .client
            .get(&url)
            .headers(self.auth_headers())
            .send()
            .await?
            .error_for_status()?
            .text()
            .await?;
        #[derive(Deserialize)]
        struct TagsResponse {
            tags: Option<Vec<String>>,
        }
        let parsed: TagsResponse = serde_json::from_str(&resp)?;
        Ok(parsed.tags.unwrap_or_default())
    }

    pub async fn catalog(&self) -> Result<Vec<String>> {
        let url = format!("{}/v2/_catalog", self.base_url());
        let resp = self
            .client
            .get(&url)
            .headers(self.auth_headers())
            .send()
            .await?
            .error_for_status()?
            .text()
            .await?;
        #[derive(Deserialize)]
        struct CatalogResponse {
            repositories: Option<Vec<String>>,
        }
        let parsed: CatalogResponse = serde_json::from_str(&resp)?;
        Ok(parsed.repositories.unwrap_or_default())
    }

    pub async fn delete(&self, name: &str, reference: &str) -> Result<()> {
        let url = format!("{}/v2/{}/manifests/{}", self.base_url(), name, reference);
        self.client
            .delete(&url)
            .headers(self.auth_headers())
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }
}

fn blob_digest(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    format!("sha256:{}", hex::encode(hasher.finalize()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn make_registry() -> OciRegistry {
        OciRegistry::with_config(RegistryConfig {
            host: "localhost:5555".to_string(),
            scheme: "http".to_string(),
            username: Some("test".to_string()),
            password: Some("pass".to_string()),
        })
    }

    #[test]
    fn test_registry_default() {
        let reg = OciRegistry::new();
        assert_eq!(reg.config.host, "localhost:5000");
    }

    #[test]
    fn test_registry_base_url() {
        let reg = OciRegistry::with_config(RegistryConfig {
            host: "registry.example.com".to_string(),
            scheme: "https".to_string(),
            username: None,
            password: None,
        });
        assert_eq!(reg.base_url(), "https://registry.example.com");
    }

    #[test]
    fn test_blob_digest() {
        let digest = blob_digest(b"hello");
        assert!(digest.starts_with("sha256:"));
        assert_eq!(digest.len(), 7 + 64);
    }

    #[test]
    fn test_manifest_for_push() {
        let manifest = OciManifest {
            schema_version: 2,
            media_type: "application/vnd.oci.image.manifest.v1+json".to_string(),
            config: OciDescriptor {
                media_type: "application/vnd.oci.image.config.v1+json".to_string(),
                digest: "sha256:config".to_string(),
                size: 100,
                annotations: None,
            },
            layers: vec![OciDescriptor {
                media_type: "application/vnd.oci.image.layer.v1.tar+gzip".to_string(),
                digest: "sha256:layer".to_string(),
                size: 200,
                annotations: None,
            }],
            annotations: HashMap::new(),
        };
        let json = serde_json::to_string(&manifest).unwrap();
        assert!(json.contains("schemaVersion"));
    }

    #[test]
    fn test_auth_headers_present() {
        let reg = make_registry();
        let headers = reg.auth_headers();
        assert!(headers.contains_key(reqwest::header::AUTHORIZATION));
    }

    #[test]
    fn test_no_auth_headers() {
        let reg = OciRegistry::new();
        let headers = reg.auth_headers();
        assert!(!headers.contains_key(reqwest::header::AUTHORIZATION));
    }

    #[tokio::test]
    async fn test_push_blob_connection_refused() {
        let reg = make_registry();
        let result = reg.push_blob("test", b"data").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_pull_manifest_connection_refused() {
        let reg = make_registry();
        let result = reg.pull_manifest("test", "latest").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_list_tags_connection_refused() {
        let reg = make_registry();
        let result = reg.list_tags("test").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_catalog_connection_refused() {
        let reg = make_registry();
        let result = reg.catalog().await;
        assert!(result.is_err());
    }
}
