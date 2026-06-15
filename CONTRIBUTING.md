# Contributing to CivitForge

## Prerequisites

- Rust 1.88+ (edition 2024) with `clippy` and `rustfmt` components
- Protobuf compiler (`protoc`) for gRPC code generation
- PostgreSQL 17+ (integration tests only)
- Redis 7+ (optional, for sessions and edge cache)
- Podman or Docker (optional, for CI runner)
- Node.js 20+ (optional, for Playwright GUI/E2E test harnesses)

## Development Setup

```bash
git clone https://github.com/WyattAu/CivitForge.git
cd CivitForge

# Install toolchain components
rustup component add clippy rustfmt

# Activate pre-commit hooks (one-time per clone)
git config core.hooksPath .githooks

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

## Pre-Commit Hook

The canonical pre-commit hook lives at `.githooks/pre-commit`. It enforces:

1. **Emoji prohibition** -- scans staged files for Unicode pictographs and escape
   sequences that produce emoji. The project standard mandates zero emoji in
   source, documentation, and rendered UI text.
2. **Format check** -- `cargo fmt --check --all`
3. **Clippy** -- `cargo clippy --workspace --all-targets -- -D warnings`
4. **Tests** -- `cargo test --workspace`
5. **Conventional Commit hint** -- non-blocking advisory for commit subjects

Activation (one command per clone):

```bash
git config core.hooksPath .githooks
```

Bypass options (use sparingly):

```bash
SKIP_PRE_COMMIT=1 git commit ...           # skip all checks
SKIP_PRE_COMMIT_TESTS=1 git commit ...     # skip slow test suite only
```

CI (GitHub Actions) runs the same fmt, clippy, test, and audit checks plus a
release build on every push to `main`.

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
