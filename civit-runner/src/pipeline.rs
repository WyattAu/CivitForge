#![forbid(unsafe_code)]

use crate::models::{PipelineSpec, PipelineStatus, PipelineStep, StepStatus};
use tracing::{debug, info};

#[derive(Debug, Clone)]
pub struct PipelineEngine {
    #[allow(dead_code)]
    namespace: String,
}

impl PipelineEngine {
    pub fn new(namespace: String) -> Self {
        Self { namespace }
    }

    pub async fn run(&self, spec: &PipelineSpec) -> anyhow::Result<PipelineStatus> {
        info!(pipeline = %spec.name, steps = spec.steps.len(), "starting pipeline");

        let mut step_results: Vec<StepStatus> = Vec::new();

        for step in &spec.steps {
            let result = self.execute_step(step, &step_results).await;
            let status = match result {
                Ok(_) => {
                    info!(step = %step.name, "step succeeded");
                    StepStatus {
                        name: step.name.clone(),
                        status: "success".into(),
                        started_at: chrono::Utc::now().to_rfc3339(),
                        finished_at: chrono::Utc::now().to_rfc3339(),
                        output: String::new(),
                    }
                }
                Err(e) => {
                    info!(step = %step.name, error = %e, "step failed");
                    StepStatus {
                        name: step.name.clone(),
                        status: "failed".into(),
                        started_at: chrono::Utc::now().to_rfc3339(),
                        finished_at: chrono::Utc::now().to_rfc3339(),
                        output: e.to_string(),
                    }
                }
            };

            let succeeded = status.status == "success";
            step_results.push(status);

            if !succeeded {
                return Ok(PipelineStatus {
                    pipeline: spec.name.clone(),
                    status: "failed".into(),
                    step_results,
                    started_at: chrono::Utc::now().to_rfc3339(),
                    finished_at: chrono::Utc::now().to_rfc3339(),
                });
            }
        }

        Ok(PipelineStatus {
            pipeline: spec.name.clone(),
            status: "success".into(),
            step_results,
            started_at: chrono::Utc::now().to_rfc3339(),
            finished_at: chrono::Utc::now().to_rfc3339(),
        })
    }

    async fn execute_step(
        &self,
        step: &PipelineStep,
        _prior_results: &[StepStatus],
    ) -> anyhow::Result<()> {
        if let Some(ref condition) = step.condition {
            if !self.evaluate_condition(condition) {
                info!(step = %step.name, "step skipped due to condition");
                return Ok(());
            }
        }

        debug!(
            step = %step.name,
            image = %step.image,
            commands = step.commands.len(),
            "executing step"
        );

        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        Ok(())
    }

    fn evaluate_condition(&self, condition: &crate::models::StepCondition) -> bool {
        match condition {
            crate::models::StepCondition::Always => true,
            crate::models::StepCondition::OnFailure => false,
            crate::models::StepCondition::OnSuccess => true,
            crate::models::StepCondition::Branch { branches: _ } => true,
            crate::models::StepCondition::EnvVar { key, value } => {
                std::env::var(key).map(|v| v == *value).unwrap_or(false)
            }
        }
    }

    pub fn validate_spec(&self, spec: &PipelineSpec) -> Vec<String> {
        let mut warnings = Vec::new();
        if spec.steps.is_empty() {
            warnings.push("pipeline has no steps".into());
        }
        for (i, step) in spec.steps.iter().enumerate() {
            if step.name.is_empty() {
                warnings.push(format!("step {i}: empty name"));
            }
            if step.image.is_empty() {
                warnings.push(format!("step {i}: empty image"));
            }
            if step.commands.is_empty() {
                warnings.push(format!("step {}: no commands", step.name));
            }
        }
        warnings
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_pipeline_success() {
        let engine = PipelineEngine::new("default".into());
        let spec = PipelineSpec {
            name: "test-pipeline".into(),
            triggers: vec!["push".into()],
            steps: vec![
                PipelineStep {
                    name: "step1".into(),
                    image: "alpine:latest".into(),
                    commands: vec!["echo hello".into()],
                    env: std::collections::HashMap::new(),
                    condition: None,
                },
                PipelineStep {
                    name: "step2".into(),
                    image: "alpine:latest".into(),
                    commands: vec!["echo world".into()],
                    env: std::collections::HashMap::new(),
                    condition: None,
                },
            ],
        };
        let result = engine.run(&spec).await.unwrap();
        assert_eq!(result.status, "success");
        assert_eq!(result.step_results.len(), 2);
    }

    #[test]
    fn test_validate_empty_pipeline() {
        let engine = PipelineEngine::new("default".into());
        let spec = PipelineSpec {
            name: "empty".into(),
            triggers: vec![],
            steps: vec![],
        };
        let warnings = engine.validate_spec(&spec);
        assert!(warnings.iter().any(|w| w.contains("no steps")));
    }

    #[test]
    fn test_validate_step_warnings() {
        let engine = PipelineEngine::new("default".into());
        let spec = PipelineSpec {
            name: "bad-steps".into(),
            triggers: vec![],
            steps: vec![PipelineStep {
                name: "".into(),
                image: "".into(),
                commands: vec![],
                env: std::collections::HashMap::new(),
                condition: None,
            }],
        };
        let warnings = engine.validate_spec(&spec);
        assert!(warnings.iter().any(|w| w.contains("empty name")));
        assert!(warnings.iter().any(|w| w.contains("empty image")));
        assert!(warnings.iter().any(|w| w.contains("no commands")));
    }
}
