# ADR-0007: otelkit & flag-kit adoption scope (deferred)

- Status: accepted — steps 1-3 shipped (complete) (otelkit subscriber `7e20ce3`;
  flag-kit bucketing + validation `dba123b`; flag-kit `FlagStore`
  adapter + `Evaluator` `d191e50`); remaining steps deferred
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

1. otelkit — DONE step 1: `otelkit::init(TelemetryConfig)` replaced the
   main.rs subscriber (`7e20ce3`), with `TelemetryGuard` flushing on drop
   and OTLP export active whenever `OTEL_EXPORTER_OTLP_ENDPOINT` is set.
   DONE step 2: middleware emits `http.server.request` through the
   tracing-opentelemetry layer with the inbound `traceparent` attached as
   parent (`d7307f4`). Two gaps fixed along the way: the `otlp` feature
   was never compiled (so the env var exported nothing) and otelkit 2.0
   installs no global propagator (so extraction was a silent no-op).
   DONE step 3 prep: otelkit 2.1.0 installs the W3C propagator itself and
   re-exports the OTel surface, so CivitForge dropped its hand-pinned
   `opentelemetry` deps rather than guessing versions that match the kit's
   (`a40fd63`). Its `prometheus` feature is now enabled, which is what the
   health gate needs for real telemetry.
   DONE step 3: metrics and traces compose. otelkit 2.2.0 accepts
   Prometheus metrics alongside any primary exporter (the old single-
   exporter match made the production pair impossible), and 2.2.1 carries
   service.name onto the meter resource after a scrape showed
   unknown_service:civit-core. CivitForge initializes both, serves
   /api/v1/metrics/prometheus from the guard's registry, and the
   middleware records real OTel instruments — the previous
   increment_metric calls wrote to counters that were never registered, a
   silent no-op since the middleware was first written. The hand-rolled
   OTLP exporter (1,152 lines, zero references) is deleted. Verified
   live: exposition text/plain with attributed target_info and counters
   split by status code.
2. flag-kit — DONE step 1: kit `bucket()` rollout + `FlagName`
   validation adopted in `FeatureFlagService` (`dba123b`). DONE step 2:
   `flags_store::DbFlagStore` implements `FlagStore` over the
   `feature_flags` table and `AppState` carries the kit `Evaluator`
   (`d191e50`). The adapter is deliberately a read model — `set`/`delete`
   return `FlagError::Storage` because kit `Flag` cannot express
   targeting lists or descriptions, so admin-managed writes stay on the
   admin API rather than being silently clobbered. Remaining: expose the
   evaluator through the admin API/UI and migrate rollout SQL off
   `hashtext` onto kit bucketing.
3. Prerequisite for both: the local dev machine's `target/` sweeper
   makes server-side builds mandatory (see deploy/civitforge.sh).

## Consequences

- civit-telemetry remains as-is (functional, E2E-verified) until the
  dedicated sessions.
- The kit ecosystem's unreleased-version risk is now handled via a
  `[patch.crates-io]` pin for throttle-kit (pending the `remaining_burst`
  release); error-codes, error-classify, and shutdown-kit pins were
  dropped once published.
