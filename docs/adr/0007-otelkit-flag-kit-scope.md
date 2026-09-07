# ADR-0007: otelkit & flag-kit adoption scope (deferred)

- Status: accepted (deferred)
- Date: 2026-09-07
- Deciders: Wyatt

## Context

ADR-0006 Phase 4 lists otelkit and flag-kit adoption. Scoping during the
kit-adoption sessions revealed both are multi-session refactors, unlike the
Phase 1-3 swaps:

- **otelkit** wraps the real `opentelemetry` SDK. CivitForge's
  `civit-telemetry` is a 4,700-line hand-rolled OTel reimplementation
  (custom TraceContext, spans, OTLP/JSON exporter, APM, error tracking,
  Prometheus). The hot-path trace middleware
  (`middleware/tracing.rs`) and observability API
  (`api/observability.rs`) consume the custom model directly. Swapping
  means rewiring W3C context extraction + span creation onto the
  official SDK — touching the request hot path.
- **flag-kit** replaces the feature-flags DB tables, admin API
  (`/admin/feature-flags`), and admin UI page. Wire + UI contracts
  change.

Additionally: `tracing_setup.rs` in civit-telemetry is dead code
(main.rs initializes its own subscriber), and the graceful kit's
`shutdown_signal` (shutdown-kit 0.2, SIGTERM-aware) is now adopted.

## Decision

Defer both to dedicated sessions. On adoption:

1. otelkit: start with `TelemetryConfig::init` replacing main.rs
   subscriber boilerplate, then migrate middleware onto
   `opentelemetry` SDK context propagation; delete the hand-rolled
   OTLP exporter last (otelkit's `otlp` feature covers it).
2. flag-kit: introduce behind the existing endpoints first
   (adapter), migrate the admin UI second, drop the old tables last.
3. Prerequisite for both: the local dev machine's `target/` sweeper
   makes server-side builds mandatory (see deploy/civitforge.sh).

## Consequences

- civit-telemetry remains as-is (functional, E2E-verified) until the
  dedicated sessions.
- The kit ecosystem's unreleased-version risk is now handled via
  `[patch.crates-io]` pins (error-codes 1.1.0, error-classify 0.3.0,
  shutdown-kit 0.2.0) — remove pins on publish.
