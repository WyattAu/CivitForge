#![forbid(unsafe_code)]

//! Migration module for civit-core.
//!
//! Re-exports from civit-db, which is the single source of truth for all
//! database migrations. This eliminates the migration divergence that
//! previously existed between civit-core and civit-db.

pub use civit_db::migrations::{Migration, MigrationManager};
