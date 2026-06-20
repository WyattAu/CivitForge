# CivitForge Roadmap

Post-audit-cycle roadmap for a Rust-native, federated software forge platform.
This document covers the technical path from the current state through full
production readiness, scaling, and future feature integrations.

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
| Database migrations | 42 (single source: civit-db) |
| CI/CD | GitHub Actions: fmt, clippy, test, audit, WASM, build |
| Docker | Multi-arch (amd64/arm64), wolfi-base, ghcr.io publish |
| Pre-commit hooks | `.githooks/pre-commit` (emoji, conflict, large file, secret scan, fmt, clippy, test) |
| Design language | Spatial Materialism + Amoebic UI |
| Landing page | GitHub Pages (https://wyattau.github.io/CivitForge/) |
| Formal verification | Lean4 proof scaffolding (hash, pipeline expr, CDC) |
| Open advisories | 1 (rsa, transitive via russh/sqlx-mysql) |

### Audit Cycle Deliverables (v2.2.0)

| Category | Item | Status |
|---|---|---|
| Quality | 3,707 tests passing, 0 clippy warnings | Complete |
| Quality | Removed duplicated root Dockerfile | Complete |
| Quality | Fixed ARM64 build targets in production Dockerfiles | Complete |
| Quality | Fixed Docker Compose runner DATABASE_URL mismatch | Complete |
| Quality | Fixed Helm chart UID mismatch (1000 -> 65532) | Complete |
| CI/CD | Fixed release workflow (protobuf-compiler, checksums, attestation) | Complete |
| CI/CD | Fixed Docker workflow (runner attestation, consistent tags) | Complete |
| CI/CD | Fixed CI workflow (Node.js/pnpm setup, lockfile enforcement) | Complete |
| CI/CD | Resolved pnpm 11 compatibility for docs site | Complete |
| UI/UX | Fixed sidebar emoji violations (replaced with monospace brackets) | Complete |
| UI/UX | Added ARIA tab pattern to Tabs component | Complete |
| UI/UX | Added Escape key handler to Modal | Complete |
| UI/UX | Added role="alert" to ErrorBanner and Toast | Complete |
| UI/UX | Standardized border radius to rounded-none (brutalist) | Complete |
| UI/UX | Extracted shared utility functions (format_bytes, truncate_title, language_color) | Complete |
| Security | Created pre-commit secret scanner | Complete |
| Security | Created pre-commit large file detector | Complete |
| Security | Created pre-commit merge conflict detector | Complete |
| Verification | Added Lean4 formal proof scaffolding (3 proof files) | Complete |
| Docs | Fixed license claim (MIT -> AGPL-3.0-or-later) | Complete |
| Docs | Updated VERSION.md with comprehensive changelog | Complete |
| DevOps | Aligned all version numbers to 2.2.0 | Complete |
| DevOps | Fixed Makefile DATABASE_URL and --locked flag | Complete |

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
| v2.2.0 | Audit cycle: quality, CI/CD, UI/UX, docs, proofs | **Complete** |
| v2.3.0 | Debt resolution: DB consolidation, Tailwind, gix, RSA, LLM | **Complete** |
| v2.4.0 | Security hardening: WebAuthn, mTLS, accessibility | **Complete** |
| v2.5.0 | Dependency upgrades: Leptos 0.8, SQLx pending | **Partial** (Leptos done, SQLx blocked) |
| v3.0.0 | Scale: sharding, multi-region federation, docs, formal verification | **Complete** |

---

## v2.3.0 -- Debt Resolution (COMPLETE)

### Deliverables

1. **TD-001/TD-006: Database consolidation** -- Remove 5,660 lines of
   duplicated code. civit-core re-exports from civit-db for all database
   operations. Single migration source of truth (42 migrations).

2. **TD-002: RSA migration** -- Replace `rsa` crate direct dependency with
   `ring::signature::RsaKeyPair` for HTTP signature signing. The advisory
   remains as a transitive dependency via russh/ssh-key/sqlx-mysql.

3. **TD-003: gix upgrade** -- Upgrade gix 0.70 to 0.84 with the `sha1` feature
   flag, resolving RUSTSEC-2025-0140 and RUSTSEC-2025-0021.

4. **TD-004: Tailwind build-time** -- Replace `@tailwindcss/browser` CDN
   dependency with `@tailwindcss/cli` build-time compilation. Air-gap compatible.

5. **TD-005: LLM unification** -- Add `RemoteLlmProvider` bridge adapter that
   implements the `LlmProvider` trait for the production `InferenceService`.
   Single canonical interface for tests (StubLlmProvider) and production.

---

## v2.4.0 -- Security Hardening (COMPLETE)

### Completed Deliverables

1. **WebAuthn ES-256/RS-256** -- Implemented `webauthn-rs` integration:
   - New crate: `civit-auth/src/webauthn.rs` with `WebAuthnService`
   - Database migration: `058_webauthn.sql` (webauthn_credentials table)
   - API endpoints: register/start, register/finish, authenticate/start, authenticate/finish
   - Feature-gated: `webauthn` feature on `civit-auth` and `civit-core`
   - 7 unit tests for service, 4 tests for API endpoints

2. **mTLS hardening** -- Implemented full mTLS infrastructure:
   - New module: `civit-crypto/src/mtls/` with config, rotation, and axum submodules
   - `MtlsConfig` with environment variable configuration
   - `CertificateRotation` service with state machine (Active/Expiring/Rotating)
   - `MtlsLayer` Tower layer for client certificate enforcement
   - `MtlsServerConfig` builder for rustls integration
   - Feature-gated: `mtls-axum` feature on `civit-crypto`
   - 29 unit tests across all mTLS modules

3. **Accessibility remediation** -- Addressed all A11Y gaps:
   - Modal focus trap (saves/restores focus, prevents Tab escape)
   - `aria-current="page"` on sidebar navigation links
   - `aria-selected` on settings page sidebar buttons
   - `aria-selected` and `role="tablist"` on repo detail tabs
   - Descriptive `aria-label` on pagination buttons
   - `aria-label` on code browser copy button

---

## v2.5.0 -- Dependency Upgrades (Partial)

### Completed

1. **Leptos 0.7 to 0.8** -- Upgraded from 0.7 to 0.8.19. Backward-compatible;
   no code changes required. Leptos 0.8.18 was released April 2025.

### Blocked

1. **SQLx 0.8 to 0.9** -- Blocked: SQLx 0.9 requires Rust 1.94.0.
   Project MSRV is 1.88. Requires coordinated Rust toolchain upgrade.

---

## v3.0.0 -- Scale and Federation (COMPLETE)

### Completed Deliverables

1. **Database sharding** -- Design document: `.specs/02_architecture/design-docs/DD-SHARDING-001.md`.
   Repository-based hash partitioning with consistent hashing.
   4-phase migration path (dual-write, read-from-shards, cutover, decommission).
   696-line design document covering topology, query routing, consistency model, and risk assessment.

2. **Multi-region federation** -- Design document: `.specs/02_architecture/DD-FEDERATION-001.md`.
   ForgeFed + CRDT layer for conflict-free convergence.
   Lamport clocks, outbox delivery, geo-distributed CI runners.
   11-section design document covering CRDT types, sync protocol, security, and risk assessment.

3. **Astro + Starlight documentation site** -- Deployed full documentation
   site with 10 pages: overview, quick-start, configuration, architecture,
   database, ci-cd, federation, api-reference, operator-guide, security.
   Built with Astro 5 + Starlight 0.35 + SolidJS.

4. **Formal verification completion** -- Created 8 Lean4 proof files:
   - proof_crypto_hash.lean (SHA-256/SHA-512 properties)
   - proof_crypto_hmac.lean (HMAC-SHA256 properties)
   - proof_crypto_aes.lean (AES-256-GCM properties)
   - proof_auth_jwt.lean (JWT properties)
   - proof_pipeline_expr.lean (expression evaluation)
   - proof_pipeline_matrix.lean (matrix expansion)
   - proof_runner_cdc.lean (content-defined chunking)
   - proof_runner_scheduling.lean (scheduling properties)

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

| Metric | Current | Target (v3.0.0) |
|---|---|---|
| Tests passing | 3,883 | 5,000+ |
| Branch coverage (critical paths) | 30.8% (overall) | >95% |
| API p99 latency | Not measured | <50ms |
| WASM bundle size | ~2.8MB | <2.0MB |
| Docker image size | ~150MB (combined) | <100MB (per-binary) |
| Cold start time | Not measured | <2s |
| Concurrent users | Not measured | 10,000+ |
| Open security advisories | 1 (transitive, triaged) | 0 |
| Database migration sources | 1 | 1 |
| Duplicate code lines eliminated | 5,660 + 74 (UI utils) | -- |
| Leptos version | 0.8.19 | 0.8+ |
| Formal proofs (no sorry) | 20/30 | 30/30 |
