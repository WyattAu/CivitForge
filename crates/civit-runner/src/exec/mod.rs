//! Standalone CI/CD Runner Execution Module.
//!
//! A lightweight runner that polls the CivitForge API for pipeline jobs
//! and executes them using Podman containers. This replaces the K8s-native
//! operator for single-machine and Podman-based deployments.

#![forbid(unsafe_code)]

pub mod client;
pub mod executor;
pub mod workspace;
