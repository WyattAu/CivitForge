#![forbid(unsafe_code)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RiskCategory {
    Security,
    Operational,
    Financial,
    Compliance,
    Technical,
    Reputational,
}

impl fmt::Display for RiskCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Security => write!(f, "security"),
            Self::Operational => write!(f, "operational"),
            Self::Financial => write!(f, "financial"),
            Self::Compliance => write!(f, "compliance"),
            Self::Technical => write!(f, "technical"),
            Self::Reputational => write!(f, "reputational"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

impl fmt::Display for RiskLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Low => write!(f, "low"),
            Self::Medium => write!(f, "medium"),
            Self::High => write!(f, "high"),
            Self::Critical => write!(f, "critical"),
        }
    }
}

impl RiskLevel {
    pub fn score(&self) -> u32 {
        match self {
            Self::Low => 1,
            Self::Medium => 2,
            Self::High => 3,
            Self::Critical => 4,
        }
    }
}

pub use crate::cmdb::RiskStatus;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Risk {
    pub id: String,
    pub title: String,
    pub description: String,
    pub category: RiskCategory,
    pub likelihood: RiskLevel,
    pub impact: RiskLevel,
    pub risk_score: u32,
    pub mitigation: Option<String>,
    pub owner: String,
    pub status: RiskStatus,
    pub last_reviewed: DateTime<Utc>,
}

impl Risk {
    pub fn compute_score(likelihood: &RiskLevel, impact: &RiskLevel) -> u32 {
        likelihood.score() * impact.score()
    }
}

#[derive(Debug, Clone)]
pub struct RiskRegister {
    risks: Vec<Risk>,
}

impl Default for RiskRegister {
    fn default() -> Self {
        Self::new()
    }
}

impl RiskRegister {
    pub fn new() -> Self {
        Self { risks: Vec::new() }
    }

    pub fn add_risk(&mut self, mut risk: Risk) {
        risk.risk_score = Risk::compute_score(&risk.likelihood, &risk.impact);
        self.risks.push(risk);
    }

    pub fn update_risk(
        &mut self,
        id: &str,
        title: Option<String>,
        likelihood: Option<RiskLevel>,
        impact: Option<RiskLevel>,
        mitigation: Option<Option<String>>,
        status: Option<RiskStatus>,
    ) -> bool {
        if let Some(risk) = self.risks.iter_mut().find(|r| r.id == id) {
            if let Some(t) = title {
                risk.title = t;
            }
            if let Some(l) = likelihood {
                risk.likelihood = l;
            }
            if let Some(i) = impact {
                risk.impact = i;
            }
            risk.risk_score = Risk::compute_score(&risk.likelihood, &risk.impact);
            if let Some(m) = mitigation {
                risk.mitigation = m;
            }
            if let Some(s) = status {
                risk.status = s;
            }
            risk.last_reviewed = Utc::now();
            true
        } else {
            false
        }
    }

    pub fn get_high_risks(&self) -> Vec<&Risk> {
        self.risks
            .iter()
            .filter(|r| {
                r.risk_score >= 6
                    || r.likelihood == RiskLevel::Critical
                    || r.impact == RiskLevel::Critical
            })
            .collect()
    }

    pub fn risk_matrix(&self) -> HashMap<RiskLevel, Vec<&Risk>> {
        let mut matrix: HashMap<RiskLevel, Vec<&Risk>> = HashMap::new();
        for risk in &self.risks {
            let max_level = if risk.likelihood.score() >= risk.impact.score() {
                risk.likelihood.clone()
            } else {
                risk.impact.clone()
            };
            matrix.entry(max_level).or_default().push(risk);
        }
        matrix
    }

    pub fn generate_report(&self) -> String {
        let mut report = String::from("=== Risk Assessment Report ===\n\n");
        report.push_str(&format!("Total Risks: {}\n", self.risks.len()));

        let open: Vec<&Risk> = self
            .risks
            .iter()
            .filter(|r| r.status == RiskStatus::Open)
            .collect();
        let mitigated: Vec<&Risk> = self
            .risks
            .iter()
            .filter(|r| r.status == RiskStatus::Mitigated)
            .collect();
        let accepted: Vec<&Risk> = self
            .risks
            .iter()
            .filter(|r| r.status == RiskStatus::Accepted)
            .collect();
        let closed: Vec<&Risk> = self
            .risks
            .iter()
            .filter(|r| r.status == RiskStatus::Closed)
            .collect();
        let transferred: Vec<&Risk> = self
            .risks
            .iter()
            .filter(|r| r.status == RiskStatus::Transferred)
            .collect();

        report.push_str(&format!("Open: {}\n", open.len()));
        report.push_str(&format!("Mitigated: {}\n", mitigated.len()));
        report.push_str(&format!("Accepted: {}\n", accepted.len()));
        report.push_str(&format!("Transferred: {}\n", transferred.len()));
        report.push_str(&format!("Closed: {}\n", closed.len()));
        report.push('\n');

        let high = self.get_high_risks();
        report.push_str(&format!("High/Critical Risks ({}):\n", high.len()));
        for r in &high {
            report.push_str(&format!(
                "  [{}] {} (score={}, likelihood={}, impact={}, owner={})\n",
                r.id, r.title, r.risk_score, r.likelihood, r.impact, r.owner
            ));
        }

        report.push_str("\nRisk Matrix:\n");
        let matrix = self.risk_matrix();
        for level in &[
            RiskLevel::Critical,
            RiskLevel::High,
            RiskLevel::Medium,
            RiskLevel::Low,
        ] {
            if let Some(risks) = matrix.get(level) {
                report.push_str(&format!("  {}: {} risk(s)\n", level, risks.len()));
            }
        }

        report
    }

    pub fn risks(&self) -> &[Risk] {
        &self.risks
    }

    pub fn len(&self) -> usize {
        self.risks.len()
    }

    pub fn is_empty(&self) -> bool {
        self.risks.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_risk(id: &str, likelihood: RiskLevel, impact: RiskLevel) -> Risk {
        Risk {
            id: id.to_string(),
            title: format!("Risk {id}"),
            description: format!("Description for risk {id}"),
            category: RiskCategory::Security,
            likelihood,
            impact,
            risk_score: 0,
            mitigation: Some("Implement controls".to_string()),
            owner: "risk-owner".to_string(),
            status: RiskStatus::Open,
            last_reviewed: Utc::now(),
        }
    }

    #[test]
    fn test_add_risk() {
        let mut register = RiskRegister::new();
        register.add_risk(sample_risk("r1", RiskLevel::Medium, RiskLevel::High));
        assert_eq!(register.len(), 1);
        assert_eq!(register.risks()[0].risk_score, 6);
    }

    #[test]
    fn test_risk_score_calculation() {
        assert_eq!(Risk::compute_score(&RiskLevel::Low, &RiskLevel::Low), 1);
        assert_eq!(Risk::compute_score(&RiskLevel::Low, &RiskLevel::Medium), 2);
        assert_eq!(Risk::compute_score(&RiskLevel::Medium, &RiskLevel::High), 6);
        assert_eq!(Risk::compute_score(&RiskLevel::High, &RiskLevel::High), 9);
        assert_eq!(
            Risk::compute_score(&RiskLevel::Critical, &RiskLevel::Critical),
            16
        );
    }

    #[test]
    fn test_update_risk() {
        let mut register = RiskRegister::new();
        register.add_risk(sample_risk("r1", RiskLevel::Medium, RiskLevel::Medium));
        assert_eq!(register.risks()[0].risk_score, 4);

        register.update_risk("r1", None, Some(RiskLevel::High), None, None, None);
        assert_eq!(register.risks()[0].risk_score, 6);
    }

    #[test]
    fn test_update_risk_not_found() {
        let mut register = RiskRegister::new();
        assert!(!register.update_risk("nonexistent", None, None, None, None, None));
    }

    #[test]
    fn test_get_high_risks() {
        let mut register = RiskRegister::new();
        register.add_risk(sample_risk("r1", RiskLevel::Low, RiskLevel::Low));
        register.add_risk(sample_risk("r2", RiskLevel::High, RiskLevel::High));
        register.add_risk(sample_risk("r3", RiskLevel::Medium, RiskLevel::Medium));
        register.add_risk(sample_risk("r4", RiskLevel::Critical, RiskLevel::Low));
        let high = register.get_high_risks();
        assert_eq!(high.len(), 2);
        assert_eq!(high[0].id, "r2");
        assert_eq!(high[1].id, "r4");
    }

    #[test]
    fn test_risk_matrix() {
        let mut register = RiskRegister::new();
        register.add_risk(sample_risk("r1", RiskLevel::Low, RiskLevel::Low));
        register.add_risk(sample_risk("r2", RiskLevel::High, RiskLevel::Medium));
        register.add_risk(sample_risk("r3", RiskLevel::Critical, RiskLevel::Low));
        let matrix = register.risk_matrix();
        assert!(!matrix.get(&RiskLevel::Critical).unwrap().is_empty());
    }

    #[test]
    fn test_generate_report() {
        let mut register = RiskRegister::new();
        register.add_risk(sample_risk("r1", RiskLevel::High, RiskLevel::High));
        register.add_risk(sample_risk("r2", RiskLevel::Low, RiskLevel::Low));
        let report = register.generate_report();
        assert!(report.contains("Risk Assessment Report"));
        assert!(report.contains("Total Risks: 2"));
        assert!(report.contains("High/Critical"));
    }

    #[test]
    fn test_empty_register() {
        let register = RiskRegister::new();
        assert!(register.is_empty());
        let high = register.get_high_risks();
        assert!(high.is_empty());
        let report = register.generate_report();
        assert!(report.contains("Total Risks: 0"));
    }

    #[test]
    fn test_risk_level_score() {
        assert_eq!(RiskLevel::Low.score(), 1);
        assert_eq!(RiskLevel::Medium.score(), 2);
        assert_eq!(RiskLevel::High.score(), 3);
        assert_eq!(RiskLevel::Critical.score(), 4);
    }

    #[test]
    fn test_risk_status_display() {
        assert_eq!(RiskStatus::Open.to_string(), "open");
        assert_eq!(RiskStatus::Closed.to_string(), "closed");
        assert_eq!(RiskStatus::Mitigated.to_string(), "mitigated");
    }
}
