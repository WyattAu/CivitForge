# CivitForge Version Tracker

Version: 2.1.1
Last Updated: 2026-06-05
Tests: 3,076 passing
Clippy: 0 warnings

## Artifact Summary

- Rust source files: 320+
- Rust lines of code: ~125,000
- Cargo workspace crates: 8 (civit-shared, civit-pipeline, civit-core, civit-runner, civit-brain, civit-vfs, civit-crypto, civit-ui)
- Cargo standalone crates: 1 (civit-desktop, buildable separately)
- Unit tests passing: 3,076
- Clippy warnings: 0
- Format violations: 0
- `#![forbid(unsafe_code)`: Enforced across all crates
- API endpoints: ~85 routes (debug gated by --debug flag)
- Migrations: 001-027 (odd-numbered)
- OpenAPI: v3.1 spec at /api/v1/openapi.json
- WASM: 2.8MB WASM + 54KB JS (trunk build)
- WASM rendering tests: 34 tests (wasm-bindgen-test, gated on wasm32+csr)
- Rust edition: 2024
- MSRV: 1.88
- E2E tests: Playwright (15 pages, all buttons/forms, benchmarks)
- Desktop: Tauri 2 (buildable, system deps required)
- Code search: Tantivy 0.22 (full-text with fuzzy, code-aware tokenization)
- Wiki: Git-backed via gix (bare repos with commit history)

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
- v2.1.1
- v2.1.0

## Next

- Tantivy index population triggered by git push hooks
- Federation inbox outbound delivery (post-processing → ActivityPub delivery)
- Real-time WebSocket log streaming from runner event bus
- WebAuthn ES-256/RS256 authentication
- Project boards / Kanban
- Merge queue with status checks
