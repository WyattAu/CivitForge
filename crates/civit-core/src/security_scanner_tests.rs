#![cfg(test)]

use super::security_scanner::*;
use chrono::Utc;

#[test]
fn test_threat_intelligence_new() {
    let threat = ThreatIntelligence::new(
        "CVE-2024-1234".into(),
        ThreatSeverity::Critical,
        "Critical RCE in libfoo".into(),
    );
    assert_eq!(threat.cve_id, "CVE-2024-1234");
    assert!(threat.is_critical());
    assert!(threat.affected_packages.is_empty());
    assert!(!threat.fix_available);
}

#[test]
fn test_threat_intelligence_with_packages() {
    let threat = ThreatIntelligence::new(
        "CVE-2024-5678".into(),
        ThreatSeverity::High,
        "SQL injection".into(),
    )
    .with_packages(vec!["libc".into(), "openssl".into()])
    .with_fix_available(true);

    assert!(threat.affects_package("libc"));
    assert!(threat.affects_package("openssl"));
    assert!(!threat.affects_package("other"));
    assert!(threat.fix_available);
}

#[test]
fn test_threat_intelligence_with_published_at() {
    let pub_date = Utc::now();
    let threat = ThreatIntelligence::new(
        "CVE-2024-9999".into(),
        ThreatSeverity::Medium,
        "XSS".into(),
    )
    .with_published_at(pub_date);

    assert!(threat.published_at.is_some());
}

#[test]
fn test_threat_intelligence_not_critical() {
    let threat = ThreatIntelligence::new(
        "CVE-2024-0001".into(),
        ThreatSeverity::Low,
        "Minor issue".into(),
    );
    assert!(!threat.is_critical());
}

#[test]
fn test_threat_severity_risk_weights() {
    assert_eq!(ThreatSeverity::Critical.risk_weight(), 5);
    assert_eq!(ThreatSeverity::High.risk_weight(), 4);
    assert_eq!(ThreatSeverity::Medium.risk_weight(), 2);
    assert_eq!(ThreatSeverity::Low.risk_weight(), 1);
    assert_eq!(ThreatSeverity::Informational.risk_weight(), 0);
}

#[test]
fn test_threat_severity_display_names() {
    assert_eq!(ThreatSeverity::Critical.display_name(), "Critical");
    assert_eq!(ThreatSeverity::High.display_name(), "High");
    assert_eq!(ThreatSeverity::Medium.display_name(), "Medium");
    assert_eq!(ThreatSeverity::Low.display_name(), "Low");
    assert_eq!(ThreatSeverity::Informational.display_name(), "Informational");
}

#[test]
fn test_dependency_tree_node_new() {
    let node = DependencyTreeNode::new("repo-1".into(), "serde".into(), "1.0.0".into());
    assert_eq!(node.package_name, "serde");
    assert_eq!(node.version, "1.0.0");
    assert_eq!(node.repo_id, "repo-1");
    assert_eq!(node.depth, 0);
    assert!(!node.is_transitive());
    assert!(node.parent_package.is_none());
    assert_eq!(node.dependency_type, DependencyType::Direct);
}

#[test]
fn test_dependency_tree_node_with_parent() {
    let node = DependencyTreeNode::new("repo-1".into(), "serde_derive".into(), "1.0.0".into())
        .with_parent("serde".into(), DependencyType::Transitive)
        .with_depth(2);

    assert!(node.is_transitive());
    assert_eq!(node.depth, 2);
    assert_eq!(node.parent_package.as_deref(), Some("serde"));
}

#[test]
fn test_dependency_tree_node_dev() {
    let node = DependencyTreeNode::new("repo-1".into(), "cargo-test".into(), "0.1.0".into())
        .with_parent("project".into(), DependencyType::Dev);

    assert_eq!(node.dependency_type, DependencyType::Dev);
    assert!(!node.is_transitive());
}

#[test]
fn test_dependency_type_display_names() {
    assert_eq!(DependencyType::Direct.display_name(), "direct");
    assert_eq!(DependencyType::Transitive.display_name(), "transitive");
    assert_eq!(DependencyType::Dev.display_name(), "dev");
    assert_eq!(DependencyType::Optional.display_name(), "optional");
    assert_eq!(DependencyType::Peer.display_name(), "peer");
}

#[test]
fn test_vulnerability_correlation_new() {
    let corr = VulnerabilityCorrelation::new(
        "threat-1".into(),
        "dep-1".into(),
        "CVE-2024-1111".into(),
        "libc".into(),
        "2.31".into(),
    );
    assert!(corr.is_high_confidence());
    assert_eq!(corr.correlation_confidence, 1.0);
    assert!(corr.fix_version.is_none());
}

#[test]
fn test_vulnerability_correlation_with_fix() {
    let corr = VulnerabilityCorrelation::new(
        "t1".into(),
        "d1".into(),
        "CVE-2024-2222".into(),
        "openssl".into(),
        "1.1.1".into(),
    )
    .with_fix_version("1.1.2".into())
    .with_confidence(0.75);

    assert_eq!(corr.fix_version.as_deref(), Some("1.1.2"));
    assert!(!corr.is_high_confidence());
}

#[test]
fn test_vulnerability_correlation_confidence_clamping() {
    let corr = VulnerabilityCorrelation::new(
        "t1".into(),
        "d1".into(),
        "CVE-1".into(),
        "pkg".into(),
        "1.0".into(),
    )
    .with_confidence(1.5);

    assert_eq!(corr.correlation_confidence, 1.0);

    let corr = corr.with_confidence(-0.5);
    assert_eq!(corr.correlation_confidence, 0.0);
}

#[test]
fn test_risk_score_new() {
    let score = RiskScore::new("libc".into(), "repo-1".into());
    assert_eq!(score.overall_score, 0.0);
    assert_eq!(score.vulnerability_count, 0);
    assert_eq!(score.risk_level(), "informational");
}

#[test]
fn test_risk_score_calculate() {
    let mut score = RiskScore::new("libc".into(), "repo-1".into());
    let corrs = vec![
        VulnerabilityCorrelation::new(
            "t1".into(), "d1".into(), "CVE-1".into(), "libc".into(), "2.31".into(),
        ),
        VulnerabilityCorrelation::new(
            "t2".into(), "d2".into(), "CVE-2".into(), "libc".into(), "2.31".into(),
        ),
    ];
    score.calculate(&corrs);
    assert_eq!(score.vulnerability_count, 2);
    assert!(score.overall_score > 0.0);
}

#[test]
fn test_risk_score_levels() {
    let mut score = RiskScore::new("pkg".into(), "repo-1".into());
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
fn test_risk_score_calculate_critical_boost() {
    let mut score = RiskScore::new("pkg".into(), "repo-1".into());
    let corrs = vec![VulnerabilityCorrelation::new(
        "t1".into(),
        "d1".into(),
        "CVE-CRITICAL-1".into(),
        "pkg".into(),
        "1.0".into(),
    )];
    score.calculate(&corrs);
    assert_eq!(score.critical_count, 1);
    assert!(score.overall_score > 20.0);
}

#[test]
fn test_risk_score_calculate_high_boost() {
    let mut score = RiskScore::new("pkg".into(), "repo-1".into());
    let corr = VulnerabilityCorrelation::new(
        "t1".into(),
        "d1".into(),
        "CVE-NORMAL".into(),
        "pkg".into(),
        "1.0".into(),
    )
    .with_confidence(0.8);
    score.calculate(&[corr]);
    assert_eq!(score.high_count, 1);
}

#[test]
fn test_threat_store_empty() {
    let store = ThreatIntelligenceStore::new();
    assert_eq!(store.total(), 0);
    assert!(store.get_by_cve("CVE-1").is_empty());
    assert!(store.critical_threats().is_empty());
}

#[test]
fn test_threat_store_add_and_query() {
    let mut store = ThreatIntelligenceStore::new();
    let threat = ThreatIntelligence::new(
        "CVE-1".into(),
        ThreatSeverity::Critical,
        "Critical vuln".into(),
    )
    .with_packages(vec!["libc".into(), "openssl".into()]);

    store.add_threat(threat);
    assert_eq!(store.total(), 1);
    assert_eq!(store.get_by_cve("CVE-1").len(), 1);
    assert_eq!(store.get_for_package("libc").len(), 1);
    assert_eq!(store.get_for_package("openssl").len(), 1);
    assert_eq!(store.critical_threats().len(), 1);
}

#[test]
fn test_threat_store_multiple_threats() {
    let mut store = ThreatIntelligenceStore::new();
    store.add_threat(ThreatIntelligence::new(
        "CVE-1".into(),
        ThreatSeverity::Critical,
        "d".into(),
    ));
    store.add_threat(ThreatIntelligence::new(
        "CVE-2".into(),
        ThreatSeverity::High,
        "d".into(),
    ));
    store.add_threat(ThreatIntelligence::new(
        "CVE-1".into(),
        ThreatSeverity::Medium,
        "d2".into(),
    ));

    assert_eq!(store.total(), 3);
    assert_eq!(store.get_by_cve("CVE-1").len(), 2);
    assert_eq!(store.get_by_cve("CVE-2").len(), 1);
    assert_eq!(store.critical_threats().len(), 1);
}

#[test]
fn test_dependency_tree_analyzer_empty() {
    let analyzer = DependencyTreeAnalyzer::new();
    assert!(analyzer.get_nodes_for_repo("repo-1").is_empty());
    assert!(analyzer.get_children("pkg").is_empty());
    assert_eq!(analyzer.max_depth_for_repo("repo-1"), 0);
    assert_eq!(analyzer.total_packages_for_repo("repo-1"), 0);
}

#[test]
fn test_dependency_tree_analyzer_tree_structure() {
    let mut analyzer = DependencyTreeAnalyzer::new();

    let root = DependencyTreeNode::new("repo-1".into(), "root".into(), "1.0".into());
    let child1 = DependencyTreeNode::new("repo-1".into(), "child1".into(), "2.0".into())
        .with_parent("root".into(), DependencyType::Direct)
        .with_depth(1);
    let child2 = DependencyTreeNode::new("repo-1".into(), "child2".into(), "3.0".into())
        .with_parent("root".into(), DependencyType::Direct)
        .with_depth(1);
    let grandchild = DependencyTreeNode::new("repo-1".into(), "grandchild".into(), "4.0".into())
        .with_parent("child1".into(), DependencyType::Transitive)
        .with_depth(2);

    analyzer.add_node(root);
    analyzer.add_node(child1);
    analyzer.add_node(child2);
    analyzer.add_node(grandchild);

    assert_eq!(analyzer.get_nodes_for_repo("repo-1").len(), 4);
    assert_eq!(analyzer.get_children("root").len(), 2);
    assert_eq!(analyzer.get_children("child1").len(), 1);
    assert_eq!(analyzer.max_depth_for_repo("repo-1"), 2);
    assert_eq!(analyzer.total_packages_for_repo("repo-1"), 4);

    let found = analyzer.find_package("child1", "repo-1");
    assert!(found.is_some());
    assert_eq!(found.unwrap().version, "2.0");

    assert!(analyzer.find_package("nonexistent", "repo-1").is_none());
}

#[test]
fn test_dependency_tree_analyzer_multi_repo() {
    let mut analyzer = DependencyTreeAnalyzer::new();
    analyzer.add_node(DependencyTreeNode::new("repo-1".into(), "a".into(), "1.0".into()));
    analyzer.add_node(DependencyTreeNode::new("repo-2".into(), "b".into(), "1.0".into()));

    assert_eq!(analyzer.get_nodes_for_repo("repo-1").len(), 1);
    assert_eq!(analyzer.get_nodes_for_repo("repo-2").len(), 1);
    assert_eq!(analyzer.get_nodes_for_repo("repo-3").len(), 0);
}

#[test]
fn test_correlation_engine_empty() {
    let engine = VulnerabilityCorrelationEngine::new();
    assert_eq!(engine.total(), 0);
    assert!(engine.get_correlations_for_threat("t1").is_empty());
    assert!(engine.get_correlations_for_dependency("d1").is_empty());
    assert!(engine.high_confidence_correlations().is_empty());
}

#[test]
fn test_correlation_engine_add_and_query() {
    let mut engine = VulnerabilityCorrelationEngine::new();
    let corr = VulnerabilityCorrelation::new(
        "t1".into(),
        "d1".into(),
        "CVE-1".into(),
        "pkg".into(),
        "1.0".into(),
    );
    engine.add_correlation(corr);

    assert_eq!(engine.total(), 1);
    assert_eq!(engine.get_correlations_for_threat("t1").len(), 1);
    assert_eq!(engine.get_correlations_for_dependency("d1").len(), 1);
    assert_eq!(engine.high_confidence_correlations().len(), 1);
}

#[test]
fn test_correlation_engine_low_confidence() {
    let mut engine = VulnerabilityCorrelationEngine::new();
    let corr = VulnerabilityCorrelation::new(
        "t1".into(),
        "d1".into(),
        "CVE-1".into(),
        "pkg".into(),
        "1.0".into(),
    )
    .with_confidence(0.5);

    engine.add_correlation(corr);
    assert!(engine.high_confidence_correlations().is_empty());
}

#[test]
fn test_correlation_engine_multiple_threats_for_dependency() {
    let mut engine = VulnerabilityCorrelationEngine::new();
    engine.add_correlation(VulnerabilityCorrelation::new(
        "t1".into(), "d1".into(), "CVE-1".into(), "pkg".into(), "1.0".into(),
    ));
    engine.add_correlation(VulnerabilityCorrelation::new(
        "t2".into(), "d1".into(), "CVE-2".into(), "pkg".into(), "1.0".into(),
    ));

    assert_eq!(engine.get_correlations_for_dependency("d1").len(), 2);
    assert_eq!(engine.get_correlations_for_threat("t1").len(), 1);
}

#[test]
fn test_risk_scoring_engine_empty() {
    let engine = RiskScoringEngine::new();
    assert!(engine.get_score("pkg", "repo-1").is_none());
    assert!(engine.highest_risk_packages("repo-1").is_empty());
}

#[test]
fn test_risk_scoring_engine_calculate_and_get() {
    let mut engine = RiskScoringEngine::new();
    let corrs = vec![
        VulnerabilityCorrelation::new(
            "t1".into(), "d1".into(), "CVE-1".into(), "libc".into(), "2.31".into(),
        ),
    ];
    let score = engine.calculate_score("libc", "repo-1", &corrs);
    assert!(score.overall_score > 0.0);

    let retrieved = engine.get_score("libc", "repo-1");
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().overall_score, score.overall_score);
}

#[test]
fn test_risk_scoring_engine_highest_risk() {
    let mut engine = RiskScoringEngine::new();
    let corrs_low = vec![VulnerabilityCorrelation::new(
        "t1".into(), "d1".into(), "CVE-1".into(), "low-pkg".into(), "1.0".into(),
    )];
    let corrs_high = vec![
        VulnerabilityCorrelation::new(
            "t2".into(), "d2".into(), "CVE-CRITICAL".into(), "high-pkg".into(), "1.0".into(),
        ),
        VulnerabilityCorrelation::new(
            "t3".into(), "d3".into(), "CVE-2".into(), "high-pkg".into(), "1.0".into(),
        ),
    ];

    engine.calculate_score("low-pkg", "repo-1", &corrs_low);
    engine.calculate_score("high-pkg", "repo-1", &corrs_high);

    let highest = engine.highest_risk_packages("repo-1");
    assert_eq!(highest.len(), 2);
    assert!(highest[0].overall_score >= highest[1].overall_score);
}

#[test]
fn test_risk_scoring_engine_repo_summary() {
    let mut engine = RiskScoringEngine::new();
    let corrs = vec![VulnerabilityCorrelation::new(
        "t1".into(), "d1".into(), "CVE-1".into(), "pkg".into(), "1.0".into(),
    )];
    engine.calculate_score("pkg", "repo-1", &corrs);

    let summary = engine.repo_risk_summary("repo-1");
    assert_eq!(summary.repo_id, "repo-1");
    assert_eq!(summary.total_packages, 1);
    assert!(summary.average_risk_score > 0.0);
}

#[test]
fn test_risk_scoring_engine_repo_summary_empty() {
    let engine = RiskScoringEngine::new();
    let summary = engine.repo_risk_summary("repo-1");
    assert_eq!(summary.total_packages, 0);
    assert_eq!(summary.average_risk_score, 0.0);
}
