# ADR-0002: SSE/EventSource authentication via `?token=` query parameter

- Status: accepted
- Date: 2026-09-06
- Deciders: Wyatt

## Context

The `EventSource` Web API cannot send custom headers (no `Authorization: Bearer`).
Server-side `extract_auth_user` only read the `Authorization` header, so the
notification stream always 401'd from the browser. The first fix (query-param
fallback using `tokio::task::block_in_place` + `block_on`) deadlocked the tokio
runtime inside async axum handlers → HTTP 500.

## Decision

1. `extract_auth_user` gains a query-parameter fallback: `Authorization` header
   first; if absent, parse `?token=` from the URI.
2. Query-param path validates **JWT synchronously** (CPU-bound, no DB) exactly like
   the header path; only `cf_pat_` tokens fall through to the async DB lookup via
   `block_in_place`.

## Consequences

- SSE streams authenticate from browsers/EventSource without protocol hacks.
- `block_on`-inside-async deadlocks are avoided for the hot path (JWT); PAT-in-query
  remains a rarely-used path and is documented as such.
- Tokens can leak into server access logs via query strings; acceptable on
  self-hosted deployments, revisit with short-lived SSE tickets if logging is
  ever centralized.
