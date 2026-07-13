#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Severity level of a vulnerability.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VulnSeverity {
    Critical,
    High,
    Medium,
    Low,
    Unknown,
}

impl VulnSeverity {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Critical => "critical",
            Self::High => "high",
            Self::Medium => "medium",
            Self::Low => "low",
            Self::Unknown => "unknown",
        }
    }
}

/// A single vulnerability finding.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VulnFinding {
    pub id: String,
    pub severity: VulnSeverity,
    pub package: String,
    pub installed_version: String,
    pub fixed_version: Option<String>,
    pub description: String,
    pub references: Vec<String>,
}

/// Scan status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScanStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
}

/// Result of a vulnerability scan.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VulnScanResult {
    pub manifest_digest: String,
    pub scanner: String,
    pub status: ScanStatus,
    pub total_vulns: usize,
    pub critical_count: usize,
    pub high_count: usize,
    pub medium_count: usize,
    pub low_count: usize,
    pub findings: Vec<VulnFinding>,
    pub scanned_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl VulnScanResult {
    pub fn new(manifest_digest: impl Into<String>) -> Self {
        Self {
            manifest_digest: manifest_digest.into(),
            scanner: "osv".to_string(),
            status: ScanStatus::Pending,
            total_vulns: 0,
            critical_count: 0,
            high_count: 0,
            medium_count: 0,
            low_count: 0,
            findings: Vec::new(),
            scanned_at: None,
        }
    }

    pub fn with_scanner(mut self, scanner: impl Into<String>) -> Self {
        self.scanner = scanner.into();
        self
    }

    /// Compute severity counts from findings.
    pub fn compute_counts(&mut self) {
        self.critical_count = self.findings.iter().filter(|f| f.severity == VulnSeverity::Critical).count();
        self.high_count = self.findings.iter().filter(|f| f.severity == VulnSeverity::High).count();
        self.medium_count = self.findings.iter().filter(|f| f.severity == VulnSeverity::Medium).count();
        self.low_count = self.findings.iter().filter(|f| f.severity == VulnSeverity::Low).count();
        self.total_vulns = self.findings.len();
    }

    pub fn mark_completed(&mut self) {
        self.status = ScanStatus::Completed;
        self.scanned_at = Some(chrono::Utc::now());
        self.compute_counts();
    }

    pub fn mark_failed(&mut self) {
        self.status = ScanStatus::Failed;
        self.scanned_at = Some(chrono::Utc::now());
    }

    /// Check if the scan has any critical or high vulnerabilities.
    pub fn has_critical_or_high(&self) -> bool {
        self.critical_count > 0 || self.high_count > 0
    }
}

/// Manages vulnerability scans for container images.
pub struct VulnScanner {
    scans: dashmap::DashMap<String, VulnScanResult>,
}

impl Default for VulnScanner {
    fn default() -> Self {
        Self::new()
    }
}

impl VulnScanner {
    pub fn new() -> Self {
        Self {
            scans: dashmap::DashMap::new(),
        }
    }

    /// Start a new scan for a manifest digest.
    pub fn start_scan(&self, manifest_digest: &str) -> VulnScanResult {
        let result = VulnScanResult::new(manifest_digest);
        self.scans
            .insert(manifest_digest.to_string(), result.clone());
        result
    }

    /// Get scan result by manifest digest.
    pub fn get_scan(&self, manifest_digest: &str) -> Option<VulnScanResult> {
        self.scans.get(manifest_digest).map(|r| r.value().clone())
    }

    /// Update scan with findings.
    pub fn update_findings(
        &self,
        manifest_digest: &str,
        findings: Vec<VulnFinding>,
    ) -> Option<VulnScanResult> {
        let mut scan = self.scans.get_mut(manifest_digest)?;
        scan.findings = findings;
        scan.mark_completed();
        Some(scan.value().clone())
    }

    /// Mark a scan as failed.
    pub fn mark_failed(&self, manifest_digest: &str) -> Option<VulnScanResult> {
        let mut scan = self.scans.get_mut(manifest_digest)?;
        scan.mark_failed();
        Some(scan.value().clone())
    }

    /// List all scans.
    pub fn list_scans(&self) -> Vec<VulnScanResult> {
        self.scans.iter().map(|r| r.value().clone()).collect()
    }

    /// Get count of scans by status.
    pub fn scan_counts(&self) -> HashMap<String, usize> {
        let mut counts = HashMap::new();
        for scan in self.scans.iter() {
            let status = format!("{:?}", scan.value().status);
            *counts.entry(status).or_insert(0) += 1;
        }
        counts
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_finding(severity: VulnSeverity) -> VulnFinding {
        VulnFinding {
            id: "CVE-2024-0001".to_string(),
            severity,
            package: "openssl".to_string(),
            installed_version: "1.1.1".to_string(),
            fixed_version: Some("1.1.2".to_string()),
            description: "Test vuln".to_string(),
            references: vec![],
        }
    }

    #[test]
    fn test_scan_result_new() {
        let r = VulnScanResult::new("sha256:abc");
        assert_eq!(r.manifest_digest, "sha256:abc");
        assert_eq!(r.status, ScanStatus::Pending);
        assert_eq!(r.total_vulns, 0);
    }

    #[test]
    fn test_scan_result_compute_counts() {
        let mut r = VulnScanResult::new("d");
        r.findings = vec![
            make_finding(VulnSeverity::Critical),
            make_finding(VulnSeverity::Critical),
            make_finding(VulnSeverity::High),
            make_finding(VulnSeverity::Low),
        ];
        r.compute_counts();
        assert_eq!(r.critical_count, 2);
        assert_eq!(r.high_count, 1);
        assert_eq!(r.low_count, 1);
        assert_eq!(r.total_vulns, 4);
    }

    #[test]
    fn test_scan_result_has_critical_or_high() {
        let mut r = VulnScanResult::new("d");
        r.findings = vec![make_finding(VulnSeverity::Medium)];
        r.compute_counts();
        assert!(!r.has_critical_or_high());

        r.findings.push(make_finding(VulnSeverity::High));
        r.compute_counts();
        assert!(r.has_critical_or_high());
    }

    #[test]
    fn test_scanner_start_and_get() {
        let scanner = VulnScanner::new();
        scanner.start_scan("sha256:abc");
        assert!(scanner.get_scan("sha256:abc").is_some());
        assert!(scanner.get_scan("sha256:other").is_none());
    }

    #[test]
    fn test_scanner_update_findings() {
        let scanner = VulnScanner::new();
        scanner.start_scan("sha256:abc");
        let result = scanner
            .update_findings("sha256:abc", vec![make_finding(VulnSeverity::High)])
            .unwrap();
        assert_eq!(result.status, ScanStatus::Completed);
        assert_eq!(result.high_count, 1);
    }

    #[test]
    fn test_scanner_mark_failed() {
        let scanner = VulnScanner::new();
        scanner.start_scan("sha256:abc");
        let result = scanner.mark_failed("sha256:abc").unwrap();
        assert_eq!(result.status, ScanStatus::Failed);
    }

    #[test]
    fn test_scanner_scan_counts() {
        let scanner = VulnScanner::new();
        scanner.start_scan("d1");
        scanner.start_scan("d2");
        let counts = scanner.scan_counts();
        assert_eq!(counts.get("Pending"), Some(&2));
    }

    #[test]
    fn test_vuln_severity_as_str() {
        assert_eq!(VulnSeverity::Critical.as_str(), "critical");
        assert_eq!(VulnSeverity::High.as_str(), "high");
        assert_eq!(VulnSeverity::Medium.as_str(), "medium");
        assert_eq!(VulnSeverity::Low.as_str(), "low");
    }

    #[test]
    fn test_serialization_roundtrip() {
        let mut r = VulnScanResult::new("d");
        r.findings = vec![make_finding(VulnSeverity::Critical)];
        r.mark_completed();
        let json = serde_json::to_string(&r).unwrap();
        let de: VulnScanResult = serde_json::from_str(&json).unwrap();
        assert_eq!(de.critical_count, 1);
    }
}
