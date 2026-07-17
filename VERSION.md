# CivitForge Version Tracker

Version: 3.0.0
Last Updated: 2026-07-17
Tests: 4,830 #[test] annotations (118 ignored, require PostgreSQL)
Clippy: 0 warnings
Format: 0 violations

## Artifact Summary

- Workspace crates: 20 (civit-shared, civit-pipeline, civit-core, civit-workflow,
  civit-federation, civit-ci, civit-storage, civit-db, civit-git, civit-runner,
  civit-brain, civit-vfs, civit-crypto, civit-auth, civit-shard, civit-types,
  civit-ui, civit-telemetry, civit-security)
- Standalone crates: 1 (civit-desktop, buildable separately)
- Total .rs files: 613
- Total lines of Rust: 347,247
- Unit + integration tests: 4,830 #[test] annotations
- Tests ignored (require PostgreSQL): 118
- Clippy warnings: 0
- Format violations: 0
- `#![forbid(unsafe_code)]`: Enforced across all crates
- API endpoints: ~85 routes (debug gated by --debug flag)
- Migrations: 612 SQL migration files in civit-db
- Versioned module files: 8 (_v*.rs)
- OpenAPI: v3.1 spec at /api/v1/openapi.json
- WASM: Leptos CSR build via trunk
- WASM rendering tests: gated on wasm32+csr
- Rust edition: 2024
- MSRV: 1.96
- E2E tests: Playwright (15 pages, all buttons/forms, benchmarks)
- GUI tests: Playwright (path routing traversal)
- Desktop smoke: Xvfb + GTK + WebKit
- Desktop: Tauri 2 (buildable, system deps required)
- Code search: Tantivy 0.22 (full-text with fuzzy, code-aware tokenization)
- Wiki: Git-backed via gix (bare repos with commit history)
- Pre-commit hook: `.githooks/pre-commit` (emoji + conflict + large file + secret scan + fmt + clippy + test)
- Formal verification scaffolding: `.specs/02_architecture/proofs/` (Lean4 proof sketches for hash, pipeline expr, CDC)

## v3.0.0 Changes

### Architecture Decomposition (5 new crates extracted)
- civit-workflow: workflow engine extracted from civit-core
- civit-federation: ActivityPub/ForgeFed inbox processing extracted from civit-core
- civit-shard: sharding/routing logic extracted from civit-core
- civit-types: shared type definitions extracted to reduce duplication across modules
- civit-telemetry: OpenTelemetry tracing and metrics extracted from civit-core
- civit-security: security scanning, compliance, and audit trail extracted from civit-core
- Consolidated 188 versioned files into 8 canonical versions

### Quality Improvements
- Resolved all 291 compiler warnings
- Fixed 174 test compilation errors, achieving 4,830 test annotations
- Eliminated unwrap() calls in favor of proper error handling
- Clippy: 0 warnings enforced across all crates
- Format: 0 violations enforced

### New Features
- Merge queue with status checks (civit-workflow)
- Full-text search indexing with Tantivy (code-aware tokenization, fuzzy matching)
- Federation inbox/ForgeFed dispatch (ActivityPub JSON parsing, tokio::spawn)
- Project boards / Kanban support (civit-workflow)
- Plugin system architecture (civit-workflow)
- Marketplace/extension API (8 endpoints, manifest validation)

### Infrastructure
- CI workflow fixes (Node.js/pnpm setup, lockfile validation)
- Dependabot configuration for dependency automation
- E2E tests with Playwright (full traversal, button/form coverage)
- Helm chart validation and production Helm manifests
- Prometheus/Grafana monitoring setup
- Production readiness hardening

### Documentation
- API examples and OpenAPI spec improvements
- Operational runbook for deployment and troubleshooting
- Security policy document
- Docker Compose production configuration
- Database migration guide

### Previous v2.x Changes
- See v2.2.0, v2.1.x, v2.0.0 sections below for full history

## v2.2.0 Changes

- Fixed broken civit-db integration tests (create_pr signature: added auto_merge arg)
- Fixed PullRequest initializer (missing auto_merge field)
- Fixed UserResponse initializer (missing avatar_url/location/website fields)
- Resolved all clippy warnings (22 in civit-ui, 7 in civit-core tests)
- Consolidated pre-commit hooks: removed husky, canonicalized .githooks/
- Pre-commit hook now enforces: no emoji, fmt, clippy (-D warnings), tests
- Added conventional-commit subject hint and SKIP_PRE_COMMIT bypass
- Removed duplicated root Dockerfile (superseded by container/civitforge/Dockerfile)
- Fixed ARM64 build targets in production Dockerfiles (was hardcoded to x86_64)
- Fixed Docker Compose runner DATABASE_URL mismatch (wrong password and database name)
- Aligned all version numbers to 2.2.0 (container manifests, Helm chart, Dockerfiles)
- Fixed Helm chart UID mismatch (1000 -> 65532 to match container images)
- Fixed Helm chart PostgreSQL version (16-alpine -> 17-alpine to match docker-compose)
- Fixed release workflow (added protobuf-compiler, removed unused llvm-tools, added checksums)
- Fixed Docker workflow (added runner attestation, consistent tag generation)
- Fixed CI workflow (added Node.js/pnpm setup, removed lockfile fallback)
- Fixed docs-site.yml (removed lockfile fallback)
- Fixed docs/index.html license claim (MIT -> AGPL-3.0-or-later)
- Created .cargo/audit.toml for cargo-audit configuration
- Enhanced pre-commit hook (merge conflict detection, large file detection, secret scanning)
- Added ARM64 target to rust-toolchain.toml
- Added node_modules to .dockerignore
- Fixed Makefile DATABASE_URL and --locked flag consistency
- Fixed smoke-test.sh (replaced Python JSON parsing with jq, removed hardcoded version)
- Fixed sidebar emoji violations (replaced unicode emoji with monospace bracket icons)
- Added ARIA tab pattern to Tabs component (role=tablist/tab/tabpanel)
- Added Escape key handler to Modal component
- Added role="alert" to ErrorBanner and ToastContainer
- Added aria-live="polite" to toast notifications
- Standardized border radius to rounded-none across Button, Input, Modal, Toast
- Added font-mono to Input component for brutalist consistency
- Added formal verification scaffolding (Lean4 proof sketches for crypto hash, pipeline expr, CDC)
- Created .specs/02_architecture/proofs/ directory with 3 proof files

## v2.1.2 Changes

- Fixed GUI traverse routing (hash URLs -> path URLs for Leptos CSR)
- Added WASM hydration wait (waitForSelector after networkidle)
- Fixed ServiceWorker MIME error (trunk copy-file for sw.js)
- Created Tauri desktop smoke test (Xvfb + GDK_BACKEND=x11)
- GUI traverse: 12/12 PASS, 63 actions, 0 errors
- Tauri smoke: 11/11 PASS (GTK init, WebKit spawn, WASM hydration, clean shutdown)

## v2.1.1 Changes

- Fixed WASM hydration bootstrap (inline_js -> js_sys::eval IIFE)
- Added SPA client-side routing (ServeDir fallback -> index.html)
- Fixed migration 025 typo (TIMSTAMPTZ -> TIMESTAMPTZ)
- Suppressed service worker registration error (try/catch)
- Fixed Tauri desktop crate for standalone builds
- Added GUI test infrastructure (full-traverse.mjs + debug-capture.mjs)
- Verified Leptos CSR route rendering (all 6 routes correct in headless)

## v2.1.0 Changes

- Federation inbox ForgeFed dispatch (parse ActivityPub JSON -> ForgeFedProcessor, tokio::spawn)
- 19 new federation inbox parsing tests
- WASM rendering tests via wasm-bindgen-test (34 tests, DOM/events/leptos signals)
- Real-time WebSocket log streaming (LogBroadcaster, SSE endpoint, pipeline topic subscriptions)
- Tantivy code search (in-memory/file-backed index, fuzzy, code-aware tokenization, repo/language filters)
- Git-backed wiki storage (gix bare repos, commit history, page CRUD, diff, search)
- Migration 027: wiki git_synced/is_git_commit columns
- E2E traversal fix for Node 26 module resolution

## v2.0.0 Changes

- OpenAPI 3.1 spec (50+ endpoints, JSON + YAML)
- Marketplace/extension API (8 endpoints, manifest validation)
- PWA manifest + service worker (network-first caching)
- Frontend error capture (window.onerror, unhandledrejection, console interceptors)
- Error boundary component (CatchError) with retry
- Debug panel (Ctrl+Shift+D, error list, stack traces)
- Backend debug middleware (request logging, slow query detection)
- Panic catcher middleware (structured 500 responses)
- Diagnostics endpoints (health, memory, routes, panic trigger)
- Client error reporting endpoint
- Playwright E2E traversal script (12+ routes, all buttons/forms, error capture)
- Tauri desktop foundation (system tray, git commands, CSP)

## Tags

- v1.0.0 (6c625cc)
- v1.1.0
- v1.1.1
- v1.2.0
- v1.3.0
- v1.4.0
- v1.5.0
- v2.0.0
- v2.1.0
- v2.1.1
- v2.1.2
- v2.2.0
- v3.0.0
