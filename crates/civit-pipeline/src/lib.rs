//! CivitForge Pipeline YAML specification.
//!
//! Parses `.civit/pipeline.yaml` into typed Rust structs with full validation.

#![forbid(unsafe_code)]

mod error;
mod expr;
mod model;
mod parser;
pub mod trigger;
mod validate;

pub use error::PipelineError;
pub use expr::PipelineExpression;
pub use model::*;
pub use parser::parse_pipeline;
pub use trigger::{TriggerContext, matches_trigger, validate_cron};
pub use validate::validate_pipeline;
