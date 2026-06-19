#![forbid(unsafe_code)]

//! LLM inference module.
//!
//! Provides a unified `LlmProvider` trait that abstracts over inference
//! backends. Production code uses `RemoteLlmProvider` (HTTP, OpenAI-compatible);
//! tests use `StubLlmProvider`.
//!
//! The low-level async `InferenceService` (inference.rs) is also available for
//! streaming and advanced use cases.

pub mod inference;
pub mod model_management;
pub mod models;
pub mod provider;

// Re-export the canonical types at the module level.
pub use provider::{ChatMessage, InferenceResult, LlmProvider, ModelConfig, RemoteLlmProvider};
