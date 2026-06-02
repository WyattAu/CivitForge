#![forbid(unsafe_code)]

//! Shared types for CivitForge.
//!
//! This crate contains API contract types, domain models, and common enums
//! used by both the backend (civit-core) and frontend (civit-ui).
//!
//! Types are organized by domain:
//! - `pagination` — list endpoint pagination
//! - `id` — typed ID newtypes for type safety
//! - `visibility` — repository visibility levels
//! - `user` — user domain types
//! - `repo` — repository domain types
//! - `org` — organization domain types
//! - `error` — API error types

pub mod error;
pub mod id;
pub mod org;
pub mod pagination;
pub mod repo;
pub mod user;
pub mod visibility;

// Re-exports for convenience
pub use error::{ApiError, ApiErrorCode};
pub use id::*;
pub use org::*;
pub use pagination::{Pagination, PaginationParams};
pub use repo::*;
pub use user::*;
pub use visibility::Visibility;
