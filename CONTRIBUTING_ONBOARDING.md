# CivitForge Contributor Onboarding

Welcome to CivitForge. This guide covers everything you need to start contributing.

## Development Environment Setup

### Prerequisites

| Tool | Version | Purpose |
|------|---------|---------|
| Rust | 1.96+ (edition 2024) | Core language |
| protoc | 3.x | gRPC code generation |
| PostgreSQL | 17+ | Database (integration tests) |
| Redis | 7+ | Sessions and edge cache |
| Podman or Docker | Latest | CI runner containers |
| Node.js | 20+ | Playwright GUI/E2E tests |

### Quick Start

```bash
git clone https://github.com/WyattAu/CivitForge.git
cd CivitForge

# Install toolchain components
rustup component add clippy rustfmt

# Activate pre-commit hooks
git config core.hooksPath .githooks

# Build the workspace
cargo build --workspace

# Start dependencies
docker compose up -d postgres redis

# Set environment variables
export DATABASE_URL="postgres://civit:civit-dev-secure-pw-2026@localhost:5432/civit"
export REDIS_URL="redis://:civit-redis-dev-2026@localhost:6379"
export JWT_SECRET="dev-secret-key-32bytes-minimum"

# Run the server (binds to 127.0.0.1:9091)
make run
```

### Environment Variables

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| `DATABASE_URL` | Yes | -- | PostgreSQL connection string |
| `REDIS_URL` | No | -- | Redis connection string |
| `JWT_SECRET` | Yes | -- | Secret for JWT signing (32+ bytes) |
| `RUST_LOG` | No | `info` | Log level filter |
| `CIVIT_LISTEN_ADDR` | No | `127.0.0.1:9091` | Server bind address |

---

## Codebase Overview

CivitForge is a 20-crate Rust workspace (19 active + 1 excluded desktop app).

### Core Infrastructure

| Crate | Purpose | Lines |
|-------|---------|-------|
| **civit-shared** | Shared API types between backend and frontend | ~2K |
| **civit-types** | Core domain types (User, Repo, Issue, etc.) | ~3K |
| **civit-db** | PostgreSQL layer: migrations, models, pool | ~8K |
| **civit-core** | HTTP API server (45 route modules), events | ~15K |

### Feature Crates

| Crate | Purpose | Lines |
|-------|---------|-------|
| **civit-git** | Git operations via gitoxide (clone, diff, blame) | ~4K |
| **civit-auth** | JWT, LDAP, PAT, SSH key authentication | ~5K |
| **civit-ci** | CI/CD orchestration: badges, caches, DAG, schedules | ~6K |
| **civit-storage** | Artifacts, LFS, mirrors, OCI registry | ~4K |
| **civit-runner** | CI execution: Podman, K8s operator | ~5K |
| **civit-brain** | AI/ML: RAG, AST parsing (19 languages), vector DB | ~4K |
| **civit-crypto** | Crypto primitives, CEL, OIDC, SAML, mTLS, HSM | ~6K |

### Supporting Crates

| Crate | Purpose | Lines |
|-------|---------|-------|
| **civit-pipeline** | Pipeline YAML parsing and validation | ~3K |
| **civit-federation** | ForgeFed: ActivityPub, WebFinger, HTTP signing | ~4K |
| **civit-vfs** | gRPC virtual filesystem | ~2K |
| **civit-telemetry** | OTLP exporter | ~1K |
| **civit-security** | Security scanning and audit | ~2K |
| **civit-workflow** | Workflow state machine | ~2K |

### Frontend

| Crate | Purpose | Lines |
|-------|---------|-------|
| **civit-ui** | Leptos WASM + SSR frontend (Tailwind CSS v4) | ~8K |
| **civit-desktop** | Tauri desktop app (excluded, requires webkit2gtk) | ~3K |

### Crate Dependency Flow

```
civit-shared (types)
    |
civit-db (database)
    |
civit-auth ── civit-git ── civit-ci ── civit-storage
    |              |            |
civit-core (HTTP server, depends on all above)
    |
civit-runner / civit-brain / civit-vfs (executables)
```

---

## First-Time Contributor Guide

### Finding Something to Work On

1. Check [GitHub Issues](https://github.com/WyattAu/CivitForge/issues) for `good-first-issue` labels
2. Review the [ROADMAP.md](ROADMAP.md) for planned features
3. Look at `docs/GAP_ANALYSIS.md` for missing functionality

### Making Your First Change

1. Fork and clone the repository
2. Create a feature branch: `git checkout -b feat/my-feature`
3. Make changes with tests
4. Run the pre-commit checks:
   ```bash
   cargo fmt --all
   cargo clippy --workspace --all-targets -- -D warnings
   cargo test --workspace --locked
   ```
5. Commit with a conventional commit message
6. Open a PR against `main`

### Code Organization Rules

- `#![forbid(unsafe_code)]` in every crate (non-negotiable)
- New dependencies require ADR approval
- Configuration via environment variables, not config files
- Prefer `thiserror` for library errors, `anyhow` for binaries
- Public interfaces in `pub mod` at crate root; internals are `pub(crate)`

---

## PR Process

### Before Submitting

- [ ] Code compiles: `cargo build --workspace`
- [ ] No clippy warnings: `cargo clippy --workspace --all-targets -- -D warnings`
- [ ] Formatting correct: `cargo fmt --check --all`
- [ ] Tests pass: `cargo test --workspace --locked`
- [ ] Documentation updated (if applicable)

### PR Template

```markdown
## Summary

Brief description of changes.

## Changes

- Change 1
- Change 2

## Testing

- [ ] Unit tests added/updated
- [ ] Integration tests pass
- [ ] Manual testing performed (if UI changes)

## Documentation

- [ ] README updated
- [ ] API docs updated
- [ ] CHANGELOG entry added (if user-facing)
```

### Review Requirements

- One approval required from a maintainer
- All CI checks must pass
- Squash merge to `main`

---

## Code Review Checklist

### Correctness

- [ ] Logic is correct and handles edge cases
- [ ] Error handling is appropriate (no silent failures)
- [ ] No unwrap() in production paths
- [ ] Concurrency is handled correctly (no data races)

### Safety

- [ ] `#![forbid(unsafe_code)]` present in new crates
- [ ] No secrets or keys committed
- [ ] Input validation on all external data
- [ ] SQL injection prevented (parameterized queries)

### Quality

- [ ] Code follows existing patterns in the codebase
- [ ] Functions are reasonably sized (< 50 lines ideal)
- [ ] Names are clear and descriptive
- [ ] No unnecessary allocations or clones

### Testing

- [ ] Unit tests cover main logic
- [ ] Edge cases tested
- [ ] No tests requiring external services (use mocks)
- [ ] Test names follow `test_<unit>_<behavior>` convention

### Documentation

- [ ] Public APIs have doc comments
- [ ] Complex algorithms have comments
- [ ] README/CHANGELOG updated if user-facing

---

## Common Patterns

### Error Handling

```rust
// Library crates: use thiserror
#[derive(Debug, thiserror::Error)]
pub enum MyError {
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("not found: {0}")]
    NotFound(String),
}

// Binary crates: use anyhow
fn main() -> anyhow::Result<()> {
    let config = load_config()?;
    Ok(())
}
```

### Configuration

```rust
// Always use environment variables
pub struct Config {
    pub database_url: String,
    pub redis_url: Option<String>,
    pub listen_addr: SocketAddr,
}

impl Config {
    pub fn from_env() -> Result<Self, ConfigError> {
        Ok(Self {
            database_url: std::env::var("DATABASE_URL")
                .map_err(|_| ConfigError::Missing("DATABASE_URL"))?,
            redis_url: std::env::var("REDIS_URL").ok(),
            listen_addr: std::env::var("CIVIT_LISTEN_ADDR")
                .unwrap_or_else(|_| "127.0.0.1:9091".into())
                .parse()?,
        })
    }
}
```

### Testing

```rust
// Unit tests in the same file
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_from_env() {
        std::env::set_var("DATABASE_URL", "postgres://localhost/test");
        let config = Config::from_env().unwrap();
        assert_eq!(config.database_url, "postgres://localhost/test");
    }
}
```

---

## Common Anti-Patterns

| Anti-Pattern | Correct Approach |
|-------------|-----------------|
| `unwrap()` in production code | Use `?` or `unwrap_or_else` with logging |
| Hardcoded config values | Use environment variables |
| Tests depending on external services | Use mocks or test containers |
| Adding dependencies without justification | Check existing workspace deps first |
| Large functions (> 100 lines) | Extract into smaller functions |
| `#[allow(unsafe_code)]` | Never -- use `#![forbid(unsafe_code)]` |
| Silent error swallowing | Log errors with `tracing::error!` |
| Stringly-typed APIs | Use typed structs with serde |

---

## Where to Find Help

| Resource | Location |
|----------|----------|
| Architecture docs | `docs/ARCHITECTURE.md` |
| API reference | `docs/API_REFERENCE.md` |
| Architecture decisions | `docs/architecture-decisions/` |
| Roadmap | `ROADMAP.md` |
| Change log | `CHANGELOG.md` |
| Gap analysis | `docs/GAP_ANALYSIS.md` |
| Feature comparison | `docs/FEATURE_COMPARISON_MATRIX.md` |
| Pre-commit hooks | `.githooks/pre-commit` |
| Docker setup | `docker-compose.yml` |
| Helm chart | `deploy/helm/civitforge/` |

### Running Specific Tests

```bash
# All tests
cargo test --workspace --locked

# Single crate
cargo test -p civit-core --locked

# Single test
cargo test -p civit-core test_health_endpoint --locked

# With output
cargo test --workspace --locked -- --nocapture
```

### Building Specific Targets

```bash
# Full workspace
cargo build --workspace

# Single crate
cargo build -p civit-core

# Release build
cargo build --release --workspace

# WASM frontend
cd crates/civit-ui && trunk build --release
```
