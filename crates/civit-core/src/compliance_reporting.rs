#![forbid(unsafe_code)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceFramework {
    pub id: Uuid,
    pub name: String,
    pub version: String,
    pub framework_type: ComplianceReportFrameworkType,
    pub description: String,
    pub controls: Vec<ComplianceControl>,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ComplianceReportFrameworkType {
    Soc2TypeII,
    Gdpr,
    Iso27001,
    Hipaa,
    PciDss,
    Custom,
}

impl ComplianceReportFrameworkType {
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Soc2TypeII => "SOC 2 Type II",
            Self::Gdpr => "GDPR",
            Self::Iso27001 => "ISO 27001",
            Self::Hipaa => "HIPAA",
            Self::PciDss => "PCI DSS",
            Self::Custom => "Custom",
        }
    }

    pub fn default_controls(&self) -> Vec<ComplianceControl> {
        match self {
            Self::Soc2TypeII => vec![
                ComplianceControl {
                    id: "CC6.1".into(),
                    name: "Logical Access Security".into(),
                    description: "Logical and physical access controls".into(),
                    status: ControlStatus::Passing,
                    last_evidence_collected: None,
                },
                ComplianceControl {
                    id: "CC7.1".into(),
                    name: "System Monitoring".into(),
                    description: "Monitoring of system components".into(),
                    status: ControlStatus::Passing,
                    last_evidence_collected: None,
                },
                ComplianceControl {
                    id: "CC8.1".into(),
                    name: "Change Management".into(),
                    description: "Configuration change controls".into(),
                    status: ControlStatus::Passing,
                    last_evidence_collected: None,
                },
            ],
            Self::Gdpr => vec![
                ComplianceControl {
                    id: "Art.5".into(),
                    name: "Principles of Processing".into(),
                    description: "Lawfulness, fairness and transparency".into(),
                    status: ControlStatus::Passing,
                    last_evidence_collected: None,
                },
                ComplianceControl {
                    id: "Art.17".into(),
                    name: "Right to Erasure".into(),
                    description: "Right to be forgotten".into(),
                    status: ControlStatus::Passing,
                    last_evidence_collected: None,
                },
                ComplianceControl {
                    id: "Art.32".into(),
                    name: "Security of Processing".into(),
                    description: "Appropriate technical measures".into(),
                    status: ControlStatus::Passing,
                    last_evidence_collected: None,
                },
            ],
            Self::Iso27001 => vec![
                ComplianceControl {
                    id: "A.8".into(),
                    name: "Asset Management".into(),
                    description: "Information asset management".into(),
                    status: ControlStatus::Passing,
                    last_evidence_collected: None,
                },
                ComplianceControl {
                    id: "A.12".into(),
                    name: "Operations Security".into(),
                    description: "Operational procedures and responsibilities".into(),
                    status: ControlStatus::Passing,
                    last_evidence_collected: None,
                },
                ComplianceControl {
                    id: "A.14".into(),
                    name: "System Acquisition & Maintenance".into(),
                    description: "Secure development and support processes".into(),
                    status: ControlStatus::Passing,
                    last_evidence_collected: None,
                },
            ],
            Self::Hipaa => vec![
                ComplianceControl {
                    id: "164.312".into(),
                    name: "Technical Safeguards".into(),
                    description: "Access control, audit controls, integrity".into(),
                    status: ControlStatus::Passing,
                    last_evidence_collected: None,
                },
                ComplianceControl {
                    id: "164.316".into(),
                    name: "Policies and Procedures".into(),
                    description: "Implement policies and procedures".into(),
                    status: ControlStatus::Passing,
                    last_evidence_collected: None,
                },
            ],
            Self::PciDss => vec![
                ComplianceControl {
                    id: "Req1".into(),
                    name: "Network Security".into(),
                    description: "Install and maintain network security controls".into(),
                    status: ControlStatus::Passing,
                    last_evidence_collected: None,
                },
                ComplianceControl {
                    id: "Req3".into(),
                    name: "Protect Stored Data".into(),
                    description: "Protect stored account data".into(),
                    status: ControlStatus::Passing,
                    last_evidence_collected: None,
                },
            ],
            Self::Custom => vec![],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceControl {
    pub id: String,
    pub name: String,
    pub description: String,
    pub status: ControlStatus,
    pub last_evidence_collected: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ControlStatus {
    Passing,
    Failing,
    PartiallyPassing,
    NotAssessed,
    NotApplicable,
}

impl ControlStatus {
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Passing => "Passing",
            Self::Failing => "Failing",
            Self::PartiallyPassing => "Partially Passing",
            Self::NotAssessed => "Not Assessed",
            Self::NotApplicable => "Not Applicable",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceReport {
    pub id: Uuid,
    pub framework: ComplianceReportFrameworkType,
    pub report_name: String,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub status: ReportStatus,
    pub controls_summary: ControlsSummary,
    pub findings: Vec<ComplianceReportFinding>,
    pub evidence_items: Vec<EvidenceItem>,
    pub overall_score: f64,
    pub generated_at: DateTime<Utc>,
    pub distributed: bool,
    pub distribution_method: Option<DistributionMethod>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReportStatus {
    Generating,
    Completed,
    Failed,
    PendingReview,
    Distributed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControlsSummary {
    pub total: u32,
    pub passing: u32,
    pub failing: u32,
    pub partially_passing: u32,
    pub not_assessed: u32,
    pub compliance_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceReportFinding {
    pub id: Uuid,
    pub control_id: String,
    pub title: String,
    pub severity: FindingSeverity,
    pub description: String,
    pub remediation: String,
    pub status: FindingStatus,
    pub identified_at: DateTime<Utc>,
    pub resolved_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FindingSeverity {
    Critical,
    High,
    Medium,
    Low,
    Informational,
}

impl FindingSeverity {
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Critical => "Critical",
            Self::High => "High",
            Self::Medium => "Medium",
            Self::Low => "Low",
            Self::Informational => "Informational",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FindingStatus {
    Open,
    InRemediation,
    Mitigated,
    Closed,
    Accepted,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceItem {
    pub id: Uuid,
    pub control_id: String,
    pub evidence_type: EvidenceType,
    pub description: String,
    pub collected_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub storage_path: Option<String>,
    pub integrity_hash: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceType {
    AuditLog,
    Configuration,
    Screenshot,
    Certificate,
    Policy,
    ScanResult,
    TestResult,
    Manual,
}

impl EvidenceType {
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::AuditLog => "Audit Log",
            Self::Configuration => "Configuration",
            Self::Screenshot => "Screenshot",
            Self::Certificate => "Certificate",
            Self::Policy => "Policy",
            Self::ScanResult => "Scan Result",
            Self::TestResult => "Test Result",
            Self::Manual => "Manual",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DistributionMethod {
    Email,
    Download,
    Both,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportSchedule {
    pub id: Uuid,
    pub framework: ComplianceReportFrameworkType,
    pub frequency: ReportFrequency,
    pub day_of_week: Option<u8>,
    pub day_of_month: Option<u8>,
    pub recipients: Vec<String>,
    pub distribution_method: DistributionMethod,
    pub enabled: bool,
    pub last_run: Option<DateTime<Utc>>,
    pub next_run: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReportFrequency {
    Weekly,
    Monthly,
    Quarterly,
    Annually,
}

impl ReportFrequency {
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Weekly => "Weekly",
            Self::Monthly => "Monthly",
            Self::Quarterly => "Quarterly",
            Self::Annually => "Annually",
        }
    }
}

pub fn supported_frameworks() -> Vec<ComplianceFramework> {
    vec![
        ComplianceFramework {
            id: Uuid::new_v4(),
            name: "SOC 2 Type II".into(),
            version: "2017".into(),
            framework_type: ComplianceReportFrameworkType::Soc2TypeII,
            description: "Service Organization Control 2 Type II".into(),
            controls: ComplianceReportFrameworkType::Soc2TypeII.default_controls(),
            enabled: true,
            created_at: Utc::now(),
        },
        ComplianceFramework {
            id: Uuid::new_v4(),
            name: "GDPR".into(),
            version: "2016/679".into(),
            framework_type: ComplianceReportFrameworkType::Gdpr,
            description: "General Data Protection Regulation".into(),
            controls: ComplianceReportFrameworkType::Gdpr.default_controls(),
            enabled: true,
            created_at: Utc::now(),
        },
        ComplianceFramework {
            id: Uuid::new_v4(),
            name: "ISO 27001".into(),
            version: "2022".into(),
            framework_type: ComplianceReportFrameworkType::Iso27001,
            description: "Information Security Management System".into(),
            controls: ComplianceReportFrameworkType::Iso27001.default_controls(),
            enabled: true,
            created_at: Utc::now(),
        },
        ComplianceFramework {
            id: Uuid::new_v4(),
            name: "HIPAA".into(),
            version: "2013".into(),
            framework_type: ComplianceReportFrameworkType::Hipaa,
            description: "Health Insurance Portability and Accountability Act".into(),
            controls: ComplianceReportFrameworkType::Hipaa.default_controls(),
            enabled: false,
            created_at: Utc::now(),
        },
        ComplianceFramework {
            id: Uuid::new_v4(),
            name: "PCI DSS".into(),
            version: "4.0".into(),
            framework_type: ComplianceReportFrameworkType::PciDss,
            description: "Payment Card Industry Data Security Standard".into(),
            controls: ComplianceReportFrameworkType::PciDss.default_controls(),
            enabled: false,
            created_at: Utc::now(),
        },
    ]
}

pub fn generate_soc2_evidence() -> Vec<EvidenceItem> {
    let now = Utc::now();
    vec![
        EvidenceItem {
            id: Uuid::new_v4(),
            control_id: "CC6.1".into(),
            evidence_type: EvidenceType::AuditLog,
            description: "Access control audit logs for the reporting period".into(),
            collected_at: now,
            expires_at: Some(now + chrono::Duration::days(365 * 7)),
            storage_path: Some("/evidence/soc2/access-logs/".into()),
            integrity_hash: None,
        },
        EvidenceItem {
            id: Uuid::new_v4(),
            control_id: "CC7.1".into(),
            evidence_type: EvidenceType::ScanResult,
            description: "System monitoring and alerting configuration".into(),
            collected_at: now,
            expires_at: Some(now + chrono::Duration::days(365 * 7)),
            storage_path: Some("/evidence/soc2/monitoring-config/".into()),
            integrity_hash: None,
        },
        EvidenceItem {
            id: Uuid::new_v4(),
            control_id: "CC8.1".into(),
            evidence_type: EvidenceType::Configuration,
            description: "Change management audit trail".into(),
            collected_at: now,
            expires_at: Some(now + chrono::Duration::days(365 * 7)),
            storage_path: Some("/evidence/soc2/change-log/".into()),
            integrity_hash: None,
        },
    ]
}

pub fn generate_gdpr_evidence() -> Vec<EvidenceItem> {
    let now = Utc::now();
    vec![
        EvidenceItem {
            id: Uuid::new_v4(),
            control_id: "Art.5".into(),
            evidence_type: EvidenceType::Policy,
            description: "Data processing policy documentation".into(),
            collected_at: now,
            expires_at: Some(now + chrono::Duration::days(365 * 3)),
            storage_path: Some("/evidence/gdpr/policies/".into()),
            integrity_hash: None,
        },
        EvidenceItem {
            id: Uuid::new_v4(),
            control_id: "Art.17".into(),
            evidence_type: EvidenceType::TestResult,
            description: "User data deletion verification test results".into(),
            collected_at: now,
            expires_at: Some(now + chrono::Duration::days(365 * 3)),
            storage_path: Some("/evidence/gdpr/deletion-tests/".into()),
            integrity_hash: None,
        },
        EvidenceItem {
            id: Uuid::new_v4(),
            control_id: "Art.32".into(),
            evidence_type: EvidenceType::Certificate,
            description: "Encryption certificates and key management records".into(),
            collected_at: now,
            expires_at: Some(now + chrono::Duration::days(365 * 3)),
            storage_path: Some("/evidence/gdpr/encryption/".into()),
            integrity_hash: None,
        },
    ]
}

pub fn generate_iso27001_evidence() -> Vec<EvidenceItem> {
    let now = Utc::now();
    vec![
        EvidenceItem {
            id: Uuid::new_v4(),
            control_id: "A.8".into(),
            evidence_type: EvidenceType::Configuration,
            description: "Asset inventory and classification records".into(),
            collected_at: now,
            expires_at: Some(now + chrono::Duration::days(365 * 3)),
            storage_path: Some("/evidence/iso27001/assets/".into()),
            integrity_hash: None,
        },
        EvidenceItem {
            id: Uuid::new_v4(),
            control_id: "A.12".into(),
            evidence_type: EvidenceType::AuditLog,
            description: "Operations audit trail and change logs".into(),
            collected_at: now,
            expires_at: Some(now + chrono::Duration::days(365 * 3)),
            storage_path: Some("/evidence/iso27001/operations/".into()),
            integrity_hash: None,
        },
        EvidenceItem {
            id: Uuid::new_v4(),
            control_id: "A.14".into(),
            evidence_type: EvidenceType::TestResult,
            description: "Secure development lifecycle test results".into(),
            collected_at: now,
            expires_at: Some(now + chrono::Duration::days(365 * 3)),
            storage_path: Some("/evidence/iso27001/sdlc/".into()),
            integrity_hash: None,
        },
    ]
}

pub fn collect_evidence_for_framework(
    framework: &ComplianceReportFrameworkType,
) -> Vec<EvidenceItem> {
    match framework {
        ComplianceReportFrameworkType::Soc2TypeII => generate_soc2_evidence(),
        ComplianceReportFrameworkType::Gdpr => generate_gdpr_evidence(),
        ComplianceReportFrameworkType::Iso27001 => generate_iso27001_evidence(),
        _ => vec![],
    }
}

pub fn compute_controls_summary(controls: &[ComplianceControl]) -> ControlsSummary {
    let total = controls.len() as u32;
    let passing = controls.iter().filter(|c| c.status == ControlStatus::Passing).count() as u32;
    let failing = controls.iter().filter(|c| c.status == ControlStatus::Failing).count() as u32;
    let partially = controls
        .iter()
        .filter(|c| c.status == ControlStatus::PartiallyPassing)
        .count() as u32;
    let not_assessed = controls
        .iter()
        .filter(|c| c.status == ControlStatus::NotAssessed)
        .count() as u32;

    let assessed = total - not_assessed;
    let compliance_score = if assessed > 0 {
        (passing as f64 / assessed as f64) * 100.0
    } else {
        0.0
    };

    ControlsSummary {
        total,
        passing,
        failing,
        partially_passing: partially,
        not_assessed,
        compliance_score,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_supported_frameworks_count() {
        let frameworks = supported_frameworks();
        assert_eq!(frameworks.len(), 5);
    }

    #[test]
    fn test_framework_type_display() {
        assert_eq!(
            ComplianceReportFrameworkType::Soc2TypeII.display_name(),
            "SOC 2 Type II"
        );
        assert_eq!(ComplianceReportFrameworkType::Gdpr.display_name(), "GDPR");
        assert_eq!(
            ComplianceReportFrameworkType::Iso27001.display_name(),
            "ISO 27001"
        );
    }

    #[test]
    fn test_control_status_display() {
        assert_eq!(ControlStatus::Passing.display_name(), "Passing");
        assert_eq!(ControlStatus::Failing.display_name(), "Failing");
    }

    #[test]
    fn test_finding_severity_display() {
        assert_eq!(FindingSeverity::Critical.display_name(), "Critical");
        assert_eq!(
            FindingSeverity::Informational.display_name(),
            "Informational"
        );
    }

    #[test]
    fn test_evidence_type_display() {
        assert_eq!(EvidenceType::AuditLog.display_name(), "Audit Log");
        assert_eq!(EvidenceType::ScanResult.display_name(), "Scan Result");
    }

    #[test]
    fn test_report_frequency_display() {
        assert_eq!(ReportFrequency::Quarterly.display_name(), "Quarterly");
        assert_eq!(ReportFrequency::Annually.display_name(), "Annually");
    }

    #[test]
    fn test_default_controls_soc2() {
        let controls = ComplianceReportFrameworkType::Soc2TypeII.default_controls();
        assert_eq!(controls.len(), 3);
        assert_eq!(controls[0].id, "CC6.1");
    }

    #[test]
    fn test_default_controls_gdpr() {
        let controls = ComplianceReportFrameworkType::Gdpr.default_controls();
        assert_eq!(controls.len(), 3);
        assert_eq!(controls[0].id, "Art.5");
    }

    #[test]
    fn test_default_controls_iso27001() {
        let controls = ComplianceReportFrameworkType::Iso27001.default_controls();
        assert_eq!(controls.len(), 3);
        assert_eq!(controls[0].id, "A.8");
    }

    #[test]
    fn test_collect_evidence_soc2() {
        let evidence = collect_evidence_for_framework(&ComplianceReportFrameworkType::Soc2TypeII);
        assert_eq!(evidence.len(), 3);
        assert_eq!(evidence[0].control_id, "CC6.1");
    }

    #[test]
    fn test_collect_evidence_gdpr() {
        let evidence = collect_evidence_for_framework(&ComplianceReportFrameworkType::Gdpr);
        assert_eq!(evidence.len(), 3);
        assert_eq!(evidence[0].control_id, "Art.5");
    }

    #[test]
    fn test_compute_controls_summary_all_passing() {
        let controls = vec![
            ComplianceControl {
                id: "A".into(),
                name: "Test A".into(),
                description: "Desc A".into(),
                status: ControlStatus::Passing,
                last_evidence_collected: None,
            },
            ComplianceControl {
                id: "B".into(),
                name: "Test B".into(),
                description: "Desc B".into(),
                status: ControlStatus::Passing,
                last_evidence_collected: None,
            },
        ];
        let summary = compute_controls_summary(&controls);
        assert_eq!(summary.total, 2);
        assert_eq!(summary.passing, 2);
        assert_eq!(summary.compliance_score, 100.0);
    }

    #[test]
    fn test_compute_controls_summary_mixed() {
        let controls = vec![
            ComplianceControl {
                id: "A".into(),
                name: "Test A".into(),
                description: "Desc".into(),
                status: ControlStatus::Passing,
                last_evidence_collected: None,
            },
            ComplianceControl {
                id: "B".into(),
                name: "Test B".into(),
                description: "Desc".into(),
                status: ControlStatus::Failing,
                last_evidence_collected: None,
            },
        ];
        let summary = compute_controls_summary(&controls);
        assert_eq!(summary.total, 2);
        assert_eq!(summary.passing, 1);
        assert_eq!(summary.failing, 1);
        assert_eq!(summary.compliance_score, 50.0);
    }

    #[test]
    fn test_compute_controls_summary_with_not_assessed() {
        let controls = vec![
            ComplianceControl {
                id: "A".into(),
                name: "A".into(),
                description: "Desc".into(),
                status: ControlStatus::Passing,
                last_evidence_collected: None,
            },
            ComplianceControl {
                id: "B".into(),
                name: "B".into(),
                description: "Desc".into(),
                status: ControlStatus::NotAssessed,
                last_evidence_collected: None,
            },
        ];
        let summary = compute_controls_summary(&controls);
        assert_eq!(summary.not_assessed, 1);
        assert_eq!(summary.compliance_score, 100.0);
    }
}
