# CivitForge Architecture

Crate-level architecture, data flow, and design decisions for the CivitForge workspace.

## Crate Dependency Graph

```
                     ┌─────────────┐
                     │  civit-core  │  HTTP API, auth, DB, sessions, events
                     │   (binary)   │
                     └──────┬──────┘
                            │ uses
               ┌────────────┼────────────┐
               │            │            │
     ┌─────────▼──────┐   │   ┌────────▼───────┐
     │   civit-brain    │   │   │   civit-vfs    │
     │  (binary, lib)   │   │   │  (binary, lib) │
     │  AI / RAG / LLM  │   │   │  gRPC / VFS    │
     └────────┬─────────┘   │   └────────┬────────┘
              │             │            │
              └──────┬──────┘            │
                     │                    │
     ┌───────────────▼────────────────────▼──────┐
     │            civit-crypto (library)            │
     │  CEL evaluator, HMAC, OIDC, SAML, WebAuthn  │
     └───────────────────────────────────────────┘
                     │
     ┌───────────────▼───────────────────────────┐
     │            civit-runner (binary)              │
     │  CI pipeline execution, K8s operator, Podman │
     └───────────────────────────────────────────┘
                     ▲
                     │
     ┌───────────────┘
     │   civit-pipeline (library)
     │   YAML spec parsing, validation
     └───────────────┘
                     ▲
                     │
     ┌───────────────┘
     │   civit-shared (library)
     │   Shared API types (backend + frontend)
     └───────────────┘
```

## Crate Responsibilities

### civit-core (binary)

Central server binary. HTTP API, database, authentication, events, Git engine, SSH daemon.

| Module | Responsibility |
|--------|---------------|
| `api/` | Axum router: ~60 routes (repos, users, orgs, auth, SSH keys, WebSocket, Git smart HTTP, pipelines, runners, OCI registry, issues, wiki, search) |
| `db/` | sqlx PostgreSQL: connection pool, migrations, sessions |
| `auth/` | JWT encode/decode, RBAC role hierarchy, TOTP 2FA, session management |
| `git/` | gitoxide (gix): bare repo init, commit walking, ref reading, smart HTTP |
| `ssh/` | russh: Ed25519 host key, pubkey auth, rate limiting, git command routing |
| `events/` | DashMap pub/sub, WebSocket broadcast, bounded log replay |
| `federation/` | ForgeFed: WebFinger lookup, HTTP signing, delivery, backoff |
| `webhooks/` | HMAC-SHA256 payload signing, HTTP dispatch |
| `notifications/` | SMTP (lettre), Slack, webhook, log-only modes |
| `scanning/` | Secret scanner (15+ regex patterns), license scanner (50+ SPDX) |
| `telemetry/` | OTLP exporter (reqwest POST) |
| `config.rs` | Environment-based configuration with validation |

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

### civit-vfs (binary, lib)

Remote filesystem via gRPC.

| Module | Responsibility |
|--------|---------------|
| `vfs.proto` | 8 RPCs (ReadDir, ReadFile, WriteFile, Delete, Stat, etc.) |
| `grpc_server.rs` | tonic gRPC server with mTLS support |
| `vfs/` | In-memory HashMap VFS with libc error codes (FUSE mount deferred to v1.2) |

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

### civit-pipeline (library)

CI/CD pipeline YAML spec parsing and validation.

| Module | Responsibility |
|--------|---------------|
| YAML parser | Full pipeline spec: triggers, services, cache, secrets, workspace, concurrency |
| Expression evaluator | CEL-based conditions for `if:`, trigger filters |
| Validation | 80+ test vectors |

### civit-shared (library)

Shared API request/response types for backend-frontend type sharing via `civit-ui`.

### civit-runner (binary)

CI/CD execution engine.

| Module | Responsibility |
|--------|---------------|
| `pipeline.rs` | Pipeline step execution in real Podman containers |
| `podman.rs` | Podman service: auto-detect Unix socket vs HTTP, CLI transport |
| `kube_controller.rs` | K8s operator: `kube::runtime::Reconciler`, leader election via Lease CRD |
| `sync.rs` | Multi-master DAG sync: `IncrementalSyncEngine`, checkpointing, conflict resolution |
| `redis_session.rs` | Redis-backed token rotation |

## Data Flow

### Git push to code review

```
Developer -> SSH/HTTP -> civit-core API -> gitoxide (store) -> EventBus
                                                               |
                                           DiffAnalyzer -> ReviewAgent<T: VectorDb>
                                                               |
                                           RAGPipeline.retrieve() -> LlmCodeReviewer
                                                               |
                                           LlmProvider.infer() -> LlmReviewResult -> Notification
```

### CI pipeline

```
Push Event -> EventBus -> PipelineEngine.execute_step()
                           |
              Podman CLI -> Container Build/Test -> stdout/stderr capture
                           |
              PipelineResult -> NotificationService.dispatch()
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

## Technology Stack

| Layer | Technology | Version |
|-------|-----------|---------|
| HTTP framework | Axum | 0.8 |
| Git operations | gitoxide (gix) | 0.70 |
| Database | sqlx (PostgreSQL) | 0.8 |
| Crypto | ring, sha2, hmac | 0.17, 0.10, 0.12 |
| TLS/mTLS | rcgen, x509-parser, rustls | 0.13, 0.17 |
| Auth | jsonwebtoken | 9 |
| Kubernetes | kube-rs, k8s-openapi | 0.98, v1_30 |
| gRPC | tonic, prost | 0.12, 0.13 |
| Edge cache | Redis + zstd | 7 |
| Serialization | serde, serde_json, serde_yaml | 1, 1, 0.9 |
| Email | lettre | 0.11 |
| Web UI | Leptos (WASM + SSR) + Tailwind CSS | 0.7, v4 |

## Workspace Structure

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
│   ├── civit-core/         # API server, auth, DB, events
│   ├── civit-runner/       # CI execution, K8s operator, Podman
│   ├── civit-brain/        # AI/ML, RAG, AST parsing
│   ├── civit-crypto/       # Crypto primitives, CEL, enterprise auth
│   ├── civit-vfs/          # gRPC filesystem
│   └── civit-ui/           # Leptos web frontend (WASM + SSR)
├── deploy/
│   └── helm/civitforge/    # Kubernetes Helm chart
├── docs/                   # Operator guide, API reference, ADRs
└── .specs/                 # Architecture specs, traceability
```
