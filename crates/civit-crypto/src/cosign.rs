#![forbid(unsafe_code)]

use crate::hash::HashService;
use serde::{Deserialize, Serialize};
use tracing::info;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageRef {
    pub registry: String,
    pub repository: String,
    pub tag: String,
    pub digest: Option<String>,
}

impl ImageRef {
    pub fn full_name(&self) -> String {
        match &self.digest {
            Some(d) => format!("{}/{}@{}", self.registry, self.repository, d),
            None => format!("{}/{}:{}", self.registry, self.repository, self.tag),
        }
    }

    pub fn without_registry(&self) -> String {
        match &self.digest {
            Some(d) => format!("{}@{}", self.repository, d),
            None => format!("{}:{}", self.repository, self.tag),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CosignSignature {
    pub base64_signature: String,
    pub payload: String,
    pub key_id: String,
    pub annotations: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyResult {
    pub verified: bool,
    pub digest: String,
    pub signer: String,
    pub reason: String,
}

pub struct CosignService;

impl Default for CosignService {
    fn default() -> Self {
        Self::new()
    }
}

impl CosignService {
    pub fn new() -> Self {
        Self
    }

    pub fn sign_image(
        &self,
        image: &ImageRef,
        key: &[u8],
        annotations: serde_json::Value,
    ) -> anyhow::Result<CosignSignature> {
        let digest = HashService::hash(
            crate::hash::HashAlgorithm::Sha256,
            image.full_name().as_bytes(),
        )
        .hex;

        let payload = serde_json::json!({
            "critical": {
                "identity": {
                    "docker-reference": image.without_registry(),
                },
                "image": {
                    "docker-manifest-digest": format!("sha256:{digest}"),
                },
                "type": "cosign container image signature",
            },
            "optional": annotations,
        });

        let payload_str = serde_json::to_string(&payload)?;
        let payload_b64 = base64::Engine::encode(
            &base64::engine::general_purpose::URL_SAFE_NO_PAD,
            payload_str.as_bytes(),
        );

        let sig = crate::hmac::HmacService::sign(key, payload_b64.as_bytes());
        let sig_b64 = base64::Engine::encode(
            &base64::engine::general_purpose::STANDARD,
            hex::decode(&sig)?,
        );

        let key_hash = HashService::hash(crate::hash::HashAlgorithm::Sha256, key).hex;
        let key_id = &key_hash[..16];

        info!(
            image = %image.full_name(),
            digest = %digest,
            "signed image"
        );

        Ok(CosignSignature {
            base64_signature: sig_b64,
            payload: payload_b64,
            key_id: key_id.to_string(),
            annotations,
        })
    }

    pub fn verify_signature(
        &self,
        image: &ImageRef,
        signature: &CosignSignature,
        key: &[u8],
    ) -> VerifyResult {
        let sig_bytes = match base64::Engine::decode(
            &base64::engine::general_purpose::STANDARD,
            &signature.base64_signature,
        ) {
            Ok(b) => b,
            Err(e) => {
                return VerifyResult {
                    verified: false,
                    digest: String::new(),
                    signer: String::new(),
                    reason: format!("base64 decode error: {e}"),
                };
            }
        };

        let sig_hex = hex::encode(&sig_bytes);
        let valid = crate::hmac::HmacService::verify(key, signature.payload.as_bytes(), &sig_hex);

        if valid {
            let digest = HashService::hash(
                crate::hash::HashAlgorithm::Sha256,
                image.full_name().as_bytes(),
            )
            .hex;
            VerifyResult {
                verified: true,
                digest,
                signer: signature.key_id.clone(),
                reason: "signature valid".into(),
            }
        } else {
            VerifyResult {
                verified: false,
                digest: String::new(),
                signer: signature.key_id.clone(),
                reason: "signature verification failed".into(),
            }
        }
    }

    pub fn parse_image_ref(input: &str) -> anyhow::Result<ImageRef> {
        let parts: Vec<&str> = input.split('/').collect();
        let (registry, rest) = if parts.len() >= 3 {
            (parts[0], parts[1..].join("/"))
        } else {
            ("docker.io", parts.join("/"))
        };

        let (repository, tag) = if let Some(at_idx) = rest.find('@') {
            let repo = &rest[..at_idx];
            let digest = &rest[at_idx + 1..];
            return Ok(ImageRef {
                registry: registry.into(),
                repository: repo.into(),
                tag: "latest".into(),
                digest: Some(digest.into()),
            });
        } else if let Some(colon_idx) = rest.rfind(':') {
            let repo = &rest[..colon_idx];
            let tag = &rest[colon_idx + 1..];
            (repo, tag)
        } else {
            (rest.as_ref(), "latest")
        };

        Ok(ImageRef {
            registry: registry.into(),
            repository: repository.into(),
            tag: tag.into(),
            digest: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_image() {
        let ref_ = CosignService::parse_image_ref("nginx:latest").unwrap();
        assert_eq!(ref_.repository, "nginx");
        assert_eq!(ref_.tag, "latest");
    }

    #[test]
    fn test_parse_docker_hub() {
        let ref_ = CosignService::parse_image_ref("library/alpine:3.18").unwrap();
        assert_eq!(ref_.repository, "library/alpine");
        assert_eq!(ref_.tag, "3.18");
    }

    #[test]
    fn test_parse_registry_image() {
        let ref_ = CosignService::parse_image_ref("ghcr.io/civitforge/app:v1.0").unwrap();
        assert_eq!(ref_.registry, "ghcr.io");
        assert_eq!(ref_.repository, "civitforge/app");
        assert_eq!(ref_.tag, "v1.0");
    }

    #[test]
    fn test_parse_digest() {
        let ref_ = CosignService::parse_image_ref("alpine@sha256:abc123").unwrap();
        assert_eq!(ref_.repository, "alpine");
        assert_eq!(ref_.digest.as_deref(), Some("sha256:abc123"));
    }

    #[test]
    fn test_full_name() {
        let ref_ = ImageRef {
            registry: "ghcr.io".into(),
            repository: "civit/app".into(),
            tag: "v1".into(),
            digest: None,
        };
        assert_eq!(ref_.full_name(), "ghcr.io/civit/app:v1");
    }

    #[test]
    fn test_sign_and_verify() {
        let svc = CosignService::new();
        let image = ImageRef {
            registry: "ghcr.io".into(),
            repository: "civit/app".into(),
            tag: "latest".into(),
            digest: None,
        };
        let key = b"signing-key-32-bytes-long-enough";
        let sig = svc
            .sign_image(&image, key, serde_json::json!({"env": "prod"}))
            .unwrap();
        let result = svc.verify_signature(&image, &sig, key);
        assert!(result.verified);
    }

    #[test]
    fn test_verify_wrong_key() {
        let svc = CosignService::new();
        let image = ImageRef {
            registry: "ghcr.io".into(),
            repository: "civit/app".into(),
            tag: "latest".into(),
            digest: None,
        };
        let sig = svc
            .sign_image(&image, b"correct-key-32-bytes-long", serde_json::json!({}))
            .unwrap();
        let result = svc.verify_signature(&image, &sig, b"wrong-key-32-bytes-longggg");
        assert!(!result.verified);
    }

    #[test]
    fn test_image_ref_serialization() {
        let ref_ = ImageRef {
            registry: "ghcr.io".into(),
            repository: "app".into(),
            tag: "v1".into(),
            digest: None,
        };
        let json = serde_json::to_string(&ref_).unwrap();
        let de: ImageRef = serde_json::from_str(&json).unwrap();
        assert_eq!(de.registry, "ghcr.io");
    }
}
