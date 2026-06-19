# CivitForge Roadmap

Post-audit-cycle roadmap for a Rust-native, federated software forge platform.
This document covers versions v2.2.0 through v3.0.0 and beyond.

---

## Current State (v2.3.0-dev)

| Metric | Value |
|---|---|
| Workspace crates | 12 active + 1 standalone (desktop) |
| Tests passing | 3,600+ (118 ignored, require PostgreSQL) |
| Clippy warnings | 0 |
| Format violations | 0 |
| `#![forbid(unsafe_code)]` | Enforced across all crates |
| API endpoints | ~85 routes |
| Database migrations | 42 (single source: civit-db) |
| CI/CD | GitHub Actions: fmt, clippy, test, audit, WASM, build |
| Docker | Multi-arch (amd64/arm64), wolfi-base, ghcr.io publish |
| Pre-commit hooks | `.githooks/pre-commit` (fmt, clippy, test, emoji scan) |
| Design language | Spatial Materialism + Amoebic UI (92/92 drift checks pass) |
| Landing page | GitHub Pages (https://wyattau.github.io/CivitForge/) |
| Open advisories | 1 (rsa, transitive via russh/sqlx-mysql) |

---

## Technical Debt Register

### Resolved

| ID | Issue | Resolution | Version |
|---|---|---|---|
| TD-001 | DB layer duplication (5,660 lines) | Consolidated civit-core/src/db to re-export civit-db | v2.3.0 |
| TD-002 | RSA Marvin Attack (RUSTSEC-2023-0071) | Migrated signing to ring; remaining is transitive | v2.3.0 |
| TD-003 | gix transitive advisories | Upgraded gix 0.70 to 0.84 with SHA-1 collision detection | v2.3.0 |
| TD-004 | Tailwind via CDN (air-gap incompatible) | Build-time compilation via @tailwindcss/cli + trunk hook | v2.3.0 |
| TD-005 | Dual LLM abstractions | Unified with RemoteLlmProvider bridge adapter | v2.3.0 |
| TD-006 | Migration divergence (35 vs 42) | Single migration source (civit-db, 42 migrations) | v2.3.0 |

### Open

| ID | Issue | Severity | Status | Blocker |
|---|---|---|---|---|
| TD-007 | RUSTSEC-2023-0071 transitive (russh/sqlx-mysql) | Medium | Triaged | Waiting for upstream to drop rsa |

---

## Version Timeline

| Version | Focus | Status |
|---|---|---|
| v2.2.0 | Audit cycle: quality, CI/CD, UI/UX, docs | **Complete** |
| v2.3.0 | Debt resolution: DB consolidation, Tailwind, gix, RSA, LLM | **Complete** |
| v2.4.0 | Security hardening: WebAuthn, mTLS | Design complete, implementation pending |
| v2.5.0 | Dependency upgrades: SQLx, Leptos | Blocked on upstream (SQLx 0.9 needs Rust 1.94, Leptos 0.8 not stable) |
| v3.0.0 | Scale: sharding, multi-region federation, Astro docs | Design complete, implementation pending |

---

## v2.3.0 -- Debt Resolution (COMPLETE)

### Completed Deliverables

1. **TD-001/TD-006: Database consolidation** -- Removed 5,660 lines of
   duplicated code. civit-core now re-exports from civit-db for all database
   operations. Single migration source of truth (42 migrations).

2. **TD-002: RSA migration** -- Replaced `rsa` crate direct dependency with
   `ring::signature::RsaKeyPair` for HTTP signature signing. The advisory
   remains as a transitive dependency via russh/ssh-key/sqlx-mysql.

3. **TD-003: gix upgrade** -- Upgraded gix 0.70 to 0.84 with the `sha1` feature
   flag, resolving RUSTSEC-2025-0140 and RUSTSEC-2025-0021.

4. **TD-004: Tailwind build-time** -- Replaced `@tailwindcss/browser` CDN
   dependency with `@tailwindcss/cli` build-time compilation. Air-gap compatible.

5. **TD-005: LLM unification** -- Added `RemoteLlmProvider` bridge adapter that
   implements the `LlmProvider` trait for the production `InferenceService`.
   Single canonical interface for tests (StubLlmProvider) and production.

---

## v2.4.0 -- Security Hardening (Design Complete)

### Deliverables

1. **WebAuthn ES-256/RS-256** -- Design document: DD-WEBAUTHN-001.
   Implementation requires `webauthn-rs` crate, new database migration,
   and new API endpoints. Registration and login flows specified.

2. **mTLS hardening** -- Design document: DD-MTLS-001. Internal CA,
   certificate rotation, Axum/gRPC mTLS configuration. SPIFFE/SPIRE
   integration optional for Kubernetes deployments.

---

## v2.5.0 -- Dependency Upgrades (Blocked)

### Blocked Items

1. **SQLx 0.8 to 0.9** -- Blocked: SQLx 0.9 requires Rust 1.94.0.
   Project MSRV is 1.88. Requires coordinated Rust toolchain upgrade.

2. **Leptos 0.7 to 0.8+** -- Blocked: Leptos 0.8 is not yet released.
   Only 0.7 (stable) and 0.9.0-alpha are available.

---

## v3.0.0 -- Scale and Federation (Design Complete)

### Deliverables

1. **Database sharding** -- Design document: DD-SHARDING-001.
   Repository-based hash partitioning with consistent hashing.
   4-phase migration path (dual-write, read-from-shards, cutover, decommission).

2. **Multi-region federation** -- Design document: DD-FEDERATION-001.
   ForgeFed + CRDT layer for conflict-free convergence.
   Lamport clocks, outbox delivery, geo-distributed CI runners.

3. **Astro + Starlight documentation site** -- Site scaffolding created at
   `site/`. Astro 5 + Starlight 0.35 + SolidJS. Spatial Materialism themed.
   Deployment workflow configured (docs-site.yml).

---

## Continuous Monitoring

| Concern | Tool | Cadence |
|---|---|---|
| Security advisories | cargo-audit (CI) | Every push |
| Dependency drift | cargo-outdated | Weekly |
| Test coverage | tarpaulin (planned) | Weekly |
| Docker image CVEs | Trivy (Docker workflow) | Every build |
| Performance regression | Criterion benchmarks | Every release |

---

## Success Metrics

| Metric | Current (v2.3.0) | Target (v3.0.0) |
|---|---|---|
| Tests passing | 3,600+ | 5,000+ |
| Branch coverage (critical paths) | Not measured | >95% |
| API p99 latency | Not measured | <50ms |
| WASM bundle size | ~2.8MB | <2.0MB |
| Docker image size | ~150MB | <100MB |
| Cold start time | Not measured | <2s |
| Concurrent users | Not measured | 10,000+ |
| Open security advisories | 1 (transitive, triaged) | 0 |
| Database migration sources | 1 (was 2 divergent) | 1 |
| Duplicate code lines eliminated | 5,660 | -- |
