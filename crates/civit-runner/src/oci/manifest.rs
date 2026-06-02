#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OciManifest {
    #[serde(rename = "schemaVersion")]
    pub schema_version: u32,
    #[serde(rename = "mediaType")]
    pub media_type: String,
    pub config: OciDescriptor,
    pub layers: Vec<OciDescriptor>,
    #[serde(default)]
    pub annotations: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OciDescriptor {
    #[serde(rename = "mediaType")]
    pub media_type: String,
    pub digest: String,
    pub size: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub annotations: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OciImageConfig {
    pub architecture: String,
    pub os: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config: Option<HashMap<String, serde_json::Value>>,
    pub rootfs: RootFs,
    #[serde(default)]
    pub history: Vec<HistoryEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RootFs {
    #[serde(rename = "type")]
    pub type_: String,
    pub diff_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created: Option<String>,
    #[serde(rename = "createdBy", skip_serializing_if = "Option::is_none")]
    pub created_by: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
    #[serde(rename = "emptyLayer", skip_serializing_if = "Option::is_none")]
    pub empty_layer: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OciIndex {
    #[serde(rename = "schemaVersion")]
    pub schema_version: u32,
    #[serde(rename = "mediaType")]
    pub media_type: String,
    #[serde(default)]
    pub manifests: Vec<OciIndexManifest>,
    #[serde(default)]
    pub annotations: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OciIndexManifest {
    #[serde(rename = "mediaType")]
    pub media_type: String,
    pub digest: String,
    pub size: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub platform: Option<OciPlatform>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub annotations: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OciPlatform {
    pub architecture: String,
    pub os: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variant: Option<String>,
}

#[derive(Debug, Clone)]
pub struct HelmChart {
    pub name: String,
    pub version: String,
    pub chart_content: Vec<u8>,
    pub values_content: Vec<u8>,
}

impl OciManifest {
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    pub fn digest(&self) -> String {
        let json = serde_json::to_vec(self).unwrap_or_default();
        let mut hasher = Sha256::new();
        hasher.update(&json);
        format!("sha256:{}", hex::encode(hasher.finalize()))
    }
}

impl OciIndex {
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}

impl HelmChart {
    pub fn to_oci_layer(&self) -> OciDescriptor {
        let mut hasher = Sha256::new();
        hasher.update(&self.chart_content);
        hasher.update(&self.values_content);
        let digest = format!("sha256:{}", hex::encode(hasher.finalize()));
        let size = (self.chart_content.len() + self.values_content.len()) as u64;
        OciDescriptor {
            media_type: "application/vnd.civit.helm.chart.v1.tar+gzip".to_string(),
            digest,
            size,
            annotations: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manifest_serialization_roundtrip() {
        let manifest = OciManifest {
            schema_version: 2,
            media_type: "application/vnd.oci.image.manifest.v1+json".to_string(),
            config: OciDescriptor {
                media_type: "application/vnd.oci.image.config.v1+json".to_string(),
                digest: "sha256:abc123".to_string(),
                size: 1234,
                annotations: None,
            },
            layers: vec![OciDescriptor {
                media_type: "application/vnd.oci.image.layer.v1.tar+gzip".to_string(),
                digest: "sha256:def456".to_string(),
                size: 5678,
                annotations: None,
            }],
            annotations: HashMap::new(),
        };
        let json = manifest.to_json().unwrap();
        let deserialized = OciManifest::from_json(&json).unwrap();
        assert_eq!(manifest.schema_version, deserialized.schema_version);
        assert_eq!(manifest.layers.len(), deserialized.layers.len());
    }

    #[test]
    fn test_manifest_digest() {
        let manifest = OciManifest {
            schema_version: 2,
            media_type: "application/vnd.oci.image.manifest.v1+json".to_string(),
            config: OciDescriptor {
                media_type: "application/vnd.oci.image.config.v1+json".to_string(),
                digest: "sha256:config".to_string(),
                size: 100,
                annotations: None,
            },
            layers: vec![],
            annotations: HashMap::new(),
        };
        let digest = manifest.digest();
        assert!(digest.starts_with("sha256:"));
        assert_eq!(digest.len(), 7 + 64);
    }

    #[test]
    fn test_index_serialization_roundtrip() {
        let index = OciIndex {
            schema_version: 2,
            media_type: "application/vnd.oci.image.index.v1+json".to_string(),
            manifests: vec![OciIndexManifest {
                media_type: "application/vnd.oci.image.manifest.v1+json".to_string(),
                digest: "sha256:abc".to_string(),
                size: 500,
                platform: Some(OciPlatform {
                    architecture: "amd64".to_string(),
                    os: "linux".to_string(),
                    variant: None,
                }),
                annotations: None,
            }],
            annotations: HashMap::new(),
        };
        let json = index.to_json().unwrap();
        let deserialized = OciIndex::from_json(&json).unwrap();
        assert_eq!(index.manifests.len(), deserialized.manifests.len());
    }

    #[test]
    fn test_helm_chart_to_oci_layer() {
        let chart = HelmChart {
            name: "my-app".to_string(),
            version: "1.0.0".to_string(),
            chart_content: b"chart data".to_vec(),
            values_content: b"values data".to_vec(),
        };
        let layer = chart.to_oci_layer();
        assert!(layer.digest.starts_with("sha256:"));
        assert_eq!(layer.size, 10 + 11);
    }

    #[test]
    fn test_descriptor_with_annotations() {
        let desc = OciDescriptor {
            media_type: "application/octet-stream".to_string(),
            digest: "sha256:1234".to_string(),
            size: 42,
            annotations: Some({
                let mut m = HashMap::new();
                m.insert(
                    "org.opencontainers.image.title".to_string(),
                    "test".to_string(),
                );
                m
            }),
        };
        let json = serde_json::to_string(&desc).unwrap();
        let d: OciDescriptor = serde_json::from_str(&json).unwrap();
        assert_eq!(
            d.annotations.unwrap()["org.opencontainers.image.title"],
            "test"
        );
    }
}
