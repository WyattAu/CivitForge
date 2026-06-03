# Contributing to CivitForge

Guidelines for developing CivitForge. See root [CONTRIBUTING.md](../CONTRIBUTING.md) for the canonical guide.

## Quick Setup

```bash
git clone https://github.com/WyattAu/CivitForge.git
cd CivitForge

rustup component add clippy rustfmt
cargo build --workspace
docker compose up -d postgres redis
export DATABASE_URL="postgres://civit:civit-dev-secure-pw-2026@localhost:5432/civit"
export JWT_SECRET="dev-secret-key-32bytes-minimum"
cargo run -p civit-core
```

## Pre-Commit Hooks

Husky enforces on every commit:
1. `cargo fmt --all -- --check`
2. `cargo clippy --workspace --all-targets -- -D warnings`
3. `cargo test --workspace`

CI (GitHub Actions) runs the same checks on push/PR to `main`.

## Coding Standards

- `#![forbid(unsafe_code)]` enforced at crate level (ADR-001; tree-sitter C FFI gated behind feature flag)
- Zero clippy warnings (`-D warnings`)
- `cargo fmt` must pass with `--check`
- All new code must include unit tests
- Conventional Commits: `type(scope): description`
- No new dependencies without ADR justification

## Feature Flags

| Flag | Description | Unsafe? |
|------|-------------|---------|
| `syn-parser` | Rust AST via syn 2 | No |
| `swc-parser` | JS/TS via swc 12 | No |
| `sql-parser` | SQL via sqlparser | No |
| `treesitter` | Tree-sitter 12+ languages | Yes (C FFI, gated) |
| `ssh-server` | russh SSH daemon | No |

## Workspace Structure

```
crates/
├── civit-shared/       # Shared API types (backend + frontend)
├── civit-pipeline/     # Pipeline YAML parsing and validation
├── civit-core/         # API server, auth, DB, events, federation
├── civit-runner/       # CI execution, K8s operator, Podman
├── civit-brain/        # AI/ML, RAG, AST parsing
├── civit-crypto/       # Crypto primitives, CEL, enterprise auth
├── civit-vfs/          # gRPC filesystem
└── civit-ui/           # Leptos web frontend (WASM + SSR)
```
