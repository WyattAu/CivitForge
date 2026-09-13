//! Contract tests pinning the `breaker` kit semantics that `civit-db`'s
//! connection pool depends on (`pool.rs` wraps `breaker::CircuitBreaker`
//! with the `standard()` preset: 5-failure streak / 10-window / 30 s open /
//! 3 half-open probes, ADR-0006 Phase 3).
//!
//! These run against the kit directly with a short open-phase duration so
//! they stay fast while still locking the state machine the pool relies on:
//! closed → open on failure streak, open rejects, half-open after backoff,
//! success closes.

use breaker::{BackoffStrategy, CircuitBreaker, CircuitBreakerConfig, State};
use std::time::Duration;

fn fast_config() -> CircuitBreakerConfig {
    CircuitBreakerConfig::builder()
        .failure_rate_threshold(0.0)
        .consecutive_failures(3)
        .sliding_window_size(10)
        .minimum_calls(10)
        .backoff(BackoffStrategy::Fixed(Duration::from_millis(50)))
        .build()
}

#[test]
fn standard_preset_matches_pool_documentation() {
    let cfg = CircuitBreakerConfig::standard();
    assert_eq!(cfg.consecutive_failures, 5, "pool comment: 5 failures");
    assert_eq!(cfg.sliding_window_size, 10, "pool comment: 10 window");
    assert_eq!(cfg.half_open_max_calls, 3, "pool comment: 3 half-open probes");
    assert_eq!(cfg.backoff.initial_wait(), Duration::from_secs(30), "pool comment: 30s wait");
}

#[test]
fn failure_streak_opens_the_circuit() {
    let breaker = CircuitBreaker::new(fast_config());
    assert_eq!(breaker.state(), State::Closed);

    for _ in 0..3 {
        breaker.record_failure();
    }
    assert_eq!(breaker.state(), State::Open);
    assert!(breaker.is_open());
}

#[test]
fn open_circuit_rejects_until_backoff_elapses_then_half_opens() {
    let breaker = CircuitBreaker::new(fast_config());
    for _ in 0..3 {
        breaker.record_failure();
    }
    assert!(breaker.is_open(), "must reject while open");

    std::thread::sleep(Duration::from_millis(80));
    assert_eq!(breaker.state(), State::HalfOpen, "after 50ms fixed backoff");
}

#[test]
fn success_resets_streak_and_closes_from_half_open() {
    let breaker = CircuitBreaker::new(fast_config());

    breaker.record_failure();
    breaker.record_failure();
    assert_eq!(breaker.state(), State::Closed, "streak below threshold stays closed");

    breaker.record_success();
    for _ in 0..2 {
        breaker.record_failure();
    }
    assert_eq!(breaker.state(), State::Closed, "success resets the failure streak");

    for _ in 0..3 {
        breaker.record_failure();
    }
    std::thread::sleep(Duration::from_millis(80));
    breaker.record_success();
    breaker.record_success();
    breaker.record_success();
    assert_eq!(breaker.state(), State::Closed, "success_threshold probes close");
}

#[test]
fn metrics_reflect_recorded_outcomes() {
    let breaker = CircuitBreaker::new(fast_config());
    breaker.record_success();
    breaker.record_failure();
    let metrics = breaker.metrics();
    assert_eq!(metrics.total_successes, 1);
    assert_eq!(metrics.total_failures, 1);
}
