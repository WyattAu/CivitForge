#![forbid(unsafe_code)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum VulnSeverity {
    None,
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vulnerability {
    pub id: String,
    pub package_name: String,
    pub installed_version: String,
    pub fixed_version: Option<String>,
    pub severity: VulnSeverity,
    pub title: String,
    pub description: String,
    pub references: Vec<String>,
    pub detected_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VulnScanReport {
    pub id: String,
    pub scanned_at: DateTime<Utc>,
    pub scanner_version: String,
    pub total_packages: usize,
    pub vulnerabilities: Vec<Vulnerability>,
    pub summary: VulnSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct VulnSummary {
    pub critical: usize,
    pub high: usize,
    pub medium: usize,
    pub low: usize,
}

impl VulnSummary {
    pub fn from_vulns(vulns: &[Vulnerability]) -> Self {
        let mut summary = Self {
            critical: 0,
            high: 0,
            medium: 0,
            low: 0,
        };
        for v in vulns {
            match v.severity {
                VulnSeverity::Critical => summary.critical += 1,
                VulnSeverity::High => summary.high += 1,
                VulnSeverity::Medium => summary.medium += 1,
                VulnSeverity::Low => summary.low += 1,
                VulnSeverity::None => {}
            }
        }
        summary
    }
}

pub trait VulnerabilityScanner: Send + Sync {
    fn scan(&self, packages: &[PackageInfo]) -> Result<VulnScanReport, String>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageInfo {
    pub name: String,
    pub version: String,
    pub ecosystem: String,
}

pub struct StubVulnScanner;

impl VulnerabilityScanner for StubVulnScanner {
    fn scan(&self, packages: &[PackageInfo]) -> Result<VulnScanReport, String> {
        Ok(VulnScanReport {
            id: uuid::Uuid::new_v4().to_string(),
            scanned_at: Utc::now(),
            scanner_version: "stub-0.1.0".into(),
            total_packages: packages.len(),
            vulnerabilities: Vec::new(),
            summary: VulnSummary::default(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pkg(name: &str, version: &str) -> PackageInfo {
        PackageInfo {
            name: name.into(),
            version: version.into(),
            ecosystem: "cargo".into(),
        }
    }

    fn vuln(id: &str, severity: VulnSeverity) -> Vulnerability {
        Vulnerability {
            id: id.into(),
            package_name: "test-pkg".into(),
            installed_version: "1.0.0".into(),
            fixed_version: Some("1.1.0".into()),
            severity,
            title: "Test".into(),
            description: "Test vulnerability".into(),
            references: vec!["https://example.com".into()],
            detected_at: Utc::now(),
        }
    }

    #[test]
    fn test_severity_ordering() {
        assert!(VulnSeverity::Critical > VulnSeverity::High);
        assert!(VulnSeverity::High > VulnSeverity::Medium);
        assert!(VulnSeverity::Medium > VulnSeverity::Low);
        assert!(VulnSeverity::Low > VulnSeverity::None);
    }

    #[test]
    fn test_severity_equality() {
        assert_eq!(VulnSeverity::Critical, VulnSeverity::Critical);
        assert_ne!(VulnSeverity::Critical, VulnSeverity::High);
    }

    #[test]
    fn test_summary_from_empty() {
        let summary = VulnSummary::from_vulns(&[]);
        assert_eq!(summary.critical, 0);
        assert_eq!(summary.high, 0);
        assert_eq!(summary.medium, 0);
        assert_eq!(summary.low, 0);
    }

    #[test]
    fn test_summary_from_mixed() {
        let vulns = vec![
            vuln("CVE-001", VulnSeverity::Critical),
            vuln("CVE-002", VulnSeverity::High),
            vuln("CVE-003", VulnSeverity::High),
            vuln("CVE-004", VulnSeverity::Medium),
            vuln("CVE-005", VulnSeverity::Low),
            vuln("CVE-006", VulnSeverity::None),
        ];
        let summary = VulnSummary::from_vulns(&vulns);
        assert_eq!(summary.critical, 1);
        assert_eq!(summary.high, 2);
        assert_eq!(summary.medium, 1);
        assert_eq!(summary.low, 1);
    }

    #[test]
    fn test_stub_scanner_empty() {
        let scanner = StubVulnScanner;
        let report = scanner.scan(&[]).unwrap();
        assert_eq!(report.total_packages, 0);
        assert!(report.vulnerabilities.is_empty());
    }

    #[test]
    fn test_stub_scanner_with_packages() {
        let scanner = StubVulnScanner;
        let packages = vec![pkg("serde", "1.0"), pkg("clap", "3.0")];
        let report = scanner.scan(&packages).unwrap();
        assert_eq!(report.total_packages, 2);
        assert_eq!(report.scanner_version, "stub-0.1.0");
    }

    #[test]
    fn test_stub_scanner_report_has_id() {
        let scanner = StubVulnScanner;
        let report = scanner.scan(&[]).unwrap();
        assert!(!report.id.is_empty());
    }

    #[test]
    fn test_stub_scanner_timestamp() {
        let scanner = StubVulnScanner;
        let before = Utc::now();
        let report = scanner.scan(&[]).unwrap();
        let after = Utc::now();
        assert!(report.scanned_at >= before);
        assert!(report.scanned_at <= after);
    }

    #[test]
    fn test_package_info() {
        let p = pkg("serde", "1.0.130");
        assert_eq!(p.name, "serde");
        assert_eq!(p.version, "1.0.130");
        assert_eq!(p.ecosystem, "cargo");
    }

    #[test]
    fn test_vulnerability_construction() {
        let v = vuln("CVE-2024-1234", VulnSeverity::Critical);
        assert_eq!(v.id, "CVE-2024-1234");
        assert_eq!(v.severity, VulnSeverity::Critical);
        assert!(v.fixed_version.is_some());
    }

    #[test]
    fn test_vulnerability_no_fix() {
        let v = Vulnerability {
            id: "CVE-2024-0000".into(),
            package_name: "pkg".into(),
            installed_version: "1.0.0".into(),
            fixed_version: None,
            severity: VulnSeverity::High,
            title: "Unfixed".into(),
            description: "No fix available".into(),
            references: vec![],
            detected_at: Utc::now(),
        };
        assert!(v.fixed_version.is_none());
    }

    #[test]
    fn test_vuln_severity_serialization() {
        let json = serde_json::to_string(&VulnSeverity::Critical).unwrap();
        let de: VulnSeverity = serde_json::from_str(&json).unwrap();
        assert_eq!(de, VulnSeverity::Critical);
    }

    #[test]
    fn test_vulnerability_serialization() {
        let v = vuln("CVE-2024-5678", VulnSeverity::Medium);
        let json = serde_json::to_string(&v).unwrap();
        let de: Vulnerability = serde_json::from_str(&json).unwrap();
        assert_eq!(de.id, "CVE-2024-5678");
        assert_eq!(de.severity, VulnSeverity::Medium);
    }

    #[test]
    fn test_report_serialization() {
        let scanner = StubVulnScanner;
        let report = scanner.scan(&[]).unwrap();
        let json = serde_json::to_string(&report).unwrap();
        let de: VulnScanReport = serde_json::from_str(&json).unwrap();
        assert_eq!(de.id, report.id);
        assert_eq!(de.total_packages, 0);
    }

    #[test]
    fn test_summary_serialization() {
        let summary = VulnSummary {
            critical: 3,
            high: 2,
            medium: 1,
            low: 0,
        };
        let json = serde_json::to_string(&summary).unwrap();
        let de: VulnSummary = serde_json::from_str(&json).unwrap();
        assert_eq!(de.critical, 3);
    }

    #[test]
    fn test_summary_default() {
        let summary = VulnSummary::default();
        assert_eq!(summary.critical, 0);
        assert_eq!(summary.high, 0);
    }

    #[test]
    fn test_package_info_serialization() {
        let p = pkg("tokio", "1.0");
        let json = serde_json::to_string(&p).unwrap();
        let de: PackageInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(de.name, "tokio");
    }

    #[test]
    fn test_multiple_critical_vulns() {
        let vulns = vec![
            vuln("CVE-001", VulnSeverity::Critical),
            vuln("CVE-002", VulnSeverity::Critical),
            vuln("CVE-003", VulnSeverity::Critical),
        ];
        let summary = VulnSummary::from_vulns(&vulns);
        assert_eq!(summary.critical, 3);
    }

    #[test]
    fn test_summary_total() {
        let vulns = vec![
            vuln("CVE-001", VulnSeverity::Critical),
            vuln("CVE-002", VulnSeverity::High),
            vuln("CVE-003", VulnSeverity::Medium),
            vuln("CVE-004", VulnSeverity::Low),
            vuln("CVE-005", VulnSeverity::None),
        ];
        let summary = VulnSummary::from_vulns(&vulns);
        let total = summary.critical + summary.high + summary.medium + summary.low;
        assert_eq!(total, 4);
    }

    #[test]
    fn test_stub_scanner_unique_ids() {
        let scanner = StubVulnScanner;
        let r1 = scanner.scan(&[]).unwrap();
        let r2 = scanner.scan(&[]).unwrap();
        assert_ne!(r1.id, r2.id);
    }
}
