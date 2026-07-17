#![forbid(unsafe_code)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceConfig {
    pub active_frameworks: Vec<ComplianceFrameworkType>,
    pub data_retention: DataRetentionPolicy,
    pub access_control: AccessControlPolicy,
    pub encryption: EncryptionPolicy,
    pub audit_log: AuditLogPolicy,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Default for ComplianceConfig {
    fn default() -> Self {
        let now = Utc::now();
        Self {
            active_frameworks: vec![ComplianceFrameworkType::Soc2],
            data_retention: DataRetentionPolicy::default(),
            access_control: AccessControlPolicy::default(),
            encryption: EncryptionPolicy::default(),
            audit_log: AuditLogPolicy::default(),
            enabled: true,
            created_at: now,
            updated_at: now,
        }
    }
}

impl ComplianceConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_frameworks(mut self, frameworks: Vec<ComplianceFrameworkType>) -> Self {
        self.active_frameworks = frameworks;
        self
    }

    pub fn with_data_retention(mut self, policy: DataRetentionPolicy) -> Self {
        self.data_retention = policy;
        self
    }

    pub fn with_access_control(mut self, policy: AccessControlPolicy) -> Self {
        self.access_control = policy;
        self
    }

    pub fn with_encryption(mut self, policy: EncryptionPolicy) -> Self {
        self.encryption = policy;
        self
    }

    pub fn with_audit_log(mut self, policy: AuditLogPolicy) -> Self {
        self.audit_log = policy;
        self
    }

    pub fn is_framework_active(&self, framework: &ComplianceFrameworkType) -> bool {
        self.active_frameworks.contains(framework)
    }

    pub fn update(&mut self) {
        self.updated_at = Utc::now();
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ComplianceFrameworkType {
    Gdpr,
    Soc2,
    Hipaa,
    PciDss,
    Iso27001,
}

impl ComplianceFrameworkType {
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Gdpr => "GDPR",
            Self::Soc2 => "SOC 2",
            Self::Hipaa => "HIPAA",
            Self::PciDss => "PCI DSS",
            Self::Iso27001 => "ISO 27001",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Self::Gdpr => "General Data Protection Regulation",
            Self::Soc2 => "Service Organization Control 2",
            Self::Hipaa => "Health Insurance Portability and Accountability Act",
            Self::PciDss => "Payment Card Industry Data Security Standard",
            Self::Iso27001 => "ISO/IEC 27001 Information Security Management",
        }
    }

    pub fn requires_audit_logging(&self) -> bool {
        true
    }

    pub fn requires_encryption_at_rest(&self) -> bool {
        matches!(
            self,
            Self::Hipaa | Self::PciDss | Self::Iso27001 | Self::Soc2
        )
    }

    pub fn requires_encryption_in_transit(&self) -> bool {
        true
    }

    pub fn max_retention_days(&self) -> u32 {
        match self {
            Self::Gdpr => 365 * 3,
            Self::Soc2 => 365 * 7,
            Self::Hipaa => 365 * 6,
            Self::PciDss => 365,
            Self::Iso27001 => 365 * 3,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataRetentionPolicy {
    pub default_retention_days: u32,
    pub min_retention_days: u32,
    pub max_retention_days: u32,
    pub auto_archive_enabled: bool,
    pub auto_delete_enabled: bool,
    pub classification_policies: HashMap<String, ClassificationRetentionPolicy>,
    pub legal_hold_override: bool,
}

impl Default for DataRetentionPolicy {
    fn default() -> Self {
        let mut classification_policies = HashMap::new();
        classification_policies.insert(
            "pii".into(),
            ClassificationRetentionPolicy {
                retention_days: 2555,
                archive_after_days: Some(365),
                encrypt_at_rest: true,
                access_logging: true,
            },
        );
        classification_policies.insert(
            "phi".into(),
            ClassificationRetentionPolicy {
                retention_days: 2190,
                archive_after_days: Some(365),
                encrypt_at_rest: true,
                access_logging: true,
            },
        );

        Self {
            default_retention_days: 365,
            min_retention_days: 30,
            max_retention_days: 2555,
            auto_archive_enabled: true,
            auto_delete_enabled: false,
            classification_policies,
            legal_hold_override: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassificationRetentionPolicy {
    pub retention_days: u32,
    pub archive_after_days: Option<u32>,
    pub encrypt_at_rest: bool,
    pub access_logging: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessControlPolicy {
    pub require_mfa_for_admin: bool,
    pub require_mfa_for_sensitive: bool,
    pub sensitive_operations: Vec<String>,
    pub role_hierarchy: HashMap<String, Vec<String>>,
    pub max_session_duration_secs: i64,
    pub idle_timeout_secs: i64,
    pub enforce_ip_restrictions: bool,
    pub allowed_ip_ranges: Vec<String>,
    pub enforce_just_in_time_access: bool,
    pub jit_max_duration_secs: i64,
}

impl Default for AccessControlPolicy {
    fn default() -> Self {
        let mut role_hierarchy = HashMap::new();
        role_hierarchy.insert("admin".into(), vec!["member".into(), "guest".into()]);
        role_hierarchy.insert("member".into(), vec!["guest".into()]);

        Self {
            require_mfa_for_admin: true,
            require_mfa_for_sensitive: true,
            sensitive_operations: vec![
                "delete_user".into(),
                "manage_sso".into(),
                "manage_api_keys".into(),
                "update_settings".into(),
                "export_data".into(),
                "manage_compliance".into(),
            ],
            role_hierarchy,
            max_session_duration_secs: 86400,
            idle_timeout_secs: 1800,
            enforce_ip_restrictions: false,
            allowed_ip_ranges: Vec::new(),
            enforce_just_in_time_access: false,
            jit_max_duration_secs: 3600,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionPolicy {
    pub require_encryption_at_rest: bool,
    pub require_encryption_in_transit: bool,
    pub minimum_tls_version: TlsVersion,
    pub allowed_cipher_suites: Vec<String>,
    pub key_rotation_days: u32,
    pub encryption_algorithm: EncryptionAlgorithm,
    pub require_per_field_encryption: bool,
    pub encrypted_fields: Vec<String>,
}

impl Default for EncryptionPolicy {
    fn default() -> Self {
        Self {
            require_encryption_at_rest: true,
            require_encryption_in_transit: true,
            minimum_tls_version: TlsVersion::Tls12,
            allowed_cipher_suites: vec![
                "TLS_AES_256_GCM_SHA384".into(),
                "TLS_CHACHA20_POLY1305_SHA256".into(),
                "TLS_AES_128_GCM_SHA256".into(),
            ],
            key_rotation_days: 90,
            encryption_algorithm: EncryptionAlgorithm::Aes256Gcm,
            require_per_field_encryption: false,
            encrypted_fields: vec![
                "password_hash".into(),
                "api_key_hash".into(),
                "email".into(),
            ],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TlsVersion {
    Tls10,
    Tls11,
    Tls12,
    Tls13,
}

impl TlsVersion {
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Tls10 => "TLS 1.0",
            Self::Tls11 => "TLS 1.1",
            Self::Tls12 => "TLS 1.2",
            Self::Tls13 => "TLS 1.3",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EncryptionAlgorithm {
    Aes128Gcm,
    Aes256Gcm,
    ChaCha20Poly1305,
}

impl EncryptionAlgorithm {
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Aes128Gcm => "AES-128-GCM",
            Self::Aes256Gcm => "AES-256-GCM",
            Self::ChaCha20Poly1305 => "ChaCha20-Poly1305",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogPolicy {
    pub enabled: bool,
    pub retention_days: u32,
    pub log_auth_events: bool,
    pub log_data_access: bool,
    pub log_admin_operations: bool,
    pub log_api_access: bool,
    pub log_system_events: bool,
    pub require_tamper_evidence: bool,
    pub export_enabled: bool,
    pub real_time_alerting: bool,
    pub alert_on_high_risk: bool,
    pub min_classification_for_logging: AuditLogLevel,
}

impl Default for AuditLogPolicy {
    fn default() -> Self {
        Self {
            enabled: true,
            retention_days: 365 * 2,
            log_auth_events: true,
            log_data_access: true,
            log_admin_operations: true,
            log_api_access: true,
            log_system_events: true,
            require_tamper_evidence: true,
            export_enabled: true,
            real_time_alerting: true,
            alert_on_high_risk: true,
            min_classification_for_logging: AuditLogLevel::Internal,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditLogLevel {
    Public,
    Internal,
    Confidential,
    Restricted,
}

impl AuditLogLevel {
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Public => "Public",
            Self::Internal => "Internal",
            Self::Confidential => "Confidential",
            Self::Restricted => "Restricted",
        }
    }

    pub fn min_severity_to_log(&self) -> u32 {
        match self {
            Self::Public => 0,
            Self::Internal => 10,
            Self::Confidential => 50,
            Self::Restricted => 80,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compliance_config_default() {
        let config = ComplianceConfig::default();
        assert!(config.enabled);
        assert_eq!(config.active_frameworks.len(), 1);
        assert_eq!(config.active_frameworks[0], ComplianceFrameworkType::Soc2);
    }

    #[test]
    fn test_compliance_config_builder() {
        let config = ComplianceConfig::new()
            .with_frameworks(vec![
                ComplianceFrameworkType::Gdpr,
                ComplianceFrameworkType::Hipaa,
            ])
            .with_encryption(EncryptionPolicy {
                require_encryption_at_rest: true,
                require_encryption_in_transit: true,
                minimum_tls_version: TlsVersion::Tls13,
                ..Default::default()
            });
        assert_eq!(config.active_frameworks.len(), 2);
        assert_eq!(config.encryption.minimum_tls_version, TlsVersion::Tls13);
    }

    #[test]
    fn test_is_framework_active() {
        let config = ComplianceConfig::new()
            .with_frameworks(vec![ComplianceFrameworkType::Soc2]);
        assert!(config.is_framework_active(&ComplianceFrameworkType::Soc2));
        assert!(!config.is_framework_active(&ComplianceFrameworkType::Gdpr));
    }

    #[test]
    fn test_framework_properties() {
        assert!(ComplianceFrameworkType::Hipaa.requires_encryption_at_rest());
        assert!(ComplianceFrameworkType::Soc2.requires_audit_logging());
        assert!(ComplianceFrameworkType::PciDss.requires_encryption_in_transit());
    }

    #[test]
    fn test_framework_max_retention() {
        assert_eq!(ComplianceFrameworkType::Hipaa.max_retention_days(), 365 * 6);
        assert_eq!(ComplianceFrameworkType::PciDss.max_retention_days(), 365);
    }

    #[test]
    fn test_data_retention_default() {
        let policy = DataRetentionPolicy::default();
        assert_eq!(policy.default_retention_days, 365);
        assert!(policy.auto_archive_enabled);
        assert!(!policy.auto_delete_enabled);
        assert!(policy.legal_hold_override);
        assert!(policy.classification_policies.contains_key("pii"));
        assert!(policy.classification_policies.contains_key("phi"));
    }

    #[test]
    fn test_access_control_default() {
        let policy = AccessControlPolicy::default();
        assert!(policy.require_mfa_for_admin);
        assert!(policy.require_mfa_for_sensitive);
        assert!(policy.sensitive_operations.contains(&"delete_user".to_string()));
        assert!(!policy.enforce_ip_restrictions);
    }

    #[test]
    fn test_encryption_policy_default() {
        let policy = EncryptionPolicy::default();
        assert!(policy.require_encryption_at_rest);
        assert!(policy.require_encryption_in_transit);
        assert_eq!(policy.minimum_tls_version, TlsVersion::Tls12);
        assert_eq!(policy.key_rotation_days, 90);
        assert_eq!(
            policy.encryption_algorithm,
            EncryptionAlgorithm::Aes256Gcm
        );
    }

    #[test]
    fn test_tls_version_display() {
        assert_eq!(TlsVersion::Tls12.display_name(), "TLS 1.2");
        assert_eq!(TlsVersion::Tls13.display_name(), "TLS 1.3");
    }

    #[test]
    fn test_encryption_algorithm_display() {
        assert_eq!(
            EncryptionAlgorithm::Aes256Gcm.display_name(),
            "AES-256-GCM"
        );
    }

    #[test]
    fn test_audit_log_policy_default() {
        let policy = AuditLogPolicy::default();
        assert!(policy.enabled);
        assert!(policy.log_auth_events);
        assert!(policy.log_data_access);
        assert!(policy.require_tamper_evidence);
        assert!(policy.export_enabled);
    }

    #[test]
    fn test_audit_log_level_display() {
        assert_eq!(AuditLogLevel::Internal.display_name(), "Internal");
        assert_eq!(AuditLogLevel::Restricted.display_name(), "Restricted");
    }

    #[test]
    fn test_audit_log_level_min_severity() {
        assert_eq!(AuditLogLevel::Public.min_severity_to_log(), 0);
        assert_eq!(AuditLogLevel::Restricted.min_severity_to_log(), 80);
    }

    #[test]
    fn test_classification_retention_policy() {
        let policy = ClassificationRetentionPolicy {
            retention_days: 2555,
            archive_after_days: Some(365),
            encrypt_at_rest: true,
            access_logging: true,
        };
        assert_eq!(policy.retention_days, 2555);
        assert!(policy.encrypt_at_rest);
    }

    #[test]
    fn test_framework_description() {
        assert!(ComplianceFrameworkType::Gdpr.description().contains("Data Protection"));
        assert!(ComplianceFrameworkType::Soc2.description().contains("Service Organization"));
    }
}
