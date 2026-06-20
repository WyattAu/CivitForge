#![forbid(unsafe_code)]

use crate::models::{PipelineSpec, PipelineStatus};
use tracing::info;

#[derive(Debug, Clone)]
pub struct OperatorConfig {
    pub namespace: String,
    pub resync_interval_secs: u64,
    pub max_parallel_pipelines: u32,
}

impl Default for OperatorConfig {
    fn default() -> Self {
        Self {
            namespace: "civit-system".into(),
            resync_interval_secs: 30,
            max_parallel_pipelines: 10,
        }
    }
}

pub struct PipelineOperator {
    #[allow(dead_code)]
    config: OperatorConfig,
    running_pipelines: dashmap::DashMap<String, PipelineStatus>,
}

impl PipelineOperator {
    pub fn new(config: OperatorConfig) -> Self {
        Self {
            config,
            running_pipelines: dashmap::DashMap::new(),
        }
    }

    pub fn reconcile(&self, spec: &PipelineSpec) -> anyhow::Result<ReconcileAction> {
        let key = spec.name.clone();
        if let Some(current) = self.running_pipelines.get(&key)
            && current.status == "running"
        {
            info!(pipeline = %spec.name, "pipeline already running, skipping");
            return Ok(ReconcileAction::Requeue { after_secs: 5 });
        }
        info!(pipeline = %spec.name, steps = spec.steps.len(), "reconciling pipeline");
        Ok(ReconcileAction::Run)
    }

    pub fn register_pipeline(&self, spec: &PipelineSpec) {
        let status = PipelineStatus {
            pipeline: spec.name.clone(),
            status: "running".into(),
            step_results: vec![],
            started_at: chrono::Utc::now().to_rfc3339(),
            finished_at: String::new(),
        };
        self.running_pipelines.insert(spec.name.clone(), status);
    }

    pub fn complete_pipeline(&self, name: &str, result: PipelineStatus) {
        self.running_pipelines.insert(name.into(), result);
    }

    pub fn get_pipeline_status(&self, name: &str) -> Option<PipelineStatus> {
        self.running_pipelines.get(name).map(|r| r.clone())
    }

    pub fn running_count(&self) -> usize {
        self.running_pipelines
            .iter()
            .filter(|e| e.value().status == "running")
            .count()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ReconcileAction {
    Run,
    Requeue { after_secs: u64 },
    Ignore,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::PipelineStep;
    use std::collections::HashMap;

    fn test_spec(name: &str) -> PipelineSpec {
        PipelineSpec {
            name: name.into(),
            triggers: vec!["push".into()],
            steps: vec![PipelineStep {
                name: "build".into(),
                image: "alpine:latest".into(),
                commands: vec!["make".into()],
                env: HashMap::new(),
                condition: None,
            }],
        }
    }

    #[test]
    fn test_reconcile_new_pipeline() {
        let operator = PipelineOperator::new(OperatorConfig::default());
        let spec = test_spec("new-pipe");
        let action = operator.reconcile(&spec).unwrap();
        assert_eq!(action, ReconcileAction::Run);
    }

    #[test]
    fn test_reconcile_running_pipeline_skips() {
        let operator = PipelineOperator::new(OperatorConfig::default());
        let spec = test_spec("running-pipe");
        operator.register_pipeline(&spec);
        let action = operator.reconcile(&spec).unwrap();
        assert_eq!(action, ReconcileAction::Requeue { after_secs: 5 });
    }

    #[test]
    fn test_running_count() {
        let operator = PipelineOperator::new(OperatorConfig::default());
        let spec = test_spec("count-pipe");
        operator.register_pipeline(&spec);
        assert_eq!(operator.running_count(), 1);
    }

    #[test]
    fn test_default_config() {
        let config = OperatorConfig::default();
        assert_eq!(config.namespace, "civit-system");
        assert_eq!(config.resync_interval_secs, 30);
        assert_eq!(config.max_parallel_pipelines, 10);
    }
}
