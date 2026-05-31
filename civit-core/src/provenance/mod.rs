#![forbid(unsafe_code)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlsaProvenance {
    #[serde(rename = "_type")]
    pub kind: String,
    pub version: u32,
    pub builder: Builder,
    pub metadata: BuildMetadata,
    pub materials: Vec<Material>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Builder {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub builder_dependencies: Option<Vec<BuilderDependency>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuilderDependency {
    pub uri: String,
    pub digest: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildMetadata {
    #[serde(rename = "buildInvocationId")]
    pub build_invocation_id: String,
    #[serde(rename = "buildStartedOn")]
    pub build_started_on: DateTime<Utc>,
    #[serde(rename = "buildFinishedOn")]
    pub build_finished_on: Option<DateTime<Utc>>,
    #[serde(default = "completeness_default")]
    pub completeness: Completeness,
    #[serde(default)]
    pub reproducible: bool,
}

fn completeness_default() -> Completeness {
    Completeness {
        parameters: true,
        environment: false,
        materials: true,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Completeness {
    #[serde(default)]
    pub parameters: bool,
    #[serde(default)]
    pub environment: bool,
    #[serde(default)]
    pub materials: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Material {
    pub uri: String,
    pub digest: HashMap<String, String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub annotations: Option<HashMap<String, String>>,
}

pub struct ProvenanceGenerator {
    builder_id: String,
    builder_version: Option<String>,
}

impl ProvenanceGenerator {
    pub fn new(builder_id: String) -> Self {
        Self {
            builder_id,
            builder_version: None,
        }
    }

    pub fn with_version(mut self, version: String) -> Self {
        self.builder_version = Some(version);
        self
    }

    pub fn generate(
        &self,
        invocation_id: &str,
        materials: Vec<Material>,
        start_time: DateTime<Utc>,
        end_time: Option<DateTime<Utc>>,
        reproducible: bool,
    ) -> SlsaProvenance {
        SlsaProvenance {
            kind: "https://in-toto.io/Statement/v0.1".to_string(),
            version: 1,
            builder: Builder {
                id: self.builder_id.clone(),
                version: self.builder_version.clone(),
                builder_dependencies: None,
            },
            metadata: BuildMetadata {
                build_invocation_id: invocation_id.to_string(),
                build_started_on: start_time,
                build_finished_on: end_time,
                completeness: completeness_default(),
                reproducible,
            },
            materials,
        }
    }

    pub fn verify(provenance: &SlsaProvenance) -> Result<VerificationResult, String> {
        let mut checks = Vec::new();

        if provenance.builder.id.is_empty() {
            checks.push(VerificationCheck {
                name: "builder.id".into(),
                passed: false,
                message: "builder ID is empty".into(),
            });
        } else {
            checks.push(VerificationCheck {
                name: "builder.id".into(),
                passed: true,
                message: String::new(),
            });
        }

        if provenance.metadata.build_invocation_id.is_empty() {
            checks.push(VerificationCheck {
                name: "invocation.id".into(),
                passed: false,
                message: "invocation ID is empty".into(),
            });
        } else {
            checks.push(VerificationCheck {
                name: "invocation.id".into(),
                passed: true,
                message: String::new(),
            });
        }

        if provenance.materials.is_empty() {
            checks.push(VerificationCheck {
                name: "materials".into(),
                passed: false,
                message: "no materials specified".into(),
            });
        } else {
            checks.push(VerificationCheck {
                name: "materials".into(),
                passed: true,
                message: String::new(),
            });
        }

        let all_passed = checks.iter().all(|c| c.passed);

        Ok(VerificationResult {
            passed: all_passed,
            checks,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationResult {
    pub passed: bool,
    pub checks: Vec<VerificationCheck>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationCheck {
    pub name: String,
    pub passed: bool,
    pub message: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_material(uri: &str, alg: &str, digest: &str) -> Material {
        Material {
            uri: uri.to_string(),
            digest: {
                let mut m = HashMap::new();
                m.insert(alg.to_string(), digest.to_string());
                m
            },
            annotations: None,
        }
    }

    #[test]
    fn test_generate_provenance_with_materials() {
        let generator = ProvenanceGenerator::new("civitforge-builder".into());
        let materials = vec![make_material(
            "git+https://example.com/repo",
            "sha256",
            "abc123",
        )];
        let start = Utc::now();
        let end = Utc::now();
        let prov = generator.generate("inv-1", materials, start, Some(end), true);
        assert_eq!(prov.builder.id, "civitforge-builder");
        assert_eq!(prov.metadata.build_invocation_id, "inv-1");
        assert_eq!(prov.materials.len(), 1);
        assert!(prov.metadata.reproducible);
        assert!(prov.metadata.build_finished_on.is_some());
    }

    #[test]
    fn test_provenance_roundtrip() {
        let generator = ProvenanceGenerator::new("builder".into());
        let materials = vec![make_material("git://repo", "sha256", "deadbeef")];
        let prov = generator.generate("inv-2", materials, Utc::now(), None, false);

        let json = serde_json::to_string(&prov).expect("serialize");
        let de: SlsaProvenance = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(de.kind, prov.kind);
        assert_eq!(de.version, prov.version);
        assert_eq!(de.builder.id, prov.builder.id);
        assert_eq!(
            de.metadata.build_invocation_id,
            prov.metadata.build_invocation_id
        );
        assert_eq!(de.materials.len(), prov.materials.len());
        assert_eq!(de.materials[0].uri, "git://repo");
        assert_eq!(de.materials[0].digest.get("sha256").unwrap(), "deadbeef");
    }

    #[test]
    fn test_verify_valid_provenance() {
        let generator = ProvenanceGenerator::new("builder".into());
        let materials = vec![make_material("git://repo", "sha256", "abc")];
        let prov = generator.generate("inv-3", materials, Utc::now(), None, false);
        let result = ProvenanceGenerator::verify(&prov).unwrap();
        assert!(result.passed);
        assert!(result.checks.iter().all(|c| c.passed));
    }

    #[test]
    fn test_verify_invalid_empty_builder() {
        let prov = SlsaProvenance {
            kind: "https://in-toto.io/Statement/v0.1".into(),
            version: 1,
            builder: Builder {
                id: String::new(),
                version: None,
                builder_dependencies: None,
            },
            metadata: BuildMetadata {
                build_invocation_id: "inv".into(),
                build_started_on: Utc::now(),
                build_finished_on: None,
                completeness: completeness_default(),
                reproducible: false,
            },
            materials: vec![make_material("git://repo", "sha256", "abc")],
        };
        let result = ProvenanceGenerator::verify(&prov).unwrap();
        assert!(!result.passed);
        assert!(
            !result
                .checks
                .iter()
                .find(|c| c.name == "builder.id")
                .unwrap()
                .passed
        );
    }

    #[test]
    fn test_verify_invalid_no_materials() {
        let prov = SlsaProvenance {
            kind: "https://in-toto.io/Statement/v0.1".into(),
            version: 1,
            builder: Builder {
                id: "builder".into(),
                version: None,
                builder_dependencies: None,
            },
            metadata: BuildMetadata {
                build_invocation_id: "inv".into(),
                build_started_on: Utc::now(),
                build_finished_on: None,
                completeness: completeness_default(),
                reproducible: false,
            },
            materials: vec![],
        };
        let result = ProvenanceGenerator::verify(&prov).unwrap();
        assert!(!result.passed);
        assert!(
            !result
                .checks
                .iter()
                .find(|c| c.name == "materials")
                .unwrap()
                .passed
        );
    }

    #[test]
    fn test_verification_result_serialization() {
        let result = VerificationResult {
            passed: false,
            checks: vec![VerificationCheck {
                name: "builder.id".into(),
                passed: false,
                message: "builder ID is empty".into(),
            }],
        };
        let json = serde_json::to_string(&result).expect("serialize");
        let de: VerificationResult = serde_json::from_str(&json).expect("deserialize");
        assert!(!de.passed);
        assert_eq!(de.checks.len(), 1);
        assert_eq!(de.checks[0].name, "builder.id");
    }

    #[test]
    fn test_provenance_with_version() {
        let generator = ProvenanceGenerator::new("builder".into()).with_version("1.0.0".into());
        let materials = vec![make_material("git://repo", "sha256", "abc")];
        let prov = generator.generate("inv-4", materials, Utc::now(), None, true);
        assert_eq!(prov.builder.version.as_deref(), Some("1.0.0"));
    }
}
