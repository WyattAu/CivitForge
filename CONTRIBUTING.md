# Contributing to CivitForge

## Prerequisites

- Rust 1.88+ (edition 2024)
- PostgreSQL 17+
- Redis 7+ (optional, for sessions/edge cache)
- Podman or Docker (optional, for CI runner)
- Node.js (optional, for husky pre-commit hooks)

## Development Setup

```bash
git clone https://github.com/WyattAu/CivitForge.git
cd CivitForge

# Install toolchain
rustup component add clippy rustfmt

# Build
cargo build --workspace

# Start dependencies
docker compose up -d postgres redis

# Set required environment variables
export DATABASE_URL="postgres://civit:civit-dev-secure-pw-2026@localhost:5432/civit"
export REDIS_URL="redis://:civit-redis-dev-2026@localhost:6379"
export JWT_SECRET="dev-secret-key-32bytes-minimum"

# Run the server (binds to 127.0.0.1:9091 by default)
make run
```

## Pre-Commit Hooks

Husky enforces three checks on every commit:

```bash
# Format
cargo fmt --all -- --check

# Lint
cargo clippy --workspace --all-targets -- -D warnings

# Test
cargo test --workspace
```

To install hooks:
```bash
npm install   # triggers husky via package.json prepare script
```

Or manually:
```bash
make hooks
```

CI (GitHub Actions) runs the same three checks plus a release build on every push/PR to `main`.

## Coding Standards

### Safety

- Every Rust crate must enforce `#![forbid(unsafe_code)]` at the crate level (ADR-001)
- The only exception is tree-sitter C FFI, which is gated behind the `treesitter` feature flag
- No new unsafe code without an ADR approval process

### Linting

All code must pass:

```bash
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --check --all
```

### Code Style

- Follow standard Rust idioms
- Prefer `thiserror` for library error types, `anyhow` for binary errors
- Module-level documentation is encouraged
- Public interfaces go in `pub mod` at the crate root; internals are `pub(crate)` or private
- Configuration via environment variables (not config files)

### Dependencies

- No new dependencies without justification in an ADR
- Prefer stdlib or existing workspace dependencies
- Prefer pure-Rust crates over C FFI
- Prefer API-based integrations over bundled ML models

## Testing

```bash
# All workspace tests
cargo test --workspace --locked

# Specific crate
cargo test -p civit-core --locked

# Specific test
cargo test -p civit-core test_health_endpoint --locked

# With output
cargo test --workspace --locked -- --nocapture
```

Test naming convention: `test_<unit_under_test>_<expected_behavior>`

Example: `test_config_missing_database_url_error`

All new code must include unit tests. Tests must not require external services (use mocks).

## Commit Message Format

Conventional Commits:

```
type(scope): description

[optional body]
```

Types: `feat`, `fix`, `refactor`, `test`, `docs`, `chore`, `perf`, `ci`

Scopes: `core`, `runner`, `brain`, `vfs`, `crypto`, `pipeline`, `shared`, `ui`, `deploy`

Examples:
```
feat(core): add repository creation endpoint
fix(runner): handle sandbox timeout gracefully
test(vfs): add deduplication benchmarks
docs(api): update WebSocket event documentation
```

## Pull Request Process

1. Create a feature branch from `main`
2. Write code with tests and documentation
3. Run `cargo fmt --all`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`
4. Update affected documentation (API reference, architecture, roadmap)
5. Open PR; one approval required; squash merge to `main`

## Architecture Decision Records

ADRs are stored in `docs/architecture-decisions/`. Consult existing ADRs before proposing changes to core architecture.

## Adding a New Crate

1. Add to `Cargo.toml` `[workspace] members` array
2. Create `Cargo.toml` with `version.workspace = true`, `edition.workspace = true`
3. Add `#![forbid(unsafe_code)]` at the top of `src/lib.rs`
4. Declare workspace dependencies in the crate's `Cargo.toml`
5. Update `ROADMAP.md` and `docs/ARCHITECTURE.md`
