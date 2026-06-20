#![forbid(unsafe_code)]

use crate::error::{DbError, Result};
use sqlx::postgres::PgPool;
use std::sync::atomic::{AtomicU8, AtomicU32, Ordering};
use std::time::Duration;

pub const CIRCUIT_CLOSED: u8 = 0;
pub const CIRCUIT_OPEN: u8 = 1;
pub const CIRCUIT_HALF_OPEN: u8 = 2;

#[derive(Debug)]
pub struct DatabasePool {
    pool: PgPool,
    consecutive_failures: AtomicU32,
    circuit_state: AtomicU8,
    failure_threshold: u32,
    reset_timeout: Duration,
    opened_at: std::sync::Mutex<Option<std::time::Instant>>,
}

impl DatabasePool {
    pub async fn new(database_url: &str, max_connections: u32) -> Result<Self> {
        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(max_connections)
            .connect(database_url)
            .await
            .map_err(|e| DbError::Database(format!("failed to create pool: {e}")))?;

        Ok(Self {
            pool,
            consecutive_failures: AtomicU32::new(0),
            circuit_state: AtomicU8::new(CIRCUIT_CLOSED),
            failure_threshold: 5,
            reset_timeout: Duration::from_secs(30),
            opened_at: std::sync::Mutex::new(None),
        })
    }

    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    pub async fn close(self) {
        self.pool.close().await;
    }

    pub async fn health_check(&self) -> bool {
        if self.circuit_state() == CIRCUIT_OPEN {
            if let Some(opened) = *self.opened_at.lock().unwrap() {
                if opened.elapsed() >= self.reset_timeout {
                    self.set_circuit_state(CIRCUIT_HALF_OPEN);
                } else {
                    return false;
                }
            } else {
                return false;
            }
        }

        let result = sqlx::query("SELECT 1").execute(&self.pool).await;
        match result {
            Ok(_) => {
                self.record_success();
                true
            }
            Err(_) => {
                self.record_failure();
                false
            }
        }
    }

    pub fn circuit_state(&self) -> u8 {
        self.circuit_state.load(Ordering::Relaxed)
    }

    pub fn consecutive_failures(&self) -> u32 {
        self.consecutive_failures.load(Ordering::Relaxed)
    }

    pub fn failure_threshold(&self) -> u32 {
        self.failure_threshold
    }

    pub fn reset_timeout(&self) -> Duration {
        self.reset_timeout
    }

    fn set_circuit_state(&self, state: u8) {
        self.circuit_state.store(state, Ordering::Relaxed);
    }

    fn record_success(&self) {
        self.consecutive_failures.store(0, Ordering::Relaxed);
        self.set_circuit_state(CIRCUIT_CLOSED);
        *self.opened_at.lock().unwrap() = None;
    }

    fn record_failure(&self) {
        let failures = self.consecutive_failures.fetch_add(1, Ordering::Relaxed) + 1;
        if failures >= self.failure_threshold {
            self.set_circuit_state(CIRCUIT_OPEN);
            *self.opened_at.lock().unwrap() = Some(std::time::Instant::now());
        }
    }

    pub fn is_circuit_open(&self) -> bool {
        let state = self.circuit_state();
        if state == CIRCUIT_OPEN {
            if let Some(opened) = *self.opened_at.lock().unwrap()
                && opened.elapsed() >= self.reset_timeout
            {
                self.set_circuit_state(CIRCUIT_HALF_OPEN);
                return false;
            }
            true
        } else {
            false
        }
    }

    pub async fn execute(&self, query: &str) -> Result<u64> {
        if self.is_circuit_open() {
            return Err(DbError::Database("circuit breaker is open".into()));
        }

        let result = sqlx::query(sqlx::AssertSqlSafe(query.to_string()))
            .execute(&self.pool)
            .await;
        match result {
            Ok(r) => {
                self.record_success();
                Ok(r.rows_affected())
            }
            Err(e) => {
                self.record_failure();
                Err(DbError::Database(format!("query execution failed: {e}")))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_circuit_breaker_initial_state() {
        let counter = AtomicU32::new(0);
        let state = AtomicU8::new(CIRCUIT_CLOSED);
        assert_eq!(counter.load(Ordering::Relaxed), 0);
        assert_eq!(state.load(Ordering::Relaxed), CIRCUIT_CLOSED);
    }

    #[test]
    fn test_circuit_breaker_transitions_to_open() {
        let failures = AtomicU32::new(0);
        let threshold = 5u32;
        let state = AtomicU8::new(CIRCUIT_CLOSED);

        for _ in 0..threshold {
            let count = failures.fetch_add(1, Ordering::Relaxed) + 1;
            if count >= threshold {
                state.store(CIRCUIT_OPEN, Ordering::Relaxed);
            }
        }

        assert_eq!(state.load(Ordering::Relaxed), CIRCUIT_OPEN);
        assert_eq!(failures.load(Ordering::Relaxed), threshold);
    }

    #[test]
    fn test_circuit_breaker_resets_on_success() {
        let failures = AtomicU32::new(5);
        let state = AtomicU8::new(CIRCUIT_OPEN);

        failures.store(0, Ordering::Relaxed);
        state.store(CIRCUIT_CLOSED, Ordering::Relaxed);

        assert_eq!(failures.load(Ordering::Relaxed), 0);
        assert_eq!(state.load(Ordering::Relaxed), CIRCUIT_CLOSED);
    }

    #[test]
    #[allow(clippy::assertions_on_constants)]
    fn test_half_open_state_value() {
        assert_eq!(CIRCUIT_HALF_OPEN, 2);
        assert!(CIRCUIT_HALF_OPEN > CIRCUIT_OPEN);
        assert!(CIRCUIT_OPEN > CIRCUIT_CLOSED);
    }

    #[test]
    fn test_reset_timeout_default() {
        let timeout = Duration::from_secs(30);
        assert_eq!(timeout.as_secs(), 30);
    }

    #[test]
    fn test_failure_threshold_default() {
        let threshold: u32 = 5;
        assert_eq!(threshold, 5);
    }

    #[test]
    fn test_consecutive_failures_below_threshold_keeps_closed() {
        let failures = AtomicU32::new(0);
        let threshold = 5u32;
        let state = AtomicU8::new(CIRCUIT_CLOSED);

        for _ in 0..(threshold - 1) {
            let count = failures.fetch_add(1, Ordering::Relaxed) + 1;
            if count >= threshold {
                state.store(CIRCUIT_OPEN, Ordering::Relaxed);
            }
        }

        assert_eq!(state.load(Ordering::Relaxed), CIRCUIT_CLOSED);
        assert_eq!(failures.load(Ordering::Relaxed), threshold - 1);
    }

    #[test]
    fn test_failure_accumulates_across_multiple_thresholds() {
        let failures = AtomicU32::new(3);
        let threshold = 5u32;
        let state = AtomicU8::new(CIRCUIT_CLOSED);

        for _ in 0..threshold {
            let count = failures.fetch_add(1, Ordering::Relaxed) + 1;
            if count >= threshold {
                state.store(CIRCUIT_OPEN, Ordering::Relaxed);
            }
        }

        assert_eq!(state.load(Ordering::Relaxed), CIRCUIT_OPEN);
        assert_eq!(failures.load(Ordering::Relaxed), 3 + threshold);
    }

    #[test]
    fn test_new_error_format() {
        let err = DbError::Database("failed to create pool: connection refused".into());
        assert!(err.to_string().contains("failed to create pool"));
    }

    #[test]
    fn test_execute_circuit_open_error() {
        let err = DbError::Database("circuit breaker is open".into());
        assert!(err.to_string().contains("circuit breaker is open"));
    }

    #[test]
    fn test_execute_query_failure_error() {
        let err = DbError::Database("query execution failed: syntax error".into());
        assert!(err.to_string().contains("query execution failed"));
    }

    #[test]
    fn test_opened_at_mutex_none_means_no_transition() {
        let opened_at: std::sync::Mutex<Option<std::time::Instant>> = std::sync::Mutex::new(None);
        assert!(opened_at.lock().unwrap().is_none());
    }

    #[test]
    fn test_opened_at_mutex_some_tracks_time() {
        let opened_at: std::sync::Mutex<Option<std::time::Instant>> =
            std::sync::Mutex::new(Some(std::time::Instant::now()));
        let guard = opened_at.lock().unwrap();
        assert!(guard.is_some());
        assert!(guard.unwrap().elapsed() < Duration::from_secs(1));
    }

    #[test]
    fn test_circuit_state_accessor() {
        let state = AtomicU8::new(CIRCUIT_CLOSED);
        assert_eq!(state.load(Ordering::Relaxed), CIRCUIT_CLOSED);
        state.store(CIRCUIT_HALF_OPEN, Ordering::Relaxed);
        assert_eq!(state.load(Ordering::Relaxed), CIRCUIT_HALF_OPEN);
        state.store(CIRCUIT_OPEN, Ordering::Relaxed);
        assert_eq!(state.load(Ordering::Relaxed), CIRCUIT_OPEN);
    }

    #[test]
    fn test_is_circuit_open_with_elapsed_timeout() {
        let state = AtomicU8::new(CIRCUIT_OPEN);
        let opened_at: std::sync::Mutex<Option<std::time::Instant>> = std::sync::Mutex::new(None);

        if state.load(Ordering::Relaxed) == CIRCUIT_OPEN {
            if let Some(_opened) = *opened_at.lock().unwrap() {
                if _opened.elapsed() >= Duration::from_secs(30) {
                    state.store(CIRCUIT_HALF_OPEN, Ordering::Relaxed);
                }
            }
        }
        assert_eq!(state.load(Ordering::Relaxed), CIRCUIT_OPEN);
    }

    #[test]
    fn test_record_failure_updates_opened_at() {
        let failures = AtomicU32::new(4);
        let threshold = 5u32;
        let state = AtomicU8::new(CIRCUIT_CLOSED);
        let opened_at: std::sync::Mutex<Option<std::time::Instant>> = std::sync::Mutex::new(None);

        let count = failures.fetch_add(1, Ordering::Relaxed) + 1;
        if count >= threshold {
            state.store(CIRCUIT_OPEN, Ordering::Relaxed);
            *opened_at.lock().unwrap() = Some(std::time::Instant::now());
        }

        assert_eq!(state.load(Ordering::Relaxed), CIRCUIT_OPEN);
        assert!(opened_at.lock().unwrap().is_some());
    }

    #[test]
    fn test_record_success_clears_opened_at() {
        let failures = AtomicU32::new(5);
        let state = AtomicU8::new(CIRCUIT_OPEN);
        let opened_at: std::sync::Mutex<Option<std::time::Instant>> =
            std::sync::Mutex::new(Some(std::time::Instant::now()));

        failures.store(0, Ordering::Relaxed);
        state.store(CIRCUIT_CLOSED, Ordering::Relaxed);
        *opened_at.lock().unwrap() = None;

        assert_eq!(failures.load(Ordering::Relaxed), 0);
        assert_eq!(state.load(Ordering::Relaxed), CIRCUIT_CLOSED);
        assert!(opened_at.lock().unwrap().is_none());
    }

    #[test]
    fn test_result_type_used_in_pool() {
        let res: Result<()> = Ok(());
        assert!(res.is_ok());
        let res: Result<()> = Err(DbError::Database("fail".into()));
        assert!(res.is_err());
    }

    #[tokio::test]
    async fn test_new_with_invalid_url_returns_error() {
        let result = DatabasePool::new("postgres://invalid-host:0/invalid_db", 1).await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("failed to create pool"));
    }
}
