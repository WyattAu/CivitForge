#![forbid(unsafe_code)]

use std::collections::HashMap;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct ComponentHealth {
    pub name: String,
    pub healthy: bool,
    pub response_time_ms: u64,
    pub error: Option<String>,
}

#[derive(Debug, Clone)]
pub struct HealthCondition {
    pub component: String,
    pub status: bool,
    pub message: String,
}

#[derive(Clone)]
pub struct DeploymentHealthChecker {
    base_url: String,
    timeout_ms: u64,
    http_client: std::sync::Arc<reqwest::Client>,
}

impl std::fmt::Debug for DeploymentHealthChecker {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DeploymentHealthChecker")
            .field("base_url", &self.base_url)
            .field("timeout_ms", &self.timeout_ms)
            .finish()
    }
}

impl DeploymentHealthChecker {
    pub fn new(base_url: String) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            timeout_ms: 10_000,
            http_client: std::sync::Arc::new(client),
        }
    }

    pub fn with_timeout(mut self, timeout_ms: u64) -> Self {
        self.timeout_ms = timeout_ms;
        let client = reqwest::Client::builder()
            .timeout(Duration::from_millis(timeout_ms))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());
        self.http_client = std::sync::Arc::new(client);
        self
    }

    pub async fn check_all(&self) -> Vec<ComponentHealth> {
        let mut checks = Vec::with_capacity(3);

        let web = self
            .check_endpoint("web", &format!("{}/health", self.base_url))
            .await;
        checks.push(web);

        let registry = self
            .check_endpoint("registry", &format!("{}/v2/", self.base_url))
            .await;
        checks.push(registry);

        let brain = self
            .check_endpoint("brain", &format!("{}/api/v1/health", self.base_url))
            .await;
        checks.push(brain);

        checks
    }

    async fn check_endpoint(&self, name: &str, url: &str) -> ComponentHealth {
        let start = Instant::now();
        match self.http_client.get(url).send().await {
            Ok(resp) => {
                let elapsed = start.elapsed().as_millis() as u64;
                let status_code = resp.status().as_u16();
                let healthy =
                    resp.status().is_success() || (name == "registry" && status_code == 401);
                ComponentHealth {
                    name: name.to_string(),
                    healthy,
                    response_time_ms: elapsed,
                    error: if healthy {
                        None
                    } else {
                        Some(format!("HTTP {status_code}"))
                    },
                }
            }
            Err(e) => {
                let elapsed = start.elapsed().as_millis() as u64;
                ComponentHealth {
                    name: name.to_string(),
                    healthy: false,
                    response_time_ms: elapsed,
                    error: Some(e.to_string()),
                }
            }
        }
    }

    pub fn is_healthy(checks: &[ComponentHealth]) -> bool {
        checks.iter().all(|c| c.healthy)
    }

    pub fn to_conditions(checks: &[ComponentHealth]) -> Vec<HealthCondition> {
        checks
            .iter()
            .map(|c| HealthCondition {
                component: c.name.clone(),
                status: c.healthy,
                message: c.error.clone().unwrap_or_else(|| "healthy".to_string()),
            })
            .collect()
    }

    pub fn summary(checks: &[ComponentHealth]) -> HashMap<String, bool> {
        checks.iter().map(|c| (c.name.clone(), c.healthy)).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_checker_new() {
        let checker = DeploymentHealthChecker::new("http://localhost:8080".into());
        assert_eq!(checker.base_url, "http://localhost:8080");
        assert_eq!(checker.timeout_ms, 10_000);
    }

    #[test]
    fn test_checker_trims_trailing_slash() {
        let checker = DeploymentHealthChecker::new("http://localhost:8080/".into());
        assert_eq!(checker.base_url, "http://localhost:8080");
    }

    #[test]
    fn test_checker_trims_multiple_slashes() {
        let checker = DeploymentHealthChecker::new("http://localhost:8080///".into());
        assert_eq!(checker.base_url, "http://localhost:8080");
    }

    #[test]
    fn test_checker_with_timeout() {
        let checker =
            DeploymentHealthChecker::new("http://localhost:8080".into()).with_timeout(5000);
        assert_eq!(checker.timeout_ms, 5000);
    }

    #[test]
    fn test_checker_debug() {
        let checker = DeploymentHealthChecker::new("http://localhost:8080".into());
        let debug = format!("{checker:?}");
        assert!(debug.contains("http://localhost:8080"));
        assert!(debug.contains("10000"));
    }

    #[test]
    fn test_is_healthy_all_healthy() {
        let checks = vec![
            ComponentHealth {
                name: "web".into(),
                healthy: true,
                response_time_ms: 10,
                error: None,
            },
            ComponentHealth {
                name: "brain".into(),
                healthy: true,
                response_time_ms: 20,
                error: None,
            },
        ];
        assert!(DeploymentHealthChecker::is_healthy(&checks));
    }

    #[test]
    fn test_is_healthy_one_unhealthy() {
        let checks = vec![
            ComponentHealth {
                name: "web".into(),
                healthy: true,
                response_time_ms: 10,
                error: None,
            },
            ComponentHealth {
                name: "brain".into(),
                healthy: false,
                response_time_ms: 20,
                error: Some("timeout".into()),
            },
        ];
        assert!(!DeploymentHealthChecker::is_healthy(&checks));
    }

    #[test]
    fn test_is_healthy_empty() {
        assert!(DeploymentHealthChecker::is_healthy(&[]));
    }

    #[test]
    fn test_is_healthy_single_unhealthy() {
        let checks = vec![ComponentHealth {
            name: "web".into(),
            healthy: false,
            response_time_ms: 5,
            error: Some("connection refused".into()),
        }];
        assert!(!DeploymentHealthChecker::is_healthy(&checks));
    }

    #[test]
    fn test_to_conditions() {
        let checks = vec![
            ComponentHealth {
                name: "web".into(),
                healthy: true,
                response_time_ms: 10,
                error: None,
            },
            ComponentHealth {
                name: "brain".into(),
                healthy: false,
                response_time_ms: 20,
                error: Some("down".into()),
            },
        ];
        let conditions = DeploymentHealthChecker::to_conditions(&checks);
        assert_eq!(conditions.len(), 2);
        assert_eq!(conditions[0].component, "web");
        assert!(conditions[0].status);
        assert_eq!(conditions[0].message, "healthy");
        assert_eq!(conditions[1].component, "brain");
        assert!(!conditions[1].status);
        assert_eq!(conditions[1].message, "down");
    }

    #[test]
    fn test_to_conditions_empty() {
        let conditions = DeploymentHealthChecker::to_conditions(&[]);
        assert!(conditions.is_empty());
    }

    #[test]
    fn test_summary() {
        let checks = vec![
            ComponentHealth {
                name: "web".into(),
                healthy: true,
                response_time_ms: 10,
                error: None,
            },
            ComponentHealth {
                name: "brain".into(),
                healthy: false,
                response_time_ms: 20,
                error: Some("down".into()),
            },
        ];
        let summary = DeploymentHealthChecker::summary(&checks);
        assert_eq!(summary.get("web"), Some(&true));
        assert_eq!(summary.get("brain"), Some(&false));
    }

    #[test]
    fn test_summary_empty() {
        let summary = DeploymentHealthChecker::summary(&[]);
        assert!(summary.is_empty());
    }

    #[test]
    fn test_component_health_clone() {
        let health = ComponentHealth {
            name: "web".into(),
            healthy: true,
            response_time_ms: 42,
            error: None,
        };
        let cloned = health.clone();
        assert_eq!(cloned.name, health.name);
        assert_eq!(cloned.healthy, health.healthy);
        assert_eq!(cloned.response_time_ms, health.response_time_ms);
    }

    #[test]
    fn test_component_health_debug() {
        let health = ComponentHealth {
            name: "web".into(),
            healthy: true,
            response_time_ms: 42,
            error: None,
        };
        let debug = format!("{health:?}");
        assert!(debug.contains("web"));
        assert!(debug.contains("42"));
    }

    #[test]
    fn test_health_condition_clone() {
        let cond = HealthCondition {
            component: "web".into(),
            status: true,
            message: "ok".into(),
        };
        let cloned = cond.clone();
        assert_eq!(cloned.component, cond.component);
        assert_eq!(cloned.status, cond.status);
    }

    #[test]
    fn test_health_condition_debug() {
        let cond = HealthCondition {
            component: "brain".into(),
            status: false,
            message: "error".into(),
        };
        let debug = format!("{cond:?}");
        assert!(debug.contains("brain"));
        assert!(debug.contains("error"));
    }

    #[tokio::test]
    async fn test_check_all_unreachable() {
        let checker =
            DeploymentHealthChecker::new("http://localhost:59301".into()).with_timeout(100);
        let checks = checker.check_all().await;
        assert_eq!(checks.len(), 3);
        for check in &checks {
            assert!(!check.healthy);
            assert!(check.error.is_some());
        }
        assert_eq!(checks[0].name, "web");
        assert_eq!(checks[1].name, "registry");
        assert_eq!(checks[2].name, "brain");
    }

    #[tokio::test]
    async fn test_check_endpoint_timing() {
        let checker =
            DeploymentHealthChecker::new("http://localhost:59302".into()).with_timeout(50);
        let check = checker
            .check_endpoint("test", "http://localhost:59302/health")
            .await;
        assert!(!check.healthy);
        assert!(check.response_time_ms < 500);
        assert!(check.error.is_some());
        assert_eq!(check.name, "test");
    }

    #[tokio::test]
    async fn test_check_all_returns_expected_order() {
        let checker =
            DeploymentHealthChecker::new("http://localhost:59303".into()).with_timeout(100);
        let checks = checker.check_all().await;
        let names: Vec<&str> = checks.iter().map(|c| c.name.as_str()).collect();
        assert_eq!(names, vec!["web", "registry", "brain"]);
    }
}
