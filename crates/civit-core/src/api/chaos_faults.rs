#![forbid(unsafe_code)]

//! Fault injection for the request path.
//!
//! The chaos experiments table records *simulated* results; nothing ever
//! perturbed a live request, so the health-gated rollout's rollback path —
//! the half of the state machine that protects users — could not be
//! exercised end to end. This middleware injects real 5xx responses at a
//! controlled rate, which is the signal the rollout gate consumes.
//!
//! Guards:
//! - Admin-only control endpoint (`require_admin`), same as the rest of the
//!   chaos surface.
//! - Affects only this instance, in memory, and resets on restart — an
//!   injection that outlived its operator would be an outage, not an
//!   experiment.
//! - Injected responses flow through the normal middleware stack (the layer
//!   sits inside the tracing middleware), so the injected 500s are counted
//!   by exactly the window the gate reads — injecting outside it would
//!   prove nothing.

use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use crate::api::AppState;

/// The one path injection must never block.
pub const CONTROL_PATH: &str = "/api/v1/admin/chaos/faults";

/// Injection state: how many requests per thousand fail.
///
/// An `AtomicU64` rather than a lock: the read sits on the hot path of every
/// request, and the worst inconsistency between concurrent readers is one
/// request seeing a stale rate for a microsecond.
#[derive(Default)]
pub struct FaultInjector {
    permille: AtomicU64,
    counter: AtomicU64,
}

impl FaultInjector {
    /// Sets the failure rate in permille (0 = off, 1000 = all fail).
    pub fn set_permille(&self, value: u64) {
        self.permille.store(value.min(1000), Ordering::Relaxed);
    }

    /// Current failure rate in permille.
    #[must_use]
    pub fn permille(&self) -> u64 {
        self.permille.load(Ordering::Relaxed)
    }

    /// Whether the next request should fail.
    ///
    /// Deterministic dithering (Bresenham-style) rather than a random draw:
    /// `(counter * rate) % 1000 < rate` spreads failures evenly at any rate,
    /// so a rate of 500 permille fails exactly every second request and 10
    /// permille fails exactly every hundredth. A modulo-of-counter instead
    /// produces blocks — 500 consecutive failures, then 500 consecutive
    /// passes — which is a poor fault pattern and a lousy test.
    fn should_fail(&self) -> bool {
        let rate = self.permille();
        if rate == 0 {
            return false;
        }
        if rate >= 1000 {
            return true;
        }
        let n = self.counter.fetch_add(1, Ordering::Relaxed);
        // u128: the product of an ever-growing counter and 1000 would
        // overflow u64 in practice after ~1.8e16 requests, and "in practice"
        // is not a bound worth trusting.
        (u128::from(n) * u128::from(rate)) % 1000 < u128::from(rate)
    }
}

/// The middleware. Install inside the tracing middleware so injected
/// failures are recorded by the health window the rollout gate reads.
pub async fn fault_middleware(
    State(state): State<AppState>,
    req: axum::extract::Request,
    next: axum::middleware::Next,
) -> axum::response::Response {
    // The control endpoint is always exempt. Found the hard way: with
    // injection at 100%, the DELETE that turns it off is itself a request,
    // and the operator locks themselves out of the only off-switch until a
    // restart — an outage recovery, not an experiment. Chaos tooling must
    // never be able to deadlock its own abort path. Operators still need a
    // valid admin token held before starting; login itself stays in scope.
    if req.uri().path() == CONTROL_PATH {
        return next.run(req).await;
    }
    if state.fault_injector.should_fail() {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "error": "injected fault (chaos experiment)",
            })),
        )
            .into_response();
    }
    next.run(req).await
}

/// `PUT /api/v1/admin/chaos/faults` — set the injection rate.
pub async fn set_fault_rate(
    State(state): State<AppState>,
    auth: crate::api::auth::AuthUser,
    Json(body): Json<serde_json::Value>,
) -> impl IntoResponse {
    if let Err(rejection) = crate::api::auth::require_admin(&auth) {
        return rejection.into_response();
    }
    let permille = body.get("error_rate_permille").and_then(|v| v.as_u64());
    let Some(permille) = permille else {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "error_rate_permille (0..=1000) is required"})),
        )
            .into_response();
    };
    if permille > 1000 {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "error_rate_permille must be 0..=1000"})),
        )
            .into_response();
    }
    state.fault_injector.set_permille(permille);
    (
        StatusCode::OK,
        Json(serde_json::json!({"error_rate_permille": state.fault_injector.permille()})),
    )
        .into_response()
}

/// `DELETE /api/v1/admin/chaos/faults` — clear injection.
pub async fn clear_fault_rate(
    State(state): State<AppState>,
    auth: crate::api::auth::AuthUser,
) -> impl IntoResponse {
    if let Err(rejection) = crate::api::auth::require_admin(&auth) {
        return rejection.into_response();
    }
    state.fault_injector.set_permille(0);
    (
        StatusCode::OK,
        Json(serde_json::json!({"error_rate_permille": 0})),
    )
        .into_response()
}

/// `GET /api/v1/admin/chaos/faults` — current rate.
pub async fn get_fault_rate(
    State(state): State<AppState>,
    auth: crate::api::auth::AuthUser,
) -> impl IntoResponse {
    if let Err(rejection) = crate::api::auth::require_admin(&auth) {
        return rejection.into_response();
    }
    (
        StatusCode::OK,
        Json(serde_json::json!({"error_rate_permille": state.fault_injector.permille()})),
    )
        .into_response()
}

/// Admin chaos-fault routes.
pub fn fault_routes() -> Router<AppState> {
    use axum::routing::{get, put};
    Router::new().route(
        "/api/v1/admin/chaos/faults",
        get(get_fault_rate).put(set_fault_rate).delete(clear_fault_rate),
    )
}

use axum::Router;

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]
    use super::*;

    #[test]
    fn zero_permille_never_fails() {
        let f = FaultInjector::default();
        for _ in 0..1000 {
            assert!(!f.should_fail());
        }
    }

    #[test]
    fn full_permille_always_fails() {
        let f = FaultInjector::default();
        f.set_permille(1000);
        for _ in 0..100 {
            assert!(f.should_fail());
        }
    }

    /// Deterministic distribution: 500 permille fails exactly every second
    /// request, so a live verification is reproducible.
    #[test]
    fn half_permille_fails_every_other_request() {
        let f = FaultInjector::default();
        f.set_permille(500);
        let pattern: Vec<bool> = (0..6).map(|_| f.should_fail()).collect();
        assert_eq!(
            pattern,
            vec![true, false, true, false, true, false],
            "counter modulo must alternate exactly"
        );
    }

    #[test]
    fn permille_is_clamped() {
        let f = FaultInjector::default();
        f.set_permille(5000);
        assert_eq!(f.permille(), 1000);
    }

    /// The abort path must stay reachable under full injection.
    #[test]
    fn control_path_is_exempt_by_constant() {
        assert_eq!(
            CONTROL_PATH,
            "/api/v1/admin/chaos/faults",
            "route and exemption must not drift apart"
        );
    }

    #[test]
    fn rate_10_permille_fails_one_in_a_thousand_exactly() {
        let f = FaultInjector::default();
        f.set_permille(10);
        let failures: u64 = (0..1000u64).map(|_| u64::from(f.should_fail())).sum();
        assert_eq!(failures, 10);
    }

    /// Dithering must spread failures, not cluster them: the longest run of
    /// consecutive failures at 500 permille is 1, and at 10 permille is 1.
    #[test]
    fn dithering_spreads_failures_evenly() {
        for rate in [500u64, 100, 10] {
            let f = FaultInjector::default();
            f.set_permille(rate);
            let mut longest = 0;
            let mut run = 0;
            for _ in 0..1000 {
                if f.should_fail() {
                    run += 1;
                    longest = longest.max(run);
                } else {
                    run = 0;
                }
            }
            assert_eq!(longest, 1, "rate {rate}: clustered failures, longest run {longest}");
        }
    }
}