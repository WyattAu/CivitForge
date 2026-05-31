# ADR-0001: Rust Edition 2024

## Status

Accepted

## Context

CivitForge is a new project starting in 2025. Rust edition 2024 stabilizes important language features and is the current recommended edition.

## Decision

Use Rust edition 2024 for all crates in the workspace.

## Considerations

- edition 2024 provides `gen` blocks, `unsafe_op_in_unsafe_fn` warning by default, improved lifetime capture rules
- Requires Rust 1.88+, which is current stable
- No migration cost since this is a greenfield project

## Consequences

- All `Cargo.toml` files specify `edition = "2024"`
- `rust-toolchain.toml` pins to 1.88+
- No `edition = "2021"` anywhere in the codebase
