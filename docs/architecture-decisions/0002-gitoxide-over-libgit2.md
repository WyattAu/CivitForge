# ADR-0002: gitoxide over libgit2

## Status

Accepted

## Context

We need a Rust-native Git library for repository operations (clone, push, fetch, tree walking, ref management).

## Decision

Use `gitoxide` (the `gix` crate) instead of `libgit2` bindings.

## Considerations

- `gix` is a pure Rust implementation of Git internals
- `libgit2` is a C library with Rust bindings (`git2`)
- `gix` avoids C dependency management and FFI overhead
- `gix` supports parallel operations via the `parallel` feature
- `gix` has active maintenance and is approaching feature parity with libgit2
- `gix` supports `max-performance-safe` feature set

## Consequences

- Dependency on `gix` crate (version 0.70+)
- No C build toolchain required for Git operations
- Parallel pack operations for large repository handling
- Some niche libgit2 features may not yet be available
