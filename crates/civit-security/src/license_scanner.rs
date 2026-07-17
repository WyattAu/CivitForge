#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use parking_lot::Mutex;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LicenseInfo {
    pub spdx_id: String,
    pub name: String,
    pub category: LicenseCategory,
    pub copyleft: bool,
    pub patent_grant: bool,
}

impl LicenseInfo {
    pub fn is_osi_approved(&self) -> bool {
        matches!(
            self.category,
            LicenseCategory::Permissive
                | LicenseCategory::Copyleft
                | LicenseCategory::CopyleftLimited
                | LicenseCategory::PublicDomain
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LicenseCategory {
    Permissive,
    Copyleft,
    CopyleftLimited,
    Proprietary,
    PublicDomain,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseScanResult {
    pub package_name: String,
    pub version: String,
    pub license: LicenseInfo,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseViolation {
    pub package_name: String,
    pub license_id: String,
    pub reason: String,
    pub severity: ViolationSeverity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ViolationSeverity {
    Error,
    Warning,
    Info,
}

fn builtin_licenses() -> HashMap<&'static str, LicenseInfo> {
    let mut map = HashMap::new();
    map.insert(
        "MIT",
        LicenseInfo {
            spdx_id: "MIT".into(),
            name: "MIT License".into(),
            category: LicenseCategory::Permissive,
            copyleft: false,
            patent_grant: false,
        },
    );
    map.insert(
        "Apache-2.0",
        LicenseInfo {
            spdx_id: "Apache-2.0".into(),
            name: "Apache License 2.0".into(),
            category: LicenseCategory::Permissive,
            copyleft: false,
            patent_grant: true,
        },
    );
    map.insert(
        "GPL-2.0",
        LicenseInfo {
            spdx_id: "GPL-2.0".into(),
            name: "GNU General Public License v2.0".into(),
            category: LicenseCategory::Copyleft,
            copyleft: true,
            patent_grant: false,
        },
    );
    map.insert(
        "GPL-3.0",
        LicenseInfo {
            spdx_id: "GPL-3.0".into(),
            name: "GNU General Public License v3.0".into(),
            category: LicenseCategory::Copyleft,
            copyleft: true,
            patent_grant: false,
        },
    );
    map.insert(
        "LGPL-2.1",
        LicenseInfo {
            spdx_id: "LGPL-2.1".into(),
            name: "GNU Lesser General Public License v2.1".into(),
            category: LicenseCategory::CopyleftLimited,
            copyleft: true,
            patent_grant: false,
        },
    );
    map.insert(
        "LGPL-3.0",
        LicenseInfo {
            spdx_id: "LGPL-3.0".into(),
            name: "GNU Lesser General Public License v3.0".into(),
            category: LicenseCategory::CopyleftLimited,
            copyleft: true,
            patent_grant: false,
        },
    );
    map.insert(
        "BSD-2-Clause",
        LicenseInfo {
            spdx_id: "BSD-2-Clause".into(),
            name: "BSD 2-Clause \"Simplified\" License".into(),
            category: LicenseCategory::Permissive,
            copyleft: false,
            patent_grant: false,
        },
    );
    map.insert(
        "BSD-3-Clause",
        LicenseInfo {
            spdx_id: "BSD-3-Clause".into(),
            name: "BSD 3-Clause \"New\" or \"Revised\" License".into(),
            category: LicenseCategory::Permissive,
            copyleft: false,
            patent_grant: false,
        },
    );
    map.insert(
        "ISC",
        LicenseInfo {
            spdx_id: "ISC".into(),
            name: "ISC License".into(),
            category: LicenseCategory::Permissive,
            copyleft: false,
            patent_grant: false,
        },
    );
    map.insert(
        "MPL-2.0",
        LicenseInfo {
            spdx_id: "MPL-2.0".into(),
            name: "Mozilla Public License 2.0".into(),
            category: LicenseCategory::CopyleftLimited,
            copyleft: true,
            patent_grant: true,
        },
    );
    map.insert(
        "Unlicense",
        LicenseInfo {
            spdx_id: "Unlicense".into(),
            name: "The Unlicense".into(),
            category: LicenseCategory::PublicDomain,
            copyleft: false,
            patent_grant: false,
        },
    );
    map.insert(
        "CC0-1.0",
        LicenseInfo {
            spdx_id: "CC0-1.0".into(),
            name: "Creative Commons Zero v1.0 Universal".into(),
            category: LicenseCategory::PublicDomain,
            copyleft: false,
            patent_grant: false,
        },
    );
    map.insert(
        "AGPL-3.0",
        LicenseInfo {
            spdx_id: "AGPL-3.0".into(),
            name: "GNU Affero General Public License v3.0".into(),
            category: LicenseCategory::Copyleft,
            copyleft: true,
            patent_grant: false,
        },
    );
    map.insert(
        "BSL-1.1",
        LicenseInfo {
            spdx_id: "BSL-1.1".into(),
            name: "Boost Software License 1.1".into(),
            category: LicenseCategory::Permissive,
            copyleft: false,
            patent_grant: false,
        },
    );
    map.insert(
        "0BSD",
        LicenseInfo {
            spdx_id: "0BSD".into(),
            name: "Zero-Clause BSD".into(),
            category: LicenseCategory::Permissive,
            copyleft: false,
            patent_grant: false,
        },
    );
    map.insert(
        "GPL-2.0-only",
        LicenseInfo {
            spdx_id: "GPL-2.0-only".into(),
            name: "GNU General Public License v2.0 only".into(),
            category: LicenseCategory::Copyleft,
            copyleft: true,
            patent_grant: false,
        },
    );
    map
}

pub struct LicenseScanner {
    allowed_licenses: Mutex<Vec<String>>,
    denied_licenses: Mutex<Vec<String>>,
}

impl LicenseScanner {
    pub fn new() -> Self {
        Self {
            allowed_licenses: Mutex::new(Vec::new()),
            denied_licenses: Mutex::new(Vec::new()),
        }
    }

    pub fn allow_license(&self, spdx_id: &str) {
        self.allowed_licenses
            .lock()
            .push(spdx_id.to_string());
    }

    pub fn deny_license(&self, spdx_id: &str) {
        self.denied_licenses
            .lock()
            .push(spdx_id.to_string());
    }

    pub fn scan(&self, results: &[LicenseScanResult]) -> Vec<LicenseViolation> {
        let allowed = self.allowed_licenses.lock();
        let denied = self.denied_licenses.lock();
        let mut violations = Vec::new();

        for result in results {
            let lid = &result.license.spdx_id;

            if denied.iter().any(|d| d.eq_ignore_ascii_case(lid)) {
                violations.push(LicenseViolation {
                    package_name: result.package_name.clone(),
                    license_id: lid.clone(),
                    reason: "License is explicitly denied".into(),
                    severity: ViolationSeverity::Error,
                });
                continue;
            }

            if !allowed.is_empty() && !allowed.iter().any(|a| a.eq_ignore_ascii_case(lid)) {
                violations.push(LicenseViolation {
                    package_name: result.package_name.clone(),
                    license_id: lid.clone(),
                    reason: "License not in allow list".into(),
                    severity: ViolationSeverity::Warning,
                });
            }

            if result.license.copyleft {
                violations.push(LicenseViolation {
                    package_name: result.package_name.clone(),
                    license_id: lid.clone(),
                    reason: "Copyleft license detected".into(),
                    severity: ViolationSeverity::Info,
                });
            }
        }

        violations
    }

    pub fn is_allowed(&self, spdx_id: &str) -> bool {
        let allowed = self.allowed_licenses.lock();
        let denied = self.denied_licenses.lock();
        if denied.iter().any(|d| d.eq_ignore_ascii_case(spdx_id)) {
            return false;
        }
        if allowed.is_empty() {
            return true;
        }
        allowed.iter().any(|a| a.eq_ignore_ascii_case(spdx_id))
    }

    pub fn lookup(&self, spdx_id: &str) -> Option<LicenseInfo> {
        builtin_licenses().get(spdx_id).cloned()
    }
}

impl Default for LicenseScanner {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mit() -> LicenseInfo {
        LicenseInfo {
            spdx_id: "MIT".into(),
            name: "MIT License".into(),
            category: LicenseCategory::Permissive,
            copyleft: false,
            patent_grant: false,
        }
    }

    fn gpl3() -> LicenseInfo {
        LicenseInfo {
            spdx_id: "GPL-3.0".into(),
            name: "GNU General Public License v3.0".into(),
            category: LicenseCategory::Copyleft,
            copyleft: true,
            patent_grant: false,
        }
    }

    fn scan_result(name: &str, version: &str, license: LicenseInfo) -> LicenseScanResult {
        LicenseScanResult {
            package_name: name.into(),
            version: version.into(),
            license,
            source: "Cargo.toml".into(),
        }
    }

    #[test]
    fn test_new_scanner() {
        let scanner = LicenseScanner::new();
        assert!(scanner.is_allowed("MIT"));
    }

    #[test]
    fn test_allow_license() {
        let scanner = LicenseScanner::new();
        scanner.allow_license("MIT");
        assert!(scanner.is_allowed("MIT"));
        assert!(!scanner.is_allowed("GPL-3.0"));
    }

    #[test]
    fn test_deny_license() {
        let scanner = LicenseScanner::new();
        scanner.deny_license("AGPL-3.0");
        assert!(!scanner.is_allowed("AGPL-3.0"));
        assert!(scanner.is_allowed("MIT"));
    }

    #[test]
    fn test_deny_overrides_allow() {
        let scanner = LicenseScanner::new();
        scanner.allow_license("MIT");
        scanner.deny_license("MIT");
        assert!(!scanner.is_allowed("MIT"));
    }

    #[test]
    fn test_scan_empty() {
        let scanner = LicenseScanner::new();
        let violations = scanner.scan(&[]);
        assert!(violations.is_empty());
    }

    #[test]
    fn test_scan_no_policy() {
        let scanner = LicenseScanner::new();
        let results = vec![scan_result("serde", "1.0", mit())];
        let violations = scanner.scan(&results);
        assert!(violations.is_empty());
    }

    #[test]
    fn test_scan_allow_list() {
        let scanner = LicenseScanner::new();
        scanner.allow_license("MIT");
        let results = vec![scan_result("serde", "1.0", mit())];
        let violations = scanner.scan(&results);
        assert!(violations.is_empty());
    }

    #[test]
    fn test_scan_allow_list_rejects_unlisted() {
        let scanner = LicenseScanner::new();
        scanner.allow_license("MIT");
        let results = vec![scan_result(
            "foo",
            "1.0",
            LicenseInfo {
                spdx_id: "Unknown-1.0".into(),
                name: "Unknown".into(),
                category: LicenseCategory::Unknown,
                copyleft: false,
                patent_grant: false,
            },
        )];
        let violations = scanner.scan(&results);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].severity, ViolationSeverity::Warning);
    }

    #[test]
    fn test_scan_deny_list() {
        let scanner = LicenseScanner::new();
        scanner.deny_license("GPL-3.0");
        let results = vec![scan_result("copyleft-crate", "1.0", gpl3())];
        let violations = scanner.scan(&results);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].severity, ViolationSeverity::Error);
    }

    #[test]
    fn test_scan_copyleft_info() {
        let scanner = LicenseScanner::new();
        let results = vec![scan_result("gpl-lib", "2.0", gpl3())];
        let violations = scanner.scan(&results);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].severity, ViolationSeverity::Info);
    }

    #[test]
    fn test_lookup_mit() {
        let scanner = LicenseScanner::new();
        let info = scanner.lookup("MIT").unwrap();
        assert_eq!(info.spdx_id, "MIT");
        assert!(!info.copyleft);
    }

    #[test]
    fn test_lookup_apache() {
        let scanner = LicenseScanner::new();
        let info = scanner.lookup("Apache-2.0").unwrap();
        assert!(info.patent_grant);
        assert!(!info.copyleft);
    }

    #[test]
    fn test_lookup_unknown() {
        let scanner = LicenseScanner::new();
        assert!(scanner.lookup("FOOBAR-1.0").is_none());
    }

    #[test]
    fn test_osi_approved() {
        let scanner = LicenseScanner::new();
        let mit = scanner.lookup("MIT").unwrap();
        assert!(mit.is_osi_approved());
        let cc0 = scanner.lookup("CC0-1.0").unwrap();
        assert!(cc0.is_osi_approved());
    }

    #[test]
    fn test_not_osi_approved() {
        let info = LicenseInfo {
            spdx_id: "PROPRIETARY".into(),
            name: "Proprietary".into(),
            category: LicenseCategory::Proprietary,
            copyleft: false,
            patent_grant: false,
        };
        assert!(!info.is_osi_approved());
    }

    #[test]
    fn test_builtin_licenses_count() {
        let map = builtin_licenses();
        assert!(map.len() >= 15);
    }

    #[test]
    fn test_all_categories_present() {
        let map = builtin_licenses();
        let has_permissive = map
            .values()
            .any(|l| l.category == LicenseCategory::Permissive);
        let has_copyleft = map
            .values()
            .any(|l| l.category == LicenseCategory::Copyleft);
        let has_limited = map
            .values()
            .any(|l| l.category == LicenseCategory::CopyleftLimited);
        let has_pd = map
            .values()
            .any(|l| l.category == LicenseCategory::PublicDomain);
        assert!(has_permissive);
        assert!(has_copyleft);
        assert!(has_limited);
        assert!(has_pd);
    }

    #[test]
    fn test_serialization_roundtrip() {
        let info = mit();
        let json = serde_json::to_string(&info).unwrap();
        let de: LicenseInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(de, info);
    }

    #[test]
    fn test_violation_serialization() {
        let v = LicenseViolation {
            package_name: "foo".into(),
            license_id: "GPL-3.0".into(),
            reason: "denied".into(),
            severity: ViolationSeverity::Error,
        };
        let json = serde_json::to_string(&v).unwrap();
        let de: LicenseViolation = serde_json::from_str(&json).unwrap();
        assert_eq!(de.package_name, "foo");
    }

    #[test]
    fn test_case_insensitive_allow() {
        let scanner = LicenseScanner::new();
        scanner.allow_license("MIT");
        assert!(scanner.is_allowed("mit"));
        assert!(scanner.is_allowed("Mit"));
    }
}
