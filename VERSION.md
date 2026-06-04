# CivitForge Version Tracker

Version: 2.0.0
Last Updated: 2026-06-04
Tests: 2,953 passing
Clippy: 0 warnings

## Artifact Summary

- Rust source files: 300+
- Rust lines of code: ~115,000
- Cargo workspace crates: 8 (civit-shared, civit-pipeline, civit-core, civit-runner, civit-brain, civit-vfs, civit-crypto, civit-ui)
- Cargo standalone crates: 1 (civit-desktop, not in workspace)
- Unit tests passing: 2,953
- Clippy warnings: 0
- Format violations: 0
- `#![forbid(unsafe_code)`: Enforced across all crates
- API endpoints: ~80 routes (including debug/diagnostics)
- Migrations: 001-025 (odd-numbered)
- OpenAPI: v3.1 spec at /api/v1/openapi.json
- Rust edition: 2024
- MSRV: 1.88

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

## Next

- WASM rendering tests via wasm-bindgen-test
- Implement HTTP signature validation in federation inbox
- Real-time WebSocket log streaming
- Tantivy code search upgrade
- Git-backed wiki storage
