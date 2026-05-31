# Contributing to CivitForge

## Development Setup

### Prerequisites

- Rust 1.85+ (edition 2024)
- PostgreSQL 16+
- Redis 7+
- Protocol Buffers compiler (for gRPC)
- `just` (optional task runner)

### Building

```bash
cargo build --workspace
```

### Running Tests

```bash
cargo test --workspace
```

### Running Locally

1. Start dependencies:
   ```bash
   docker compose up postgres redis
   ```

2. Set environment variables:
   ```bash
   export DATABASE_URL=postgres://civitforge:civitforge@localhost:5432/civitforge
   export REDIS_URL=redis://localhost:6379
   export JWT_SECRET=dev-secret-change-in-production
   export FEDERATION_ENABLED=false
   ```

3. Run the API server:
   ```bash
   cargo run --bin civit-core
   ```

## Coding Standards

### Mandatory

- Every `.rs` file **must** start with `#![forbid(unsafe_code)]`
- Rust edition 2024
- No emojis in code or commit messages

### Linting

All code must pass:

```bash
cargo clippy --workspace -- -D warnings
cargo fmt --check --all
```

Pre-commit hooks enforce both checks.

### Code Style

- Follow standard Rust idioms
- Prefer `thiserror` for error types
- Use `Result<T, CoreError>` for fallible operations
- Module-level documentation is encouraged
- No `unsafe` blocks under any circumstances

## Pull Request Process

1. Create a feature branch from `main`:
   ```bash
   git checkout -b feat/my-feature
   ```

2. Make your changes, ensure all tests pass:
   ```bash
   cargo test --workspace
   cargo clippy --workspace -- -D warnings
   cargo fmt --check --all
   ```

3. Open a PR with a description of the change

4. Ensure CI passes (build, test, clippy, fmt)

5. One approval required for merge

6. Squash merge to `main`

## Commit Message Format

We use [Conventional Commits](https://www.conventionalcommits.org/):

```
<type>(<scope>): <description>

[optional body]
```

### Types

- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation only
- `refactor`: Code change that neither fixes a bug nor adds a feature
- `test`: Adding or updating tests
- `chore`: Maintenance tasks
- `ci`: CI/CD configuration changes

### Scopes

- `core`: civit-core crate
- `runner`: civit-runner crate
- `brain`: civit-brain crate
- `vfs`: civit-vfs crate
- `crypto`: civit-crypto crate
- `deploy`: Helm charts and Kubernetes configs

### Examples

```
feat(core): add repository creation endpoint
fix(runner): handle sandbox timeout gracefully
test(vfs): add deduplication benchmarks
docs(api): update WebSocket event documentation
```

## Testing Requirements

- All new code must include unit tests
- Integration tests for API endpoints in `civit-core/tests/`
- Minimum 80% line coverage for new modules
- Load test validation for performance-sensitive paths
- Tests must not require external services (use mocks)

## Architecture Decisions

Significant design decisions are documented as ADRs in `docs/architecture-decisions/`.
Consult existing ADRs before proposing changes to core architecture.
