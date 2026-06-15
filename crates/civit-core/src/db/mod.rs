#![forbid(unsafe_code)]

//! Database module for civit-core.
//!
//! This module re-exports the database layer from the `civit-db` crate,
//! eliminating the previous code duplication. civit-core-specific concerns
//! (replica routing, AppConfig integration) live here as thin wrappers.

pub mod migrations;
pub mod replica_router;

// Re-export the single source of truth from civit-db.
pub use civit_db::models;
pub use civit_db::pool;
pub use civit_db::repository;
pub use civit_db::session;

// Convenience re-exports matching the old API surface.
pub use civit_db::pool::DatabasePool;
pub use civit_db::repository::DbRepository;
pub use civit_db::session::{Session, SessionManager};

// Re-export commonly used model types (backward-compatible with old imports).
pub use civit_db::models::{
    ActivityEvent, Issue, Org, Pipeline, PullRequest, Repository, SshKey, Team, TeamMember, User,
};

use crate::config::AppConfig;
use crate::error::Result;

/// Create a DatabasePool from the application configuration.
/// This is a civit-core convenience wrapper around civit_db::DatabasePool::new.
/// The DbError-to-CoreError conversion is handled by the From impl in error.rs.
pub async fn pool_from_config(config: &AppConfig) -> Result<DatabasePool> {
    let pool = DatabasePool::new(&config.database_url, 20).await?;
    Ok(pool)
}
