#![forbid(unsafe_code)]

pub const M_001_INITIAL_SCHEMA_UP: &str = include_str!("001_initial_schema.sql");
pub const M_001_INITIAL_SCHEMA_DOWN: &str = include_str!("002_initial_schema_down.sql");

#[derive(Debug, Clone)]
pub struct Migration {
    pub version: i64,
    pub name: String,
    pub up_sql: String,
    pub down_sql: String,
}

#[derive(Debug, Default)]
pub struct MigrationManager {
    migrations: Vec<Migration>,
}

impl MigrationManager {
    pub fn new() -> Self {
        let mut mgr = Self {
            migrations: Vec::new(),
        };
        mgr.register_builtins();
        mgr
    }

    fn register_builtins(&mut self) {
        self.add_migration(Migration {
            version: 1,
            name: "initial_schema".into(),
            up_sql: M_001_INITIAL_SCHEMA_UP.into(),
            down_sql: M_001_INITIAL_SCHEMA_DOWN.into(),
        });
    }

    pub fn add_migration(&mut self, migration: Migration) {
        if let Some(last) = self.migrations.last() {
            assert!(
                migration.version > last.version,
                "migration version {} must be greater than last version {}",
                migration.version,
                last.version,
            );
        }
        self.migrations.push(migration);
    }

    pub fn all(&self) -> &[Migration] {
        &self.migrations
    }

    pub fn get_pending(&self, db_version: i64) -> Vec<&Migration> {
        self.migrations
            .iter()
            .filter(|m| m.version > db_version)
            .collect()
    }
}

impl std::ops::Deref for MigrationManager {
    type Target = [Migration];

    fn deref(&self) -> &Self::Target {
        &self.migrations
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_manager_has_initial_migration() {
        let mgr = MigrationManager::new();
        assert_eq!(mgr.all().len(), 1);
        assert_eq!(mgr.all()[0].version, 1);
        assert_eq!(mgr.all()[0].name, "initial_schema");
    }

    #[test]
    fn test_add_migration_sequential() {
        let mut mgr = MigrationManager::new();
        mgr.add_migration(Migration {
            version: 2,
            name: "add_index".into(),
            up_sql: "CREATE INDEX test;".into(),
            down_sql: "DROP INDEX test;".into(),
        });
        assert_eq!(mgr.all().len(), 2);
        assert_eq!(mgr.all()[1].version, 2);
    }

    #[test]
    #[should_panic(expected = "must be greater")]
    fn test_add_migration_out_of_order_panics() {
        let mut mgr = MigrationManager::new();
        mgr.add_migration(Migration {
            version: 0,
            name: "bad".into(),
            up_sql: "".into(),
            down_sql: "".into(),
        });
    }

    #[test]
    fn test_get_pending_none_applied() {
        let mgr = MigrationManager::new();
        let pending = mgr.get_pending(0);
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].version, 1);
    }

    #[test]
    fn test_get_pending_all_applied() {
        let mgr = MigrationManager::new();
        let pending = mgr.get_pending(1);
        assert!(pending.is_empty());
    }

    #[test]
    fn test_get_pending_partial() {
        let mut mgr = MigrationManager::new();
        mgr.add_migration(Migration {
            version: 2,
            name: "second".into(),
            up_sql: "".into(),
            down_sql: "".into(),
        });
        let pending = mgr.get_pending(1);
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].version, 2);
    }

    #[test]
    fn test_initial_schema_sql_not_empty() {
        assert_ne!(M_001_INITIAL_SCHEMA_UP, "");
        assert!(M_001_INITIAL_SCHEMA_UP.contains("CREATE TABLE IF NOT EXISTS users"));
        assert!(M_001_INITIAL_SCHEMA_UP.contains("CREATE TABLE IF NOT EXISTS schema_migrations"));
    }

    #[test]
    fn test_initial_schema_down_sql_not_empty() {
        assert_ne!(M_001_INITIAL_SCHEMA_DOWN, "");
        assert!(M_001_INITIAL_SCHEMA_DOWN.contains("DROP TABLE IF EXISTS"));
    }
}
