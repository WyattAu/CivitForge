#![forbid(unsafe_code)]

use chrono::{DateTime, Utc};
use dashmap::DashMap;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum AssetType {
    Server,
    Database,
    Network,
    Service,
    Certificate,
    Application,
    Storage,
}

impl std::fmt::Display for AssetType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Server => write!(f, "server"),
            Self::Database => write!(f, "database"),
            Self::Network => write!(f, "network"),
            Self::Service => write!(f, "service"),
            Self::Certificate => write!(f, "certificate"),
            Self::Application => write!(f, "application"),
            Self::Storage => write!(f, "storage"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Criticality {
    High,
    Medium,
    Low,
}

impl std::fmt::Display for Criticality {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::High => write!(f, "high"),
            Self::Medium => write!(f, "medium"),
            Self::Low => write!(f, "low"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum AssetStatus {
    Active,
    Retired,
    Maintenance,
    Decommissioned,
}

impl std::fmt::Display for AssetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Active => write!(f, "active"),
            Self::Retired => write!(f, "retired"),
            Self::Maintenance => write!(f, "maintenance"),
            Self::Decommissioned => write!(f, "decommissioned"),
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Asset {
    pub id: String,
    pub name: String,
    pub asset_type: AssetType,
    pub owner: String,
    pub criticality: Criticality,
    pub location: String,
    pub status: AssetStatus,
    pub configuration: HashMap<String, serde_json::Value>,
    pub tags: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Likelihood {
    Low,
    Medium,
    High,
}

impl std::fmt::Display for Likelihood {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Low => write!(f, "low"),
            Self::Medium => write!(f, "medium"),
            Self::High => write!(f, "high"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Impact {
    Low,
    Medium,
    High,
}

impl std::fmt::Display for Impact {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Low => write!(f, "low"),
            Self::Medium => write!(f, "medium"),
            Self::High => write!(f, "high"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Treatment {
    Avoid,
    Mitigate,
    Transfer,
    Accept,
}

impl std::fmt::Display for Treatment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Avoid => write!(f, "avoid"),
            Self::Mitigate => write!(f, "mitigate"),
            Self::Transfer => write!(f, "transfer"),
            Self::Accept => write!(f, "accept"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum RiskStatus {
    Open,
    Closed,
    Accepted,
}

impl std::fmt::Display for RiskStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Open => write!(f, "open"),
            Self::Closed => write!(f, "closed"),
            Self::Accepted => write!(f, "accepted"),
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RiskEntry {
    pub id: String,
    pub asset_id: String,
    pub description: String,
    pub likelihood: Likelihood,
    pub impact: Impact,
    pub severity: u32,
    pub treatment: Treatment,
    pub owner: String,
    pub status: RiskStatus,
}

impl RiskEntry {
    fn compute_severity(likelihood: &Likelihood, impact: &Impact) -> u32 {
        let l = match likelihood {
            Likelihood::Low => 1,
            Likelihood::Medium => 2,
            Likelihood::High => 3,
        };
        let i = match impact {
            Impact::Low => 1,
            Impact::Medium => 2,
            Impact::High => 3,
        };
        l * i
    }
}

#[derive(Debug, Clone, Default)]
pub struct Cmdb {
    assets: DashMap<String, Asset>,
    risks: DashMap<String, RiskEntry>,
}

impl Cmdb {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register_asset(&self, mut asset: Asset) {
        let now = Utc::now();
        asset.created_at = now;
        asset.updated_at = now;
        asset.status = AssetStatus::Active;
        self.assets.insert(asset.id.clone(), asset);
    }

    #[allow(clippy::too_many_arguments)]
    pub fn update_asset(
        &self,
        id: &str,
        name: Option<String>,
        owner: Option<String>,
        criticality: Option<Criticality>,
        location: Option<String>,
        configuration: Option<HashMap<String, serde_json::Value>>,
        tags: Option<Vec<String>>,
    ) -> bool {
        if let Some(mut asset) = self.assets.get_mut(id) {
            if let Some(n) = name {
                asset.name = n;
            }
            if let Some(o) = owner {
                asset.owner = o;
            }
            if let Some(c) = criticality {
                asset.criticality = c;
            }
            if let Some(l) = location {
                asset.location = l;
            }
            if let Some(cfg) = configuration {
                asset.configuration = cfg;
            }
            if let Some(t) = tags {
                asset.tags = t;
            }
            asset.updated_at = Utc::now();
            true
        } else {
            false
        }
    }

    pub fn retire_asset(&self, id: &str) -> bool {
        if let Some(mut asset) = self.assets.get_mut(id) {
            asset.status = AssetStatus::Retired;
            asset.updated_at = Utc::now();
            true
        } else {
            false
        }
    }

    pub fn list_assets(&self) -> Vec<Asset> {
        self.assets.iter().map(|r| r.value().clone()).collect()
    }

    pub fn get_asset(&self, id: &str) -> Option<Asset> {
        self.assets.get(id).map(|r| r.value().clone())
    }

    pub fn add_risk(&self, mut risk: RiskEntry) {
        risk.severity = RiskEntry::compute_severity(&risk.likelihood, &risk.impact);
        self.risks.insert(risk.id.clone(), risk);
    }

    pub fn close_risk(&self, id: &str) -> bool {
        if let Some(mut risk) = self.risks.get_mut(id) {
            risk.status = RiskStatus::Closed;
            true
        } else {
            false
        }
    }

    pub fn risk_register(&self) -> Vec<RiskEntry> {
        self.risks.iter().map(|r| r.value().clone()).collect()
    }

    pub fn access_review_report(&self) -> String {
        let assets: Vec<Asset> = self
            .assets
            .iter()
            .filter(|r| r.value().status == AssetStatus::Active)
            .map(|r| r.value().clone())
            .collect();
        let mut report = String::from("=== Access Review Report ===\n\n");
        report.push_str(&format!("Total Active Assets: {}\n\n", assets.len()));

        let high_crit: Vec<&Asset> = assets
            .iter()
            .filter(|a| a.criticality == Criticality::High)
            .collect();
        let med_crit: Vec<&Asset> = assets
            .iter()
            .filter(|a| a.criticality == Criticality::Medium)
            .collect();
        let low_crit: Vec<&Asset> = assets
            .iter()
            .filter(|a| a.criticality == Criticality::Low)
            .collect();

        report.push_str(&format!("High Criticality: {}\n", high_crit.len()));
        report.push_str(&format!("Medium Criticality: {}\n", med_crit.len()));
        report.push_str(&format!("Low Criticality: {}\n", low_crit.len()));
        report.push('\n');

        for asset in &high_crit {
            report.push_str(&format!(
                "  [{}] {} (type={}, owner={})\n",
                asset.id, asset.name, asset.asset_type, asset.owner
            ));
        }

        report
    }

    pub fn compliance_summary(&self) -> HashMap<String, usize> {
        let mut summary = HashMap::new();
        let total_assets = self.assets.len();
        summary.insert("total_assets".to_string(), total_assets);
        let active: usize = self
            .assets
            .iter()
            .filter(|r| r.value().status == AssetStatus::Active)
            .count();
        summary.insert("active_assets".to_string(), active);
        let retired: usize = self
            .assets
            .iter()
            .filter(|r| r.value().status == AssetStatus::Retired)
            .count();
        summary.insert("retired_assets".to_string(), retired);
        let high: usize = self
            .assets
            .iter()
            .filter(|r| r.value().criticality == Criticality::High)
            .count();
        summary.insert("high_criticality".to_string(), high);
        let open_risks = self
            .risks
            .iter()
            .filter(|r| r.value().status == RiskStatus::Open)
            .count();
        summary.insert("open_risks".to_string(), open_risks);
        let closed_risks = self
            .risks
            .iter()
            .filter(|r| r.value().status == RiskStatus::Closed)
            .count();
        summary.insert("closed_risks".to_string(), closed_risks);
        let asset_types: HashMap<AssetType, usize> =
            self.assets.iter().fold(HashMap::new(), |mut acc, r| {
                *acc.entry(r.value().asset_type.clone()).or_insert(0) += 1;
                acc
            });
        for (t, c) in &asset_types {
            summary.insert(format!("type_{t}"), *c);
        }
        summary
    }
}

#[allow(dead_code)]
fn make_asset(id: &str, name: &str, asset_type: AssetType, criticality: Criticality) -> Asset {
    Asset {
        id: id.to_string(),
        name: name.to_string(),
        asset_type,
        owner: "team-infra".to_string(),
        criticality,
        location: "us-east-1".to_string(),
        status: AssetStatus::Active,
        configuration: HashMap::new(),
        tags: vec!["production".to_string()],
        created_at: Utc::now(),
        updated_at: Utc::now(),
    }
}

#[allow(dead_code)]
fn make_risk(id: &str, asset_id: &str, likelihood: Likelihood, impact: Impact) -> RiskEntry {
    RiskEntry {
        id: id.to_string(),
        asset_id: asset_id.to_string(),
        description: format!("Risk {id} for asset {asset_id}"),
        likelihood,
        impact,
        severity: 0,
        treatment: Treatment::Mitigate,
        owner: "risk-owner".to_string(),
        status: RiskStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_asset() {
        let cmdb = Cmdb::new();
        let asset = make_asset(
            "srv-001",
            "web-server",
            AssetType::Server,
            Criticality::High,
        );
        cmdb.register_asset(asset);
        let retrieved = cmdb.get_asset("srv-001").unwrap();
        assert_eq!(retrieved.name, "web-server");
        assert_eq!(retrieved.status, AssetStatus::Active);
    }

    #[test]
    fn test_register_sets_timestamps() {
        let cmdb = Cmdb::new();
        let before = Utc::now();
        let asset = make_asset("srv-002", "db", AssetType::Database, Criticality::High);
        cmdb.register_asset(asset);
        let retrieved = cmdb.get_asset("srv-002").unwrap();
        assert!(retrieved.created_at >= before);
        assert_eq!(retrieved.created_at, retrieved.updated_at);
    }

    #[test]
    fn test_update_asset_name() {
        let cmdb = Cmdb::new();
        cmdb.register_asset(make_asset(
            "srv-003",
            "old-name",
            AssetType::Server,
            Criticality::Low,
        ));
        let updated = cmdb.update_asset(
            "srv-003",
            Some("new-name".to_string()),
            None,
            None,
            None,
            None,
            None,
        );
        assert!(updated);
        let retrieved = cmdb.get_asset("srv-003").unwrap();
        assert_eq!(retrieved.name, "new-name");
    }

    #[test]
    fn test_update_asset_not_found() {
        let cmdb = Cmdb::new();
        let updated = cmdb.update_asset("nonexistent", None, None, None, None, None, None);
        assert!(!updated);
    }

    #[test]
    fn test_update_asset_owner() {
        let cmdb = Cmdb::new();
        cmdb.register_asset(make_asset(
            "srv-004",
            "app",
            AssetType::Application,
            Criticality::Medium,
        ));
        cmdb.update_asset(
            "srv-004",
            None,
            Some("new-team".to_string()),
            None,
            None,
            None,
            None,
        );
        let retrieved = cmdb.get_asset("srv-004").unwrap();
        assert_eq!(retrieved.owner, "new-team");
    }

    #[test]
    fn test_retire_asset() {
        let cmdb = Cmdb::new();
        cmdb.register_asset(make_asset(
            "srv-005",
            "old",
            AssetType::Server,
            Criticality::Low,
        ));
        let retired = cmdb.retire_asset("srv-005");
        assert!(retired);
        let retrieved = cmdb.get_asset("srv-005").unwrap();
        assert_eq!(retrieved.status, AssetStatus::Retired);
    }

    #[test]
    fn test_retire_asset_not_found() {
        let cmdb = Cmdb::new();
        assert!(!cmdb.retire_asset("nonexistent"));
    }

    #[test]
    fn test_list_assets() {
        let cmdb = Cmdb::new();
        cmdb.register_asset(make_asset(
            "srv-006",
            "a",
            AssetType::Server,
            Criticality::High,
        ));
        cmdb.register_asset(make_asset(
            "srv-007",
            "b",
            AssetType::Database,
            Criticality::Medium,
        ));
        let list = cmdb.list_assets();
        assert_eq!(list.len(), 2);
    }

    #[test]
    fn test_get_asset_not_found() {
        let cmdb = Cmdb::new();
        assert!(cmdb.get_asset("nonexistent").is_none());
    }

    #[test]
    fn test_add_risk() {
        let cmdb = Cmdb::new();
        cmdb.add_risk(make_risk(
            "risk-001",
            "srv-001",
            Likelihood::High,
            Impact::High,
        ));
        let register = cmdb.risk_register();
        assert_eq!(register.len(), 1);
        assert_eq!(register[0].severity, 9);
    }

    #[test]
    fn test_risk_severity_calculation() {
        assert_eq!(
            RiskEntry::compute_severity(&Likelihood::Low, &Impact::Low),
            1
        );
        assert_eq!(
            RiskEntry::compute_severity(&Likelihood::Medium, &Impact::Low),
            2
        );
        assert_eq!(
            RiskEntry::compute_severity(&Likelihood::High, &Impact::High),
            9
        );
    }

    #[test]
    fn test_close_risk() {
        let cmdb = Cmdb::new();
        cmdb.add_risk(make_risk(
            "risk-002",
            "srv-001",
            Likelihood::Medium,
            Impact::Medium,
        ));
        assert!(cmdb.close_risk("risk-002"));
        let register = cmdb.risk_register();
        assert_eq!(register[0].status, RiskStatus::Closed);
    }

    #[test]
    fn test_close_risk_not_found() {
        let cmdb = Cmdb::new();
        assert!(!cmdb.close_risk("nonexistent"));
    }

    #[test]
    fn test_access_review_report() {
        let cmdb = Cmdb::new();
        cmdb.register_asset(make_asset(
            "srv-010",
            "prod-api",
            AssetType::Server,
            Criticality::High,
        ));
        cmdb.register_asset(make_asset(
            "srv-011",
            "staging-api",
            AssetType::Server,
            Criticality::Low,
        ));
        let report = cmdb.access_review_report();
        assert!(report.contains("Access Review Report"));
        assert!(report.contains("Total Active Assets: 2"));
        assert!(report.contains("High Criticality: 1"));
        assert!(report.contains("Low Criticality: 1"));
        assert!(report.contains("prod-api"));
    }

    #[test]
    fn test_compliance_summary() {
        let cmdb = Cmdb::new();
        cmdb.register_asset(make_asset(
            "srv-020",
            "web",
            AssetType::Server,
            Criticality::High,
        ));
        cmdb.register_asset(make_asset(
            "srv-021",
            "db",
            AssetType::Database,
            Criticality::Medium,
        ));
        cmdb.add_risk(make_risk(
            "risk-010",
            "srv-020",
            Likelihood::High,
            Impact::High,
        ));
        let summary = cmdb.compliance_summary();
        assert_eq!(summary.get("total_assets").copied(), Some(2));
        assert_eq!(summary.get("active_assets").copied(), Some(2));
        assert_eq!(summary.get("high_criticality").copied(), Some(1));
        assert_eq!(summary.get("open_risks").copied(), Some(1));
        assert_eq!(summary.get("closed_risks").copied(), Some(0));
    }

    #[test]
    fn test_compliance_summary_with_retired() {
        let cmdb = Cmdb::new();
        cmdb.register_asset(make_asset(
            "srv-030",
            "active",
            AssetType::Server,
            Criticality::High,
        ));
        cmdb.register_asset(make_asset(
            "srv-031",
            "retired",
            AssetType::Server,
            Criticality::Low,
        ));
        cmdb.retire_asset("srv-031");
        let summary = cmdb.compliance_summary();
        assert_eq!(summary.get("total_assets").copied(), Some(2));
        assert_eq!(summary.get("active_assets").copied(), Some(1));
        assert_eq!(summary.get("retired_assets").copied(), Some(1));
    }

    #[test]
    fn test_compliance_summary_empty() {
        let cmdb = Cmdb::new();
        let summary = cmdb.compliance_summary();
        assert_eq!(summary.get("total_assets").copied(), Some(0));
        assert_eq!(summary.get("open_risks").copied(), Some(0));
    }

    #[test]
    fn test_asset_type_display() {
        assert_eq!(AssetType::Server.to_string(), "server");
        assert_eq!(AssetType::Database.to_string(), "database");
        assert_eq!(AssetType::Network.to_string(), "network");
        assert_eq!(AssetType::Service.to_string(), "service");
        assert_eq!(AssetType::Certificate.to_string(), "certificate");
        assert_eq!(AssetType::Application.to_string(), "application");
        assert_eq!(AssetType::Storage.to_string(), "storage");
    }

    #[test]
    fn test_update_asset_updates_timestamp() {
        let cmdb = Cmdb::new();
        cmdb.register_asset(make_asset(
            "srv-040",
            "a",
            AssetType::Server,
            Criticality::Medium,
        ));
        let before = cmdb.get_asset("srv-040").unwrap().updated_at;
        cmdb.update_asset("srv-040", None, None, None, None, None, None);
        let after = cmdb.get_asset("srv-040").unwrap().updated_at;
        assert!(after >= before);
    }
}
