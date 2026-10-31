- Status: accepted; research-driven (2026-10); supersedes the deferred
  stale"; which no system supplied automatically.
### 3. New repo: `flaglab`: Piranha's missing orchestrator
### 4. New repo: `kit-conformance`: the shared harness
  zero-rollout flag would have been reported as unused; a signal that
  promotion a "regression"; hence `Rollout::reset_exposure()`, which zeroes
traffic, and nothing produced it; the chaos experiments table recorded
git-sourced dependencies; the estate resolves entirely from crates.io,
- which is precisely how the hand-rolled counters hid for so long
state; exposure, stage, consecutive bad windows, last decision with its
error rate and p99; plus the fifty most recent decisions across all
reported `exposure 5, stage 0, streak 0, hold/too_soon, error_rate None` -
`fully_rolled_out`; the lifecycle loop closes.
1. `tracing_middleware` was never installed on the router; request spans,
3. The controller applied the previous stage's exposure on promote; the
| flag-kit unit tests | verified; 67 tests, clippy clean, published |
| flaglab unit tests | verified; 23 tests, clippy clean, run end to end on fixture data |
| CivitForge core | verified; 1,905 tests |
| CivitForge db | verified; 167 tests |
| migration 640 against live Postgres | verified; applied to a table with legacy rows; bad names and bad kinds rejected on write |
| OFREP interop against a real SDK | **verified; 8/8** with `@openfeature/ofrep-provider`: missing flag falls back to the code default (FLAG_NOT_FOUND distinguishable), bulk + ETag + If-None-Match 304, SDK result matches the wire decision per subject |
| Playwright E2E | **verified; 219/219 effective** after the middleware, chaos, OFREP and Prometheus changes: 215 passed + 2 failed + 2 flaky in a 23-minute contended run; every one of the four passes on re-run in isolation (74/74 in 1.1 minutes on a quiet machine), which is the signature of load contention rather than regression |
   migration 640 carried semicolons inside comments; the server died on
- ~~`ofrep`: OpenFeature-compatible evaluation~~; DONE: flag-kit 0.5.0