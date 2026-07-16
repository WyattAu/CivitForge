#![forbid(unsafe_code)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreatIntelligenceV24 {
    pub id: String,
    pub cve_id: String,
    pub severity: ThreatSeverityV24,
    pub description: String,
    pub affected_packages: Vec<String>,
    pub fix_available: bool,
    pub published_at: Option<DateTime<Utc>>,
    pub fetched_at: DateTime<Utc>,
}

impl ThreatIntelligenceV24 {
    pub fn new(cve_id: String, severity: ThreatSeverityV24, description: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            cve_id,
            severity,
            description,
            affected_packages: Vec::new(),
            fix_available: false,
            published_at: None,
            fetched_at: Utc::now(),
        }
    }

    pub fn with_packages(mut self, packages: Vec<String>) -> Self {
        self.affected_packages = packages;
        self
    }

    pub fn with_fix_available(mut self, fix: bool) -> Self {
        self.fix_available = fix;
        self
    }

    pub fn with_published_at(mut self, published_at: DateTime<Utc>) -> Self {
        self.published_at = Some(published_at);
        self
    }

    pub fn is_critical(&self) -> bool {
        self.severity == ThreatSeverityV24::Critical
    }

    pub fn affects_package(&self, package: &str) -> bool {
        self.affected_packages.iter().any(|p| p == package)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ThreatSeverityV24 {
    Critical,
    High,
    Medium,
    Low,
    Informational,
}

impl ThreatSeverityV24 {
    pub fn risk_weight(&self) -> u32 {
        match self {
            Self::Critical => 5,
            Self::High => 4,
            Self::Medium => 2,
            Self::Low => 1,
            Self::Informational => 0,
        }
    }

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyTreeNodeV24 {
    pub id: String,
    pub repo_id: String,
    pub package_name: String,
    pub version: String,
    pub parent_package: Option<String>,
    pub dependency_type: DependencyTypeV24,
    pub depth: u32,
    pub scanned_at: DateTime<Utc>,
}

impl DependencyTreeNodeV24 {
    pub fn new(
        repo_id: String,
        package_name: String,
        version: String,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            repo_id,
            package_name,
            version,
            parent_package: None,
            dependency_type: DependencyTypeV24::Direct,
            depth: 0,
            scanned_at: Utc::now(),
        }
    }

    pub fn with_parent(mut self, parent: String, dep_type: DependencyTypeV24) -> Self {
        self.parent_package = Some(parent);
        self.dependency_type = dep_type;
        self
    }

    pub fn with_depth(mut self, depth: u32) -> Self {
        self.depth = depth;
        self
    }

    pub fn is_transitive(&self) -> bool {
        self.dependency_type == DependencyTypeV24::Transitive
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DependencyTypeV24 {
    Direct,
    Transitive,
    Dev,
    Optional,
    Peer,
}

impl DependencyTypeV24 {
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Direct => "direct",
            Self::Transitive => "transitive",
            Self::Dev => "dev",
            Self::Optional => "optional",
            Self::Peer => "peer",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VulnerabilityCorrelationV24 {
    pub threat_intel_id: String,
    pub dependency_node_id: String,
    pub cve_id: String,
    pub package_name: String,
    pub affected_version: String,
    pub fix_version: Option<String>,
    pub correlation_confidence: f64,
    pub correlated_at: DateTime<Utc>,
}

impl VulnerabilityCorrelationV24 {
    pub fn new(
        threat_intel_id: String,
        dependency_node_id: String,
        cve_id: String,
        package_name: String,
        affected_version: String,
    ) -> Self {
        Self {
            threat_intel_id,
            dependency_node_id,
            cve_id,
            package_name,
            affected_version,
            fix_version: None,
            correlation_confidence: 1.0,
            correlated_at: Utc::now(),
        }
    }

    pub fn with_fix_version(mut self, fix: String) -> Self {
        self.fix_version = Some(fix);
        self
    }

    pub fn with_confidence(mut self, confidence: f64) -> Self {
        self.correlation_confidence = confidence.clamp(0.0, 1.0);
        self
    }

    pub fn is_high_confidence(&self) -> bool {
        self.correlation_confidence >= 0.8
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskScoreV24 {
    pub package_name: String,
    pub repo_id: String,
    pub overall_score: f64,
    pub vulnerability_count: u32,
    pub critical_count: u32,
    pub high_count: u32,
    pub transitive_risk: f64,
    pub last_calculated: DateTime<Utc>,
}

impl RiskScoreV24 {
    pub fn new(package_name: String, repo_id: String) -> Self {
        Self {
            package_name,
            repo_id,
            overall_score: 0.0,
            vulnerability_count: 0,
            critical_count: 0,
            high_count: 0,
            transitive_risk: 0.0,
            last_calculated: Utc::now(),
        }
    }

    pub fn calculate(&mut self, correlations: &[VulnerabilityCorrelationV24]) {
        self.vulnerability_count = correlations.len() as u32;
        self.critical_count = 0;
        self.high_count = 0;

        let mut score = 0.0;
        for corr in correlations {
            score += corr.correlation_confidence * 20.0;
            if corr.cve_id.contains("CRITICAL") || corr.correlation_confidence > 0.9 {
                self.critical_count += 1;
                score += 15.0;
            } else if corr.correlation_confidence > 0.7 {
                self.high_count += 1;
                score += 8.0;
            }
        }

        self.overall_score = score.min(100.0);
        self.last_calculated = Utc::now();
    }

    pub fn risk_level(&self) -> &'static str {
        if self.overall_score >= 80.0 {
            "critical"
        } else if self.overall_score >= 60.0 {
            "high"
        } else if self.overall_score >= 40.0 {
            "medium"
        } else if self.overall_score >= 20.0 {
            "low"
        } else {
            "informational"
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ThreatIntelligenceStoreV24 {
    threats: Vec<ThreatIntelligenceV24>,
    by_cve: HashMap<String, Vec<usize>>,
    by_package: HashMap<String, Vec<usize>>,
}

impl ThreatIntelligenceStoreV24 {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_threat(&mut self, threat: ThreatIntelligenceV24) {
        let idx = self.threats.len();
        self.by_cve
            .entry(threat.cve_id.clone())
            .or_default()
            .push(idx);
        for pkg in &threat.affected_packages {
            self.by_package
                .entry(pkg.clone())
                .or_default()
                .push(idx);
        }
        self.threats.push(threat);
    }

    pub fn get_by_cve(&self, cve_id: &str) -> Vec<&ThreatIntelligenceV24> {
        self.by_cve
            .get(cve_id)
            .map(|indices| indices.iter().map(|&idx| &self.threats[idx]).collect())
            .unwrap_or_default()
    }

    pub fn get_for_package(&self, package: &str) -> Vec<&ThreatIntelligenceV24> {
        self.by_package
            .get(package)
            .map(|indices| indices.iter().map(|&idx| &self.threats[idx]).collect())
            .unwrap_or_default()
    }

    pub fn critical_threats(&self) -> Vec<&ThreatIntelligenceV24> {
        self.threats
            .iter()
            .filter(|t| t.is_critical())
            .collect()
    }

    pub fn total(&self) -> usize {
        self.threats.len()
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DependencyTreeAnalyzerV24 {
    nodes: Vec<DependencyTreeNodeV24>,
    by_repo: HashMap<String, Vec<usize>>,
    by_package: HashMap<String, Vec<usize>>,
    parent_index: HashMap<String, Vec<usize>>,
}

impl DependencyTreeAnalyzerV24 {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_node(&mut self, node: DependencyTreeNodeV24) {
        let idx = self.nodes.len();
        self.by_repo
            .entry(node.repo_id.clone())
            .or_default()
            .push(idx);
        self.by_package
            .entry(node.package_name.clone())
            .or_default()
            .push(idx);
        if let Some(ref parent) = node.parent_package {
            self.parent_index
                .entry(parent.clone())
                .or_default()
                .push(idx);
        }
        self.nodes.push(node);
    }

    pub fn get_nodes_for_repo(&self, repo_id: &str) -> Vec<&DependencyTreeNodeV24> {
        self.by_repo
            .get(repo_id)
            .map(|indices| indices.iter().map(|&idx| &self.nodes[idx]).collect())
            .unwrap_or_default()
    }

    pub fn get_children(&self, package_name: &str) -> Vec<&DependencyTreeNodeV24> {
        self.parent_index
            .get(package_name)
            .map(|indices| indices.iter().map(|&idx| &self.nodes[idx]).collect())
            .unwrap_or_default()
    }

    pub fn find_package(&self, name: &str, repo_id: &str) -> Option<&DependencyTreeNodeV24> {
        self.nodes.iter().find(|n| n.package_name == name && n.repo_id == repo_id)
    }

    pub fn max_depth_for_repo(&self, repo_id: &str) -> u32 {
        self.get_nodes_for_repo(repo_id)
            .iter()
            .map(|n| n.depth)
            .max()
            .unwrap_or(0)
    }

    pub fn total_packages_for_repo(&self, repo_id: &str) -> usize {
        self.get_nodes_for_repo(repo_id).len()
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct VulnerabilityCorrelationEngineV24 {
    correlations: Vec<VulnerabilityCorrelationV24>,
    by_threat: HashMap<String, Vec<usize>>,
    by_dependency: HashMap<String, Vec<usize>>,
}

impl VulnerabilityCorrelationEngineV24 {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_correlation(&mut self, corr: VulnerabilityCorrelationV24) {
        let idx = self.correlations.len();
        self.by_threat
            .entry(corr.threat_intel_id.clone())
            .or_default()
            .push(idx);
        self.by_dependency
            .entry(corr.dependency_node_id.clone())
            .or_default()
            .push(idx);
        self.correlations.push(corr);
    }

    pub fn get_correlations_for_threat(
        &self,
        threat_id: &str,
    ) -> Vec<&VulnerabilityCorrelationV24> {
        self.by_threat
            .get(threat_id)
            .map(|indices| indices.iter().map(|&idx| &self.correlations[idx]).collect())
            .unwrap_or_default()
    }

    pub fn get_correlations_for_dependency(
        &self,
        dep_id: &str,
    ) -> Vec<&VulnerabilityCorrelationV24> {
        self.by_dependency
            .get(dep_id)
            .map(|indices| indices.iter().map(|&idx| &self.correlations[idx]).collect())
            .unwrap_or_default()
    }

    pub fn high_confidence_correlations(&self) -> Vec<&VulnerabilityCorrelationV24> {
        self.correlations
            .iter()
            .filter(|c| c.is_high_confidence())
            .collect()
    }

    pub fn total(&self) -> usize {
        self.correlations.len()
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RiskScoringEngineV24 {
    scores: HashMap<String, RiskScoreV24>,
}

impl RiskScoringEngineV24 {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn calculate_score(
        &mut self,
        package_name: &str,
        repo_id: &str,
        correlations: &[VulnerabilityCorrelationV24],
    ) -> RiskScoreV24 {
        let key = format!("{}:{}", repo_id, package_name);
        let mut score = RiskScoreV24::new(package_name.into(), repo_id.into());
        score.calculate(correlations);
        self.scores.insert(key, score.clone());
        score
    }

    pub fn get_score(&self, package_name: &str, repo_id: &str) -> Option<&RiskScoreV24> {
        let key = format!("{}:{}", repo_id, package_name);
        self.scores.get(&key)
    }

    pub fn highest_risk_packages(&self, repo_id: &str) -> Vec<&RiskScoreV24> {
        let mut scores: Vec<_> = self
            .scores
            .values()
            .filter(|s| s.repo_id == repo_id)
            .collect();
        scores.sort_by(|a, b| b.overall_score.partial_cmp(&a.overall_score).unwrap());
        scores
    }

    pub fn repo_risk_summary(&self, repo_id: &str) -> RepoRiskSummaryV24 {
        let repo_scores: Vec<_> = self
            .scores
            .values()
            .filter(|s| s.repo_id == repo_id)
            .collect();
        let total_packages = repo_scores.len() as u32;
        let total_vulns: u32 = repo_scores.iter().map(|s| s.vulnerability_count).sum();
        let critical: u32 = repo_scores.iter().map(|s| s.critical_count).sum();
        let avg_score = if total_packages == 0 {
            0.0
        } else {
            repo_scores.iter().map(|s| s.overall_score).sum::<f64>() / total_packages as f64
        };

        RepoRiskSummaryV24 {
            repo_id: repo_id.into(),
            total_packages,
            total_vulnerabilities: total_vulns,
            critical_vulnerabilities: critical,
            average_risk_score: avg_score,
            calculated_at: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoRiskSummaryV24 {
    pub repo_id: String,
    pub total_packages: u32,
    pub total_vulnerabilities: u32,
    pub critical_vulnerabilities: u32,
    pub average_risk_score: f64,
    pub calculated_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_threat_intelligence_v24_new() {
        let threat = ThreatIntelligenceV24::new(
            "CVE-2024-0001".into(),
            ThreatSeverityV24::Critical,
            "Test vuln".into(),
        );
        assert_eq!(threat.cve_id, "CVE-2024-0001");
        assert!(threat.is_critical());
    }

    #[test]
    fn test_threat_intelligence_v24_affects_package() {
        let threat = ThreatIntelligenceV24::new(
            "CVE-2024-0001".into(),
            ThreatSeverityV24::High,
            "Desc".into(),
        )
        .with_packages(vec!["libc".into(), "openssl".into()]);
        assert!(threat.affects_package("libc"));
        assert!(!threat.affects_package("other"));
    }

    #[test]
    fn test_dependency_tree_node_v24_new() {
        let node = DependencyTreeNodeV24::new("repo-1".into(), "serde".into(), "1.0".into());
        assert_eq!(node.package_name, "serde");
        assert!(!node.is_transitive());
        assert_eq!(node.depth, 0);
    }

    #[test]
    fn test_dependency_tree_node_v24_transitive() {
        let node = DependencyTreeNodeV24::new("repo-1".into(), "serde_derive".into(), "1.0".into())
            .with_parent("serde".into(), DependencyTypeV24::Transitive)
            .with_depth(2);
        assert!(node.is_transitive());
        assert_eq!(node.depth, 2);
    }

    #[test]
    fn test_vulnerability_correlation_v24_new() {
        let corr = VulnerabilityCorrelationV24::new(
            "t1".into(),
            "d1".into(),
            "CVE-2024-0001".into(),
            "libc".into(),
            "2.31".into(),
        );
        assert!(corr.is_high_confidence());
    }

    #[test]
    fn test_risk_score_v24_calculate() {
        let mut score = RiskScoreV24::new("libc".into(), "repo-1".into());
        let corrs = vec![
            VulnerabilityCorrelationV24::new("t1".into(), "d1".into(), "CVE-1".into(), "libc".into(), "2.31".into()),
            VulnerabilityCorrelationV24::new("t2".into(), "d2".into(), "CVE-2".into(), "libc".into(), "2.31".into()),
        ];
        score.calculate(&corrs);
        assert_eq!(score.vulnerability_count, 2);
        assert!(score.overall_score > 0.0);
    }

    #[test]
    fn test_risk_score_v24_risk_level() {
        let mut score = RiskScoreV24::new("pkg".into(), "repo-1".into());
        score.overall_score = 85.0;
        assert_eq!(score.risk_level(), "critical");
        score.overall_score = 65.0;
        assert_eq!(score.risk_level(), "high");
        score.overall_score = 45.0;
        assert_eq!(score.risk_level(), "medium");
        score.overall_score = 25.0;
        assert_eq!(score.risk_level(), "low");
        score.overall_score = 5.0;
        assert_eq!(score.risk_level(), "informational");
    }

    #[test]
    fn test_threat_intelligence_store_v24() {
        let mut store = ThreatIntelligenceStoreV24::new();
        let threat = ThreatIntelligenceV24::new(
            "CVE-1".into(),
            ThreatSeverityV24::Critical,
            "Desc".into(),
        )
        .with_packages(vec!["libc".into()]);
        store.add_threat(threat);
        assert_eq!(store.total(), 1);
        assert_eq!(store.get_for_package("libc").len(), 1);
        assert_eq!(store.critical_threats().len(), 1);
    }

    #[test]
    fn test_dependency_tree_analyzer_v24() {
        let mut analyzer = DependencyTreeAnalyzerV24::new();
        let root = DependencyTreeNodeV24::new("repo-1".into(), "root".into(), "1.0".into());
        let child = DependencyTreeNodeV24::new("repo-1".into(), "child".into(), "2.0".into())
            .with_parent("root".into(), DependencyTypeV24::Direct)
            .with_depth(1);
        analyzer.add_node(root);
        analyzer.add_node(child);
        assert_eq!(analyzer.get_nodes_for_repo("repo-1").len(), 2);
        assert_eq!(analyzer.get_children("root").len(), 1);
        assert_eq!(analyzer.max_depth_for_repo("repo-1"), 1);
    }

    #[test]
    fn test_vulnerability_correlation_engine_v24() {
        let mut engine = VulnerabilityCorrelationEngineV24::new();
        let corr = VulnerabilityCorrelationV24::new(
            "t1".into(), "d1".into(), "CVE-1".into(), "pkg".into(), "1.0".into(),
        );
        engine.add_correlation(corr);
        assert_eq!(engine.total(), 1);
        assert_eq!(engine.get_correlations_for_threat("t1").len(), 1);
        assert_eq!(engine.high_confidence_correlations().len(), 1);
    }

    #[test]
    fn test_risk_scoring_engine_v24() {
        let mut engine = RiskScoringEngineV24::new();
        let corrs = vec![
            VulnerabilityCorrelationV24::new("t1".into(), "d1".into(), "CVE-1".into(), "pkg".into(), "1.0".into()),
        ];
        let score = engine.calculate_score("pkg", "repo-1", &corrs);
        assert!(score.overall_score > 0.0);
        assert!(engine.get_score("pkg", "repo-1").is_some());
        let summary = engine.repo_risk_summary("repo-1");
        assert_eq!(summary.total_packages, 1);
    }

    #[test]
    fn test_threat_severity_v24_risk_weight() {
        assert_eq!(ThreatSeverityV24::Critical.risk_weight(), 5);
        assert_eq!(ThreatSeverityV24::High.risk_weight(), 4);
        assert_eq!(ThreatSeverityV24::Medium.risk_weight(), 2);
        assert_eq!(ThreatSeverityV24::Low.risk_weight(), 1);
        assert_eq!(ThreatSeverityV24::Informational.risk_weight(), 0);
    }

    #[test]
    fn test_dependency_type_v24_display() {
        assert_eq!(DependencyTypeV24::Direct.display_name(), "direct");
        assert_eq!(DependencyTypeV24::Transitive.display_name(), "transitive");
    }
}
