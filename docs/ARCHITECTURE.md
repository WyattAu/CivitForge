# CivitForge Architecture

Crate-level architecture, data flow, and design decisions for the CivitForge workspace.

## Workspace Structure

12-crate workspace (plus `civit-desktop`, excluded from workspace build), Rust edition 2024, `#![forbid(unsafe_code)]` enforced.

```
CivitForge/
├── Cargo.toml              # Workspace root
├── rust-toolchain.toml     # Rust 1.88, clippy, rustfmt
├── docker-compose.yml      # Full-stack local deployment
├── Dockerfile              # Convenience build
├── container/
│   ├── civitforge/         # Main server image (tier: critical)
│   └── runner/             # CI pipeline daemon (tier: standard)
├── crates/
│   ├── civit-shared/       # Shared types (backend + frontend)
│   ├── civit-pipeline/     # Pipeline YAML parsing and validation
│   ├── civit-db/           # Database layer, migrations, models
│   ├── civit-git/          # Git operations (gitoxide)
│   ├── civit-auth/         # Authentication: JWT, LDAP, PAT, SSH keys
│   ├── civit-ci/           # CI/CD: badges, caches, DAG, schedules
│   ├── civit-storage/      # Storage: artifacts, LFS, mirrors, OCI
│   ├── civit-core/         # HTTP API server, events, federation
│   ├── civit-runner/       # CI execution, K8s operator, Podman
│   ├── civit-brain/        # AI/ML, RAG, AST parsing
│   ├── civit-crypto/       # Crypto primitives, CEL, enterprise auth
│   ├── civit-vfs/          # gRPC filesystem
│   ├── civit-ui/           # Leptos web frontend (WASM + SSR)
│   └── civit-desktop/      # Tauri desktop app (excluded, requires webkit2gtk)
├── deploy/
│   └── helm/civitforge/    # Kubernetes Helm chart
├── docs/                   # Operator guide, API reference, ADRs
└── .specs/                 # Architecture specs, traceability
```

## Crate Dependency Graph

```
                         ┌──────────────┐
                         │   civit-ui   │  Leptos WASM + SSR frontend
                         └──────┬───────┘
                                │ uses types from
                    ┌───────────▼───────────┐
                    │     civit-shared       │  Shared API request/response types
                    └───────┬───────┬───────┘
                            │       │
               ┌────────────▼─┐   ┌─▼────────────┐
               │  civit-auth   │   │  civit-storage │
               │ JWT, LDAP,   │   │ Artifacts, LFS,│
               │ PAT, SSH keys│   │ Mirrors, OCI   │
               └──────┬───────┘   └────────────────┘
                      │ depends on
               ┌──────▼───────┐
               │    civit-db   │  PostgreSQL: migrations, models, pool
               └──────┬───────┘
                      │
         ┌────────────▼────────────┐
         │       civit-core        │  HTTP API (45 route modules), events,
         │  (main binary)          │  federation, search, notifications
         └──┬─────────┬──────────┬─┘
            │         │          │
  ┌─────────▼──┐  ┌───▼────┐  ┌─▼──────────┐
  │  civit-ci   │  │civit-  │  │  civit-brain │
  │ Badges,     │  │git     │  │ AI/ML, RAG,  │
  │ Caches, DAG │  │Archive,│  │ AST (19 lang)│
  │ Schedules   │  │Blame,  │  │ Vector DB    │
  └──────┬──────┘  │Diff,   │  └──────────────┘
         │         │Graph   │
         ▼         └────────┘
  ┌──────────────┐
  │ civit-pipeline│  YAML spec parsing, validation
  └──────────────┘

  ┌──────────────┐    ┌──────────────┐
  │ civit-crypto  │    │  civit-runner │
  │ CEL, HMAC,   │    │ CI execution, │
  │ OIDC, SAML,  │    │ K8s operator, │
  │ WebAuthn,    │    │ Podman sandbox│
  │ mTLS, HSM    │    └──────────────┘
  └──────────────┘

  ┌──────────────┐
  │  civit-vfs    │
  │ gRPC filesystem│
  └──────────────┘
```

## Crate Responsibilities

### civit-shared (library)

Shared API request/response types for backend-frontend type sharing via `civit-ui`. Standalone, no internal dependencies.

### civit-pipeline (library)

CI/CD pipeline YAML spec parsing and validation.

| Module | Responsibility |
|--------|---------------|
| YAML parser | Full pipeline spec: triggers, services, cache, secrets, workspace, concurrency |
| Expression evaluator | CEL-based conditions for `if:`, trigger filters |
| Validation | 80+ test vectors |

### civit-db (library)

Database abstraction layer. Standalone, no internal dependencies.

| Module | Responsibility |
|--------|---------------|
| `migrations/` | 32 numbered SQL migrations (001-049) with rollback scripts |
| `models/` | Data structs: User, Org, Repository, Pipeline, Issue, PullRequest, Wiki, etc. |
| `pool/` | sqlx connection pool wrapper |
| `repository/` | Database access methods, org usage tracking |
| `session/` | Session management and storage |
| `error.rs` | `DbError` type with thiserror |

### civit-git (library)

Git operations via gitoxide. Depends on `civit-db`.

| Module | Responsibility |
|--------|---------------|
| `operations/` | `GitService`: clone, commit, merge, ref management |
| `diff/` | Diff generation between commits/refs |
| `blame/` | Line-by-line blame information |
| `graph/` | Commit graph traversal and branch info |
| `archive/` | Archive generation (tar.gz, zip) |
| `tree/` | Tree walking, blob reading, language statistics |

### civit-auth (library)

Authentication and authorization. Depends on `civit-db`, `civit-shared`.

| Module | Responsibility |
|--------|---------------|
| `jwt/` | JWT encode/decode with RS256/HS256 |
| `ldap/` | LDAP connection pool, bind, user/group search |
| `password/` | bcrypt hashing, password policy enforcement |
| `pat/` | Personal access token management |
| `ssh/` | SSH key storage and validation |
| `middleware/` | Auth middleware for Axum routes |
| `error.rs` | `AuthError` type |

### civit-ci (library)

CI/CD orchestration. Depends on `civit-pipeline`.

| Module | Responsibility |
|--------|---------------|
| `pipeline/` | Pipeline execution engine, step sequencing |
| `graph/` | DAG-based dependency resolution |
| `caches/` | Cache management and restoration |
| `schedules/` | Cron-based pipeline scheduling |
| `secrets/` | Secret injection into pipeline steps |
| `badges/` | Status badge SVG generation |
| `runner_protocol/` | Runner communication protocol types |

### civit-storage (library)

File and artifact storage. Depends on `civit-shared`.

| Module | Responsibility |
|--------|---------------|
| `artifacts/` | Build artifact storage and retrieval |
| `lfs/` | Git LFS (Large File Storage) pointer and blob management |
| `mirrors/` | Repository mirror storage |
| `oci/` | OCI container registry storage backend |

### civit-core (binary)

Central HTTP API server. Depends on all library crates.

| Module | Responsibility |
|--------|---------------|
| `api/` | Axum router: 45 route modules (repos, users, orgs, auth, pipelines, runners, OCI, issues, wiki, search, boards, releases, PRs, teams, tokens, webhooks, federation, etc.) |
| `config.rs` | Environment-based configuration with validation |
| `events/` | DashMap pub/sub, WebSocket broadcast, bounded log replay |
| `federation/` | ForgeFed: ActivityPub, WebFinger, HTTP signing, inbox/outbox, multi-master sync, vector clocks |
| `ssh/` | russh: Ed25519 host key, pubkey auth, rate limiting, git command routing |
| `webhooks/` | HMAC-SHA256 payload signing, HTTP dispatch |
| `notifications/` | SMTP (lettre), Slack, webhook, log-only modes |
| `search/` | Full-text code search across repositories |
| `wiki/` | Wiki page CRUD, history, diff |
| `health/` | Health check endpoints |
| `telemetry/` | OTLP exporter (reqwest POST) |

### civit-runner (binary)

CI/CD execution engine.

| Module | Responsibility |
|--------|---------------|
| `pipeline.rs` | Pipeline step execution in real Podman containers |
| `podman.rs` | Podman service: auto-detect Unix socket vs HTTP, CLI transport |
| `kube_controller.rs` | K8s operator: `kube::runtime::Reconciler`, leader election via Lease CRD |
| `sync.rs` | Multi-master DAG sync: `IncrementalSyncEngine`, checkpointing, conflict resolution |
| `redis_session.rs` | Redis-backed token rotation |

### civit-brain (binary, lib)

AI/ML services. Air-gap compatible -- all inference via local or self-hosted endpoints.

| Module | Responsibility |
|--------|---------------|
| `ast/` | 3-tier parser: `syn`/`swc`/`sqlparser` > tree-sitter (feature-gated) > regex. 19 languages. |
| `embedding.rs` | HTTP client to `/v1/embeddings` (vLLM/Ollama/OpenAI). API + Deterministic backends. |
| `vectordb.rs` | `VectorDb` async trait (RPITIT). In-memory + Qdrant backends. |
| `rag.rs` | `RAGPipeline<T: VectorDb>` generic. `LlmCodeReviewer<T, P>`. |
| `inference.rs` | `InferenceService` HTTP client to `/v1/chat/completions`. SSE streaming. |
| `review/` | `DiffAnalyzerReviewAgent` -- static analysis to `ReviewAgent` trait. |

### civit-crypto (library)

Cryptographic primitives and enterprise auth protocols.

| Module | Responsibility |
|--------|---------------|
| `cel/` | CEL evaluator: arithmetic, 15 functions, parenthesized sub-expressions |
| `hmac.rs` | SHA-256/512, HMAC-SHA256 via sha2/hmac crates |
| `oidc.rs` | JWKS fetch, RS256 signature verification via ring |
| `saml.rs` | SHA-256 digest integrity verification |
| `webauthn.rs` | CBOR parsing, structure validation for registration/authentication |
| `hsm/` | Software key fallback: ECDSA/HMAC/AES-GCM via ring |
| `vuln.rs` | OSV API client, CVSS classification |
| `provenance.rs` | SLSA provenance signer + PEM codec |
| `mtls.rs` | rcgen X.509 CA creation, cert issuance, SHA-256 fingerprints |
| `policy.rs` | CAS-style policy engine (Subject/Action/Resource/Condition/Effect) |

### civit-vfs (binary, lib)

Remote filesystem via gRPC.

| Module | Responsibility |
|--------|---------------|
| `vfs.proto` | 8 RPCs (ReadDir, ReadFile, WriteFile, Delete, Stat, etc.) |
| `grpc_server.rs` | tonic gRPC server with mTLS support |
| `vfs/` | In-memory HashMap VFS with libc error codes |

### civit-ui (library)

Leptos CSR WASM frontend with Tailwind CSS v4. Includes Kanban boards for issue tracking.

## Data Flow

### Git push to code review

```
Developer -> SSH/HTTP -> civit-core API -> civit-git (gitoxide store) -> EventBus
                                                                      |
                                              DiffAnalyzer -> ReviewAgent<T: VectorDb>
                                                                      |
                                              RAGPipeline.retrieve() -> LlmCodeReviewer
                                                                      |
                                              LlmProvider.infer() -> LlmReviewResult -> Notification
```

### CI pipeline

```
Push Event -> EventBus -> civit-ci PipelineEngine.execute_step()
                          |
             civit-pipeline YAML parse -> step validation
                          |
             civit-runner -> Podman CLI -> Container Build/Test -> stdout/stderr capture
                          |
             PipelineResult -> civit-core NotificationService.dispatch()
```

### Authentication flow

```
Request -> civit-core middleware -> civit-auth (JWT/LDAP/PAT validation)
                                        |
                               civit-db session lookup
                                        |
                               RBAC role check -> Route handler
```

## Feature Flags

| Flag | Purpose | Safety Impact |
|------|---------|--------------|
| `syn-parser` | Rust AST parsing via `syn` 2 | Zero unsafe |
| `swc-parser` | JavaScript/TypeScript via `swc` 12 | Zero unsafe |
| `sql-parser` | SQL via `sqlparser` 0.62 | Zero unsafe |
| `treesitter` | Tree-sitter 0.24 (12+ languages) | C FFI (feature-gated via ADR-001) |
| `ssh-server` | russh SSH daemon | Zero unsafe |

## Design Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| API-based embeddings | `/v1/embeddings` HTTP call | Avoids bundled ML model binary (50-200 MB) |
| 3-tier AST routing | native > tree-sitter > regex | Pure-Rust preferred; tree-sitter as fallback |
| RPITIT VectorDb trait | `impl Future<Output=...> + Send` | Native Rust 1.88 RPITIT, zero overhead |
| Podman CLI transport | `tokio::process::Command` | Zero new dependencies |
| Direct OTLP export | Raw reqwest POST | Avoids transitive unsafe from opentelemetry SDK |
| Crate decomposition | 12 focused crates | Single-responsibility, faster incremental builds |

## Technology Stack

| Layer | Technology | Version |
|-------|-----------|---------|
| HTTP framework | Axum | 0.8 |
| Git operations | gitoxide (gix) | 0.70 |
| Database | sqlx (PostgreSQL) | 0.8 |
| Crypto | ring, sha2, hmac | 0.17, 0.10, 0.12 |
| TLS/mTLS | rcgen, x509-parser, rustls | 0.13, 0.17 |
| Auth | jsonwebtoken, ldap3, bcrypt | 9, 0.11, 0.17 |
| Kubernetes | kube-rs, k8s-openapi | 0.98, v1_30 |
| gRPC | tonic, prost | 0.12, 0.13 |
| Edge cache | Redis + zstd | 7 |
| Serialization | serde, serde_json, serde_yaml | 1, 1, 0.9 |
| Email | lettre | 0.11 |
| Web UI | Leptos (WASM + SSR) + Tailwind CSS | 0.7, v4 |
