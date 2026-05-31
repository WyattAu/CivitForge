#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleaseInfo {
    pub version: String,
    pub commit_hash: String,
    pub build_timestamp: String,
    pub build_profile: String,
    pub rust_version: String,
    pub target: String,
}

impl ReleaseInfo {
    pub fn new(version: &str) -> Self {
        Self {
            version: version.to_string(),
            commit_hash: option_env!("CIVITFORGE_COMMIT_HASH")
                .unwrap_or("dev")
                .to_string(),
            build_timestamp: option_env!("CIVITFORGE_BUILD_TIMESTAMP")
                .unwrap_or("dev")
                .to_string(),
            build_profile: option_env!("CIVITFORGE_BUILD_PROFILE")
                .unwrap_or("debug")
                .to_string(),
            rust_version: option_env!("CIVITFORGE_RUST_VERSION")
                .unwrap_or("unknown")
                .to_string(),
            target: option_env!("CIVITFORGE_TARGET")
                .unwrap_or(std::env::consts::ARCH)
                .to_string(),
        }
    }
}

impl std::fmt::Display for ReleaseInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "CivitForge v{} ({}) built on {} [{}]",
            self.version, self.commit_hash, self.build_timestamp, self.build_profile
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_release_info_new() {
        let info = ReleaseInfo::new("0.1.0");
        assert_eq!(info.version, "0.1.0");
        assert!(!info.version.is_empty());
    }

    #[test]
    fn test_release_info_has_commit_hash() {
        let info = ReleaseInfo::new("1.0.0");
        assert!(!info.commit_hash.is_empty());
    }

    #[test]
    fn test_release_info_has_build_timestamp() {
        let info = ReleaseInfo::new("1.0.0");
        assert!(!info.build_timestamp.is_empty());
    }

    #[test]
    fn test_release_info_has_build_profile() {
        let info = ReleaseInfo::new("1.0.0");
        assert!(!info.build_profile.is_empty());
    }

    #[test]
    fn test_release_info_has_rust_version() {
        let info = ReleaseInfo::new("1.0.0");
        assert!(!info.rust_version.is_empty());
    }

    #[test]
    fn test_release_info_has_target() {
        let info = ReleaseInfo::new("1.0.0");
        assert!(!info.target.is_empty());
    }

    #[test]
    fn test_release_info_serialization() {
        let info = ReleaseInfo::new("2.0.0");
        let json = serde_json::to_string(&info).unwrap();
        let de: ReleaseInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(de.version, "2.0.0");
        assert_eq!(de.commit_hash, info.commit_hash);
        assert_eq!(de.build_timestamp, info.build_timestamp);
        assert_eq!(de.build_profile, info.build_profile);
        assert_eq!(de.rust_version, info.rust_version);
        assert_eq!(de.target, info.target);
    }

    #[test]
    fn test_release_info_display() {
        let info = ReleaseInfo::new("1.0.0");
        let display = format!("{info}");
        assert!(display.contains("CivitForge"));
        assert!(display.contains("1.0.0"));
        assert!(display.contains(&info.commit_hash));
    }

    #[test]
    fn test_release_info_clone() {
        let info = ReleaseInfo::new("3.0.0");
        let cloned = info.clone();
        assert_eq!(cloned.version, info.version);
        assert_eq!(cloned.commit_hash, info.commit_hash);
    }

    #[test]
    fn test_release_info_debug() {
        let info = ReleaseInfo::new("0.1.0");
        let debug = format!("{info:?}");
        assert!(debug.contains("ReleaseInfo"));
        assert!(debug.contains("0.1.0"));
    }

    #[test]
    fn test_release_info_deserialization_roundtrip() {
        let info = ReleaseInfo::new("4.0.0");
        let json = serde_json::to_value(&info).unwrap();
        assert!(json.is_object());
        assert_eq!(json["version"], "4.0.0");
        let de: ReleaseInfo = serde_json::from_value(json).unwrap();
        assert_eq!(de.version, "4.0.0");
    }

    #[test]
    fn test_release_info_version_matches_input() {
        for v in ["0.0.1", "1.0.0", "10.20.30", "dev", "nightly"] {
            let info = ReleaseInfo::new(v);
            assert_eq!(info.version, v);
        }
    }

    #[test]
    fn test_release_info_fields_not_default_empty() {
        let info = ReleaseInfo::new("1.0.0");
        assert!(!info.commit_hash.is_empty());
        assert!(!info.build_timestamp.is_empty());
        assert!(!info.build_profile.is_empty());
        assert!(!info.rust_version.is_empty());
        assert!(!info.target.is_empty());
    }

    #[test]
    fn test_release_info_json_keys() {
        let info = ReleaseInfo::new("1.0.0");
        let json = serde_json::to_value(&info).unwrap();
        let obj = json.as_object().unwrap();
        assert!(obj.contains_key("version"));
        assert!(obj.contains_key("commit_hash"));
        assert!(obj.contains_key("build_timestamp"));
        assert!(obj.contains_key("build_profile"));
        assert!(obj.contains_key("rust_version"));
        assert!(obj.contains_key("target"));
    }

    #[test]
    fn test_release_info_target_fallback() {
        let info = ReleaseInfo::new("1.0.0");
        assert!(!info.target.is_empty());
        assert_ne!(info.target, "");
    }

    #[test]
    fn test_release_info_partial_eq() {
        let a = ReleaseInfo::new("1.0.0");
        let b = ReleaseInfo::new("1.0.0");
        assert_eq!(a.version, b.version);
        assert_eq!(a.commit_hash, b.commit_hash);
        assert_eq!(a.target, b.target);
    }
}
