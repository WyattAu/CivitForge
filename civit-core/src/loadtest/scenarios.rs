#![forbid(unsafe_code)]

use crate::loadtest::runner::LoadTestConfig;
use std::time::Duration;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScenarioTarget {
    ApiEndpoints,
    GitOperations,
    CiPipeline,
    FederationSync,
}

impl std::fmt::Display for ScenarioTarget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ApiEndpoints => write!(f, "api-endpoints"),
            Self::GitOperations => write!(f, "git-operations"),
            Self::CiPipeline => write!(f, "ci-pipeline"),
            Self::FederationSync => write!(f, "federation-sync"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Scenario {
    pub name: String,
    pub description: String,
    pub config: LoadTestConfig,
    pub target: ScenarioTarget,
}

impl Scenario {
    pub fn new(
        name: &str,
        description: &str,
        config: LoadTestConfig,
        target: ScenarioTarget,
    ) -> Self {
        Self {
            name: name.to_owned(),
            description: description.to_owned(),
            config,
            target,
        }
    }
}

pub fn default_scenarios() -> Vec<Scenario> {
    vec![
        Scenario::new(
            "api_read_heavy",
            "Simulates heavy read traffic against API endpoints",
            LoadTestConfig::new()
                .with_concurrent_users(50)
                .with_duration(Duration::from_secs(120))
                .with_target_rps(500),
            ScenarioTarget::ApiEndpoints,
        ),
        Scenario::new(
            "api_write_heavy",
            "Simulates heavy write traffic with repository creation and updates",
            LoadTestConfig::new()
                .with_concurrent_users(20)
                .with_duration(Duration::from_secs(120))
                .with_target_rps(100),
            ScenarioTarget::ApiEndpoints,
        ),
        Scenario::new(
            "git_clone_burst",
            "Simulates burst git clone operations (short duration, high concurrency)",
            LoadTestConfig::new()
                .with_concurrent_users(100)
                .with_duration(Duration::from_secs(30))
                .with_target_rps(200),
            ScenarioTarget::GitOperations,
        ),
        Scenario::new(
            "ci_pipeline_sustained",
            "Sustained CI pipeline execution over extended period",
            LoadTestConfig::new()
                .with_concurrent_users(15)
                .with_duration(Duration::from_secs(300))
                .with_target_rps(50),
            ScenarioTarget::CiPipeline,
        ),
        Scenario::new(
            "federation_multi_node",
            "Tests federation synchronization across multiple nodes",
            LoadTestConfig::new()
                .with_concurrent_users(10)
                .with_duration(Duration::from_secs(180))
                .with_target_rps(30),
            ScenarioTarget::FederationSync,
        ),
        Scenario::new(
            "mixed_workload",
            "Combined read/write/git/CI traffic representative of production",
            LoadTestConfig::new()
                .with_concurrent_users(40)
                .with_duration(Duration::from_secs(240))
                .with_target_rps(200),
            ScenarioTarget::ApiEndpoints,
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scenario_new() {
        let s = Scenario::new(
            "test",
            "desc",
            LoadTestConfig::new(),
            ScenarioTarget::ApiEndpoints,
        );
        assert_eq!(s.name, "test");
        assert_eq!(s.description, "desc");
        assert_eq!(s.target, ScenarioTarget::ApiEndpoints);
    }

    #[test]
    fn test_scenario_target_display() {
        assert_eq!(ScenarioTarget::ApiEndpoints.to_string(), "api-endpoints");
        assert_eq!(ScenarioTarget::GitOperations.to_string(), "git-operations");
        assert_eq!(ScenarioTarget::CiPipeline.to_string(), "ci-pipeline");
        assert_eq!(
            ScenarioTarget::FederationSync.to_string(),
            "federation-sync"
        );
    }

    #[test]
    fn test_default_scenarios_count() {
        let scenarios = default_scenarios();
        assert_eq!(scenarios.len(), 6);
    }

    #[test]
    fn test_default_scenarios_names() {
        let scenarios = default_scenarios();
        let names: Vec<&str> = scenarios.iter().map(|s| s.name.as_str()).collect();
        assert!(names.contains(&"api_read_heavy"));
        assert!(names.contains(&"api_write_heavy"));
        assert!(names.contains(&"git_clone_burst"));
        assert!(names.contains(&"ci_pipeline_sustained"));
        assert!(names.contains(&"federation_multi_node"));
        assert!(names.contains(&"mixed_workload"));
    }

    #[test]
    fn test_default_scenarios_targets() {
        let scenarios = default_scenarios();
        let api_read = &scenarios[0];
        assert_eq!(api_read.target, ScenarioTarget::ApiEndpoints);
        assert_eq!(api_read.config.concurrent_users, 50);
        assert_eq!(api_read.config.target_rps, 500);

        let git_clone = scenarios
            .iter()
            .find(|s| s.name == "git_clone_burst")
            .unwrap();
        assert_eq!(git_clone.target, ScenarioTarget::GitOperations);
        assert_eq!(git_clone.config.concurrent_users, 100);
        assert_eq!(git_clone.config.duration, Duration::from_secs(30));

        let ci = scenarios
            .iter()
            .find(|s| s.name == "ci_pipeline_sustained")
            .unwrap();
        assert_eq!(ci.target, ScenarioTarget::CiPipeline);
        assert_eq!(ci.config.duration, Duration::from_secs(300));

        let fed = scenarios
            .iter()
            .find(|s| s.name == "federation_multi_node")
            .unwrap();
        assert_eq!(fed.target, ScenarioTarget::FederationSync);
    }

    #[test]
    fn test_default_scenarios_descriptions() {
        let scenarios = default_scenarios();
        for scenario in &scenarios {
            assert!(!scenario.description.is_empty());
        }
    }

    #[test]
    fn test_scenario_clone() {
        let s = Scenario::new(
            "test",
            "desc",
            LoadTestConfig::new(),
            ScenarioTarget::CiPipeline,
        );
        let cloned = s.clone();
        assert_eq!(cloned.name, s.name);
        assert_eq!(cloned.target, s.target);
    }

    #[test]
    fn test_scenario_target_equality() {
        assert_eq!(ScenarioTarget::ApiEndpoints, ScenarioTarget::ApiEndpoints);
        assert_ne!(ScenarioTarget::ApiEndpoints, ScenarioTarget::GitOperations);
    }
}
