# Changelog

Format based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
Adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [2.0.0] - 2026-06-04

### Added

- **OpenAPI 3.1 spec** (`GET /api/v1/openapi.json`, `GET /api/v1/openapi.yaml`) — auto-generated spec covering 50+ endpoints across auth, repos, issues, wiki, pipelines, runners, search, activity, users, orgs, federation, registry, marketplace
- **Marketplace/extension API** — 8 endpoints: list, get, publish, delete, verify, install, uninstall extensions with manifest validation and permission sandbox
- **PWA support** — `manifest.json` (standalone display, dark theme, green accent), service worker (`sw.js`) with network-first caching, stale cache cleanup, `SKIP_WAITING` message handler
- **Frontend error capture** (`civit-ui::error_capture`) — global `onerror`/`unhandledrejection`/`console.error`/`console.warn` interceptors via WASM inline JS, `OnceLock<RwLock<Vec>>` in-memory store (cap 500), JS-Rust sync bridge
- **Error boundary component** (`CatchError`) — wraps child components with retry-on-error fallback UI, retry counter
- **Debug panel** (`DebugPanel`, feature-gated) — Ctrl+Shift+D toggle, floating error badge, scrollable error list with source-colored badges, stack traces, clear/refresh buttons
- **Backend debug middleware** — request/response logging with timing (error/warn/info/slow), `SlowQueryDetector` for DB queries
- **Panic catcher middleware** — converts handler panics to structured 500 JSON responses
- **Diagnostics endpoints** — `GET /debug/health` (DB/Redis/memory with latencies), `GET /debug/diagnostics` (process stats), `GET /debug/routes`, `GET /debug/panic` (debug-only)
- **Client error reporting** — `POST /debug/error-reports` endpoint for frontend error submission
- **Playwright E2E traversal script** (`tests/e2e/`) — visits all 12+ routes, fills all forms, clicks all buttons, captures console errors/page errors/network failures, screenshots on failure, JSON report
- **Tauri desktop foundation** (`crates/civit-desktop/`) — Tauri 2 shell, tauri.conf.json, system tray (Show/Quit), native git commands (list/status/clone via gix), CSP for WASM

### Changed

- **Debug panel feature-gated** behind `debug-panel` cargo feature (off by default)
- **Error boundary renamed** from `ErrorBoundary` to `CatchError` to avoid conflict with Leptos built-in
- **Service worker registered** on CSR mount via `js_sys::eval`

### Infrastructure

- 2,953 total tests (+65 from v1.5.0)
- Zero clippy warnings, zero fmt issues
- All new code `#![forbid(unsafe_code)]`

## [1.5.0] - 2026-06-04

### Added

- **Read replica router** (`civit-core::db::replica_router`) — primary/replica pool splitting with automatic failover, lag monitoring, health checks
- **Multi-region replication transport** (`civit-core::federation::replication`) — channel-based replication transport with SHA-256 checksum verification, heartbeat loop, peer health monitoring
- **Vector clocks** (`civit-core::federation::vector_clock`) — generic vector clock for multi-region conflict detection with happened-before, concurrent, merge, descends-from operations
- **Kubernetes operator** (`civit-brain::operator`) — CivitForgeApp CRD (group `civitforge.dev/v1alpha1`), reconciler with finalizer management, deployment health checker
- **CDN artifact pre-signed URLs** (`civit-core::cache::pre_signed`) — HMAC-SHA256 signed download URLs with configurable TTL, token validation, expiry checking
- **Artifact serving API** — download (token-validated), pre-signed URL generation, HEAD (ETag/cache validation), cache invalidation endpoints
- **Cache headers utility** (`CacheHeaders`) — public/private Cache-Control, ETag generation, If-None-Match parsing, 304 Not Modified support
- **Password change now verifies current password** — fetches stored hash, compares with submitted value, returns 403 on mismatch
- **Code browser directory detection via tree lookup** — uses gix `EntryMode::is_tree()` instead of fragile `.` heuristic
- **Federation HTTP signature validation on inbox** — validates `Signature` header via `SignatureVerifier`, returns 401 on failure
- **Real Ed25519 keypair for federation actor** — lazy-generated via `OnceLock`, SPKI DER-to-PEM encoding

### Changed

- **Shared UI utilities extracted** — `relative_time()`, `truncate_uuid()`, `status_badge_color()`, `status_label()`, `get_input_value()` moved from 8 pages into `civit-ui::utils`
- **Wiki page content fetch wired** — clicking sidebar page now fetches page content via `Effect::new()` watching `active_slug`
- **Repo settings page wired** — fetches repo data on mount, form populated, submit sends `PUT /repos/{owner}/{name}`
- **Explore page search wired** — search query passed as `?q=` parameter to API

### Infrastructure

- 2,888 total tests (+14 from v1.4.0)
- Zero clippy warnings, zero fmt issues
- All new code `#![forbid(unsafe_code)]`

## [1.4.0] - 2026-06-04

### Added

- **Activity feed API** (`GET /api/v1/activity`) — chronologically sorted platform events with filtering by repo/org
- **Code browser API** (`GET /api/v1/repos/{owner}/{name}/tree`, `GET /api/v1/repos/{owner}/{name}/blob`) — directory listing and file content via gix bare repo traversal
- **Password change endpoint** (`POST /api/v1/users/{id}/password`) — authenticated user password update with SHA-256 hashing
- **ActivityPub federation routes** — WebFinger discovery (`.well-known/webfinger`), actor endpoint, inbox/outbox, all gated by `federation_enabled`
- **Per-repo encryption keys** (`civit-crypto::repo_keys`) — HKDF-derived AES-256-GCM keys per repository with key rotation support and in-memory key store
- **SAML SSO foundation** (`civit-crypto::saml`) — SAMLResponse parser, attribute extraction, SHA-256 digest verification
- **Multi-tenancy methods** — org-scoped repository counting, user-accessible repo listing, org usage statistics, active runner counting
- **Activity page wired to real API** — filterable activity feed (push, issues, PRs, repos, wiki, forks, stars)
- **Code browser page wired to real API** — auto-detects file vs directory, table view for trees, syntax-highlighted file viewer for blobs
- **Migration 025** — `activity_events` table (indexed by actor, repo, org, timestamp), `federation_actors` table, `federation_activities` table

### Infrastructure

- 13th migration (025): activity + federation schema
- 2,874 total tests (+159 from v1.3.0)
- Zero clippy warnings, zero fmt issues
- All new code `#![forbid(unsafe_code)]`

## [1.1.0] - 2026-06-02

### Added

- Token refresh endpoint (`POST /api/v1/auth/refresh`) -- validates existing JWT, issues new token
- RSA-SHA256 and ECDSA-P256 signing in federation HTTP signatures (Ed25519, HMAC-SHA256, RSA-SHA256, ECDSA-P256)
- `Retriever` trait for RAG pipeline -- `RagOrchestrator` accepts `Box<dyn Retriever>` for swappable backends
- `KeywordRetriever` -- keyword-based retrieval with thread-safe `RwLock` internals
- Unified diff for wiki via LCS algorithm (`unified_diff()`, `DiffHunk`, `DiffLine`, `lcs_lines()`, `apply_diff_hunks()`)
- Wiki content snapshot column (`wiki_revisions.content_snapshot`, migration 019/020)
- AES-256-GCM encryption for pipeline variables using `ring::aead`
- Checkout, cache, and artifact action handlers in CI executor
- Service container lifecycle (`start_services()`, `stop_all_services()`, `ServiceGuard` RAII)
- CEL expression evaluator (`civit-pipeline::expr`) -- `==`, `!=`, `contains`, `startsWith`, `endsWith`, `matches`, `&&`, `||`, `!`, `${{ var }}` expansion
- PostgreSQL full-text search (`tsvector`/`tsquery`, GIN indexes, auto-update triggers, migration 021/022)
- Git archive-based pipeline YAML reading (tries `git archive` first, falls back to filesystem)

### Changed

- Runner owner lookup joins `users` table instead of hardcoded value
- Code search uses `plainto_tsquery` for ranked full-text results (replaces ILIKE)
- Wiki search uses `search_vector @@ plainto_tsquery` with `ts_rank` ordering

### Fixed

- Podman log paths return proper `Err` instead of placeholder text
- OCI dedup `get_layer()` corrected with `ChunkStore::put_direct()` for digest-based storage
- Test stubs gated behind `#[cfg(test)]` -- `StubLlmProvider`, `StubReviewAgent`, `StubVulnScanner`, `MockRemoteProvider`
- CEL evaluator doc comment updated to reflect actual implementation
- Encryption key warning logged when `CIVIT_ENCRYPTION_KEY` env var not set

### Dependencies

- Added `rsa 0.9` (RSA PKCS1v15 signing) to `civit-core`
- Added `pkcs8 0.10` (PKCS#8 DER key parsing) to `civit-core`
- Added `tar 0.4` (tar archive extraction) to `civit-core`
- Added `sha2` `oid` feature for pkcs8 ecosystem

## [1.0.0] - 2026-06-02

### Added

#### Workspace restructure

- Moved all crates into `crates/` directory layout
- Created `civit-shared` crate with shared API request/response types
- Created `civit-pipeline` crate for YAML spec parsing and validation

#### Permission system

- Full RBAC with 6-role hierarchy: Owner, Admin, Maintainer, Developer, Reporter, Guest
- Deny-always-wins policy evaluation with hierarchical inheritance (org, repo, branch)
- 11 permission resources, 22 permission actions with conditional policy checks
- Branch protection: push restrictions, required reviews, force-push control
- DB schema: `member_roles`, `repo_policies`, `branch_protections`, `pipeline_variables`
- Auth middleware applied to all API routes

#### CI/CD pipeline backend

- Pipeline YAML spec (`.civit/pipeline.yaml`): triggers (push, tag, PR, schedule, dispatch), services, cache, secrets, workspace, concurrency, expressions
- `civit-pipeline` crate: YAML parser with 80+ test vectors, validation, expression evaluation
- Runner registration and token-based auth protocol (6-endpoint internal API)
- `civit-runner` standalone daemon: Podman CLI transport, volume mounts, log streaming
- Concurrency groups with cancel-in-progress semantics
- Secret resolution via API (runner calls `/api/v1/runners/{id}/secrets`)
- CEL-based expression evaluation for conditional step execution
- 17 pipeline API endpoints + 11 runner management endpoints

#### OCI container registry

- 20 OCI Distribution Spec v1.1 endpoints
- 8 management API endpoints (list images, tags, layers, SBOM, vulns, RBAC policies, GC trigger)
- Per-image RBAC with namespace enforcement
- Content-addressed blob deduplication, tag immutability, multi-arch manifest lists
- OSV vulnerability scanning per push, Cosign image signature verification, SLSA provenance attestation

#### Issue tracking

- 18 API endpoints, state machine (open, in_progress, closed, reopen)
- Timeline audit trail, comments with reactions, labels, milestones with progress tracking, assignees

#### Wiki

- 9 API endpoints, page CRUD with Markdown content
- Page history with diff between revisions, raw content export, full-text search

#### Code search

- 3 search endpoints (global, per-repo, language filter)
- PostgreSQL `tsvector`/`tsquery` full-text search with GIN indexes

#### Leptos web UI

- `civit-ui` crate: Leptos 0.7 CSR + SSR
- 11 reusable UI components, 10 page shells
- Typed API client (reqwest), auth state management with Leptos signals, Tailwind CSS dark mode

#### Integration and polish

- All API endpoints documented
- 2,611 tests passing, 0 clippy warnings, 0 format violations
- `#![forbid(unsafe_code)]` enforced across all crates

### Security

- LICENSE replaced with AGPL-3.0-or-later
- Runner `run()` fails-closed when Podman unreachable
- CEL `matches()` fails-closed when regex engine not integrated
- CI workflow: `permissions: contents: read` (least-privilege)
- SAML signature validation: fail-closed until XML-DSig

### Fixed

- `civit-brain/vectordb.rs`: moved `QdrantVectorDbAdapter` before `#[cfg(test)]` module
- `civit-core/ssh/auth.rs`: fixed DashMap deadlock in `RateLimiter::check()`
- Runner execution: fixed missing `volumes`, nested match for timeout/container-error, double-ref
- `runners.rs`: fixed path parameter types, collapsible if-let
- Multiple clippy fixes across workspace

## [0.1.0] - 2026-05-30

### Added

- Project specification foundation (requirements, domain analysis, traceability matrix, capability matrix)
- 69 EARS-format requirements across 5 areas (VCS, LFS+, CI/CD, AI, Federation)
- 16 non-functional requirements with measurable acceptance criteria
