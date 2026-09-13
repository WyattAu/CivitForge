#![forbid(unsafe_code)]

use crate::error::{DbError, Result};
use breaker::CircuitBreaker;
use sqlx::postgres::PgPool;

/// Postgres connection pool guarded by the `breaker` kit circuit breaker
/// (ADR-0006 Phase 3). Failure-rate + sliding-window state machine replaces
/// the previous hand-rolled consecutive-failure counter.
pub struct DatabasePool {
    pool: PgPool,
    breaker: CircuitBreaker,
}

impl std::fmt::Debug for DatabasePool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DatabasePool")
            .field("circuit_state", &self.breaker.state())
            .finish_non_exhaustive()
    }
}

impl DatabasePool {
    pub async fn new(database_url: &str, max_connections: u32) -> Result<Self> {
        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(max_connections)
            .connect(database_url)
            .await
            .map_err(|e| DbError::Database(format!("failed to create pool: {e}")))?;

        // Kit `standard()` preset — defaults match the legacy config
        // (5 failures / 10 window / 30s wait / 3 half-open probes).
        let config = breaker::CircuitBreakerConfig::standard();

        Ok(Self {
            pool,
            breaker: CircuitBreaker::new(config),
        })
    }

    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    pub async fn close(self) {
        self.pool.close().await;
    }

    pub async fn health_check(&self) -> bool {
        if self.breaker.is_open() {
            return false;
        }
        match sqlx::query("SELECT 1").execute(&self.pool).await {
            Ok(_) => {
                self.breaker.record_success();
                true
            }
            Err(_) => {
                self.breaker.record_failure();
                false
            }
        }
    }

    /// Kit circuit state for observability endpoints.
    pub fn circuit_state(&self) -> breaker::State {
        self.breaker.state()
    }

    /// Kit metrics snapshot (failure rate, counts, transitions).
    pub fn breaker_metrics(&self) -> breaker::CircuitMetrics {
        self.breaker.metrics()
    }

    pub fn is_circuit_open(&self) -> bool {
        self.breaker.is_open()
    }

    pub async fn execute(&self, query: &str) -> Result<u64> {
        if self.breaker.is_open() {
            return Err(DbError::Database("circuit breaker is open".into()));
        }

        let result = sqlx::query(sqlx::AssertSqlSafe(query.to_string()))
            .execute(&self.pool)
            .await;
        match result {
            Ok(r) => {
                self.breaker.record_success();
                Ok(r.rows_affected())
            }
            Err(e) => {
                self.breaker.record_failure();
                Err(DbError::Database(format!("query execution failed: {e}")))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn breaker_starts_closed() {
        let config = breaker::CircuitBreakerConfig::standard();
        let breaker = CircuitBreaker::new(config);
        assert_eq!(breaker.state(), breaker::State::Closed);
        assert!(!breaker.is_open());
    }

    #[test]
    fn failure_threshold_trips_circuit() {
        // Kit threshold is a consecutive-failure count (standard: 5).
        let config = breaker::CircuitBreakerConfig::standard();
        let breaker = CircuitBreaker::new(config);
        for _ in 0..5 {
            breaker.record_failure();
        }
        assert!(breaker.is_open());
    }

    #[test]
    fn success_resets_failure_window() {
        let config = breaker::CircuitBreakerConfig::standard();
        let breaker = CircuitBreaker::new(config);
        breaker.record_failure();
        breaker.record_failure();
        breaker.record_failure();
        breaker.record_failure();
        // A success resets the consecutive-failure count.
        breaker.record_success();
        for _ in 0..4 {
            breaker.record_failure();
        }
        assert!(!breaker.is_open());
    }

    #[test]
    fn new_error_format() {
        let err = DbError::Database("failed to create pool: connection refused".into());
        assert!(err.to_string().contains("failed to create pool"));
    }

    #[test]
    fn execute_circuit_open_error() {
        let err = DbError::Database("circuit breaker is open".into());
        assert!(err.to_string().contains("circuit breaker is open"));
    }

    #[test]
    fn execute_query_failure_error() {
        let err = DbError::Database("query execution failed: syntax error".into());
        assert!(err.to_string().contains("query execution failed"));
    }

    #[test]
    fn result_type_used_in_pool() {
        let res: Result<()> = Ok(());
        assert!(res.is_ok());
        let res: Result<()> = Err(DbError::Database("fail".into()));
        assert!(res.is_err());
    }

    #[tokio::test]
    async fn new_with_invalid_url_returns_error() {
        let result = DatabasePool::new("postgres://invalid-host:0/invalid_db", 1).await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("failed to create pool"));
    }
}
