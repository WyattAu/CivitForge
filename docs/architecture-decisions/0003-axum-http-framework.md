# ADR-0003: Axum HTTP Framework

## Status

Accepted

## Context

The API server needs a production-grade async HTTP framework supporting routing, middleware, WebSockets, and multipart uploads.

## Decision

Use Axum as the HTTP framework.

## Considerations

- Axum is built on `tokio` and `hyper` with Tower middleware ecosystem
- First-class WebSocket support via `axum::extract::ws`
- Multipart form support built-in
- Type-safe routing with extractors
- Compatible with Tower middleware stack (auth, rate limiting, tracing)
- Maintained by the Tokio team, aligning with our async runtime choice

## Alternatives Considered

- **Actix-web**: High performance but different actor model, less Tower ecosystem integration
- **Warp**: Filter-based routing, less ergonomic for complex API surfaces
- **Rocket**: Synchronous-first, less suitable for our async-heavy workload

## Consequences

- All HTTP routing defined via `axum::Router`
- Middleware uses Tower layers
- WebSocket handling through axum extractors
- Shared state via `axum::State`
