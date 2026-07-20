# CivitForge

Federated, Rust-native software forge for large-scale monorepos. Provides Git hosting, CI/CD pipeline execution with rootless Podman, OCI container registry, issue tracking with Kanban boards, wiki, code search, and an air-gapped AI subsystem.

[![Rust Version](https://img.shields.io/badge/rust-1.88%2B-blue.svg)](https://www.rust-lang.org)
[![License: AGPL v3](https://img.shields.io/badge/License-AGPL_v3-blue.svg)](https://www.gnu.org/licenses/agpl-3.0)

## Architecture

12-workspace Cargo crate, Rust edition 2024, `#![forbid(unsafe_code)]` enforced, 3,707 tests passing (118 integration tests require PostgreSQL):

| Crate | Role |
|-------|------|
| `civit-core` | Axum HTTP server, authentication, gitoxide Git engine, SSH daemon, federation, events, notifications |
| `civit-db` | Database layer: migrations, models, connection pool, session management |
| `civit-git` | Git operations: archive, blame, diff, commit graph, tree walking |
| `civit-auth` | Authentication: JWT, LDAP, personal access tokens, SSH key validation |
| `civit-ci` | CI/CD: pipeline execution, DAG scheduling, badges, caches, secrets |
| `civit-storage` | Storage: build artifacts, Git LFS, mirrors, OCI registry backend |
| `civit-runner` | CI/CD pipeline execution, Kubernetes operator (kube-rs), rootless Podman sandbox |
| `civit-brain` | AI agent workflows, AST parsing (19 languages, 3-tier), RAG pipeline, vector DB, LLM inference |
| `civit-vfs` | gRPC filesystem server (tonic/prost), remote file operations |
| `civit-crypto` | CEL expression evaluator, HMAC/SHA, OIDC, SAML, WebAuthn, HSM, OSV vuln scanning, SLSA provenance, mTLS |
| `civit-pipeline` | YAML pipeline spec parsing and validation (80+ test vectors) |
| `civit-shared` | Shared API request/response types for backend-frontend type sharing |

Plus `civit-ui` (Leptos CSR WASM frontend with Tailwind CSS v4) and `civit-desktop` (Tauri desktop app, excluded from workspace build).

## Quick Start

### Platform Support

| Platform | Status | Method |
|----------|--------|--------|
| Desktop Web | ✅ | Docker / Native |
| Mobile Web | ✅ | Responsive SPA |
| Android | ✅ | Tauri Mobile |
| iOS | ✅ | Tauri Mobile |
| Linux | ✅ | Docker (amd64/arm64) / Tauri |
| Windows | ✅ | Tauri Desktop / Native build |
| macOS | ✅ | Tauri Desktop |

### Docker Compose

```bash
git clone https://github.com/WyattAu/CivitForge.git
cd CivitForge
docker compose up -d
```

Services exposed on the host:

| Host Port | Container | Service |
|-----------|-----------|---------|
| 9091 | 8080 | REST API + WebSocket |
| 2222 | 2222 | Git SSH |
| 9090 | 9090 | VFS gRPC |

### Health check

```bash
curl http://localhost:9091/healthz
```

### Source build

```bash
cargo build --release --workspace
```

### Mobile Build

```bash
# Android
cd crates/civit-desktop
cargo tauri android init
cargo tauri android build --target aarch64

# iOS (macOS only)
cd crates/civit-desktop
cargo tauri ios init
cargo tauri ios build
```

### Helm (Kubernetes)

```bash
helm install civitforge deploy/helm/civitforge \
  --namespace civitforge --create-namespace \
  --set postgresql.host=your-pg-host \
  --set redis.host=your-redis-host \
  --set jwtSecret=$(openssl rand -base64 32)
```

## Configuration

All configuration is via environment variables.

### Required

| Variable | Description | Example |
|----------|-------------|---------|
| `DATABASE_URL` | PostgreSQL connection string | `postgres://user:pass@host:5432/civit` |
| `JWT_SECRET` | JWT signing key (min 32 chars) | `openssl rand -base64 32` |

### Optional

| Variable | Default | Description |
|----------|---------|-------------|
| `CIVIT_HOST` | `127.0.0.1` | Bind address; use `0.0.0.0` for all interfaces |
| `CIVIT_PORT` | `8080` | Bind port |
| `REDIS_URL` | `redis://127.0.0.1:6379` | Redis for sessions and edge cache |
| `JWT_EXPIRY_HOURS` | `24` | JWT expiration |
| `CIVIT_STORAGE_PATH` | `/var/lib/civit/repos` | Git repository storage path |
| `CIVIT_ENCRYPTION_KEY` | *(none)* | AES-256-GCM key for pipeline variable encryption |
| `FEDERATION_ENABLED` | `false` | Enable ForgeFed ActivityPub federation |
| `FEDERATION_INSTANCE_ID` | `default-instance` | Federation instance ID |
| `FEDERATION_INSTANCE_DOMAIN` | `localhost` | Public domain for federation |
| `LDAP_ENABLED` | `false` | Enable LDAP authentication backend |
| `LDAP_URL` | `ldap://localhost:389` | LDAP server URL |
| `TLS_CERT_PATH` | *(none)* | TLS certificate path (enables HTTPS) |
| `TLS_KEY_PATH` | *(none)* | TLS private key path |
| `RUST_LOG` | `civit_core=info,tower_http=debug` | Log filter |

See [docs/OPERATOR_GUIDE.md](docs/OPERATOR_GUIDE.md) for the full configuration reference including LDAP, security, rate limiting, and CORS settings.

### Default credentials (Docker Compose, development only)

- PostgreSQL: `civit` / `civit-dev-secure-pw-2026` on port 5432
- Redis: password `civit-redis-dev-2026` on port 6379

## API

Base URL: `http://localhost:9091/api/v1`

Authentication: `Authorization: Bearer <jwt-token>` (register-on-login, token via `POST /auth/login`)

Full reference: [docs/API_REFERENCE.md](docs/API_REFERENCE.md)

Key endpoint groups: auth, users, organizations, repositories, branches, tags, pipelines, runners, OCI registry, issues, wiki, code search, SSH keys, Kanban boards, releases, pull requests, teams, tokens, webhooks, federation, WebSocket events.

## Dependency Services

| Service | Port | Purpose |
|---------|------|---------|
| PostgreSQL 17 | 5432 | Relational storage (users, repos, pipelines, issues, wiki, OCI metadata) |
| Redis 7 | 6379 | Session cache, edge cache (zstd), pub/sub event bus |

## Contributing

1. Install Rust 1.88+ with clippy and rustfmt components, plus `protoc` (protobuf compiler)
2. Activate pre-commit hooks: `git config core.hooksPath .githooks`
3. Run `docker compose up -d postgres redis` for local dependencies
4. Pre-commit hooks enforce: emoji scan, `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`
5. All Rust files require `#![forbid(unsafe_code)]` at the crate level (ADR-001)
6. No emoji in source, documentation, or rendered UI text (enforced by pre-commit scanner)
7. Conventional Commits: `type(scope): description`
8. See [CONTRIBUTING.md](CONTRIBUTING.md) for full guidelines

## CI

GitHub Actions runs on push/PR to `main`: fmt, clippy, test, security audit (cargo-audit), WASM build (all with `--locked`). Docker images build and push to `ghcr.io/wyattau/civitforge` on `main` and version tags.

## License

AGPL-3.0-or-later ([LICENSE](LICENSE)).
