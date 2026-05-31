#![forbid(unsafe_code)]

use base64::Engine as _;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SlsaLevel {
    One,
    Two,
    Three,
    Four,
}

impl std::fmt::Display for SlsaLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::One => write!(f, "1"),
            Self::Two => write!(f, "2"),
            Self::Three => write!(f, "3"),
            Self::Four => write!(f, "4"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlsaProvenance {
    pub version: u32,
    pub predicate_type: String,
    pub subject: Vec<Subject>,
    pub predicate: Predicate,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subject {
    pub name: String,
    pub digest: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Predicate {
    pub builder: Builder,
    pub build_type: String,
    pub invocation: Invocation,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub build_config: Option<BuildConfig>,
    pub metadata: BuildMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Builder {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(default)]
    pub builder_dependencies: Vec<BuilderDep>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuilderDep {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uri: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub digest: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Invocation {
    pub config_source: ConfigSource,
    #[serde(default)]
    pub parameters: HashMap<String, serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub environment: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigSource {
    pub uri: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub digest: Option<HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entry_point: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildMetadata {
    pub invocation_id: String,
    pub started_on: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub finished_on: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildConfig {
    #[serde(default)]
    pub values: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransitiveAttestation {
    pub attestations: Vec<SlsaProvenance>,
    pub verified: bool,
}

impl SlsaProvenance {
    pub fn v1(
        subject_name: &str,
        subject_digest: &str,
        builder_id: &str,
        build_type: &str,
        config_uri: &str,
    ) -> Self {
        let mut digest_map = HashMap::new();
        digest_map.insert("sha256".to_string(), subject_digest.to_string());

        Self {
            version: 1,
            predicate_type: "https://slsa.dev/provenance/v1".to_string(),
            subject: vec![Subject {
                name: subject_name.to_string(),
                digest: digest_map,
            }],
            predicate: Predicate {
                builder: Builder {
                    id: builder_id.to_string(),
                    version: None,
                    builder_dependencies: vec![],
                },
                build_type: build_type.to_string(),
                invocation: Invocation {
                    config_source: ConfigSource {
                        uri: config_uri.to_string(),
                        digest: None,
                        entry_point: None,
                    },
                    parameters: HashMap::new(),
                    environment: None,
                },
                build_config: None,
                metadata: BuildMetadata {
                    invocation_id: uuid::Uuid::new_v4().to_string(),
                    started_on: Utc::now(),
                    finished_on: None,
                },
            },
        }
    }

    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    pub fn to_envelope(&self) -> Result<String, serde_json::Error> {
        let payload =
            base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(serde_json::to_vec(self)?);
        let envelope = serde_json::json!({
            "payloadType": "application/vnd.in-toto+json",
            "payload": payload,
            "signatures": [],
        });
        serde_json::to_string_pretty(&envelope)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_v1_creation() {
        let p = SlsaProvenance::v1(
            "my-app",
            "sha256:abc123",
            "civit-runner",
            "https://civitforge.dev/pipeline/v1",
            ".civit/pipeline.yaml@main",
        );
        assert_eq!(p.version, 1);
        assert_eq!(p.predicate_type, "https://slsa.dev/provenance/v1");
        assert_eq!(p.subject.len(), 1);
        assert_eq!(p.subject[0].name, "my-app");
        assert_eq!(p.subject[0].digest["sha256"], "sha256:abc123");
        assert_eq!(p.predicate.builder.id, "civit-runner");
        assert_eq!(
            p.predicate.invocation.config_source.uri,
            ".civit/pipeline.yaml@main"
        );
    }

    #[test]
    fn test_json_roundtrip() {
        let p = SlsaProvenance::v1(
            "test-binary",
            "sha256:deadbeef",
            "test-builder",
            "https://example.com/build/v1",
            "pipeline.yaml@v1.0",
        );
        let json = p.to_json().unwrap();
        let restored = SlsaProvenance::from_json(&json).unwrap();
        assert_eq!(restored.version, p.version);
        assert_eq!(restored.subject[0].name, p.subject[0].name);
        assert_eq!(restored.predicate.builder.id, p.predicate.builder.id);
    }

    #[test]
    fn test_envelope_format() {
        let p = SlsaProvenance::v1("app", "sha256:abc", "builder", "type", "uri");
        let envelope = p.to_envelope().unwrap();
        let val: serde_json::Value = serde_json::from_str(&envelope).unwrap();
        assert_eq!(val["payloadType"], "application/vnd.in-toto+json");
        assert!(val["payload"].is_string());
        assert!(val["signatures"].is_array());
    }

    #[test]
    fn test_slse_level_display() {
        assert_eq!(SlsaLevel::One.to_string(), "1");
        assert_eq!(SlsaLevel::Two.to_string(), "2");
        assert_eq!(SlsaLevel::Three.to_string(), "3");
        assert_eq!(SlsaLevel::Four.to_string(), "4");
    }

    #[test]
    fn test_transitive_attestation() {
        let a1 = SlsaProvenance::v1("a", "sha256:a", "b1", "t", "u");
        let a2 = SlsaProvenance::v1("b", "sha256:b", "b2", "t", "u");
        let ta = TransitiveAttestation {
            attestations: vec![a1, a2],
            verified: false,
        };
        let json = serde_json::to_string(&ta).unwrap();
        let restored: TransitiveAttestation = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.attestations.len(), 2);
        assert!(!restored.verified);
    }

    #[test]
    fn test_build_metadata() {
        let p = SlsaProvenance::v1("app", "sha256:x", "b", "t", "u");
        assert!(!p.predicate.metadata.invocation_id.is_empty());
    }
}
