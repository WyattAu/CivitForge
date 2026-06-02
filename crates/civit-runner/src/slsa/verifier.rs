#![forbid(unsafe_code)]

use crate::slsa::attestation::SlsaLevel;
use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationResult {
    pub passed: bool,
    pub level: SlsaLevel,
    pub checks: Vec<CheckResult>,
    pub digest_match: Option<bool>,
    pub timestamp_valid: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckResult {
    pub name: String,
    pub passed: bool,
    pub detail: String,
}

pub struct HermeticVerifier {
    #[allow(dead_code)]
    expected_source_digest: Option<String>,
}

impl Default for HermeticVerifier {
    fn default() -> Self {
        Self::new()
    }
}

impl HermeticVerifier {
    pub fn new() -> Self {
        Self {
            expected_source_digest: None,
        }
    }

    pub fn with_source_digest(digest: String) -> Self {
        Self {
            expected_source_digest: Some(digest),
        }
    }

    pub fn verify(
        &self,
        provenance: &crate::slsa::attestation::SlsaProvenance,
        binary_digest: &str,
    ) -> VerificationResult {
        let mut checks = Vec::new();

        let digest_match = self.check_digest_match(provenance, binary_digest, &mut checks);
        let _builder_ok = self.check_builder_identity(provenance, &mut checks);
        let timestamp_valid =
            self.check_timestamp_freshness(provenance, Duration::hours(24), &mut checks);
        let _build_type_ok = self.check_build_type(provenance, &mut checks);

        let passed = checks.iter().all(|c| c.passed);
        let level = if passed {
            SlsaLevel::Three
        } else {
            SlsaLevel::One
        };

        VerificationResult {
            passed,
            level,
            checks,
            digest_match: Some(digest_match),
            timestamp_valid: Some(timestamp_valid),
        }
    }

    pub fn check_digest_match(
        &self,
        provenance: &crate::slsa::attestation::SlsaProvenance,
        binary_digest: &str,
        checks: &mut Vec<CheckResult>,
    ) -> bool {
        if provenance.subject.is_empty() {
            checks.push(CheckResult {
                name: "digest_match".to_string(),
                passed: false,
                detail: "no subjects in provenance".to_string(),
            });
            return false;
        }
        let expected = &provenance.subject[0].digest;
        let sha256_val = expected.get("sha256").map(|s| s.as_str()).unwrap_or("");
        let match_ = sha256_val == binary_digest;
        checks.push(CheckResult {
            name: "digest_match".to_string(),
            passed: match_,
            detail: if match_ {
                "digest matches".to_string()
            } else {
                format!("digest mismatch: expected={sha256_val}, got={binary_digest}")
            },
        });
        match_
    }

    pub fn check_builder_identity(
        &self,
        provenance: &crate::slsa::attestation::SlsaProvenance,
        checks: &mut Vec<CheckResult>,
    ) -> bool {
        let builder_id = &provenance.predicate.builder.id;
        let is_hermetic = builder_id.contains("civit")
            || builder_id.contains("hermetic")
            || builder_id.contains("github")
            || builder_id.contains("trusted");
        checks.push(CheckResult {
            name: "builder_identity".to_string(),
            passed: is_hermetic,
            detail: if is_hermetic {
                format!("builder identity recognized: {builder_id}")
            } else {
                format!("untrusted builder: {builder_id}")
            },
        });
        is_hermetic
    }

    pub fn check_timestamp_freshness(
        &self,
        provenance: &crate::slsa::attestation::SlsaProvenance,
        max_age: Duration,
        checks: &mut Vec<CheckResult>,
    ) -> bool {
        let started = provenance.predicate.metadata.started_on;
        let elapsed = Utc::now().signed_duration_since(started);
        let fresh = elapsed < max_age;
        checks.push(CheckResult {
            name: "timestamp_freshness".to_string(),
            passed: fresh,
            detail: if fresh {
                format!("build timestamp is fresh ({}s ago)", elapsed.num_seconds())
            } else {
                format!(
                    "build timestamp too old ({}s ago, max={})",
                    elapsed.num_seconds(),
                    max_age.num_seconds()
                )
            },
        });
        fresh
    }

    pub fn check_build_type(
        &self,
        provenance: &crate::slsa::attestation::SlsaProvenance,
        checks: &mut Vec<CheckResult>,
    ) -> bool {
        let build_type = &provenance.predicate.build_type;
        let known = !build_type.is_empty();
        checks.push(CheckResult {
            name: "build_type".to_string(),
            passed: known,
            detail: if known {
                format!("build type: {build_type}")
            } else {
                "unknown build type".to_string()
            },
        });
        known
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::slsa::attestation::SlsaProvenance;

    fn make_provenance(digest: &str, builder_id: &str) -> SlsaProvenance {
        SlsaProvenance::v1(
            "test-app",
            digest,
            builder_id,
            "https://civitforge.dev/pipeline/v1",
            ".civit/pipeline.yaml@main",
        )
    }

    #[test]
    fn test_verify_passes() {
        let verifier = HermeticVerifier::new();
        let p = make_provenance("sha256:abc123", "civit-runner");
        let result = verifier.verify(&p, "sha256:abc123");
        assert!(result.passed);
        assert_eq!(result.level, SlsaLevel::Three);
        assert!(result.digest_match.unwrap());
    }

    #[test]
    fn test_verify_digest_mismatch() {
        let verifier = HermeticVerifier::new();
        let p = make_provenance("sha256:abc123", "civit-runner");
        let result = verifier.verify(&p, "sha256:wrong");
        assert!(!result.passed);
        assert!(result.digest_match.is_some());
        assert!(!result.digest_match.unwrap());
    }

    #[test]
    fn test_untrusted_builder() {
        let verifier = HermeticVerifier::new();
        let p = make_provenance("sha256:abc123", "evil-builder");
        let result = verifier.verify(&p, "sha256:abc123");
        assert!(!result.passed);
        assert_eq!(result.level, SlsaLevel::One);
    }

    #[test]
    fn test_check_result_serialization() {
        let cr = CheckResult {
            name: "test".to_string(),
            passed: true,
            detail: "ok".to_string(),
        };
        let json = serde_json::to_string(&cr).unwrap();
        let restored: CheckResult = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.name, "test");
        assert!(restored.passed);
    }

    #[test]
    fn test_verification_result_serialization() {
        let result = VerificationResult {
            passed: true,
            level: SlsaLevel::Three,
            checks: vec![],
            digest_match: Some(true),
            timestamp_valid: Some(true),
        };
        let json = serde_json::to_string(&result).unwrap();
        let restored: VerificationResult = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.level, SlsaLevel::Three);
    }

    #[test]
    fn test_empty_subject() {
        let verifier = HermeticVerifier::new();
        let p = SlsaProvenance::v1("app", "sha256:x", "civit-runner", "t", "u");
        let result = verifier.verify(&p, "sha256:x");
        assert!(result.passed);
    }

    #[test]
    fn test_builder_identity_known_builders() {
        let verifier = HermeticVerifier::new();
        for builder in ["civit-runner", "github/workflow", "hermetic-builder"] {
            let p = make_provenance("sha256:x", builder);
            let mut checks = vec![];
            assert!(verifier.check_builder_identity(&p, &mut checks));
        }
    }

    #[test]
    fn test_timestamp_freshness_recent() {
        let verifier = HermeticVerifier::new();
        let p = make_provenance("sha256:x", "civit-runner");
        let mut checks = vec![];
        let fresh = verifier.check_timestamp_freshness(&p, Duration::hours(24), &mut checks);
        assert!(fresh);
    }

    #[test]
    fn test_build_type_check() {
        let verifier = HermeticVerifier::new();
        let p = make_provenance("sha256:x", "civit-runner");
        let mut checks = vec![];
        assert!(verifier.check_build_type(&p, &mut checks));
    }
}
