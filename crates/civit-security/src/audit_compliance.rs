#![forbid(unsafe_code)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::Mutex;

// -- Core Audit Log Entry --

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogEntry {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub actor_id: String,
    pub actor_email: String,
    pub action: String,
    pub resource_type: String,
    pub resource_id: String,
    pub resource_name: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub details: serde_json::Value,
    pub data_classification: DataClassification,
    pub integrity_hash: String,
    pub previous_hash: Option<String>,
    pub compliance_frameworks: Vec<ComplianceFramework>,
}

impl AuditLogEntry {
    pub fn new(
        actor_id: &str,
        actor_email: &str,
        action: &str,
        resource_type: &str,
        resource_id: &str,
        previous_hash: Option<&str>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: now,
            actor_id: actor_id.into(),
            actor_email: actor_email.into(),
            action: action.into(),
            resource_type: resource_type.into(),
            resource_id: resource_id.into(),
            resource_name: None,
            ip_address: None,
            user_agent: None,
            details: serde_json::Value::Object(serde_json::Map::new()),
            data_classification: DataClassification::Internal,
            integrity_hash: String::new(),
            previous_hash: previous_hash.map(String::from),
            compliance_frameworks: Vec::new(),
        }
    }

    pub fn compute_hash(&self) -> String {
        let payload = format!(
            "{}:{}:{}:{}:{}:{}",
            self.timestamp.to_rfc3339(),
            self.actor_id,
            self.action,
            self.resource_type,
            self.resource_id,
            self.previous_hash.as_deref().unwrap_or(""),
        );
        let mut hasher = Sha256::new();
        hasher.update(payload.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    pub fn with_resource_name(mut self, name: &str) -> Self {
        self.resource_name = Some(name.into());
        self
    }

    pub fn with_ip(mut self, ip: &str) -> Self {
        self.ip_address = Some(ip.into());
        self
    }

    pub fn with_user_agent(mut self, ua: &str) -> Self {
        self.user_agent = Some(ua.into());
        self
    }

    pub fn with_details(mut self, details: serde_json::Value) -> Self {
        self.details = details;
        self
    }

    pub fn with_classification(mut self, classification: DataClassification) -> Self {
        self.data_classification = classification;
        self
    }

    pub fn with_frameworks(mut self, frameworks: Vec<ComplianceFramework>) -> Self {
        self.compliance_frameworks = frameworks;
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DataClassification {
    Public,
    Internal,
    Confidential,
    Restricted,
    Pii,
    Phi,
}

impl DataClassification {
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Public => "Public",
            Self::Internal => "Internal",
            Self::Confidential => "Confidential",
            Self::Restricted => "Restricted",
            Self::Pii => "PII",
            Self::Phi => "PHI",
        }
    }

    pub fn retention_days(&self) -> u32 {
        match self {
            Self::Public => 365,
            Self::Internal => 730,
            Self::Confidential => 1095,
            Self::Restricted => 2555,
            Self::Pii => 2555,
            Self::Phi => 2555,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ComplianceFramework {
    Gdpr,
    Soc2,
    Hipaa,
    PciDss,
    Iso27001,
}

impl ComplianceFramework {
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Gdpr => "GDPR",
            Self::Soc2 => "SOC 2",
            Self::Hipaa => "HIPAA",
            Self::PciDss => "PCI DSS",
            Self::Iso27001 => "ISO 27001",
        }
    }
}

// -- Tamper-Evident Hash Chain --

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditHashChain {
    pub chain_id: String,
    pub entries: Vec<AuditLogEntry>,
    pub last_hash: Option<String>,
    pub chain_length: u64,
    pub created_at: DateTime<Utc>,
    pub last_updated: DateTime<Utc>,
}

impl AuditHashChain {
    pub fn new() -> Self {
        let now = Utc::now();
        Self {
            chain_id: uuid::Uuid::new_v4().to_string(),
            entries: Vec::new(),
            last_hash: None,
            chain_length: 0,
            created_at: now,
            last_updated: now,
        }
    }

    pub fn append_entry(&mut self, mut entry: AuditLogEntry) {
        entry.previous_hash = self.last_hash.clone();
        entry.integrity_hash = entry.compute_hash();
        self.last_hash = Some(entry.integrity_hash.clone());
        self.entries.push(entry);
        self.chain_length += 1;
        self.last_updated = Utc::now();
    }

    pub fn verify_integrity(&self) -> ChainIntegrityResult {
        let mut previous_hash: Option<String> = None;
        let mut broken_at: Option<usize> = None;

        for (i, entry) in self.entries.iter().enumerate() {
            if entry.previous_hash != previous_hash {
                broken_at = Some(i);
                break;
            }
            let computed = entry.compute_hash();
            if computed != entry.integrity_hash {
                broken_at = Some(i);
                break;
            }
            previous_hash = Some(entry.integrity_hash.clone());
        }

        ChainIntegrityResult {
            valid: broken_at.is_none(),
            chain_length: self.chain_length as usize,
            broken_at_index: broken_at,
            verified_at: Utc::now(),
        }
    }

    pub fn get_entries_since(
        &self,
        since: DateTime<Utc>,
    ) -> Vec<&AuditLogEntry> {
        self.entries
            .iter()
            .filter(|e| e.timestamp >= since)
            .collect()
    }

    pub fn get_entries_for_actor(&self, actor_id: &str) -> Vec<&AuditLogEntry> {
        self.entries
            .iter()
            .filter(|e| e.actor_id == actor_id)
            .collect()
    }

    pub fn get_entries_for_resource(
        &self,
        resource_type: &str,
        resource_id: &str,
    ) -> Vec<&AuditLogEntry> {
        self.entries
            .iter()
            .filter(|e| e.resource_type == resource_type && e.resource_id == resource_id)
            .collect()
    }
}

impl Default for AuditHashChain {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainIntegrityResult {
    pub valid: bool,
    pub chain_length: usize,
    pub broken_at_index: Option<usize>,
    pub verified_at: DateTime<Utc>,
}

// -- GDPR Audit Trail --

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GdprAuditTrail {
    pub data_access_events: Vec<AuditLogEntry>,
    pub data_processing_events: Vec<AuditLogEntry>,
    pub consent_events: Vec<AuditLogEntry>,
    pub data_breach_events: Vec<AuditLogEntry>,
}

impl GdprAuditTrail {
    pub fn new() -> Self {
        Self {
            data_access_events: Vec::new(),
            data_processing_events: Vec::new(),
            consent_events: Vec::new(),
            data_breach_events: Vec::new(),
        }
    }

    pub fn record_data_access(&mut self, entry: AuditLogEntry) {
        self.data_access_events.push(entry);
    }

    pub fn record_data_processing(&mut self, entry: AuditLogEntry) {
        self.data_processing_events.push(entry);
    }

    pub fn record_consent(&mut self, entry: AuditLogEntry) {
        self.consent_events.push(entry);
    }

    pub fn record_data_breach(&mut self, entry: AuditLogEntry) {
        self.data_breach_events.push(entry);
    }

    pub fn who_accessed(
        &self,
        resource_id: &str,
        since: DateTime<Utc>,
    ) -> Vec<&AuditLogEntry> {
        self.data_access_events
            .iter()
            .filter(|e| e.resource_id == resource_id && e.timestamp >= since)
            .collect()
    }

    pub fn subject_access_request(
        &self,
        actor_id: &str,
        since: DateTime<Utc>,
    ) -> Vec<&AuditLogEntry> {
        self.data_access_events
            .iter()
            .filter(|e| e.actor_id == actor_id && e.timestamp >= since)
            .collect()
    }
}

impl Default for GdprAuditTrail {
    fn default() -> Self {
        Self::new()
    }
}

// -- SOC2 Control Evidence --

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Soc2ControlEvidence {
    pub control_id: String,
    pub control_name: String,
    pub category: Soc2Category,
    pub evidence_items: Vec<EvidenceItem>,
    pub status: ComplianceCheckStatus,
    pub last_verified: Option<DateTime<Utc>>,
    pub next_review: Option<DateTime<Utc>>,
}

impl Soc2ControlEvidence {
    pub fn new(control_id: &str, control_name: &str, category: Soc2Category) -> Self {
        Self {
            control_id: control_id.into(),
            control_name: control_name.into(),
            category,
            evidence_items: Vec::new(),
            status: ComplianceCheckStatus::Pending,
            last_verified: None,
            next_review: None,
        }
    }

    pub fn add_evidence(&mut self, item: EvidenceItem) {
        self.evidence_items.push(item);
    }

    pub fn mark_verified(&mut self) {
        self.status = ComplianceCheckStatus::Verified;
        self.last_verified = Some(Utc::now());
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Soc2Category {
    Security,
    Availability,
    ProcessingIntegrity,
    Confidentiality,
    Privacy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceItem {
    pub id: String,
    pub evidence_type: EvidenceType,
    pub description: String,
    pub content: String,
    pub collected_at: DateTime<Utc>,
    pub collected_by: String,
}

impl EvidenceItem {
    pub fn new(
        evidence_type: EvidenceType,
        description: &str,
        content: &str,
        collected_by: &str,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            evidence_type,
            description: description.into(),
            content: content.into(),
            collected_at: Utc::now(),
            collected_by: collected_by.into(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceType {
    AuditLog,
    Configuration,
    ScanResult,
    UserAccess,
    SystemConfig,
    AutomatedCheck,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ComplianceCheckStatus {
    Pending,
    Verified,
    Failed,
    Partial,
    NotApplicable,
}

// -- HIPAA Access Logging --

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HipaaAccessLog {
    pub access_events: Vec<AuditLogEntry>,
    pub phi_access_events: Vec<AuditLogEntry>,
    pub minimum_necessary_violations: Vec<MinimumNecessaryViolation>,
}

impl HipaaAccessLog {
    pub fn new() -> Self {
        Self {
            access_events: Vec::new(),
            phi_access_events: Vec::new(),
            minimum_necessary_violations: Vec::new(),
        }
    }

    pub fn record_phi_access(&mut self, entry: AuditLogEntry) {
        self.phi_access_events.push(entry);
    }

    pub fn record_violation(&mut self, violation: MinimumNecessaryViolation) {
        self.minimum_necessary_violations.push(violation);
    }

    pub fn phi_access_by_user(
        &self,
        user_id: &str,
        since: DateTime<Utc>,
    ) -> Vec<&AuditLogEntry> {
        self.phi_access_events
            .iter()
            .filter(|e| e.actor_id == user_id && e.timestamp >= since)
            .collect()
    }

    pub fn violations_since(
        &self,
        since: DateTime<Utc>,
    ) -> Vec<&MinimumNecessaryViolation> {
        self.minimum_necessary_violations
            .iter()
            .filter(|v| v.detected_at >= since)
            .collect()
    }
}

impl Default for HipaaAccessLog {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinimumNecessaryViolation {
    pub id: String,
    pub user_id: String,
    pub accessed_resource: String,
    pub justification: Option<String>,
    pub detected_at: DateTime<Utc>,
    pub severity: ViolationSeverity,
}

impl MinimumNecessaryViolation {
    pub fn new(user_id: &str, accessed_resource: &str) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            user_id: user_id.into(),
            accessed_resource: accessed_resource.into(),
            justification: None,
            detected_at: Utc::now(),
            severity: ViolationSeverity::Medium,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ViolationSeverity {
    Low,
    Medium,
    High,
    Critical,
}

// -- Audit Log Retention --

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetentionPolicy {
    pub id: String,
    pub name: String,
    pub classification: DataClassification,
    pub retention_days: u32,
    pub archive_after_days: Option<u32>,
    pub delete_after_days: Option<u32>,
    pub enabled: bool,
}

impl RetentionPolicy {
    pub fn new(name: &str, classification: DataClassification, retention_days: u32) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: name.into(),
            classification,
            retention_days,
            archive_after_days: None,
            delete_after_days: None,
            enabled: true,
        }
    }

    pub fn with_archive_after(mut self, days: u32) -> Self {
        self.archive_after_days = Some(days);
        self
    }

    pub fn with_delete_after(mut self, days: u32) -> Self {
        self.delete_after_days = Some(days);
        self
    }

    pub fn should_archive(&self, age_days: u32) -> bool {
        self.archive_after_days
            .is_some_and(|days| age_days >= days)
    }

    pub fn should_delete(&self, age_days: u32) -> bool {
        self.delete_after_days
            .is_some_and(|days| age_days >= days)
    }
}

// -- Compliance Report --

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceReport {
    pub id: String,
    pub framework: ComplianceFramework,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub total_events_audited: u64,
    pub events_by_action: HashMap<String, u64>,
    pub events_by_classification: HashMap<String, u64>,
    pub high_risk_events: u64,
    pub integrity_check_passed: bool,
    pub findings: Vec<ComplianceFinding>,
    pub recommendations: Vec<String>,
    pub generated_at: DateTime<Utc>,
    pub generated_by: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceFinding {
    pub id: String,
    pub category: String,
    pub severity: ViolationSeverity,
    pub title: String,
    pub description: String,
    pub affected_events: u64,
    pub recommendation: String,
}

// -- Export Formats --

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogExport {
    pub entries: Vec<AuditLogEntry>,
    pub total: usize,
    pub exported_at: DateTime<Utc>,
    pub format: ExportFormat,
    pub chain_integrity: Option<ChainIntegrityResult>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExportFormat {
    Json,
    Csv,
}

// -- Tamper-Evident Audit Log Store --

#[derive(Debug, Clone)]
pub struct TamperEvidentAuditLog {
    pub hash_chain: Arc<Mutex<AuditHashChain>>,
    pub gdpr_trail: Arc<Mutex<GdprAuditTrail>>,
    pub soc2_evidence: Arc<Mutex<Vec<Soc2ControlEvidence>>>,
    pub hipaa_log: Arc<Mutex<HipaaAccessLog>>,
    pub retention_policies: Arc<Mutex<Vec<RetentionPolicy>>>,
}

impl TamperEvidentAuditLog {
    pub fn new() -> Self {
        Self {
            hash_chain: Arc::new(Mutex::new(AuditHashChain::new())),
            gdpr_trail: Arc::new(Mutex::new(GdprAuditTrail::new())),
            soc2_evidence: Arc::new(Mutex::new(Vec::new())),
            hipaa_log: Arc::new(Mutex::new(HipaaAccessLog::new())),
            retention_policies: Arc::new(Mutex::new(Self::default_policies())),
        }
    }

    pub fn record_event(&self, mut entry: AuditLogEntry) {
        {
            let mut chain = self.hash_chain.lock();
            chain.append_entry(entry.clone());
            entry.previous_hash = chain.last_hash.clone();
        }

        if entry.data_classification == DataClassification::Pii
            || entry.data_classification == DataClassification::Phi
        {
            let mut gdpr = self.gdpr_trail.lock();
            gdpr.record_data_access(entry.clone());
        }

        if entry.data_classification == DataClassification::Phi {
            let mut hipaa = self.hipaa_log.lock();
            hipaa.record_phi_access(entry);
        }
    }

    pub fn verify_integrity(&self) -> ChainIntegrityResult {
        let chain = self.hash_chain.lock();
        chain.verify_integrity()
    }

    pub fn export_logs(
        &self,
        format: ExportFormat,
        since: Option<DateTime<Utc>>,
        until: Option<DateTime<Utc>>,
    ) -> AuditLogExport {
        let chain = self.hash_chain.lock();
        let entries: Vec<AuditLogEntry> = chain
            .entries
            .iter()
            .filter(|e| {
                if let Some(s) = since {
                    if e.timestamp < s {
                        return false;
                    }
                }
                if let Some(u) = until {
                    if e.timestamp > u {
                        return false;
                    }
                }
                true
            })
            .cloned()
            .collect();

        let total = entries.len();
        let integrity = chain.verify_integrity();

        AuditLogExport {
            entries,
            total,
            exported_at: Utc::now(),
            format,
            chain_integrity: Some(integrity),
        }
    }

    pub fn generate_compliance_report(
        &self,
        framework: ComplianceFramework,
        period_start: DateTime<Utc>,
        period_end: DateTime<Utc>,
        generated_by: &str,
    ) -> ComplianceReport {
        let chain = self.hash_chain.lock();
        let entries: Vec<&AuditLogEntry> = chain
            .entries
            .iter()
            .filter(|e| e.timestamp >= period_start && e.timestamp <= period_end)
            .collect();

        let total = entries.len() as u64;

        let mut events_by_action: HashMap<String, u64> = HashMap::new();
        let mut events_by_classification: HashMap<String, u64> = HashMap::new();
        let mut high_risk = 0u64;

        for entry in &entries {
            *events_by_action
                .entry(entry.action.clone())
                .or_insert(0) += 1;
            *events_by_classification
                .entry(entry.data_classification.display_name().into())
                .or_insert(0) += 1;
            if entry.data_classification == DataClassification::Restricted
                || entry.data_classification == DataClassification::Phi
            {
                high_risk += 1;
            }
        }

        let integrity = chain.verify_integrity();
        let findings = Self::generate_findings(&entries, &framework);

        ComplianceReport {
            id: uuid::Uuid::new_v4().to_string(),
            framework,
            period_start,
            period_end,
            total_events_audited: total,
            events_by_action,
            events_by_classification,
            high_risk_events: high_risk,
            integrity_check_passed: integrity.valid,
            findings,
            recommendations: Self::generate_recommendations(&framework),
            generated_at: Utc::now(),
            generated_by: generated_by.into(),
        }
    }

    pub fn add_retention_policy(&self, policy: RetentionPolicy) {
        let mut policies = self.retention_policies.lock();
        policies.push(policy);
    }

    pub fn get_hash_chain(&self) -> AuditHashChain {
        self.hash_chain.lock().clone()
    }

    fn generate_findings(
        entries: &[&AuditLogEntry],
        framework: &ComplianceFramework,
    ) -> Vec<ComplianceFinding> {
        let mut findings = Vec::new();

        let missing_ip = entries
            .iter()
            .filter(|e| e.ip_address.is_none())
            .count();
        if missing_ip > 0 {
            findings.push(ComplianceFinding {
                id: uuid::Uuid::new_v4().to_string(),
                category: "Audit Trail Completeness".into(),
                severity: ViolationSeverity::Medium,
                title: "Missing IP addresses in audit logs".into(),
                description: format!(
                    "{} events are missing IP address information",
                    missing_ip
                ),
                affected_events: missing_ip as u64,
                recommendation: "Ensure IP addresses are captured for all audit events".into(),
            });
        }

        let high_risk_count = entries
            .iter()
            .filter(|e| {
                e.data_classification == DataClassification::Restricted
                    || e.data_classification == DataClassification::Phi
            })
            .count();
        if high_risk_count > 0 && *framework == ComplianceFramework::Hipaa {
            findings.push(ComplianceFinding {
                id: uuid::Uuid::new_v4().to_string(),
                category: "PHI Access Monitoring".into(),
                severity: ViolationSeverity::High,
                title: "High-risk PHI access events detected".into(),
                description: format!(
                    "{} restricted/PHI events require enhanced monitoring",
                    high_risk_count
                ),
                affected_events: high_risk_count as u64,
                recommendation: "Review all PHI access events for minimum necessary compliance"
                    .into(),
            });
        }

        findings
    }

    fn generate_recommendations(framework: &ComplianceFramework) -> Vec<String> {
        match framework {
            ComplianceFramework::Gdpr => vec![
                "Maintain records of processing activities (Article 30)".into(),
                "Implement data subject access request tracking".into(),
                "Ensure right to erasure is logged".into(),
                "Document lawful basis for each processing activity".into(),
            ],
            ComplianceFramework::Soc2 => vec![
                "Document logical access controls (CC6.1)".into(),
                "Maintain change management records (CC8.1)".into(),
                "Monitor vulnerability management (CC7.1)".into(),
                "Review system boundaries regularly (CC6.6)".into(),
            ],
            ComplianceFramework::Hipaa => vec![
                "Track all PHI access with minimum necessary justification".into(),
                "Maintain audit logs for 6 years minimum".into(),
                "Implement automatic logoff policies".into(),
                "Document emergency access procedures".into(),
            ],
            ComplianceFramework::PciDss => vec![
                "Maintain audit trail for all access to cardholder data".into(),
                "Review logs at least daily".into(),
                "Retain audit trail history for at least one year".into(),
                "Synchronize time sources across all systems".into(),
            ],
            ComplianceFramework::Iso27001 => vec![
                "Maintain information security event records".into(),
                "Conduct regular internal audits".into(),
                "Document risk treatment decisions".into(),
                "Review security controls effectiveness".into(),
            ],
        }
    }

    fn default_policies() -> Vec<RetentionPolicy> {
        vec![
            RetentionPolicy::new("Public Events", DataClassification::Public, 365),
            RetentionPolicy::new("Internal Events", DataClassification::Internal, 730),
            RetentionPolicy::new(
                "Confidential Events",
                DataClassification::Confidential,
                1095,
            ),
            RetentionPolicy::new("Restricted Events", DataClassification::Restricted, 2555)
                .with_archive_after(365),
            RetentionPolicy::new("PII Events", DataClassification::Pii, 2555)
                .with_archive_after(365),
            RetentionPolicy::new("PHI Events", DataClassification::Phi, 2555)
                .with_archive_after(365),
        ]
    }
}

impl Default for TamperEvidentAuditLog {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_entry(actor: &str, action: &str) -> AuditLogEntry {
        AuditLogEntry::new(actor, &format!("{}@example.com", actor), action, "repo", "r-1", None)
    }

    #[test]
    fn test_audit_log_entry_new() {
        let entry = make_entry("user-1", "read");
        assert_eq!(entry.actor_id, "user-1");
        assert_eq!(entry.action, "read");
        assert!(entry.integrity_hash.is_empty());
    }

    #[test]
    fn test_audit_log_entry_compute_hash() {
        let entry = make_entry("user-1", "read");
        let hash = entry.compute_hash();
        assert_eq!(hash.len(), 64);
    }

    #[test]
    fn test_audit_log_entry_hash_deterministic() {
        let entry = make_entry("user-1", "read");
        let h1 = entry.compute_hash();
        let h2 = entry.compute_hash();
        assert_eq!(h1, h2);
    }

    #[test]
    fn test_audit_log_entry_with_fields() {
        let entry = AuditLogEntry::new("u-1", "u@example.com", "write", "repo", "r-1", None)
            .with_resource_name("my-repo")
            .with_ip("192.168.1.1")
            .with_user_agent("Mozilla/5.0")
            .with_classification(DataClassification::Confidential)
            .with_details(serde_json::json!({"key": "value"}));
        assert_eq!(entry.resource_name.as_deref(), Some("my-repo"));
        assert_eq!(entry.ip_address.as_deref(), Some("192.168.1.1"));
        assert_eq!(entry.data_classification, DataClassification::Confidential);
    }

    #[test]
    fn test_data_classification_retention() {
        assert_eq!(DataClassification::Public.retention_days(), 365);
        assert_eq!(DataClassification::Pii.retention_days(), 2555);
        assert_eq!(DataClassification::Phi.retention_days(), 2555);
    }

    #[test]
    fn test_hash_chain_append_and_verify() {
        let mut chain = AuditHashChain::new();
        let entry1 = make_entry("user-1", "create");
        let entry2 = make_entry("user-2", "delete");

        chain.append_entry(entry1);
        chain.append_entry(entry2);

        assert_eq!(chain.chain_length, 2);
        assert!(chain.last_hash.is_some());

        let result = chain.verify_integrity();
        assert!(result.valid);
        assert_eq!(result.chain_length, 2);
    }

    #[test]
    fn test_hash_chain_empty_valid() {
        let chain = AuditHashChain::new();
        let result = chain.verify_integrity();
        assert!(result.valid);
        assert_eq!(result.chain_length, 0);
    }

    #[test]
    fn test_hash_chain_get_entries_since() {
        let mut chain = AuditHashChain::new();
        let mut entry = make_entry("user-1", "read");
        entry.timestamp = Utc::now() - chrono::Duration::hours(2);
        chain.append_entry(entry);

        let recent_entry = make_entry("user-2", "write");
        chain.append_entry(recent_entry);

        let since = Utc::now() - chrono::Duration::hours(1);
        let entries = chain.get_entries_since(since);
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].actor_id, "user-2");
    }

    #[test]
    fn test_hash_chain_get_entries_for_actor() {
        let mut chain = AuditHashChain::new();
        chain.append_entry(make_entry("user-1", "read"));
        chain.append_entry(make_entry("user-2", "write"));
        chain.append_entry(make_entry("user-1", "delete"));

        let entries = chain.get_entries_for_actor("user-1");
        assert_eq!(entries.len(), 2);
    }

    #[test]
    fn test_gdpr_audit_trail() {
        let mut trail = GdprAuditTrail::new();
        let entry = make_entry("user-1", "access_pii");
        trail.record_data_access(entry.clone());
        trail.record_consent(entry);

        assert_eq!(trail.data_access_events.len(), 1);
        assert_eq!(trail.consent_events.len(), 1);

        let accessed = trail.who_accessed("r-1", Utc::now() - chrono::Duration::hours(1));
        assert_eq!(accessed.len(), 1);
    }

    #[test]
    fn test_gdpr_subject_access_request() {
        let mut trail = GdprAuditTrail::new();
        trail.record_data_access(make_entry("user-1", "read"));
        trail.record_data_access(make_entry("user-1", "export"));
        trail.record_data_access(make_entry("user-2", "read"));

        let sar = trail.subject_access_request("user-1", Utc::now() - chrono::Duration::hours(1));
        assert_eq!(sar.len(), 2);
    }

    #[test]
    fn test_soc2_control_evidence() {
        let mut evidence = Soc2ControlEvidence::new(
            "CC6.1",
            "Logical Access Controls",
            Soc2Category::Security,
        );
        evidence.add_evidence(EvidenceItem::new(
            EvidenceType::AuditLog,
            "Access log review",
            "All access logs reviewed and approved",
            "auditor-1",
        ));
        evidence.mark_verified();

        assert_eq!(evidence.status, ComplianceCheckStatus::Verified);
        assert!(evidence.last_verified.is_some());
        assert_eq!(evidence.evidence_items.len(), 1);
    }

    #[test]
    fn test_hipaa_access_log() {
        let mut log = HipaaAccessLog::new();
        let entry = make_entry("doctor-1", "view_patient_record");
        log.record_phi_access(entry);

        let violations = log.phi_access_by_user(
            "doctor-1",
            Utc::now() - chrono::Duration::hours(1),
        );
        assert_eq!(violations.len(), 1);
    }

    #[test]
    fn test_hipaa_violation() {
        let mut log = HipaaAccessLog::new();
        let mut violation = MinimumNecessaryViolation::new("user-1", "full-db-dump");
        violation.severity = ViolationSeverity::High;
        log.record_violation(violation);

        let violations =
            log.violations_since(Utc::now() - chrono::Duration::hours(1));
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].severity, ViolationSeverity::High);
    }

    #[test]
    fn test_retention_policy() {
        let policy =
            RetentionPolicy::new("test", DataClassification::Internal, 365).with_archive_after(90);
        assert!(!policy.should_archive(89));
        assert!(policy.should_archive(90));
        assert!(!policy.should_delete(364));
    }

    #[test]
    fn test_tamper_evident_log_record_and_verify() {
        let log = TamperEvidentAuditLog::new();
        log.record_event(make_entry("user-1", "create"));
        log.record_event(make_entry("user-2", "delete"));

        let result = log.verify_integrity();
        assert!(result.valid);
        assert_eq!(result.chain_length, 2);
    }

    #[test]
    fn test_tamper_evident_log_export() {
        let log = TamperEvidentAuditLog::new();
        log.record_event(make_entry("user-1", "read"));

        let export = log.export_logs(ExportFormat::Json, None, None);
        assert_eq!(export.total, 1);
        assert!(export.chain_integrity.unwrap().valid);
    }

    #[test]
    fn test_tamper_evident_log_export_with_time_range() {
        let log = TamperEvidentAuditLog::new();
        log.record_event(make_entry("user-1", "read"));

        let export = log.export_logs(
            ExportFormat::Csv,
            Some(Utc::now() - chrono::Duration::hours(1)),
            Some(Utc::now() + chrono::Duration::hours(1)),
        );
        assert_eq!(export.total, 1);
    }

    #[test]
    fn test_compliance_report_generation() {
        let log = TamperEvidentAuditLog::new();
        log.record_event(make_entry("user-1", "read"));
        log.record_event(make_entry("user-2", "write"));

        let report = log.generate_compliance_report(
            ComplianceFramework::Soc2,
            Utc::now() - chrono::Duration::hours(1),
            Utc::now() + chrono::Duration::hours(1),
            "admin-1",
        );

        assert_eq!(report.total_events_audited, 2);
        assert!(report.integrity_check_passed);
        assert_eq!(report.framework, ComplianceFramework::Soc2);
    }

    #[test]
    fn test_compliance_framework_display() {
        assert_eq!(ComplianceFramework::Gdpr.display_name(), "GDPR");
        assert_eq!(ComplianceFramework::Soc2.display_name(), "SOC 2");
        assert_eq!(ComplianceFramework::Hipaa.display_name(), "HIPAA");
    }

    #[test]
    fn test_data_classification_display() {
        assert_eq!(DataClassification::Public.display_name(), "Public");
        assert_eq!(DataClassification::Pii.display_name(), "PII");
        assert_eq!(DataClassification::Phi.display_name(), "PHI");
    }

    #[test]
    fn test_default_retention_policies() {
        let policies = TamperEvidentAuditLog::default_policies();
        assert_eq!(policies.len(), 6);
    }

    #[test]
    fn test_add_retention_policy() {
        let log = TamperEvidentAuditLog::new();
        let policy = RetentionPolicy::new("custom", DataClassification::Public, 100);
        log.add_retention_policy(policy);
        let policies = log.retention_policies.lock();
        assert_eq!(policies.len(), 7);
    }
}
