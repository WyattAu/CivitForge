# Contributing to CivitForge

Guidelines for developing CivitForge.

## Development Environment

### Prerequisites

- Rust 1.88+ (edition 2024)
- PostgreSQL 15+
- Redis 7+ (optional, for sessions/edge cache)
- Podman or Docker (optional, for CI runner)

### Quick Setup

```bash
git clone https://github.com/WyattAu/CivitForge.git
cd CivitForge

# Install toolchain
rustup component add clippy rustfmt

# Build
cargo build --workspace

# Test
cargo test --workspace --locked
```

### Local Stack

```bash
# Start PostgreSQL and Redis
docker compose up -d postgres redis

# Set required environment variables
export DATABASE_URL="postgres://civit:civit@localhost:5432/civit"
export JWT_SECRET="dev-secret-key-32bytes-minimum"

# Run the server
cargo run -p civit-core
```

## Pre-Commit Hooks

CivitForge enforces three checks on every commit:

```bash
# Format
cargo fmt --all -- --check

# Lint
cargo clippy --workspace --all-targets -- -D warnings

# Test
cargo test --workspace --locked
```

These are run automatically by CI. To set up as git hooks:

```bash
cat > .git/hooks/pre-commit << 'EOF'
#!/bin/sh
cargo fmt --all -- --check && \
cargo clippy --workspace --all-targets -- -D warnings && \
cargo test --workspace --locked
EOF
chmod +x .git/hooks/pre-commit
```

## Coding Standards

### Safety

- **All Rust files MUST start with `#![forbid(unsafe_code)]`** — ADR-001
- The only exception is `treesitter_backend.rs` which uses C FFI behind the `treesitter` feature flag
- No new unsafe code without an ADR approval process

### Code Quality

- Zero clippy warnings with `-D warnings` (warnings treated as errors)
- `cargo fmt` must pass with `--check` (no formatting drift)
- All new code must have test coverage
- Critical paths require >95% branch coverage

### Architecture

- Each crate has a single responsibility
- Public interfaces go in `pub mod` at the crate root
- Internal modules are `pub(crate)` or private
- Configuration via environment variables (not config files)
- Errors use `thiserror` for library errors, `anyhow` for binary errors

### Dependencies

- No new dependencies without justification in an ADR
- Prefer stdlib or existing workspace deps
- Prefer pure-Rust crates over C FFI
- Prefer API-based integrations over bundled ML models

## Workspace Structure

```
civit-core/     # HTTP API server, auth, DB, events, federation
civit-brain/    # AI/ML: AST parsing, embeddings, RAG, LLM inference
civit-crypto/   # Crypto: CEL, HMAC, OIDC, SAML, WebAuthn, HSM
civit-runner/   # CI: pipeline execution, K8s operator, Podman
civit-vfs/      # gRPC filesystem
```

### Adding a New Module

1. Create the module file in the appropriate crate
2. Add `pub mod module_name;` in `src/lib.rs` (or `src/main.rs` for binary-only)
3. Write tests in a `#[cfg(test)] mod tests` block at the bottom of the file
4. Run `cargo fmt`, `cargo clippy`, `cargo test`
5. Update this contributing guide and API reference if public API changed

### Adding a New Crate

1. Add to `Cargo.toml` `[workspace] members` array
2. Create `Cargo.toml` with `version.workspace = true`, `edition.workspace = true`
3. Add `#![forbid(unsafe_code)]` at the top of `src/lib.rs`
4. Declare workspace dependencies in the crate's `Cargo.toml`
5. Update `ROADMAP.md` and `docs/ARCHITECTURE.md`

## Testing

### Running Tests

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

### Test Organization

- Unit tests: `#[cfg(test)] mod tests` in the source file
- Integration tests: `tests/` directory at crate root (not currently used)
- Benchmark tests: `cargo run --bin civit-bench` (requires running server)

### Test Naming

- Test functions: `test_<unit_under_test>_<expected_behavior>`
- Example: `test_config_missing_database_url_error`
- Example: `test_dispatch_email_log_only_when_no_smtp_config`

## Commit Messages

Follow Conventional Commits:

```
type(scope): description

feat(core): add OIDC JWKS fetch endpoint
fix(runner): Podman CLI timeout handling
refactor(crypto): CEL evaluator arithmetic support
test(brain): vector db trait integration tests
docs: update operator guide for SMTP configuration
chore: bump ring to 0.17.1
```

Types: `feat`, `fix`, `refactor`, `test`, `docs`, `chore`, `perf`, `ci`

## Pull Request Process

1. Create a feature branch from `main`
2. Write code with tests and documentation
3. Run `cargo fmt --all`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace --locked`
4. Update affected documentation (API reference, architecture, roadmap)
5. Submit PR with description of changes

## Architecture Decision Records

ADRs are stored in `.adrs/`. Each ADR documents:

- **Context:** The technical decision being made
- **Decision:** What was chosen and why
- **Consequences:** Impact on the codebase

Current ADRs:
- **ADR-001:** Scoped unsafe features — unsafe code only allowed behind feature gates

## Feature Flags

| Flag | Description | Unsafe? |
|------|-------------|---------|
| `syn-parser` | Rust AST parsing via syn 2 | No |
| `swc-parser` | JS/TS parsing via swc 12 | No |
| `sql-parser` | SQL parsing via sqlparser | No |
| `treesitter` | Tree-sitter for 12+ languages | **Yes** (C FFI) |
| `ssh-server` | russh SSH daemon | No |

Build with features:
```bash
cargo build --workspace --features "treesitter,ssh-server"
```

## Getting Help

- Open an issue on [GitHub](https://github.com/WyattAu/CivitForge/issues)
- Check [ROADMAP.md](../ROADMAP.md) for planned work
- Check [docs/ARCHITECTURE.md](ARCHITECTURE.md) for design context
- Check [docs/OPERATOR_GUIDE.md](OPERATOR_GUIDE.md) for deployment help
