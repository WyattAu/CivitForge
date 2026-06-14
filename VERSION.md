# CivitForge Version Tracker

Version: 2.2.0
Last Updated: 2026-06-14
Tests: 3,707 passing (118 ignored, require PostgreSQL)
Clippy: 0 warnings
Format: 0 violations

## Artifact Summary

- Rust workspace crates: 12 (civit-shared, civit-pipeline, civit-core, civit-ci,
  civit-storage, civit-db, civit-git, civit-runner, civit-brain, civit-vfs,
  civit-crypto, civit-auth, civit-ui)
- Cargo standalone crates: 1 (civit-desktop, buildable separately)
- Unit + integration tests passing: 3,707
- Tests ignored (require PostgreSQL): 118
- Clippy warnings: 0
- Format violations: 0
- `#![forbid(unsafe_code)]`: Enforced across all crates
- API endpoints: ~85 routes (debug gated by --debug flag)
- Migrations: 001-027 (odd-numbered) in civit-db; mirrored 050+ in civit-core
- OpenAPI: v3.1 spec at /api/v1/openapi.json
- WASM: Leptos CSR build via trunk
- WASM rendering tests: gated on wasm32+csr
- Rust edition: 2024
- MSRV: 1.88
- E2E tests: Playwright (15 pages, all buttons/forms, benchmarks)
- GUI tests: Playwright (path routing traversal)
- Desktop smoke: Xvfb + GTK + WebKit
- Desktop: Tauri 2 (buildable, system deps required)
- Code search: Tantivy 0.22 (full-text with fuzzy, code-aware tokenization)
- Wiki: Git-backed via gix (bare repos with commit history)
- Pre-commit hook: `.githooks/pre-commit` (fmt + clippy + test + emoji scan)

## v2.2.0 Changes

- Fixed broken civit-db integration tests (create_pr signature: added auto_merge arg)
- Fixed PullRequest initializer (missing auto_merge field)
- Fixed UserResponse initializer (missing avatar_url/location/website fields)
- Resolved all clippy warnings (22 in civit-ui, 7 in civit-core tests)
- Consolidated pre-commit hooks: removed husky, canonicalized .githooks/
- Pre-commit hook now enforces: no emoji, fmt, clippy (-D warnings), tests
- Added conventional-commit subject hint and SKIP_PRE_COMMIT bypass

## v2.1.2 Changes

- Fixed GUI traverse routing (hash URLs → path URLs for Leptos CSR)
- Added WASM hydration wait (waitForSelector after networkidle)
- Fixed ServiceWorker MIME error (trunk copy-file for sw.js)
- Created Tauri desktop smoke test (Xvfb + GDK_BACKEND=x11)
- GUI traverse: 12/12 PASS, 63 actions, 0 errors
- Tauri smoke: 11/11 PASS (GTK init, WebKit spawn, WASM hydration, clean shutdown)

## v2.1.1 Changes

- Fixed WASM hydration bootstrap (inline_js → js_sys::eval IIFE)
- Added SPA client-side routing (ServeDir fallback → index.html)
- Fixed migration 025 typo (TIMSTAMPTZ → TIMESTAMPTZ)
- Suppressed service worker registration error (try/catch)
- Fixed Tauri desktop crate for standalone builds
- Added GUI test infrastructure (full-traverse.mjs + debug-capture.mjs)
- Verified Leptos CSR route rendering (all 6 routes correct in headless)

## v2.1.0 Changes

- Federation inbox ForgeFed dispatch (parse ActivityPub JSON → ForgeFedProcessor, tokio::spawn)
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

## Next

- Tantivy index population triggered by git push hooks
- Federation inbox outbound delivery (post-processing → ActivityPub delivery)
- Real-time WebSocket log streaming from runner event bus
- WebAuthn ES-256/RS256 authentication
- Project boards / Kanban
- Merge queue with status checks
