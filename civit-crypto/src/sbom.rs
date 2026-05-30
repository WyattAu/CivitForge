#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use tracing::info;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SbomPackage {
    pub spdx_id: String,
    pub name: String,
    pub version: String,
    pub source: String,
    pub checksum: String,
    pub license: String,
    pub url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SbomRelationship {
    pub spdx_id: String,
    pub relationship_type: String,
    pub related_spdx_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SbomDocument {
    pub spdx_version: String,
    pub name: String,
    pub document_namespace: String,
    pub creation_info: SbomCreationInfo,
    pub packages: Vec<SbomPackage>,
    pub relationships: Vec<SbomRelationship>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SbomCreationInfo {
    pub created: String,
    pub creators: Vec<String>,
    pub comment: String,
}

pub struct SbomGenerator;

impl Default for SbomGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl SbomGenerator {
    pub fn new() -> Self {
        Self
    }

    pub fn generate(name: &str, version: &str, packages: Vec<SbomPackageInput>) -> SbomDocument {
        let document_namespace = format!("https://civitforge.dev/sbom/{name}-{version}");
        let packages: Vec<SbomPackage> = packages
            .iter()
            .enumerate()
            .map(|(i, pkg)| {
                let checksum = crate::hash::HashService::hash_string(
                    crate::hash::HashAlgorithm::Sha256,
                    &format!("{}:{}:{}", pkg.name, pkg.version, pkg.source),
                )
                .hex;
                SbomPackage {
                    spdx_id: format!("SPDXRef-Package-{i}"),
                    name: pkg.name.clone(),
                    version: pkg.version.clone(),
                    source: pkg.source.clone(),
                    checksum,
                    license: pkg.license.clone().unwrap_or_else(|| "NOASSERTION".into()),
                    url: pkg.url.clone(),
                }
            })
            .collect();

        let relationships: Vec<SbomRelationship> = packages
            .iter()
            .map(|pkg| SbomRelationship {
                spdx_id: format!("SPDXRef-{}", name),
                relationship_type: "DEPENDS_ON".into(),
                related_spdx_id: pkg.spdx_id.clone(),
            })
            .collect();

        info!(
            name = %name,
            version = %version,
            packages = packages.len(),
            "generated SPDX SBOM"
        );

        SbomDocument {
            spdx_version: "SPDX-2.3".into(),
            name: name.into(),
            document_namespace,
            creation_info: SbomCreationInfo {
                created: chrono::Utc::now().to_rfc3339(),
                creators: vec!["Tool: CivitForge-SPDX-Generator".into()],
                comment: format!("Generated for {name} v{version}"),
            },
            packages,
            relationships,
        }
    }

    pub fn generate_cyclonedx(
        name: &str,
        version: &str,
        packages: Vec<SbomPackageInput>,
    ) -> serde_json::Value {
        let components: Vec<serde_json::Value> = packages
            .iter()
            .map(|pkg| {
                let checksum = crate::hash::HashService::hash_string(
                    crate::hash::HashAlgorithm::Sha256,
                    &format!("{}:{}", pkg.name, pkg.version),
                )
                .hex;
                serde_json::json!({
                    "type": "library",
                    "name": pkg.name,
                    "version": pkg.version,
                    "purl": format!("pkg:generic/{}@{}", pkg.name, pkg.version),
                    "hashes": [{ "alg": "SHA-256", "content": checksum }],
                })
            })
            .collect();

        serde_json::json!({
            "bomFormat": "CycloneDX",
            "specVersion": "1.6",
            "version": 1,
            "metadata": {
                "component": {
                    "name": name,
                    "version": version,
                },
                "tools": [{ "name": "CivitForge", "version": env!("CARGO_PKG_VERSION") }],
            },
            "components": components,
        })
    }

    pub fn to_json(document: &SbomDocument) -> anyhow::Result<String> {
        Ok(serde_json::to_string_pretty(document)?)
    }

    pub fn from_json(json: &str) -> anyhow::Result<SbomDocument> {
        Ok(serde_json::from_str(json)?)
    }
}

#[derive(Debug, Clone)]
pub struct SbomPackageInput {
    pub name: String,
    pub version: String,
    pub source: String,
    pub license: Option<String>,
    pub url: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_packages() -> Vec<SbomPackageInput> {
        vec![
            SbomPackageInput {
                name: "tokio".into(),
                version: "1.35.0".into(),
                source: "crates.io".into(),
                license: Some("MIT".into()),
                url: Some("https://crates.io/crates/tokio".into()),
            },
            SbomPackageInput {
                name: "serde".into(),
                version: "1.0.195".into(),
                source: "crates.io".into(),
                license: Some("MIT".into()),
                url: None,
            },
            SbomPackageInput {
                name: "axum".into(),
                version: "0.7.4".into(),
                source: "crates.io".into(),
                license: None,
                url: None,
            },
        ]
    }

    #[test]
    fn test_generate_spdx() {
        let doc = SbomGenerator::generate("my-app", "1.0.0", sample_packages());
        assert_eq!(doc.spdx_version, "SPDX-2.3");
        assert_eq!(doc.name, "my-app");
        assert_eq!(doc.packages.len(), 3);
        assert_eq!(doc.relationships.len(), 3);
        assert_eq!(doc.packages[0].name, "tokio");
        assert_eq!(doc.packages[0].license, "MIT");
        assert_eq!(doc.packages[2].license, "NOASSERTION");
    }

    #[test]
    fn test_generate_cyclonedx() {
        let doc = SbomGenerator::generate_cyclonedx("my-app", "1.0.0", sample_packages());
        assert_eq!(doc["bomFormat"], "CycloneDX");
        assert_eq!(doc["metadata"]["component"]["name"], "my-app");
        assert_eq!(doc["components"].as_array().unwrap().len(), 3);
    }

    #[test]
    fn test_sbom_serialization_roundtrip() {
        let doc = SbomGenerator::generate(
            "app",
            "1.0.0",
            vec![SbomPackageInput {
                name: "dep".into(),
                version: "2.0.0".into(),
                source: "crates.io".into(),
                license: Some("Apache-2.0".into()),
                url: None,
            }],
        );
        let json = SbomGenerator::to_json(&doc).unwrap();
        let parsed = SbomGenerator::from_json(&json).unwrap();
        assert_eq!(parsed.name, "app");
        assert_eq!(parsed.packages.len(), 1);
        assert_eq!(parsed.packages[0].name, "dep");
    }

    #[test]
    fn test_empty_packages() {
        let doc = SbomGenerator::generate("empty", "1.0.0", vec![]);
        assert_eq!(doc.packages.len(), 0);
        assert_eq!(doc.relationships.len(), 0);
    }

    #[test]
    fn test_document_namespace_format() {
        let doc = SbomGenerator::generate("test-app", "2.5.0", vec![]);
        assert!(doc.document_namespace.contains("test-app-2.5.0"));
    }
}
