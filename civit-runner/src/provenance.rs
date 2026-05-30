#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tracing::info;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SbomDocument {
    pub name: String,
    pub version: String,
    pub sbom_version: String,
    pub packages: Vec<SbomPackage>,
    pub relationships: Vec<SbomRelationship>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SbomPackage {
    pub spdx_id: String,
    pub name: String,
    pub version: String,
    pub source: String,
    pub checksum_sha256: String,
    pub license: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SbomRelationship {
    pub from: String,
    pub r#type: String,
    pub to: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CosignSignature {
    pub digest: String,
    pub signature: String,
    pub signer: String,
    pub timestamp: String,
}

pub struct ProvenanceEngine;

impl Default for ProvenanceEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl ProvenanceEngine {
    pub fn new() -> Self {
        Self
    }

    pub fn generate_sbom(&self, name: &str, version: &str, lockfile_content: &str) -> SbomDocument {
        let packages: Vec<SbomPackage> = lockfile_content
            .lines()
            .filter(|l| !l.is_empty() && !l.starts_with('#'))
            .enumerate()
            .map(|(i, line)| {
                let parts: Vec<&str> = line.split_whitespace().collect();
                let pkg_name = parts
                    .first()
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| format!("pkg-{i}"));
                let pkg_version = parts
                    .get(1)
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| "0.0.0".into());
                let mut hasher = Sha256::new();
                hasher.update(line.as_bytes());
                let hash = hex::encode(hasher.finalize());
                SbomPackage {
                    spdx_id: format!("SPDXRef-Package-{i}"),
                    name: pkg_name,
                    version: pkg_version,
                    source: "lockfile".into(),
                    checksum_sha256: hash,
                    license: "NOASSERTION".into(),
                }
            })
            .collect();

        let relationships: Vec<SbomRelationship> = packages
            .iter()
            .map(|pkg| SbomRelationship {
                from: format!("SPDXRef-{}", name),
                r#type: "DEPENDS_ON".into(),
                to: pkg.spdx_id.clone(),
            })
            .collect();

        info!(
            name = %name,
            packages = packages.len(),
            "generated SBOM"
        );

        SbomDocument {
            name: name.into(),
            version: version.into(),
            sbom_version: "SPDX-2.3".into(),
            packages,
            relationships,
        }
    }

    pub fn sign_artifact(&self, digest: &str, signer: &str) -> CosignSignature {
        let mut hasher = Sha256::new();
        hasher.update(digest.as_bytes());
        hasher.update(signer.as_bytes());
        let signature = hex::encode(hasher.finalize());

        info!(digest = %digest, signer = %signer, "signed artifact");

        CosignSignature {
            digest: digest.into(),
            signature,
            signer: signer.into(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        }
    }

    pub fn verify_signature(&self, sig: &CosignSignature) -> bool {
        let mut hasher = Sha256::new();
        hasher.update(sig.digest.as_bytes());
        hasher.update(sig.signer.as_bytes());
        let expected = hex::encode(hasher.finalize());
        expected == sig.signature
    }

    pub fn generate_build_attestation(
        &self,
        pipeline: &str,
        commit: &str,
        image_digest: &str,
    ) -> serde_json::Value {
        serde_json::json!({
            "predicateType": "https://slsa.dev/provenance/v1",
            "subject": [
                {
                    "name": pipeline,
                    "digest": { "sha256": image_digest },
                }
            ],
            "predicate": {
                "builder": { "id": "civit-runner" },
                "buildType": "https://civitforge.dev/pipeline/v1",
                "invocation": {
                    "configSource": {
                        "uri": format!(".civit/pipeline.yaml@{commit}"),
                    }
                },
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn engine() -> ProvenanceEngine {
        ProvenanceEngine::new()
    }

    #[test]
    fn test_generate_sbom() {
        let engine = engine();
        let lockfile = "alpine 3.18.0\nbash 5.2.15\ngit 2.40.0";
        let sbom = engine.generate_sbom("test-app", "1.0.0", lockfile);
        assert_eq!(sbom.name, "test-app");
        assert_eq!(sbom.version, "1.0.0");
        assert_eq!(sbom.sbom_version, "SPDX-2.3");
        assert_eq!(sbom.packages.len(), 3);
        assert_eq!(sbom.relationships.len(), 3);
        assert_eq!(sbom.packages[0].name, "alpine");
        assert_eq!(sbom.packages[0].version, "3.18.0");
    }

    #[test]
    fn test_sign_and_verify() {
        let engine = engine();
        let sig = engine.sign_artifact("sha256:abc123", "civit-runner");
        assert!(engine.verify_signature(&sig));
    }

    #[test]
    fn test_verify_tampered_signature() {
        let engine = engine();
        let mut sig = engine.sign_artifact("sha256:abc123", "civit-runner");
        sig.signature = "deadbeef".into();
        assert!(!engine.verify_signature(&sig));
    }

    #[test]
    fn test_sbom_empty_lockfile() {
        let engine = engine();
        let sbom = engine.generate_sbom("empty", "1.0.0", "");
        assert_eq!(sbom.packages.len(), 0);
        assert_eq!(sbom.relationships.len(), 0);
    }

    #[test]
    fn test_build_attestation() {
        let engine = engine();
        let att = engine.generate_build_attestation("ci-pipeline", "abc123", "sha256:def456");
        assert_eq!(att["subject"][0]["name"], "ci-pipeline");
        assert_eq!(att["predicate"]["builder"]["id"], "civit-runner");
    }

    #[test]
    fn test_sbom_serialization() {
        let engine = engine();
        let sbom = engine.generate_sbom("app", "1.0.0", "pkg1 2.0.0");
        let json = serde_json::to_string(&sbom).unwrap();
        assert!(json.contains("SPDX-2.3"));
        assert!(json.contains("pkg1"));
        let deserialized: SbomDocument = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.packages.len(), 1);
    }
}
