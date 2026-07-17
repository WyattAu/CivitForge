#![forbid(unsafe_code)]

pub mod workflow_engine;
pub mod automation_rules;
pub mod scheduled_tasks;
pub mod pipeline_action_reviews;
pub mod environment_deployment;
pub mod cache_hit_analysis;
pub mod scheduler;

pub use workflow_engine::WorkflowService;
pub use automation_rules::AutomationRuleService;
pub use scheduled_tasks::ScheduledTaskService;
pub use pipeline_action_reviews::PipelineActionReviewsService;
pub use environment_deployment::EnvironmentDeploymentService;
pub use cache_hit_analysis::CacheHitAnalysisService;
pub use scheduler::start_scheduler;

#[cfg(test)]
mod workflow_engine_tests;
#[cfg(test)]
mod automation_rules_tests;
