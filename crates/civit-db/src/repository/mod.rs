#![forbid(unsafe_code)]
#![allow(clippy::too_many_arguments)]

use sqlx::postgres::PgPool;

mod api;
mod auth;
mod import_job;
mod issue;
mod org;
mod pipeline;
mod repo;
mod user;

pub use import_job::ImportJob;

#[derive(Debug, Clone)]
pub struct DbRepository {
    pool: PgPool,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct OrgUsage {
    pub org_id: uuid::Uuid,
    pub repo_count: i64,
    pub member_count: i64,
}

impl DbRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Expose the underlying pool for permission engine queries.
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }
}
