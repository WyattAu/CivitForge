# CivitForge Roadmap

Post-audit-cycle roadmap for a Rust-native, federated software forge platform.
This document covers versions v2.2.0 through v3.0.0 and beyond.

---

## Current State (v2.2.0)

| Metric | Value |
|---|---|
| Workspace crates | 12 active + 1 standalone (desktop) |
| Tests passing | 3,707 (118 ignored, require PostgreSQL) |
| Clippy warnings | 0 |
| Format violations | 0 |
| `#![forbid(unsafe_code)]` | Enforced across all crates |
| API endpoints | ~85 routes |
| Database migrations | civit-core: 35, civit-db: 42 (divergent) |
| CI/CD | GitHub Actions: fmt, clippy, test, audit, WASM, build |
| Docker | Multi-arch (amd64/arm64), wolfi-base, ghcr.io publish |
| Pre-commit hooks | `.githooks/pre-commit` (fmt, clippy, test, emoji scan) |
| Design language | Spatial Materialism + Amoebic UI (92/92 drift checks pass) |
| Landing page | GitHub Pages (https://wyattau.github.io/CivitForge/) |

### Audit Cycle Completion (v2.2.0)

**Phase 1 -- Code Quality:** Fixed 13 compile errors (create_pr signature,
PullRequest/UserResponse initializers). Resolved 22 clippy warnings. Enforced
`-D warnings` in CI and pre-commit hook. Zero warnings across workspace.

**Phase 2 -- CI/CD:** Pinned action versions, added concurrency control,
fixed security audit gating (cargo-audit via taiki-e/install-action),
consolidated WASM build into main CI, added Docker publish workflow with
SBOM and provenance attestation.

**Phase 3 -- GUI/UX:** Eliminated all emoji from UI source (sidebar nav,
theme toggle, home cards, issue templates). Enhanced emoji scanner to catch
`\u{1Fxxx}` escape sequences. Created Playwright design-language drift
traversal (23 routes, 92/92 checks pass).

**Phase 4 -- Documentation:** Updated CONTRIBUTING.md (husky to .githooks),
deduplicated architecture.md via symlink, fixed version references.

**Phase 5 -- CI/CD Debug:** Resolved OOM on GitHub runners (CARGO_BUILD_JOBS=2
+ line-tables-only debuginfo). Fixed Docker build (missing crate directories,
trunk binary installation). CI pipeline fully green.

---

## Technical Debt Register

Issues identified during the audit cycle, prioritized by severity.

### TD-001: Database Layer Duplication (Critical)

`civit-core/src/db/` duplicates `civit-db/src/` with 392 lines of drift in
`repository.rs`. The two copies have diverged: civit-db has 42 migrations,
civit-core has 35. Changes to one (e.g., auto_merge field) do not propagate.

**Remediation:** Consolidate civit-core to use civit-db as a dependency,
extending it via trait composition rather than copying. Target: v2.3.0.

### TD-002: Security Advisory -- rsa Marvin Attack (Medium)

RUSTSEC-2023-0071 affects RSA decryption timing. CivitForge uses rsa only
for HTTP signature signing (federation/http_signatures.rs), which is not
the vulnerable code path.

**Remediation:** Migrate to `ring` or `aws-lc-rs` RSA signing. Target: v2.4.0.

### TD-003: Security Advisory -- gix Transitive (Medium)

RUSTSEC-2025-0140 (gix-date) and RUSTSEC-2025-0021 (gix-features SHA-1)
are transitive via gix 0.70. SHA-1 collision detection is available in
newer gix versions.

**Remediation:** Upgrade gix 0.70 to 0.84 (breaking API changes require
phased migration). Target: v2.5.0.

### TD-004: Tailwind via CDN (Medium)

The WASM UI loads Tailwind CSS v4 from a CDN (`@tailwindcss/browser`). This
is incompatible with air-gapped deployments and adds runtime overhead.

**Remediation:** Compile Tailwind at build time via trunk's Tailwind
integration. Target: v2.3.0.

### TD-005: Dual LLM Abstractions (Low)

`LlmProvider` trait (provider.rs, test-only StubLlmProvider) and
`InferenceEngine` (inference.rs, production HTTP client) are parallel
abstractions for LLM inference.

**Remediation:** Unify behind a single trait. Target: v3.0.0.

### TD-006: Migration Divergence (Critical)

civit-core and civit-db have separate migration sets (35 vs 42) that create
the same tables with different schemas. This causes runtime errors
(e.g., "site_settings does not exist" when civit-core's migrations don't
include newer tables).

**Remediation:** Single migration source of truth. Target: v2.3.0
(depends on TD-001).

---

## Version Timeline

| Version | Focus | Target | Status |
|---|---|---|---|
| v2.2.0 | Audit cycle: quality, CI/CD, UI/UX, docs | Complete | Done |
| v2.3.0 | Debt resolution: DB consolidation, Tailwind build-time | Q3 2026 | Planned |
| v2.4.0 | Security hardening: rsa migration, WebAuthn ES-256 | Q4 2026 | Planned |
| v2.5.0 | Dependency upgrades: gix 0.84, SQLx 0.9 | Q1 2027 | Planned |
| v3.0.0 | Scale: sharding, multi-region federation, LLM unification | Q2 2027 | Planned |

---

## v2.3.0 -- Debt Resolution

**Goal:** Eliminate critical technical debt blocking production deployment.

### Deliverables

1. **TD-001/TD-006: Database consolidation**
   - Remove `civit-core/src/db/{models,repository,pool,session}.rs` duplication
   - civit-core depends on civit-db for all database operations
   - Single migration source of truth (unified migration directory)
   - Replica router moves to civit-core as a pool wrapper

2. **TD-004: Tailwind build-time compilation**
   - Replace CDN `@tailwindcss/browser` with trunk Tailwind plugin
   - Generate `tailwind.css` at build time
   - Verify air-gapped compatibility (no external CDN dependencies)

3. **Flaky test investigation**
   - Identify and stabilize the intermittent test failure observed in
     parallel test execution (1 failure in 3,707, non-reproducible)

---

## v2.4.0 -- Security Hardening

**Goal:** Close all open security advisories and add missing auth features.

### Deliverables

1. **TD-002: RSA migration**
   - Replace `rsa` crate with `ring` or `aws-lc-rs` for HTTP signature signing
   - Update federation/http_signatures.rs
   - Remove RUSTSEC-2023-0071 from audit triage

2. **WebAuthn ES-256/RS-256**
   - Add WebAuthn authentication with ES-256 (P-256) primary
   - RS-256 fallback for compatibility
   - Hardware security key support (YubiKey, Titan, FIDO2)

3. **mTLS hardening**
   - Enforce mutual TLS for inter-service communication
   - Certificate rotation automation
   - SPIFFE/SPIRE integration (optional)

---

## v2.5.0 -- Dependency Upgrades

**Goal:** Modernize the dependency tree to resolve all transitive advisories.

### Deliverables

1. **TD-003: gix 0.70 to 0.84 migration**
   - Phased migration: API changes in commit graph, tree walking, archive
   - Enable SHA-1 collision detection (RUSTSEC-2025-0021)
   - Fix gix-date non-UTF8 handling (RUSTSEC-2025-0140)

2. **SQLx 0.8 to 0.9**
   - New query macros, connection pool improvements
   - Prepared statement caching

3. **Leptos 0.7 to latest**
   - Reactive primitives updates
   - SSR hydration improvements

---

## v3.0.0 -- Scale and Federation

**Goal:** Production-grade horizontal scalability and multi-region federation.

### Deliverables

1. **Database sharding**
   - Repository-based sharding (repo_id hash partitioning)
   - Cross-shard query routing
   - Shard rebalancing without downtime

2. **Multi-region federation**
   - Active-active replication via ForgeFed ActivityPub
   - Conflict-free replicated data types (CRDTs) for issues/PRs
   - Geo-distributed CI runner pools

3. **TD-005: LLM abstraction unification**
   - Single `LlmProvider` trait for all inference paths
   - Air-gapped local inference (vLLM/Ollama)
   - Streaming inference with SSE

4. **Documentation site (Astro + Starlight)**
   - Replace raw HTML landing page with Astro static site
   - Starlight-powered documentation with search
   - SolidJS interactive components for API explorer

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

| Metric | Current (v2.2.0) | Target (v3.0.0) |
|---|---|---|
| Tests passing | 3,707 | 5,000+ |
| Branch coverage (critical paths) | Not measured | >95% |
| API p99 latency | Not measured | <50ms |
| WASM bundle size | ~2.8MB | <2.0MB |
| Docker image size | ~150MB | <100MB |
| Cold start time | Not measured | <2s |
| Concurrent users | Not measured | 10,000+ |
| Open security advisories | 3 (triaged) | 0 |
| Database migration drift | 2 sources | 1 source |
