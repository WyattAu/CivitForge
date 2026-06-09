#![forbid(unsafe_code)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

pub use crate::cmdb::AssetType;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Classification {
    Public,
    Internal,
    Confidential,
    Restricted,
}

impl fmt::Display for Classification {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Public => write!(f, "public"),
            Self::Internal => write!(f, "internal"),
            Self::Confidential => write!(f, "confidential"),
            Self::Restricted => write!(f, "restricted"),
        }
    }
}

pub use crate::cmdb::Criticality;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Asset {
    pub id: String,
    pub name: String,
    pub asset_type: AssetType,
    pub owner: String,
    pub classification: Classification,
    pub location: String,
    pub criticality: Criticality,
}

#[derive(Debug, Clone)]
pub struct AssetInventory {
    assets: Vec<Asset>,
    categories: HashMap<String, Vec<String>>,
}

impl Default for AssetInventory {
    fn default() -> Self {
        Self::new()
    }
}

impl AssetInventory {
    pub fn new() -> Self {
        Self {
            assets: Vec::new(),
            categories: HashMap::new(),
        }
    }

    pub fn add_asset(&mut self, asset: Asset) {
        let type_name = asset.asset_type.to_string();
        self.categories
            .entry(type_name)
            .or_default()
            .push(asset.id.clone());
        self.assets.push(asset);
    }

    pub fn remove_asset(&mut self, id: &str) -> bool {
        if let Some(pos) = self.assets.iter().position(|a| a.id == id) {
            let removed = self.assets.remove(pos);
            let type_name = removed.asset_type.to_string();
            if let Some(list) = self.categories.get_mut(&type_name) {
                list.retain(|x| x != id);
            }
            true
        } else {
            false
        }
    }

    pub fn get_assets_by_type(&self, asset_type: &AssetType) -> Vec<&Asset> {
        self.assets
            .iter()
            .filter(|a| &a.asset_type == asset_type)
            .collect()
    }

    pub fn get_critical_assets(&self) -> Vec<&Asset> {
        self.assets
            .iter()
            .filter(|a| {
                a.criticality == Criticality::Critical || a.criticality == Criticality::High
            })
            .collect()
    }

    pub fn generate_report(&self) -> String {
        let mut report = String::from("=== ISO 27001 Asset Inventory Report ===\n\n");
        report.push_str(&format!("Total Assets: {}\n\n", self.assets.len()));

        for (category, ids) in &self.categories {
            report.push_str(&format!("[{category}]\n"));
            for id in ids {
                if let Some(asset) = self.assets.iter().find(|a| a.id == *id) {
                    report.push_str(&format!(
                        "  - {} ({}): owner={}, classification={}, criticality={}, location={}\n",
                        asset.name,
                        asset.id,
                        asset.owner,
                        asset.classification,
                        asset.criticality,
                        asset.location
                    ));
                }
            }
            report.push('\n');
        }

        let critical = self.get_critical_assets();
        report.push_str(&format!(
            "Critical/High Priority Assets: {}\n",
            critical.len()
        ));
        for a in &critical {
            report.push_str(&format!("  - {} ({})\n", a.name, a.id));
        }

        report
    }

    pub fn assets(&self) -> &[Asset] {
        &self.assets
    }

    pub fn len(&self) -> usize {
        self.assets.len()
    }

    pub fn is_empty(&self) -> bool {
        self.assets.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComplianceStatus {
    NotStarted,
    InProgress,
    PartiallyCompliant,
    Compliant,
    NotApplicable,
}

impl fmt::Display for ComplianceStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotStarted => write!(f, "not_started"),
            Self::InProgress => write!(f, "in_progress"),
            Self::PartiallyCompliant => write!(f, "partially_compliant"),
            Self::Compliant => write!(f, "compliant"),
            Self::NotApplicable => write!(f, "not_applicable"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChecklistItem {
    pub id: String,
    pub control: String,
    pub description: String,
    pub status: ComplianceStatus,
    pub evidence: Vec<String>,
    pub last_reviewed: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone)]
pub struct ComplianceChecklist {
    items: Vec<ChecklistItem>,
}

impl Default for ComplianceChecklist {
    fn default() -> Self {
        Self::new()
    }
}

impl ComplianceChecklist {
    pub fn new() -> Self {
        let items = vec![
            ChecklistItem {
                id: "A.5.1".into(),
                control: "A.5.1 Policies for information security".into(),
                description: "Management direction and support for information security".into(),
                status: ComplianceStatus::Compliant,
                evidence: vec!["infosec-policy-v3.pdf".into()],
                last_reviewed: Some(Utc::now()),
            },
            ChecklistItem {
                id: "A.5.7".into(),
                control: "A.5.7 Threat intelligence".into(),
                description: "Information about threats relevant to the organization".into(),
                status: ComplianceStatus::InProgress,
                evidence: vec!["threat-intel-feed-config.json".into()],
                last_reviewed: Some(Utc::now()),
            },
            ChecklistItem {
                id: "A.6.1".into(),
                control: "A.6.1 Screening".into(),
                description: "Background verification checks on personnel".into(),
                status: ComplianceStatus::Compliant,
                evidence: vec!["background-check-policy.pdf".into()],
                last_reviewed: Some(Utc::now()),
            },
            ChecklistItem {
                id: "A.7.1".into(),
                control: "A.7.1 Physical security perimeters".into(),
                description: "Defining and using security perimeters".into(),
                status: ComplianceStatus::Compliant,
                evidence: vec!["physical-security-map.pdf".into()],
                last_reviewed: Some(Utc::now()),
            },
            ChecklistItem {
                id: "A.8.1".into(),
                control: "A.8.1 User endpoint devices".into(),
                description: "Securing user endpoint devices".into(),
                status: ComplianceStatus::PartiallyCompliant,
                evidence: vec!["endpoint-mdm-policy.pdf".into()],
                last_reviewed: Some(Utc::now()),
            },
            ChecklistItem {
                id: "A.8.9".into(),
                control: "A.8.9 Configuration management".into(),
                description: "Secure configuration for hardware, software, services".into(),
                status: ComplianceStatus::Compliant,
                evidence: vec!["iac-terraform-repo".into(), "config-baseline.yaml".into()],
                last_reviewed: Some(Utc::now()),
            },
            ChecklistItem {
                id: "A.8.10".into(),
                control: "A.8.10 Information deletion".into(),
                description: "Secure deletion of data on storage devices".into(),
                status: ComplianceStatus::InProgress,
                evidence: vec![],
                last_reviewed: None,
            },
            ChecklistItem {
                id: "A.8.12".into(),
                control: "A.8.12 Data leakage prevention".into(),
                description: "Preventing data leakage via DLP measures".into(),
                status: ComplianceStatus::Compliant,
                evidence: vec!["dlp-ruleset.json".into()],
                last_reviewed: Some(Utc::now()),
            },
            ChecklistItem {
                id: "A.9.1".into(),
                control: "A.9.1 Access control".into(),
                description: "Rules to control physical and logical access".into(),
                status: ComplianceStatus::Compliant,
                evidence: vec![
                    "abac-policy-engine.md".into(),
                    "iam-audit-report.pdf".into(),
                ],
                last_reviewed: Some(Utc::now()),
            },
            ChecklistItem {
                id: "A.12.1".into(),
                control: "A.12.1 Operational procedures".into(),
                description: "Documented and secure operational procedures".into(),
                status: ComplianceStatus::Compliant,
                evidence: vec!["runbook-incident-response.md".into()],
                last_reviewed: Some(Utc::now()),
            },
            ChecklistItem {
                id: "A.12.4".into(),
                control: "A.12.4 Logging and monitoring".into(),
                description: "Logging events and generating evidence".into(),
                status: ComplianceStatus::Compliant,
                evidence: vec!["audit-trail-module.md".into(), "siem-dashboard.png".into()],
                last_reviewed: Some(Utc::now()),
            },
            ChecklistItem {
                id: "A.14.1".into(),
                control: "A.14.1 Information security in development".into(),
                description: "Secure development lifecycle".into(),
                status: ComplianceStatus::Compliant,
                evidence: vec!["sdlc-policy.pdf".into()],
                last_reviewed: Some(Utc::now()),
            },
        ];
        Self { items }
    }

    pub fn items(&self) -> &[ChecklistItem] {
        &self.items
    }

    pub fn compliance_rate(&self) -> f64 {
        let total = self.items.len();
        if total == 0 {
            return 0.0;
        }
        let compliant = self
            .items
            .iter()
            .filter(|i| i.status == ComplianceStatus::Compliant)
            .count();
        (compliant as f64 / total as f64) * 100.0
    }

    pub fn add_item(&mut self, item: ChecklistItem) {
        self.items.push(item);
    }

    pub fn update_status(&mut self, id: &str, status: ComplianceStatus) -> bool {
        if let Some(item) = self.items.iter_mut().find(|i| i.id == id) {
            item.status = status;
            item.last_reviewed = Some(Utc::now());
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_asset(id: &str, asset_type: AssetType, criticality: Criticality) -> Asset {
        Asset {
            id: id.to_string(),
            name: format!("Asset {id}"),
            asset_type,
            owner: "owner-1".to_string(),
            classification: Classification::Confidential,
            location: "us-east-1".to_string(),
            criticality,
        }
    }

    #[test]
    fn test_add_asset() {
        let mut inventory = AssetInventory::new();
        inventory.add_asset(sample_asset("a1", AssetType::Hardware, Criticality::High));
        assert_eq!(inventory.len(), 1);
    }

    #[test]
    fn test_remove_asset() {
        let mut inventory = AssetInventory::new();
        inventory.add_asset(sample_asset("a1", AssetType::Hardware, Criticality::Low));
        assert!(inventory.remove_asset("a1"));
        assert_eq!(inventory.len(), 0);
        assert!(!inventory.remove_asset("nonexistent"));
    }

    #[test]
    fn test_get_assets_by_type() {
        let mut inventory = AssetInventory::new();
        inventory.add_asset(sample_asset("a1", AssetType::Hardware, Criticality::Low));
        inventory.add_asset(sample_asset("a2", AssetType::Software, Criticality::Low));
        inventory.add_asset(sample_asset("a3", AssetType::Hardware, Criticality::Low));
        let hw = inventory.get_assets_by_type(&AssetType::Hardware);
        assert_eq!(hw.len(), 2);
        let sw = inventory.get_assets_by_type(&AssetType::Software);
        assert_eq!(sw.len(), 1);
    }

    #[test]
    fn test_get_critical_assets() {
        let mut inventory = AssetInventory::new();
        inventory.add_asset(sample_asset("a1", AssetType::Data, Criticality::Low));
        inventory.add_asset(sample_asset("a2", AssetType::Data, Criticality::Critical));
        inventory.add_asset(sample_asset("a3", AssetType::Data, Criticality::High));
        inventory.add_asset(sample_asset("a4", AssetType::Data, Criticality::Medium));
        let critical = inventory.get_critical_assets();
        assert_eq!(critical.len(), 2);
    }

    #[test]
    fn test_generate_report() {
        let mut inventory = AssetInventory::new();
        inventory.add_asset(sample_asset(
            "srv-1",
            AssetType::Hardware,
            Criticality::Critical,
        ));
        inventory.add_asset(sample_asset(
            "app-1",
            AssetType::Software,
            Criticality::High,
        ));
        let report = inventory.generate_report();
        assert!(report.contains("Asset Inventory Report"));
        assert!(report.contains("srv-1"));
        assert!(report.contains("app-1"));
        assert!(report.contains("Total Assets: 2"));
    }

    #[test]
    fn test_compliance_checklist() {
        let checklist = ComplianceChecklist::new();
        assert_eq!(checklist.items().len(), 12);
    }

    #[test]
    fn test_compliance_rate() {
        let checklist = ComplianceChecklist::new();
        let rate = checklist.compliance_rate();
        assert!(rate > 0.0);
        assert!(rate <= 100.0);
    }

    #[test]
    fn test_add_checklist_item() {
        let mut checklist = ComplianceChecklist::new();
        let item = ChecklistItem {
            id: "A.99.1".into(),
            control: "Custom control".into(),
            description: "A custom control".into(),
            status: ComplianceStatus::NotStarted,
            evidence: Vec::new(),
            last_reviewed: None,
        };
        checklist.add_item(item);
        assert_eq!(checklist.items().len(), 13);
    }

    #[test]
    fn test_update_checklist_status() {
        let mut checklist = ComplianceChecklist::new();
        assert!(checklist.update_status("A.8.10", ComplianceStatus::Compliant));
        let item = checklist.items().iter().find(|i| i.id == "A.8.10").unwrap();
        assert_eq!(item.status, ComplianceStatus::Compliant);
        assert!(item.last_reviewed.is_some());
        assert!(!checklist.update_status("nonexistent", ComplianceStatus::Compliant));
    }

    #[test]
    fn test_asset_classification_display() {
        assert_eq!(Classification::Public.to_string(), "public");
        assert_eq!(Classification::Restricted.to_string(), "restricted");
    }

    #[test]
    fn test_asset_type_display() {
        assert_eq!(AssetType::Hardware.to_string(), "hardware");
        assert_eq!(AssetType::Software.to_string(), "software");
        assert_eq!(AssetType::Data.to_string(), "data");
    }
}
