# CivitForge Architecture Overview

CivitForge is a federated, Rust-native software forge designed for extreme-scale monorepos, rootless CI/CD, and air-gapped AI. This document describes the crate architecture, data flow, and design decisions.

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
```

## Crate Responsibilities

### civit-core (~1,590 LOC API + services)

Central server binary. Owns the HTTP API, database layer, authentication, and event system.

| Module | Responsibility |
|--------|---------------|
| `api/` | Axum router: 20 routes (repos, users, orgs, auth, SSH keys, WebSocket, git smart HTTP) |
| `db/` | sqlx PostgreSQL: 7 tables, 34 methods, connection pool, migrations, sessions |
| `auth/` | JWT encode/decode, RBAC role hierarchy, TOTP 2FA, session management |
| `git/` | gitoxide (gix): bare repo init, commit walking, ref reading, smart HTTP |
| `ssh/` | russh: Ed25519 host key, pubkey auth, rate limiting, git command routing |
| `events/` | DashMap pub/sub, WebSocket broadcast, bounded log replay |
| `federation/` | ForgeFed: WebFinger lookup, Ed25511 HTTP signing, delivery, backoff |
| `webhooks/` | HMAC-SHA256 payload signing, HTTP dispatch |
| `notifications/` | Channel branching (SMTP via lettre, Slack chat.postMessage, webhook, InApp) |
| `scanning/` | Secret scanner (15+ regex patterns), license scanner (50+ SPDX) |
| `telemetry/` | OTLP exporter (reqwest POST, JSON types, spans/metrics/logs) |
| `config.rs` | Environment-based configuration with validation |

### civit-brain (~1,400 LOC)

AI/ML services. Designed as air-gap compatible — all inference via local or self-hosted endpoints.

| Module | Responsibility |
|--------|---------------|
| `ast/` | 3-tier parser: `syn`/`swc`/`sqlparser` > tree-sitter (feature-gated) > regex. 19 languages. |
| `embedding.rs` | HTTP client to `/v1/embeddings` (vLLM/Ollama/OpenAI). API + Deterministic backends. |
| `vectordb.rs` | `VectorDb` async trait (RPITIT). In-memory + Qdrant backends. |
| `rag.rs` | `RAGPipeline<T: VectorDb>` generic. `LlmCodeReviewer<T, P>`. |
| `inference.rs` | `InferenceService` HTTP client to `/v1/chat/completions`. SSE streaming. |
| `review/` | `DiffAnalyzerReviewAgent` — bridges static analysis to `ReviewAgent` trait. |
| `agent.rs` | `ReviewAgent<T: VectorDb>` — generic review orchestrator. |

### civit-vfs (~900 LOC)

Remote filesystem via gRPC.

| Module | Responsibility |
|--------|---------------|
| `vfs.proto` | 8 RPCs (ReadDir, ReadFile, WriteFile, Delete, Stat, etc.) |
| `grpc_server.rs` | tonic gRPC server with mTLS support |
| `vfs/` | In-memory HashMap VFS with libc error codes (FUSE mount deferred to v1.1) |

### civit-crypto (~1,220 LOC)

Cryptographic primitives and enterprise auth protocols. Library-only (no binary).

| Module | Responsibility |
|--------|---------------|
| `cel/` | CEL evaluator: arithmetic, 15 functions, parenthesized sub-expressions |
| `hmac.rs` | SHA-256/512, HMAC-SHA256 via sha2/hmac crates |
| `oidc.rs` | JWKS fetch, RS256 signature verification via ring |
| `saml.rs` | SHA-256 digest integrity verification |
| `webauthn.rs` | CBOR parsing, structure validation for registration/authentication |
| `hsm/` | Software key fallback: ECDSA/HMAC/AES-GCM via ring (67 tests) |
| `vuln.rs` | `OsvVulnScanner` — OSV API client, CVSS classification |
| `provenance.rs` | SLSA `ProvenanceSigner` + PEM codec |
| `mtls.rs` | rcgen X.509 CA creation, cert issuance, SHA-256 fingerprints |
| `policy.rs` | CAS-style policy engine (Subject/Action/Resource/Condition/Effect) |

### civit-runner (~515 LOC)

CI/CD execution engine.

| Module | Responsibility |
|--------|---------------|
| `pipeline.rs` | Pipeline step execution in real Podman containers |
| `podman.rs` | Podman service: auto-detect Unix socket vs HTTP, CLI transport |
| `kube_controller.rs` | K8s operator: `kube::runtime::Reconciler`, leader election via Lease CRD |
| `sync.rs` | Multi-master DAG sync: `IncrementalSyncEngine`, checkpointing, conflict resolution |
| `redis_session.rs` | Redis-backed token rotation for session management |

## Data Flow

### Git Push → Code Review

```
Developer → SSH/HTTP → civit-core API → gitoxide (store) → EventBus
                                                              ↓
                                          DiffAnalyzer → ReviewAgent<T: VectorDb>
                                                              ↓
                                          RAGPipeline.retrieve() → LlmCodeReviewer
                                                              ↓
                                          LlmProvider.infer() → LlmReviewResult → Notification
```

### CI Pipeline

```
Push Event → EventBus → PipelineEngine.execute_step()
                           ↓
              Podman CLI → Container Build/Test → stdout/stderr capture
                           ↓
              PipelineResult → NotificationService.dispatch()
```

### Federation

```
Remote Forge → ForgeFed Delivery → Ed25511 HTTP Sign → WebFinger Discovery
                                                    ↓
              Local CivitForge → Verify Signature → Process Activity → Store
```

## Feature Flags

| Flag | Purpose | Safety Impact |
|------|---------|--------------|
| `syn-parser` | Rust AST parsing via `syn` 2 | Zero unsafe |
| `swc-parser` | JavaScript/TypeScript via `swc` 12 | Zero unsafe |
| `sql-parser` | SQL via `sqlparser` 0.62 | Zero unsafe |
| `treesitter` | Tree-sitter 0.24 (12+ languages) | **C FFI** (feature-gated via ADR-001) |
| `ssh-server` | russh SSH daemon | Zero unsafe |

## Design Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| API-based embeddings | `/v1/embeddings` HTTP call | ML consumer, not ML platform. Saves ~160h, avoids 50-200MB binary. |
| 3-tier AST routing | native > tree-sitter > regex | Pure-Rust preferred; tree-sitter as fallback. ADR-001 for unsafe. |
| RPITIT VectorDb trait | `impl Future<Output=...> + Send` | Native Rust 1.88, zero overhead, explicit Send bounds. |
| Podman CLI transport | `tokio::process::Command` | Zero new deps. Works without Unix socket transport crate. |
| Direct OTLP export | Raw reqwest POST | Avoids transitive unsafe from opentelemetry SDK. |
| lettre for SMTP | `lettre` 0.11 | Standard Rust email library. AsyncSmtpTransport + Tokio1Executor. |

## Technology Stack

| Layer | Technology |
|-------|-----------|
| HTTP framework | Axum 0.8 (WebSocket, multipart) |
| Git operations | gitoxide (gix) 0.70 — C-free, pure Rust |
| Database | sqlx 0.8 (PostgreSQL) |
| Crypto | ring 0.17, sha2 0.10, hmac 0.12 |
| TLS/mTLS | rcgen 0.13, x509-parser 0.17, rustls (via reqwest) |
| Auth | jsonwebtoken 9 |
| Kubernetes | kube-rs 0.98, k8s-openapi v1_30 |
| gRPC | tonic 0.12, prost 0.13 |
| Edge cache | Redis 7 + zstd compression |
| Serialization | serde 1, serde_json 1 |
| Email | lettre 0.11 |

## Workspace Structure

```
CivitForge/
├── Cargo.toml              # Workspace root
├── Dockerfile              # Convenience build (Evergreen-compliant)
├── docker-compose.yml      # Full-stack local deployment
├── container/
│   ├── civitforge/         # Main server image (tier: critical)
│   ├── runner/              # CI pipeline daemon (tier: standard)
├── crates/
│   ├── civit-shared/        # Shared types (backend + frontend)
│   ├── civit-core/          # API server, auth, DB, events
│   ├── civit-runner/        # CI execution, K8s operator, Podman
│   ├── civit-brain/         # AI/ML, RAG, AST parsing
│   ├── civit-crypto/        # Crypto primitives, CEL, enterprise auth
│   ├── civit-vfs/           # gRPC filesystem
│   └── (future: civit-ui)  # Leptos web frontend
├── docs/                   # Operator guide, API reference
├── .specs/                 # Architecture specs, tests, constraints
└── .adrs/                  # Architecture Decision Records
```
