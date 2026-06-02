#![forbid(unsafe_code)]

use dashmap::DashMap;
use std::collections::HashMap;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, PartialEq)]
pub struct AutoscaleConfig {
    pub min_replicas: u32,
    pub max_replicas: u32,
    pub target_cpu_percent: f32,
    pub target_memory_percent: f32,
    pub scale_up_cooldown: Duration,
    pub scale_down_cooldown: Duration,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScalingDecision {
    pub component: String,
    pub current_replicas: u32,
    pub desired_replicas: u32,
    pub reason: String,
    pub metric: ScalingMetric,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ScalingMetric {
    CpuUsage(f32),
    MemoryUsage(f32),
    RequestRate(f64),
    ConnectionCount(u32),
}

pub struct ScalingMetrics {
    pub cpu_percent: f32,
    pub memory_percent: f32,
    pub request_rate: f64,
    pub active_connections: u32,
}

pub struct Autoscaler {
    config: HashMap<String, AutoscaleConfig>,
    last_scale: DashMap<String, Instant>,
}

impl Autoscaler {
    pub fn new() -> Self {
        Self {
            config: HashMap::new(),
            last_scale: DashMap::new(),
        }
    }

    pub fn register_component(&mut self, name: String, config: AutoscaleConfig) {
        self.config.insert(name, config);
    }

    pub fn evaluate(&self, component: &str, metrics: &ScalingMetrics) -> ScalingDecision {
        let default_config = AutoscaleConfig {
            min_replicas: 1,
            max_replicas: 10,
            target_cpu_percent: 70.0,
            target_memory_percent: 80.0,
            scale_up_cooldown: Duration::from_secs(60),
            scale_down_cooldown: Duration::from_secs(300),
        };

        let config = self.config.get(component).unwrap_or(&default_config);
        let current = config.min_replicas;

        let (metric, reason, desired) = self.compute_decision(component, config, current, metrics);

        ScalingDecision {
            component: component.to_string(),
            current_replicas: current,
            desired_replicas: desired,
            reason,
            metric,
        }
    }

    fn compute_decision(
        &self,
        component: &str,
        config: &AutoscaleConfig,
        current: u32,
        metrics: &ScalingMetrics,
    ) -> (ScalingMetric, String, u32) {
        let last_scale_time = self.last_scale.get(component).map(|r| *r);

        let scale_up_allowed = match last_scale_time {
            Some(t) => t.elapsed() >= config.scale_up_cooldown,
            None => true,
        };

        let scale_down_allowed = match last_scale_time {
            Some(t) => t.elapsed() >= config.scale_down_cooldown,
            None => true,
        };

        if metrics.cpu_percent > config.target_cpu_percent {
            if !scale_up_allowed {
                return (
                    ScalingMetric::CpuUsage(metrics.cpu_percent),
                    "scale up blocked by cooldown".into(),
                    current,
                );
            }
            let multiplier = metrics.cpu_percent / config.target_cpu_percent;
            let desired = (current as f32 * multiplier).ceil() as u32;
            let desired = desired.min(config.max_replicas).max(config.min_replicas);
            return (
                ScalingMetric::CpuUsage(metrics.cpu_percent),
                format!(
                    "cpu usage {:.1}% exceeds target {:.1}%",
                    metrics.cpu_percent, config.target_cpu_percent
                ),
                desired,
            );
        }

        if metrics.memory_percent > config.target_memory_percent {
            if !scale_up_allowed {
                return (
                    ScalingMetric::MemoryUsage(metrics.memory_percent),
                    "scale up blocked by cooldown".into(),
                    current,
                );
            }
            let multiplier = metrics.memory_percent / config.target_memory_percent;
            let desired = (current as f32 * multiplier).ceil() as u32;
            let desired = desired.min(config.max_replicas).max(config.min_replicas);
            return (
                ScalingMetric::MemoryUsage(metrics.memory_percent),
                format!(
                    "memory usage {:.1}% exceeds target {:.1}%",
                    metrics.memory_percent, config.target_memory_percent
                ),
                desired,
            );
        }

        if metrics.cpu_percent < config.target_cpu_percent * 0.5
            && metrics.memory_percent < config.target_memory_percent * 0.5
        {
            if !scale_down_allowed {
                return (
                    ScalingMetric::CpuUsage(metrics.cpu_percent),
                    "scale down blocked by cooldown".into(),
                    current,
                );
            }
            if current > config.min_replicas {
                let desired = (current - 1).max(config.min_replicas);
                return (
                    ScalingMetric::CpuUsage(metrics.cpu_percent),
                    "low utilization, scaling down".into(),
                    desired,
                );
            }
        }

        (
            ScalingMetric::CpuUsage(metrics.cpu_percent),
            "within target range".into(),
            current,
        )
    }
}

impl Default for Autoscaler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_scaler() -> Autoscaler {
        Autoscaler::new()
    }

    fn default_metrics() -> ScalingMetrics {
        ScalingMetrics {
            cpu_percent: 50.0,
            memory_percent: 40.0,
            request_rate: 100.0,
            active_connections: 50,
        }
    }

    #[test]
    fn test_register_component() {
        let mut scaler = make_scaler();
        scaler.register_component(
            "api".into(),
            AutoscaleConfig {
                min_replicas: 2,
                max_replicas: 10,
                target_cpu_percent: 70.0,
                target_memory_percent: 80.0,
                scale_up_cooldown: Duration::from_secs(60),
                scale_down_cooldown: Duration::from_secs(300),
            },
        );
        let decision = scaler.evaluate("api", &default_metrics());
        assert_eq!(decision.current_replicas, 2);
    }

    #[test]
    fn test_scale_up_on_cpu() {
        let mut scaler = make_scaler();
        scaler.register_component(
            "api".into(),
            AutoscaleConfig {
                min_replicas: 2,
                max_replicas: 10,
                target_cpu_percent: 70.0,
                target_memory_percent: 80.0,
                scale_up_cooldown: Duration::from_secs(0),
                scale_down_cooldown: Duration::from_secs(0),
            },
        );
        let metrics = ScalingMetrics {
            cpu_percent: 90.0,
            memory_percent: 40.0,
            request_rate: 1000.0,
            active_connections: 200,
        };
        let decision = scaler.evaluate("api", &metrics);
        assert!(decision.desired_replicas > decision.current_replicas);
        assert!(decision.reason.contains("cpu"));
    }

    #[test]
    fn test_scale_up_on_memory() {
        let mut scaler = make_scaler();
        scaler.register_component(
            "api".into(),
            AutoscaleConfig {
                min_replicas: 2,
                max_replicas: 10,
                target_cpu_percent: 70.0,
                target_memory_percent: 80.0,
                scale_up_cooldown: Duration::from_secs(0),
                scale_down_cooldown: Duration::from_secs(0),
            },
        );
        let metrics = ScalingMetrics {
            cpu_percent: 30.0,
            memory_percent: 95.0,
            request_rate: 100.0,
            active_connections: 50,
        };
        let decision = scaler.evaluate("api", &metrics);
        assert!(decision.desired_replicas > decision.current_replicas);
        assert!(decision.reason.contains("memory"));
    }

    #[test]
    fn test_scale_down_low_usage() {
        let mut scaler = make_scaler();
        scaler.register_component(
            "api".into(),
            AutoscaleConfig {
                min_replicas: 1,
                max_replicas: 10,
                target_cpu_percent: 70.0,
                target_memory_percent: 80.0,
                scale_up_cooldown: Duration::from_secs(0),
                scale_down_cooldown: Duration::from_secs(0),
            },
        );
        let metrics = ScalingMetrics {
            cpu_percent: 10.0,
            memory_percent: 15.0,
            request_rate: 5.0,
            active_connections: 5,
        };
        let decision = scaler.evaluate("api", &metrics);
        assert!(decision.desired_replicas <= decision.current_replicas);
        assert!(decision.reason.contains("target"));
    }

    #[test]
    fn test_scale_down_respects_minimum() {
        let mut scaler = make_scaler();
        scaler.register_component(
            "api".into(),
            AutoscaleConfig {
                min_replicas: 1,
                max_replicas: 10,
                target_cpu_percent: 70.0,
                target_memory_percent: 80.0,
                scale_up_cooldown: Duration::from_secs(0),
                scale_down_cooldown: Duration::from_secs(0),
            },
        );
        let metrics = ScalingMetrics {
            cpu_percent: 5.0,
            memory_percent: 5.0,
            request_rate: 0.0,
            active_connections: 0,
        };
        let decision = scaler.evaluate("api", &metrics);
        assert_eq!(decision.desired_replicas, 1);
    }

    #[test]
    fn test_scale_up_respects_maximum() {
        let mut scaler = make_scaler();
        scaler.register_component(
            "api".into(),
            AutoscaleConfig {
                min_replicas: 2,
                max_replicas: 5,
                target_cpu_percent: 50.0,
                target_memory_percent: 80.0,
                scale_up_cooldown: Duration::from_secs(0),
                scale_down_cooldown: Duration::from_secs(0),
            },
        );
        let metrics = ScalingMetrics {
            cpu_percent: 99.0,
            memory_percent: 40.0,
            request_rate: 10000.0,
            active_connections: 500,
        };
        let decision = scaler.evaluate("api", &metrics);
        assert!(decision.desired_replicas <= 5);
    }

    #[test]
    fn test_stable_within_range() {
        let mut scaler = make_scaler();
        scaler.register_component(
            "api".into(),
            AutoscaleConfig {
                min_replicas: 3,
                max_replicas: 10,
                target_cpu_percent: 70.0,
                target_memory_percent: 80.0,
                scale_up_cooldown: Duration::from_secs(0),
                scale_down_cooldown: Duration::from_secs(0),
            },
        );
        let decision = scaler.evaluate("api", &default_metrics());
        assert_eq!(decision.desired_replicas, decision.current_replicas);
        assert!(decision.reason.contains("within target"));
    }

    #[test]
    fn test_unknown_component_uses_defaults() {
        let scaler = make_scaler();
        let decision = scaler.evaluate("unknown", &default_metrics());
        assert_eq!(decision.component, "unknown");
    }

    #[test]
    fn test_scaling_decision_fields() {
        let mut scaler = make_scaler();
        scaler.register_component(
            "web".into(),
            AutoscaleConfig {
                min_replicas: 2,
                max_replicas: 8,
                target_cpu_percent: 70.0,
                target_memory_percent: 80.0,
                scale_up_cooldown: Duration::from_secs(0),
                scale_down_cooldown: Duration::from_secs(0),
            },
        );
        let metrics = ScalingMetrics {
            cpu_percent: 85.0,
            memory_percent: 40.0,
            request_rate: 500.0,
            active_connections: 100,
        };
        let decision = scaler.evaluate("web", &metrics);
        assert_eq!(decision.component, "web");
        assert!(!decision.reason.is_empty());
    }
}
