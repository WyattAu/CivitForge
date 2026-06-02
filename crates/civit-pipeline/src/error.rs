//! Pipeline error types.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum PipelineError {
    #[error("YAML parse error: {0}")]
    YamlParse(String),

    #[error("validation error: {0}")]
    Validation(String),

    #[error("invalid version: {0}, expected '1'")]
    InvalidVersion(String),

    #[error("job '{name}' has no steps")]
    EmptyJob { name: String },

    #[error("invalid timeout format: '{0}'")]
    InvalidTimeout(String),

    #[error("circular dependency in job '{name}': {chain:?}")]
    CircularDependency { name: String, chain: Vec<String> },

    #[error("invalid cron expression: '{0}")]
    InvalidCron(String),

    #[error("trigger pattern syntax error: '{0}'")]
    PatternSyntax(String),

    #[error("duplicate job name: '{0}'")]
    DuplicateJob(String),

    #[error("job '{job}' depends on unknown job '{dep}'")]
    UnknownDependency { job: String, dep: String },

    #[error("unsupported feature: {0}")]
    Unsupported(String),
}

pub type Result<T> = std::result::Result<T, PipelineError>;
