# ADR-0008: flag lifecycle, progressive delivery, and multi-repo dogfooding

- Status: accepted — research-driven (2026-10); supersedes the deferred
  "flag-kit remaining work" bullet of ADR-0007 and the Tier D/E backlog
- Date: 2026-10-05
- Research inputs: CMU ICSESEIP'20 feature-flag study, Uber Piranha
  (InfoQ'21), Datadog / Flagsmith stale-flag docs, LaunchDarkly + IBM
  two-level rollout randomization, Harness FME governance whitepaper,
  OpenFeature/OFREP 2026 landscape, staleflags, stale-flag-detector,
  bye-bye-flag

## Context

ADR-0007 shipped flag-kit bucketing, a DB read model, and an `Evaluator`.
Dogfooding exposed that the *evaluation* half of feature management was the
easy half. The research is unanimous on where the real cost sits:

- CMU: "removal of obsolete flags from the code and configurations was
  consistently identified as the key challenge"; few practical tools
  exist.
- Harness: 80% of flag removals touch more than one file, so manual
  cleanup does not scale.
- Uber Piranha exists *because* the hard part is deleting flag-gated code
  (AST + partial program evaluation), and its input is "this flag is
  stale" — which no system supplied automatically.
- LaunchDarkly/IBM describe two-level randomization: stickiness *within* a
  rollout cycle, deliberate re-randomization *across* cycles. A single
  unsalted hash gives the first and makes the second impossible without a
  code change.
- Datadog: a flag is stale when it is unarchived, not permanent, not in a
  running experiment, and either unchanged for N days or fully rolled out
  while still receiving evaluation traffic. Kill switches and permission
  gates are explicitly exempt ("mark as permanent").
- Governance is table stakes: owner, ticket linkage, default-off in
  production, RBAC per environment, audit trail (Harness/OPA).

Every one of these is a kit concern, not an application concern. Putting
them in CivitForge would make one consumer's needs look like a platform's.

## Decision

Four tracks. Each lands in the kit it belongs to, is verified by that
kit's tests, and is consumed by CivitForge.

### 1. flag-kit: lifecycle primitives

- `FlagKind` (`Release`, `Experiment`, `Operational`, `Permission`) with
  default lifetimes (operational 7d, experiment 40d, permission/permanent
  never). Rationale from stale-flag-detector's per-kind lifetimes and
  Datadog's permanent exemption.
- `FlagPolicy::is_stale(&Flag, now) -> Staleness` returning
  `Fresh | Aging | Stale | Permanent`, with the individual *signals*
  (age since last change, fully-rolled-out, never evaluated) rather than
  a bare bool, so callers can explain the verdict.
- `bucket_with_salt(subject, salt)` for two-level randomization:
  unsalted `bucket()` keeps today's behavior, the salted form gives a
  re-randomizable rollout cycle without a deploy.
- `Rollout` stages with monotonic, health-gated advancement:
  `advance(&health)` promotes only while the gate passes, and `rollback`
  is always permitted. Harness/Ops-level behavior kept small and total.
- OFREP-shaped evaluation response (`ofrep` module) so CivitForge can
  serve OpenFeature-compatible clients.

### 2. CivitForge: governance + stale surface

- `feature_flags` gains `kind`, `owner`, `ticket`, `salt`, and
  `last_evaluated_at`. Governance on create: owner and description
  required, `enabled` defaults false (Harness rules), kind defaults
  `Release`.
- `GET /api/v1/feature-flags/stale` classifies every flag using
  `FlagPolicy` and returns signals plus the audit-derived age, so the UI
  and CI can act on the same verdict the kit computed.
- The existing `Evaluator` gains salted evaluation; the admin API keeps
  ownership of writes (per ADR-0007 step 2, the store stays a read model).

### 3. New repo: `flaglab` — Piranha's missing orchestrator

Uber's weekly job "queries the flag management system for potential stale
flags and triggers Piranha" is the part nobody ships. `flaglab` is that
job: it audits flags across many repositories, computes staleness with
`FlagPolicy`, and emits removal candidates (and, later, refactorings).
It dogfoods flag-kit, otelkit, typed-id, error-classify, and ratelimit at
once, and it gives the kits a second and third consumer so regressions
surface somewhere other than CivitForge.

### 4. New repo: `kit-conformance` — the shared harness

A reusable workflow + `cargo` conformance suite every WyattAu kit repo
runs: `#![forbid(unsafe_code)]`, `#![deny(missing_docs)]`, MSRV check,
public-API semver diff, doc examples, and a no-`unwrap`-in-lib rule. This
is the "other repos can dogfood with" surface: new kits inherit the
harness on day one instead of rediscovering standards.

## Consequences

- `FlagKind` and `Staleness` are new public API: a minor version bump on
  flag-kit, no breaking changes (`bucket()` keeps its signature).
- The staleness signals depend on `last_evaluated_at`, which only the
  evaluation path can populate. Until the middleware records it,
  `is_stale` degrades to age-plus-rollout and says so in the signal list
  rather than claiming more than it knows.
- `flaglab` and `kit-conformance` are new public repos and must keep their
  own CI green; a kit that only CivitForge exercises is a kit with
  unknown blast radius.
- The throttle-kit `[patch.crates-io]` pin stays until
  `remaining_burst` ships; every new kit consumer inherits it, which is
  exactly the coupling `kit-conformance` is meant to make visible.

## Progress

- flag-kit 0.3.0: lifecycle primitives published (FlagKind, Staleness +
  StaleSignal, FlagPolicy, salted bucketing, Rollout). 38 kit tests.
- flag-kit 0.4.0: `NeverEvaluated` no longer fires when the source has no
  evaluation telemetry. Found by dogfooding in flaglab, where a static scan
  was accusing innocent flags of being unused. Absence of evidence is not
  evidence of absence of use.
- `flaglab` (new, public): 23 tests, clippy clean, verified end to end on
  fixture data. Second consumer of the policy, so CI and production review
  cannot disagree.
- `kit-conformance` (new, public): reusable harness; `flaglab`'s own CI
  calls it, so the harness is exercised rather than trusted.
- CivitForge migration 640 + `GET /api/v1/admin/feature-flags/stale` +
  governance on create. 1,891 core tests, 167 db tests.
- Verified against a live Postgres, which caught what unit tests could not:
  a validated name-format CHECK aborts the entire migration when any
  legacy row has a non-conforming name. It is `NOT VALID` now, so legacy
  rows are grandfathered while new writes are still rejected.
- The evaluation endpoint now records `last_evaluated_at`. Without it every
  zero-rollout flag would have been reported as unused — a signal that
  always fires is worse than no signal.

## Remaining in this ADR

- Expose governance and staleness verdicts in the flag admin UI.
- ~~`ofrep`: OpenFeature-compatible evaluation~~ — DONE: flag-kit 0.5.0
  publishes the wire types (three serialization bugs caught pre-publish:
  `targetingKey` swept into flattened attributes by a missing rename,
  `errorCode`/`errorDetails` serializing as null, and `#[serde(untagged)]`
  always writing the first variant so every bulk entry became a success).
  CivitForge serves both endpoints (14 tests) with a weak ETag that tracks
  rollout state rather than just flag names.
- Health-gated rollout controller: promote on green, roll back on red,
  using the OTel metrics that now exist.
- Wire `kit-conformance` into the remaining kit repos.

## Alternatives rejected

- **Build staleness detection only in CivitForge.** Rejected: it would
  fork flag-kit and leave every other consumer without it. The CMU
  finding is ecosystem-wide, so the fix belongs in the kit.
- **Adopt LaunchDarkly/Unleash/GO Feature Flag as a service.** Rejected:
  the kits are the product here; a hosted evaluator would make
  CivitForge a client of its own dependency.
- **AI-agent flag removal (bye-bye-flag) as the first step.** Deferred,
  not rejected: `flaglab` produces the candidate set first, because an
  agent removing code from an unverified staleness verdict is worse than
  no automation. Human-reviewable candidates first, automation once the
  verdicts are trustworthy.
- **Single unsalted hash for all rollouts.** Rejected by LaunchDarkly's
  two-level model: it makes cohort re-randomization impossible without a
  deploy, which is the exact thing rollout cycles exist to avoid.