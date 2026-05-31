# ADR-0005: Zero Unsafe Code

## Status

Accepted

## Context

As a security-critical forge handling user code, authentication secrets, and SSH keys, memory safety is non-negotiable.

## Decision

All Rust crates MUST include `#![forbid(unsafe_code)]` at the crate root. No unsafe blocks permitted.

## Considerations

- `#![forbid(unsafe_code)]` is a hard ban, not a warning
- Rust's safe subset provides sufficient performance for our workload
- Security audit surface is minimized by eliminating unsafe code paths
- Cryptographic operations use safe abstractions (ring, sha2, hmac)
- File I/O and network operations have safe std library implementations

## Exceptions

None. If unsafe code is ever needed, it requires:
1. A new ADR documenting the specific exception
2. Security review
3. `unsafe` wrapped in a minimal, auditable module

## Consequences

- Every `.rs` file begins with `#![forbid(unsafe_code)]`
- CI enforces the directive via `cargo clippy -- -D warnings`
- No FFI to C libraries (consistent with gitoxide choice)
- Performance optimization must use safe Rust idioms
