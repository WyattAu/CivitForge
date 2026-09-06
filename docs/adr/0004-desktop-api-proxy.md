# ADR-0004: Desktop file server proxies `/api/*` to the configured backend

- Status: accepted
- Date: 2026-09-06
- Deciders: Wyatt

## Context

In `--server-url` (remote backend) mode the desktop app serves the WASM dist from
`http://127.0.0.1:9092` and injects `window.__CIVIT_API_URL` pointing at the remote
API. A race exists: WASM `spawn_local` fetches can fire before Tauri's `eval`
injection lands, so requests hit the *file server* origin, receive `index.html`
(SPA fallback) instead of JSON, and pages show "Failed to load/parse" errors.

## Decision

The embedded file server **proxies any `/api/*` request** to the `--server-url`
backend (URL-agnostic, not hardcoded to `localhost:9091`). WASM requests are
therefore correct whether or not the API-URL injection has landed.

Supporting hardening in the same subsystem:
- File server binds ports 9092–9100 with retry (a stale prior instance panicking
  the thread on `AddrInUse` was killing the whole app).
- Dist dir resolution walks up from the binary to find the workspace root
  (no reliance on `CARGO_MANIFEST_DIR`).

## Consequences

- Same-origin API calls from WASM: zero CORS exposure in desktop mode.
- The API URL injection remains as the fast path (avoids the local proxy hop).
- `reqwest::blocking` inside the connection thread is acceptable (dedicated
  thread per request, not the tokio runtime).
