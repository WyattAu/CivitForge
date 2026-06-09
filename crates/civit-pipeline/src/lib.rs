//! CivitForge Pipeline YAML specification.
//!
//! Parses `.civit/pipeline.yaml` into typed Rust structs with full validation.

#![forbid(unsafe_code)]

mod error;
mod expand;
mod expr;
mod model;
mod parser;
pub mod trigger;
mod validate;

pub use error::PipelineError;
pub use expand::expand_matrix;
pub use expr::PipelineExpression;
pub use model::*;
pub use parser::parse_pipeline;
pub use trigger::{TriggerContext, compute_next_cron_run, matches_trigger, validate_cron};
pub use validate::validate_pipeline;
