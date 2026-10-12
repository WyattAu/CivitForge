# ADR-0010: reachability audit of civit-core

- Status: accepted (2026-10)

## Context

ADR-0009 asserted, three times and wrongly, what in `civit-core` was live.
It designed a kit that already existed, claimed four live modules by
counting files, and called `events/websocket.rs` live because the router
reaches it — without checking that anything produced events for it to
deliver.

Every one of those errors was the same shape: treating what exists as what
runs. Each was corrected the same cheap way, a grep for call sites, which
is precisely the reason to do the sweep once, systematically, instead of
module by module as each claim is made.

## Method

For each `pub mod` in `civit-core/src/lib.rs`, count references to its path
across the whole workspace, excluding the module's own directory:

```
for m in $(pub mods of lib.rs); do
  grep -rn "\b$m::" crates/ --include=*.rs | grep -v "civit-core/src/$m/"
done
```

A grep is a starting point, not proof, so every candidate was then:

1. checked for a `pub use <module>::` re-export in `lib.rs` (a re-export
   is an inbound reference the module-path grep can miss),
2. spot-checked by searching for its public types by name, and
3. proven by `cargo check --workspace --all-targets` after removal, which
   is the authority: if anything referenced it, the compiler says so.

### One near-miss worth recording

The type spot-check for `distributed_tracing_v2` returned 11 hits, which
contradicted the module count of 0. The cause was a name collision:
`civit-telemetry` defines its own `DistributedTracer`, and
`lib.rs` re-exports *that* one via `pub use telemetry::distributed_tracing`.
The two types are unrelated. Without resolving the collision this audit
would have wrongly retained a 9,950-line module.

## Finding

Thirty modules in `civit-core` had zero inbound references anywhere in the
workspace, totalling **54,453 lines**. Not one was reachable from the
router, `AppState`, or any other crate.

Two further facts shaped the response:

- The three largest — `distributed_tracing_v2`, `log_aggregation`,
  `dashboard_reporting`, together ~27,500 lines — contained **zero tests**.
  They duplicated capabilities already shipped properly through otelkit and
  `civit-telemetry`.
- The rest ranged from 0 to 54 tests, which is evidence of how much
  deliberate work went into each.

Migrations exist for some of these features (`145_add_log_aggregation`,
`147_add_dashboard_reporting`, `190/192/211` follow-ups). They are
deliberately retained: applied migrations are history, not code, and
removing them would corrupt the upgrade path for deployed databases.

## Decision

Split the removal on a stated line rather than on convenience.

**Removed — unreferenced and unverified (11 modules, ~43,300 lines):**
`distributed_tracing_v2`, `log_aggregation`, `dashboard_reporting`,
`test_suite_management`, `performance_testing`, `code_quality`,
`test_coverage`, `tenant_isolation`, `review_automation`, `quality_gates`,
`plugins`. Nothing referenced them and nothing tested them, so they
contributed to every build while providing neither a code path nor a
safety net.

**Retained — unreferenced but tested (~11,300 lines):** `scaling`,
`backup`, `loadtest`, `static_assets`, `sla_monitoring`, `performance`,
`release_manager`, `protection`, `mirror`, `release`, `shutdown`,
`compliance_config`, `database_replication`, `data_residency`,
`data_archival`, `data_migration`, `search_index_sync`, `project_boards`,
`object_storage`, `data_retention`, `webhook`.

Test coverage is evidence that someone invested real work in these.
Whether to keep an unwired *feature* is a product decision — a forge may
genuinely intend multi-region, marketplace, or tenant isolation — and not
one this audit is entitled to make by deleting it. They stay, unwired,
with this record as the argument for the decision.

`realtime` and `events` are also unwired but are deliberately kept for the
same reason, and their queues were bounded in ADR-0009 so that wiring them
does not inherit an unbounded backlog.

## Consequences

- ~43,300 lines of dead code removed; the `civit-core` lib suite is
  unchanged at 1917 throughout, which independently confirms the removed
  code held no tests.
- The public API of `civit-core` narrows. Nothing in the workspace
  referenced the removed paths, and `civit-core` is not published to a
  registry, so there is no external consumer.
- ~11,300 lines of tested-but-unwired code remain, now explicitly
  documented rather than silently present. Carrying it is still a cost;
  it is a decision to make deliberately, not by accident.
- The audit is a snapshot. The cheapest guard against repeating ADR-0009's
  mistake is to run this reachability check before making claims about what
  runs — and to treat `cargo check --workspace --all-targets` as the
  authority when a removal is contested.