# ADR-0006: Adopt the WyattAu kit ecosystem; delete hand-rolled equivalents

- Status: accepted
- Date: 2026-09-06
- Deciders: Wyatt

## Context

CivitForge hand-rolls infrastructure that already exists, production-grade, in
WyattAu's own crate ecosystem (`typed-id`, `json-envelope`, `paginate`,
`http-error`, `breaker`, `ws-kit`, `eventbus`, `webhookkit`, `auditlog`,
`flag-kit`, `otelkit`, `graceful`, `axum-stack`, `retry-backoff`, `cas-kit`,
`blobkit`, `poolkit`, `cachekit`, `healthkit`, `testkit`, `simd-tokenizer`…).
Duplication caused the majority of this cycle's bugs: three independently
hand-shaped response types produced a systemic "Failed to parse" bug class;
the hand-rolled circuit breaker and shutdown logic lag their kit counterparts.
`salting`, `tokenkit`, `cryptkit`, `validkit` adoption has already begun.

Standards enforcement exists as a reusable CI workflow:
`WyattAu/engineering-standards/.github/workflows/rust-kit.yml` with a
tiered gate matrix (Tier A: clippy-pedantic, `unwrap_used`/`panic` denied,
llvm-cov ≥90%, cargo-deny/audit, semver-checks, criterion latency baselines).

## Decision

Adopt kits in five risk-ordered phases, each gated on 219 E2E tests staying green:

1. **Data types** (zero behavior risk): `typed-id`, `json-envelope`, `paginate`,
   `http-error`, `errcode`, `app-error` — delete hand-rolled equivalents.
2. **Standards CI**: wire workspace to `rust-kit.yml` at Tier B; promote to Tier A.
3. **Infrastructure**: `breaker`, `graceful`, `axum-stack`, `retry-backoff`,
   `cachekit`, `poolkit`, `healthkit`, `testkit`.
4. **Features**: `flag-kit`, `auditlog`, `eventbus`, `otelkit`, `ws-kit`.
5. **Scale-out**: `cas-kit`+`blobkit` (LFS/artifacts), `delta-kit`, `actor-kit`
   (CI runners), `simd-tokenizer`, `plychart`.

## Consequences

- Net code deletion (~3–5k lines) and structural bug-class elimination
  (response-shape drift becomes impossible with `json-envelope`).
- SOLID arrives by construction: kits are single-responsibility with trait seams.
- Supply-chain coupling to our own crates is intentional — one ecosystem, one
  standards repo, one upgrade path (`cargo-semver-checks` guards both sides).
- Tier A promotion surfaces existing lint debt (the ~139 unwraps) as work items
  rather than silent risk.
