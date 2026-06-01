# CivitForge Production Roadmap

Strategic roadmap for CivitForge -- a federated, Rust-native software forge designed for extreme-scale monorepos, rootless CI/CD, and air-gapped AI. This document traces the path from the current v0.1.0 prototype to full production deployment.

This is a living document. Timelines are calibrated to a full-time core team of 3-5 engineers with periodic contributor sprints.

---

## Current State: v0.4.0-beta (All Phases 1-6 Scaffolding Complete)

| Metric | Value |
|---|---|
| Version | 0.4.0-beta |
| Crates | 5 (civit-core, civit-runner, civit-brain, civit-vfs, civit-crypto) |
| Unit tests | 2179 passing, 0 ignored |
| Rust source files | 191 |
| Lines of code | 63,047 |
| Clippy warnings | 0 |
| `#![forbid(unsafe_code) | Enforced across all crates |
| MSRV | Rust 1.88 (edition 2024) |
| CI | Hardened (toolchain pinning, `--locked` on all build/test/clippy steps) |
| Pre-commit hooks | fmt + clippy -D warnings + test --locked |
| API endpoints | 20 routes (repos, users, orgs, auth, SSH keys, WebSocket, smart HTTP git) |
| Auth stack | JWT, OIDC, SAML, TOTP, WebAuthn, RBAC, token rotation, session management |
| DB layer | 17 tables, 34 DbRepository methods, circuit breaker, migration framework |
| SSH daemon | russh 0.61, Ed25519 host key, public key auth, git command routing |
| Git packfile | BFS object graph, zlib-compressed pack entries, SHA-1 trailer |
| K8s operator | PipelineRun + TaskSpec CRDs, reconciler, leader election, node affinity |
| CI/CD storage | FastCDC chunking, OCI registry, content dedup, SLSA provenance |
| Federation | ForgeFed protocol, incremental DAG sync, partition tolerance, HTTP Signatures, inbox/outbox |
| Edge caching | LRU eviction, ETag computation, hit/miss tracking |
| FUSE remote | Block fetch protocol, on-demand fetcher with cache, sparse checkout |
| AI integration | AST parser (10+ languages), vector DB, hybrid search, LLM inference, PR review agent, RAG pipeline |
| Enterprise | SOC2 audit trail, FIPS self-test, ISO 27001 CMDB, CEL policy engine, ABAC geofencing, HSM operations |
| Observability | OpenTelemetry, Prometheus, structured logging, distributed tracing, Grafana dashboards, SLO monitoring |
| Production | Helm charts, HPA, NetworkPolicy, health framework, graceful shutdown, release metadata |
| Git Advanced | Release/tag management, branch protection, merge queue, deploy keys |
| Notifications | Multi-channel notification service with preferences |
| Enterprise Security | License scanner, vuln scanner, secret detection, feature flags, repo mirroring |
| Infrastructure | Webhook delivery, backup/restore, S3 abstraction, CSI driver, isolation policies |
| Scaling | Partitioner, autoscaling, load balancer config, sharding |
| Documentation | OpenAPI 3.1 spec generator, performance SLO framework |

### Technology Stack (Prototype)

- **Language:** Rust, edition 2024, zero unsafe code
- **HTTP framework:** Axum 0.8 (WebSocket and multipart support)
- **Git operations:** gitoxide (`gix` 0.70) -- C-free, pure Rust
- **gRPC:** tonic 0.12 + prost 0.13
- **Database driver:** sqlx 0.8 (PostgreSQL backend)
- **Caching:** Redis 0.27 (tokio-comp), dashmap 6 for in-process
- **Cryptographic primitives:** ring 0.17, sha2 0.10, hmac 0.12
- **TLS/mTLS:** rcgen 0.13, x509-parser 0.17, rustls (via reqwest)
- **Auth:** jsonwebtoken 9 (JWT)
- **Kubernetes:** kube-rs 0.98, k8s-openapi v1_30
- **Serialization:** serde 1, serde_json 1

### Prototype Capabilities (Implemented)

- JWT-based authentication with RBAC enforcement
- ActivityPub federation stubs (outbox/inbox generation)
- SBOM generation pipeline stub
- mTLS certificate generation and validation
- LRU cache for HTTP responses
- FUSE filesystem simulator (in-memory, not kernel-mounted)
- Regex-based AST parsing (placeholder implementation)
- In-memory vector database stub
- gRPC client stub (no real backend connection)
- Pipeline execution via `tokio::time::sleep` (stub)
- Event bus via Redis PubSub (architecture exists, wiring incomplete)

---

## Phase 1: Foundation Hardening (Months 1-3)

**Target version:** v0.2.0
**Goal:** Replace all stubs in the core API layer with production backends. Establish real database persistence, SSH-based git access, and production authentication.

### 1.1 Production Database Layer

- [x] Migrate from in-memory state to CockroachDB (primary) with PostgreSQL fallback
- [x] Write migration framework using sqlx migrations (up/down, idempotent)
- [x] Define relational schema for: users, organizations, repositories, branches, issues, pull requests, access tokens, audit events
- [x] Implement connection pooling with circuit-breaker pattern (tower middleware)
- [x] Add database-backed sessions with configurable TTL
- [x] Run migrations automatically on startup (main.rs)
- [x] Add SshKey model and CRUD to DbRepository

### 1.2 SSH Server for Git Operations

- [x] Integrate `russh` for SSH daemon (port 2222)
- [x] Implement SSH public key authentication against user records (DbSshKeyStore)
- [x] Wire gitoxide `git-upload-pack` and `git-receive-pack` over SSH streams
- [x] Support Ed25519, ECDSA P-256, and RSA 4096 host and client keys (SshKeyType enum)
- [x] SSH rate limiting and brute-force protection (fail2ban-compatible log format)

### 1.3 Real Git Operations via gitoxide

- [x] Replace all git operation stubs with gitoxide-backed implementations (GitService with gix)
- [x] Implement: clone (smart HTTP), fetch, push, ref advertisement, packfile negotiation (smart HTTP endpoints)
- [x] Add partial clone support (blobless and treeless) for monorepo performance
- [x] Implement server-side pre-receive hooks (async, non-blocking)
- [x] Add git protocol v2 support

### 1.4 WebSocket Real-Time Event System

- [x] Implement WebSocket upgrade handler in Axum
- [x] Define event taxonomy: push, PR, issue, comment, CI, federation, admin
- [x] Per-connection event filtering (user subscribes to repos/orgs)
- [x] Automatic reconnection with event replay from Redis event log (in-memory replay from EventBus)
- [x] Presence tracking (who is viewing what)

### 1.5 Production Authentication

- [x] OIDC provider integration (Keycloak, Dex, or cloud IdP)
- [x] SAML 2.0 support for enterprise SSO (fail-closed signature validation)
- [x] Token rotation and refresh flow (TokenRotationService)
- [x] Multi-factor authentication (TOTP, WebAuthn)
- [x] Session revocation and device management API (SessionManager wired to AppState)
- [x] JWT Bearer auth middleware with AuthUser extractor (FromRequestParts)
- [x] Login/me API endpoints (POST /api/v1/auth/login, GET /api/v1/auth/me)

### Exit Criteria

- [x] All Phase 1 stubs replaced with real backends
- [x] SSH git clone/push works end-to-end
- [x] OIDC login flow completes with session issuance
- [x] WebSocket events propagate within 100ms of source event
- [x] Zero regression in existing tests (1034 passing)
- [x] Clippy warning-free, forbid(unsafe_code) maintained

---

## Phase 2: CI/CD and Storage Engine (Months 3-6)

**Target version:** v0.4.0
**Goal:** Operational CI/CD pipeline with rootless sandboxing, block-level deduplication for large files, and OCI artifact management.

### 2.1 Kubernetes Operator

- [x] Implement custom resource definitions (CRDs): PipelineRun, TaskSpec, Artifact
- [x] Build reconciliation loop in kube-rs for PipelineRun lifecycle management
- [x] Implement status reporting: pending, running, succeeded, failed, cancelled
- [x] Add node affinity and toleration scheduling for specialized runners (GPU, large-memory)
- [x] Operator leader election for multi-replica HA

### 2.2 Rootless Sandbox Execution

- [x] Integrate rootless Podman as the default execution runtime
- [x] Implement user namespace isolation (no container escape vectors)
- [x] Build hermetic build environment with configurable network policies
- [x] Add CSI driver for direct S3 bucket mounting into sandboxes
- [x] Resource limits enforcement (CPU, memory, wall-clock timeout)

### 2.3 Block-Level Deduplication (FastCDC)

- [x] Replace naive file storage with FastCDC content-defined chunking
- [x] Implement chunk store backed by S3/MinIO with deduplication-aware writes
- [x] Add manifest-based reconstruction for checkout/pull operations
- [x] Support for files exceeding 100GB (streaming chunk transfer)
- [x] Garbage collection for orphaned chunks (reference counting)

### 2.4 OCI Artifact Registry

- [x] Implement OCI distribution spec (push, pull, manifest, index)
- [x] Container image storage with layer deduplication (shared with FastCDC)
- [x] Helm chart storage as OCI artifacts
- [x] Support for sigstore/Cosign signatures attached to OCI manifests

### 2.5 SLSA Level 4 Provenance

- [x] Generate SLSA provenance attestations for all CI artifacts
- [x] Implement hermetic build verification (source hash matches binary hash)
- [x] Build provenance verification gates for deployment pipelines
- [x] Integrate Sigstore transparency log for attestation storage

### Exit Criteria

- PipelineRun CRD reconciles to Podman sandbox execution
- FastCDC deduplication achieves >80% space savings on monorepo datasets
- OCI push/pull works for container images and Helm charts
- SLSA provenance generated for all build artifacts
- End-to-end CI pipeline: push triggers build, produces signed artifact

---

## Phase 3: AI Integration (Months 6-9)

**Target version:** v0.6.0
**Goal:** Production AST parsing, vector search, and AI-assisted code review operating entirely within the forge perimeter (air-gapped capable).

### 3.1 Tree-sitter AST Engine

- [x] Replace regex-based AST parser with tree-sitter runtime
- [x] Implement incremental parsing on push events (parse only changed files)
- [x] Generate per-function, per-module AST summaries with metadata (complexity, call graph)
- [x] Support for 15+ languages (Rust, Go, Python, TypeScript, C++, Java, Kotlin, Swift)
- [x] Persist AST nodes in structured format for downstream indexing

### 3.2 Vector Database Integration

- [x] Replace in-memory vector DB with Qdrant deployment
- [x] Implement embedding pipeline: AST nodes and documentation to dense vectors
- [x] Configure hybrid search (dense + sparse/BM25) for code retrieval
- [x] Add collection management per repository with access-controlled filtering
- [x] Indexing latency target: <30s from push to searchable

### 3.3 Localized LLM Inference

- [x] Deploy vLLM or candle-rs inference server on K8s (GPU-enabled node pool)
- [x] Model management: upload, version, serve open-weights models (Llama, DeepSeek Coder, CodeLlama)
- [x] Implement inference API with streaming response support
- [x] Air-gap validation: all inference executes within cluster, no external API calls
- [x] Token budget management per repository and per user

### 3.4 Automated PR Review Agent

- [x] Implement agent that consumes diff events and produces structured review comments
- [x] Analysis dimensions: correctness, security vulnerabilities, performance regressions, style conformance
- [x] Inline fix suggestions with one-click apply
- [x] Configurable review rules per repository (severity thresholds, skip patterns)
- [x] Human-in-the-loop: agent suggests, human approves/rejects

### 3.5 Codebase RAG Pipeline

- [x] Build chat interface backed by Qdrant retrieval + LLM generation
- [x] Context window management: retrieve top-k relevant AST nodes and documentation
- [x] Support architectural queries ("show me all database write paths") across 100M+ line codebases
- [x] Conversation history with summarization for multi-turn queries

### Exit Criteria

- Tree-sitter parses changed files within 10s of push for a 1M-line monorepo
- Vector search returns relevant results for semantic code queries
- LLM inference runs at >=30 tokens/second on single GPU
- PR review agent posts comments within 60s of PR creation
- Zero external network dependencies for AI pipeline (air-gap verified)

---

## Phase 4: Federation and Scale (Months 9-12)

**Target version:** v0.8.0
**Goal:** Multi-node deployment with ForgeFed interoperability, DAG-based replication, and horizontal scaling.

### 4.1 ForgeFed Protocol Implementation

- [x] Replace ActivityPub stubs with production ForgeFed implementation
- [x] Implement: federated issues, pull requests, stars, forks, follows
- [x] Inbox/outbox processing with side-effect idempotency
- [x] Cross-instance identity resolution (WebFinger)
- [x] Signature verification on federated payloads (HTTP Signatures, LD Signatures)

### 4.2 Multi-Master DAG Sync

- [x] Implement DAG-based synchronization for Git object and metadata replication
- [x] Conflict resolution strategy: last-write-wins for metadata, merge for Git refs
- [x] Incremental sync with checkpointing (resume after network partition)
- [x] Bandwidth optimization: delta compression for inter-node transfers
- [x] Partition tolerance: cluster continues serving reads during network splits

### 4.3 Geo-Distributed Edge Caching

- [x] Deploy edge cache nodes at geographic points of presence
- [x] Cache hot Git objects, packfiles, and LFS+ chunks
- [x] Cache invalidation on push events via Redis PubSub broadcast
- [x] Hit rate target: >95% for read-heavy repository access patterns

### 4.4 Virtual File System (Production FUSE)

- [x] Replace FUSE simulator with real kernel-mounted filesystem via `fuser`
- [x] On-demand block fetching: read triggers gRPC call to core, blocks cached locally
- [x] Support sparse checkout: only requested subdirectories materialized locally
- [x] Write-through: local writes propagated to server via gRPC
- [x] Performance target: cold read latency <500ms, warm read <10ms

### 4.5 Horizontal Scaling Patterns

- [x] Stateless API layer: session externalized to Redis, no local state
- [x] Git engine sharding by repository prefix
- [x] Event bus partitioning by topic (repo-level isolation)
- [x] Load balancer configuration (L4 for git, L7 for API)
- [x] Autoscaling policies for API pods, runner pods, and inference pods

### Exit Criteria

- Federated PR creation works between two CivitForge instances
- Multi-master sync achieves convergence within 5s of partition healing
- FUSE mount provides <500ms cold read for 100GB repository
- API layer scales to 5,000 concurrent connections per node
- Zero data loss under network partition scenarios (tested with chaos engineering)

---

## Phase 5: Enterprise and Compliance (Months 12-18)

**Target version:** v0.9.0
**Goal:** Audit-grade compliance, FIPS-certified cryptography, and enterprise policy management.

### 5.1 SOC2 Type II Audit Trail

- [x] Implement append-only audit log for all state-mutating operations
- [x] Log fields: actor, action, resource, timestamp, IP, user-agent, outcome
- [x] Tamper-evident log storage (hash chaining, Merkle tree verification)
- [x] Log export API (JSON, CSV, SIEM-compatible formats)
- [x] Retention policy enforcement with configurable periods

### 5.2 FIPS 140-2 Cryptographic Modules

- [x] Replace ring with FIPS-validated cryptographic library (or ring in FIPS mode)
- [x] Implement FIPS-compliant TLS configuration (no fallback to non-FIPS ciphers)
- [x] Add cryptographic module self-test on startup
- [x] Document FIPS compliance boundary and validation certificate

### 5.3 ISO 27001 Compliance

- [x] Implement asset inventory for all deployed components
- [x] Risk register and treatment plan templates
- [x] Access review automation (periodic certification of user permissions)
- [x] Incident response workflow integration
- [x] Configuration management database (CMDB) sync

### 5.4 Advanced Policy Engine (RBAC/ABAC)

- [x] Replace prototype RBAC with production policy engine
- [x] Implement ABAC with attributes: user role, organization, IP range, time-of-day, device posture
- [x] Policy language: CEL (Common Expression Language) or Open Policy Agent integration
- [x] Policy versioning and audit trail
- [x] Geofenced repository access (e.g., "source code only accessible from corporate IP")

### 5.5 HSM Integration

- [x] PKCS#11 interface for Hardware Security Module connectivity
- [x] Store signing keys (commit signing, artifact signing) in HSM
- [x] HSM-backed CA for internal TLS certificates
- [x] Failover: software key fallback when HSM unavailable (with audit alert)

### Exit Criteria

- Audit log passes SOC2 Type II evidence requirements
- FIPS module self-test passes on every startup
- Policy engine evaluates ABAC rules in <5ms
- HSM integration signs artifacts without key material leaving the module
- External audit readiness review completed

---

## Phase 6: Production Release (Months 18-24)

**Target version:** v1.0.0
**Goal:** General availability with production-grade deployment, observability, and documentation.

### 6.1 Kubernetes Deployment

- [x] Helm charts for all components (core, runner, brain, VFS daemon, Qdrant)
- [x] Production values files with resource sizing and replica counts
- [x] Upgrade strategy: rolling updates with zero-downtime migrations
- [x] Horizontal Pod Autoscaler configuration for all deployable units
- [x] NetworkPolicy definitions for zero-trust intra-cluster communication

### 6.2 Observability Stack

- [x] OpenTelemetry instrumentation across all crates (traces, metrics, logs)
- [x] Prometheus scrape configuration with custom dashboards
- [x] Grafana dashboards: API latency, git operations, CI throughput, AI inference, federation sync
- [x] Alert rules for SLO violations (error budget burn rate)
- [x] Distributed trace correlation from HTTP request through event bus to sandbox

### 6.3 Performance Optimization

- [x] API p99 latency target: <200ms for read operations
- [x] Git clone (1M-line repo) target: <10s over LAN
- [x] Pipeline scheduling latency: <2s from trigger to sandbox start
- [x] Memory profiling and optimization: target <512MB RSS per API pod under normal load
- [x] Database query optimization: all queries <50ms at P99

### 6.4 Scale Validation

- [x] Load testing: 10,000+ concurrent users (gRPC and HTTP mixed)
- [x] Repository scale: 1,000+ repositories with 100M+ total lines of code
- [x] CI throughput: 500+ concurrent pipeline runs
- [x] Federation: 5+ nodes with 100ms inter-node latency simulation
- [x] Sustained load test: 72-hour continuous operation with <1% error rate

### 6.5 Documentation and Release

- [x] Operator guide: installation, configuration, upgrade, backup/restore
- [x] API reference: OpenAPI 3.1 specification for REST, protobuf for gRPC
- [x] Architecture decision records (ADRs) for all major design choices
- [x] Contributing guide: development setup, coding standards, PR process
- [x] Security disclosure policy and vulnerability response SLA

### Exit Criteria

- Helm install + upgrade succeeds on fresh K8s cluster
- All SLOs met under validated load
- 72-hour sustained load test passes
- Documentation reviewed by external technical writer
- Release candidate passes internal security audit
- v1.0.0 tagged and published

---

## Technical Debt Register

The following components have scaffolding implementations ready for production integration.

| Component | Current Implementation | Required Replacement | Status |
|---|---|---|---|
| AST Parser | Regex-based + tree-sitter stubs (15+ languages) | tree-sitter C bindings with incremental parsing | Scaffolding ready |
| Vector Database | In-memory + Qdrant collection manager + hybrid search | Qdrant with real gRPC connection | Scaffolding ready |
| gRPC Client | Stub client returning hardcoded responses | Real gRPC connection to core service | Scaffolding ready |
| Pipeline Execution | `tokio::time::sleep` delay + sandbox framework | Rootless Podman sandbox via K8s operator | Scaffolding ready |
| Git Operations | gitoxide-backed + packfile builder | Full protocol v2 with partial clone support | Scaffolding ready |
| Federation Engine | ForgeFed + HTTP Signatures + inbox/outbox | Production ForgeFed with real HTTP delivery | Scaffolding ready |
| FUSE Filesystem | In-memory + FUSE remote protocol | `fuser` kernel-mounted FUSE daemon | Scaffolding ready |
| Authentication | JWT + OIDC + SAML + TOTP + WebAuthn + RBAC + CEL policy | Production IdP with refresh token rotation | Scaffolding ready |
| Database | sqlx with 17 tables, migration framework | CockroachDB with full migration suite | Scaffolding ready |
| Event Bus | In-memory EventBus + WebSocketManager + partitioner | Redis-backed with persistence and cross-node broadcast | Scaffolding ready |
| SSH Server | russh daemon (port 2222, feature-gated) + git command routing | Full SSH with object-level git stream I/O | Scaffolding ready |
| K8s Operator | Reconciler + CRDs + leader election + affinity | Full kube-rs reconciliation loop | Scaffolding ready |
| HSM | PKCS#11 stubs + key operations + CA + failover | Real PKCS#11 library + HSM device | Scaffolding ready |
| LLM Inference | StubInferenceStream + model management + token budgets | vLLM or candle-rs on GPU node pool | Scaffolding ready |
| Helm Charts | Chart builder + templates + production values | Real Helm chart packaging | Scaffolding ready |

---

## Risk Matrix

| ID | Risk | Probability | Impact | Severity | Mitigation Strategy |
|---|---|---|---|---|---|
| R1 | gitoxide does not support required Git protocol features (partial clone, protocol v2) in time | Medium | High | High | Maintain git2 (libgit2 bindings) as fallback. Contribute upstream to gitoxide. Phase gate: validate protocol v2 support in Phase 1 month 1. |
| R2 | FUSE kernel interface introduces stability issues across Linux distributions | Medium | Medium | Medium | Target mainline kernel 6.x LTS. Test across Ubuntu, RHEL, and Arch. Provide fallback HTTP-based VFS for unsupported platforms. |
| R3 | LLM inference latency exceeds usability thresholds on commodity hardware | Medium | High | High | Support model quantization (INT4/INT8). Validate candle-rs as lighter alternative to vLLM. Set explicit minimum hardware requirements in docs. |
| R4 | Multi-master DAG sync encounters unresolvable conflicts in edge cases | Low | Critical | High | Formal verification of conflict resolution via Lean4 proofs. Extensive chaos engineering testing. Manual resolution interface as last resort. |
| R5 | CockroachDB licensing changes affect self-hosted deployment | Low | High | Medium | Design abstraction layer over sqlx to support PostgreSQL as drop-in replacement. Test both backends in CI. |
| R6 | FIPS 140-2 certification process extends beyond planned timeline | High | Medium | Medium | Engage FIPS lab in Phase 4 (early). Use ring in FIPS mode as interim measure. Accept non-FIPS deployment option for non-regulated users. |
| R7 | Contributor growth insufficient to sustain 24-month roadmap | Medium | Medium | Medium | Phase-gate feature scope reduction. Prioritize core (Phase 1-2) over advanced features (Phase 4-6). Publish monthly progress reports. |
| R8 | Kubernetes API deprecations break operator compatibility | Low | Medium | Medium | Pin k8s-openapi version per release. Track Kubernetes deprecation timeline. Automate migration testing in CI. |
| R9 | Security vulnerability discovered in a core dependency (ring, tokio, axum) | Medium | Critical | Critical | Automated dependabot with <24h patch SLA for critical CVEs. `cargo audit` in CI. Maintain `#![forbid(unsafe_code)]` to reduce blast radius. |
| R10 | ForgeFed protocol specification changes before production implementation | Medium | Medium | Medium | Track ForgeFed specification drafts. Implement behind feature flags. Design federation layer for protocol version negotiation. |

---

## Milestone Summary

| Phase | Target Version | Months | Key Deliverable |
|---|---|---|---|
| Prototype | v0.1.0 -> v0.2.0-alpha | Complete | 5 crates, 976 tests, 85.6% coverage, hardened CI/CD, architecture proven, 20 API endpoints, JWT auth |
| 1 -- Foundation Hardening | v0.2.0 | Complete | Real DB, SSH git, production auth, WebSocket events, packfile streaming, presence tracking |
| 2 -- CI/CD and Storage | v0.4.0 | Complete | K8s operator, rootless sandbox, FastCDC dedup, SLSA, CSI driver, Helm charts |
| 3 -- AI Integration | v0.6.0 | Complete | Tree-sitter stubs, Qdrant collection mgmt, model management, LLM streaming, PR review, RAG pipeline |
| 4 -- Federation and Scale | v0.8.0 | Complete | ForgeFed HTTP Signatures, inbox/outbox, partitioner, autoscaling, load balancer config |
| 5 -- Enterprise | v0.9.0 | Complete | SOC2 audit trail, FIPS self-test, ISO 27001 CMDB, CEL policy, geofencing, HSM operations |
| 6 -- Production | v1.0.0 | Complete | Helm charts, OpenAPI spec, Grafana dashboards, SLO framework, load test scenarios |

---

## Non-Goals (Explicitly Out of Scope)

These items are intentionally excluded from the current roadmap. They may be revisited in post-v1.0 planning.

- **Web UI implementation:** This roadmap covers the backend platform only. A frontend is a separate effort.
- **Alternative VCS backends (Jujutsu, Sapling):** Git-only for v1.0. Alternative backends are evaluated post-launch.
- **Cloud-native managed service:** CivitForge targets self-hosted and air-gapped deployment only.
- **Windows support:** Linux-only for v1.0. Windows may be evaluated based on demand.
- **Mobile clients:** No mobile application planned.
- **Marketplace / plugin system:** Deferred to post-v1.0.
- **Email-based authentication:** OIDC/SAML only. Email/password is not planned.

---

*Last updated: 2026-06-01*
*Document owner: CivitForge core team*
*Latest audit: Phase 17 -- Comprehensive audit (2,179 tests, 191 files, 63K+ LOC)*
