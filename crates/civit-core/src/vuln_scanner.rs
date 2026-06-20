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

#[cfg(test)]
pub struct StubVulnScanner;

#[cfg(test)]
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

const OSV_API_URL: &str = "https://api.osv.dev/v1/query";

#[derive(Debug, Serialize)]
struct OsvRequest {
    package: OsvPackage,
    version: String,
}

#[derive(Debug, Serialize)]
struct OsvPackage {
    name: String,
    ecosystem: String,
}

#[derive(Debug, Deserialize)]
struct OsvResponse {
    vulns: Vec<OsvVuln>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct OsvVuln {
    id: String,
    #[serde(default)]
    summary: Option<String>,
    #[serde(default)]
    details: Option<String>,
    #[serde(default)]
    aliases: Vec<String>,
    #[serde(default)]
    severity: Vec<OsvSeverityEntry>,
    #[serde(default)]
    references: Vec<OsvReference>,
}

#[derive(Debug, Deserialize)]
struct OsvSeverityEntry {
    score: Option<String>,
    #[serde(rename = "type")]
    severity_type: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OsvReference {
    url: Option<String>,
}

pub struct OsvVulnScanner {
    client: reqwest::Client,
    scanner_version: String,
}

impl Default for OsvVulnScanner {
    fn default() -> Self {
        Self::new()
    }
}

impl OsvVulnScanner {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
            scanner_version: "osv-0.1.0".into(),
        }
    }

    pub fn with_client(client: reqwest::Client) -> Self {
        Self {
            client,
            scanner_version: "osv-0.1.0".into(),
        }
    }

    pub fn with_version(mut self, version: &str) -> Self {
        self.scanner_version = version.into();
        self
    }

    fn parse_cvss_score(score_str: &str) -> Option<f64> {
        let score_str = score_str.trim();
        if score_str.starts_with("CVSS:") {
            let parts: Vec<&str> = score_str.split('/').collect();
            if let Some(last) = parts.last()
                && let Some((_, val)) = last.split_once(':')
            {
                return val.parse().ok();
            }
            None
        } else {
            score_str.parse().ok()
        }
    }

    fn classify_severity(score: Option<f64>) -> VulnSeverity {
        match score {
            Some(s) if s >= 9.0 => VulnSeverity::Critical,
            Some(s) if s >= 7.0 => VulnSeverity::High,
            Some(s) if s >= 4.0 => VulnSeverity::Medium,
            Some(_) => VulnSeverity::Low,
            None => VulnSeverity::Low,
        }
    }

    fn extract_max_severity(entries: &[OsvSeverityEntry]) -> VulnSeverity {
        let mut max_score: Option<f64> = None;
        for entry in entries {
            if entry.severity_type.as_deref() == Some("CVSS_V3")
                && let Some(score_str) = &entry.score
                && let Some(score) = Self::parse_cvss_score(score_str)
            {
                max_score = Some(max_score.map_or(score, |m| m.max(score)));
            }
        }
        Self::classify_severity(max_score)
    }

    fn parse_response_vulns(
        response: &OsvResponse,
        package_name: &str,
        package_version: &str,
    ) -> Vec<Vulnerability> {
        let now = Utc::now();
        response
            .vulns
            .iter()
            .map(|v| Vulnerability {
                id: v.id.clone(),
                package_name: package_name.into(),
                installed_version: package_version.into(),
                fixed_version: None,
                severity: Self::extract_max_severity(&v.severity),
                title: v.summary.clone().unwrap_or_else(|| v.id.clone()),
                description: v.details.clone().unwrap_or_default(),
                references: v.references.iter().filter_map(|r| r.url.clone()).collect(),
                detected_at: now,
            })
            .collect()
    }

    async fn query_package(&self, package: &PackageInfo) -> Result<Vec<Vulnerability>, String> {
        let request_body = OsvRequest {
            package: OsvPackage {
                name: package.name.clone(),
                ecosystem: package.ecosystem.clone(),
            },
            version: package.version.clone(),
        };

        let response = self
            .client
            .post(OSV_API_URL)
            .json(&request_body)
            .send()
            .await
            .map_err(|e| format!("OSV API request failed for {}: {}", package.name, e))?;

        if !response.status().is_success() {
            return Err(format!(
                "OSV API returned {} for package {}",
                response.status(),
                package.name
            ));
        }

        let osv_response: OsvResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse OSV response for {}: {}", package.name, e))?;

        Ok(Self::parse_response_vulns(
            &osv_response,
            &package.name,
            &package.version,
        ))
    }
}

impl VulnerabilityScanner for OsvVulnScanner {
    fn scan(&self, packages: &[PackageInfo]) -> Result<VulnScanReport, String> {
        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| format!("Failed to create tokio runtime: {e}"))?;
        rt.block_on(self.scan_async(packages))
    }
}

impl OsvVulnScanner {
    async fn scan_async(&self, packages: &[PackageInfo]) -> Result<VulnScanReport, String> {
        let mut all_vulns = Vec::new();

        for package in packages {
            match self.query_package(package).await {
                Ok(vulns) => all_vulns.extend(vulns),
                Err(e) => {
                    tracing::warn!("Skipping package {}: {}", package.name, e);
                }
            }
        }

        let summary = VulnSummary::from_vulns(&all_vulns);

        Ok(VulnScanReport {
            id: uuid::Uuid::new_v4().to_string(),
            scanned_at: Utc::now(),
            scanner_version: self.scanner_version.clone(),
            total_packages: packages.len(),
            vulnerabilities: all_vulns,
            summary,
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

    #[test]
    fn test_osv_scanner_empty_packages() {
        let scanner = OsvVulnScanner::new();
        let report = scanner.scan(&[]).unwrap();
        assert_eq!(report.total_packages, 0);
        assert!(report.vulnerabilities.is_empty());
        assert_eq!(report.summary.critical, 0);
        assert_eq!(report.summary.high, 0);
        assert_eq!(report.summary.medium, 0);
        assert_eq!(report.summary.low, 0);
        assert!(!report.id.is_empty());
        assert_eq!(report.scanner_version, "osv-0.1.0");
    }

    #[test]
    fn test_osv_scanner_parse_vuln_response() {
        let json = r#"{
            "vulns": [
                {
                    "id": "CVE-2024-1234",
                    "summary": "Buffer overflow in foo",
                    "details": "A buffer overflow vulnerability exists...",
                    "aliases": ["GHSA-xxxx-yyyy-zzzz"],
                    "severity": [
                        {"score": "CVSS:3.1/AV:N/AC:L/PR:N/UI:N/S:U/C:H/I:H/A:H", "type": "CVSS_V3"}
                    ],
                    "references": [
                        {"url": "https://github.com/example/foo/issues/1"}
                    ]
                },
                {
                    "id": "CVE-2024-5678",
                    "summary": "XSS in bar",
                    "details": "A cross-site scripting vulnerability...",
                    "aliases": [],
                    "severity": [
                        {"score": "CVSS:3.1/AV:N/AC:L/PR:N/UI:R/S:C/C:L/I:L/A:N", "type": "CVSS_V3"}
                    ],
                    "references": []
                }
            ]
        }"#;

        let response: OsvResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.vulns.len(), 2);

        let vulns = OsvVulnScanner::parse_response_vulns(&response, "test-pkg", "1.0.0");
        assert_eq!(vulns.len(), 2);

        assert_eq!(vulns[0].id, "CVE-2024-1234");
        assert_eq!(vulns[0].package_name, "test-pkg");
        assert_eq!(vulns[0].installed_version, "1.0.0");
        assert_eq!(vulns[0].title, "Buffer overflow in foo");
        assert_eq!(
            vulns[0].description,
            "A buffer overflow vulnerability exists..."
        );
        assert_eq!(vulns[0].references.len(), 1);
        assert_eq!(
            vulns[0].references[0],
            "https://github.com/example/foo/issues/1"
        );

        assert_eq!(vulns[1].id, "CVE-2024-5678");
        assert_eq!(vulns[1].title, "XSS in bar");
        assert_eq!(vulns[1].references.len(), 0);
    }

    #[test]
    fn test_osv_scanner_severity_classification() {
        assert_eq!(
            OsvVulnScanner::classify_severity(Some(9.5)),
            VulnSeverity::Critical
        );
        assert_eq!(
            OsvVulnScanner::classify_severity(Some(9.0)),
            VulnSeverity::Critical
        );
        assert_eq!(
            OsvVulnScanner::classify_severity(Some(7.5)),
            VulnSeverity::High
        );
        assert_eq!(
            OsvVulnScanner::classify_severity(Some(7.0)),
            VulnSeverity::High
        );
        assert_eq!(
            OsvVulnScanner::classify_severity(Some(4.5)),
            VulnSeverity::Medium
        );
        assert_eq!(
            OsvVulnScanner::classify_severity(Some(4.0)),
            VulnSeverity::Medium
        );
        assert_eq!(
            OsvVulnScanner::classify_severity(Some(3.9)),
            VulnSeverity::Low
        );
        assert_eq!(
            OsvVulnScanner::classify_severity(Some(0.0)),
            VulnSeverity::Low
        );
        assert_eq!(OsvVulnScanner::classify_severity(None), VulnSeverity::Low);
    }
}
