# CivitForge Production Roadmap

Strategic roadmap for CivitForge -- a federated, Rust-native software forge designed for extreme-scale monorepos, rootless CI/CD, and air-gapped AI. This document traces the path from the current v0.1.0 prototype to full production deployment.

This is a living document. Timelines are calibrated to a full-time core team of 3-5 engineers with periodic contributor sprints.

---

## Current State: v0.1.3 (Deadlock Fix, Clippy Clean, Version Synchronized)

| Metric | Value |
|---|---|
| Version | 0.1.3 |
| Crates | 5 (civit-core, civit-runner, civit-brain, civit-vfs, civit-crypto) |
| Unit tests | 882 passing, 0 ignored |
| Lean4 proofs | 5/5 compiling |
| Rust source files | 115 |
| Lines of code | 29,086 |
| Spec artifacts | 42 |
| EARS requirements | 69 |
| Clippy warnings | 0 |
| Test coverage (line) | 85.61% |
| Test coverage (region) | 86.61% |
| `#![forbid(unsafe_code)]` | Enforced across all crates |
| MSRV | Rust 1.88 (edition 2024) |
| CI | Hardened (toolchain pinning, `--locked` on all build/test/clippy steps) |
| Pre-commit hooks | fmt + clippy --locked + test --locked |
| SAML security | Fail-closed signature validation |
| Health endpoints | /healthz, /ready, /api/v1/health (matching Helm probes) |
| Helm chart | Version-synchronized with workspace (0.1.0) |
| Documentation | 8 ADRs, CONTRIBUTING.md, CHANGELOG.md, landing page at GitHub Pages |
| Known bugs fixed | DashMap deadlock in RateLimiter, items_after_test_module clippy error |

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

- [ ] Migrate from in-memory state to CockroachDB (primary) with PostgreSQL fallback
- [ ] Write migration framework using sqlx migrations (up/down, idempotent)
- [ ] Define relational schema for: users, organizations, repositories, branches, issues, pull requests, access tokens, audit events
- [ ] Implement connection pooling with circuit-breaker pattern (tower middleware)
- [ ] Add database-backed sessions with configurable TTL

### 1.2 SSH Server for Git Operations

- [ ] Integrate `russh` for SSH daemon (port 2222)
- [ ] Implement SSH public key authentication against user records
- [ ] Wire gitoxide `git-upload-pack` and `git-receive-pack` over SSH streams
- [ ] Support Ed25519, ECDSA P-256, and RSA 4096 host and client keys
- [ ] SSH rate limiting and brute-force protection (fail2ban-compatible log format)

### 1.3 Real Git Operations via gitoxide

- [ ] Replace all git operation stubs with gitoxide-backed implementations
- [ ] Implement: clone (smart HTTP), fetch, push, ref advertisement, packfile negotiation
- [ ] Add partial clone support (blobless and treeless) for monorepo performance
- [ ] Implement server-side pre-receive hooks (async, non-blocking)
- [ ] Add git protocol v2 support

### 1.4 WebSocket Real-Time Event System

- [ ] Implement WebSocket upgrade handler in Axum
- [ ] Define event taxonomy: push, PR, issue, comment, CI, federation, admin
- [ ] Per-connection event filtering (user subscribes to repos/orgs)
- [ ] Automatic reconnection with event replay from Redis event log
- [ ] Presence tracking (who is viewing what)

### 1.5 Production Authentication

- [ ] OIDC provider integration (Keycloak, Dex, or cloud IdP)
- [ ] SAML 2.0 support for enterprise SSO
- [ ] Token rotation and refresh flow
- [ ] Multi-factor authentication (TOTP, WebAuthn)
- [ ] Session revocation and device management API

### Exit Criteria

- All Phase 1 stubs replaced with real backends
- SSH git clone/push works end-to-end
- OIDC login flow completes with session issuance
- WebSocket events propagate within 100ms of source event
- Zero regression in existing 773 tests
- Clippy warning-free, forbid(unsafe_code) maintained

---

## Phase 2: CI/CD and Storage Engine (Months 3-6)

**Target version:** v0.4.0
**Goal:** Operational CI/CD pipeline with rootless sandboxing, block-level deduplication for large files, and OCI artifact management.

### 2.1 Kubernetes Operator

- [ ] Implement custom resource definitions (CRDs): PipelineRun, TaskSpec, Artifact
- [ ] Build reconciliation loop in kube-rs for PipelineRun lifecycle management
- [ ] Implement status reporting: pending, running, succeeded, failed, cancelled
- [ ] Add node affinity and toleration scheduling for specialized runners (GPU, large-memory)
- [ ] Operator leader election for multi-replica HA

### 2.2 Rootless Sandbox Execution

- [ ] Integrate rootless Podman as the default execution runtime
- [ ] Implement user namespace isolation (no container escape vectors)
- [ ] Build hermetic build environment with configurable network policies
- [ ] Add CSI driver for direct S3 bucket mounting into sandboxes
- [ ] Resource limits enforcement (CPU, memory, wall-clock timeout)

### 2.3 Block-Level Deduplication (FastCDC)

- [ ] Replace naive file storage with FastCDC content-defined chunking
- [ ] Implement chunk store backed by S3/MinIO with deduplication-aware writes
- [ ] Add manifest-based reconstruction for checkout/pull operations
- [ ] Support for files exceeding 100GB (streaming chunk transfer)
- [ ] Garbage collection for orphaned chunks (reference counting)

### 2.4 OCI Artifact Registry

- [ ] Implement OCI distribution spec (push, pull, manifest, index)
- [ ] Container image storage with layer deduplication (shared with FastCDC)
- [ ] Helm chart storage as OCI artifacts
- [ ] Support for sigstore/Cosign signatures attached to OCI manifests

### 2.5 SLSA Level 4 Provenance

- [ ] Generate SLSA provenance attestations for all CI artifacts
- [ ] Implement hermetic build verification (source hash matches binary hash)
- [ ] Build provenance verification gates for deployment pipelines
- [ ] Integrate Sigstore transparency log for attestation storage

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

- [ ] Replace regex-based AST parser with tree-sitter runtime
- [ ] Implement incremental parsing on push events (parse only changed files)
- [ ] Generate per-function, per-module AST summaries with metadata (complexity, call graph)
- [ ] Support for 15+ languages (Rust, Go, Python, TypeScript, C++, Java, Kotlin, Swift)
- [ ] Persist AST nodes in structured format for downstream indexing

### 3.2 Vector Database Integration

- [ ] Replace in-memory vector DB with Qdrant deployment
- [ ] Implement embedding pipeline: AST nodes and documentation to dense vectors
- [ ] Configure hybrid search (dense + sparse/BM25) for code retrieval
- [ ] Add collection management per repository with access-controlled filtering
- [ ] Indexing latency target: <30s from push to searchable

### 3.3 Localized LLM Inference

- [ ] Deploy vLLM or candle-rs inference server on K8s (GPU-enabled node pool)
- [ ] Model management: upload, version, serve open-weights models (Llama, DeepSeek Coder, CodeLlama)
- [ ] Implement inference API with streaming response support
- [ ] Air-gap validation: all inference executes within cluster, no external API calls
- [ ] Token budget management per repository and per user

### 3.4 Automated PR Review Agent

- [ ] Implement agent that consumes diff events and produces structured review comments
- [ ] Analysis dimensions: correctness, security vulnerabilities, performance regressions, style conformance
- [ ] Inline fix suggestions with one-click apply
- [ ] Configurable review rules per repository (severity thresholds, skip patterns)
- [ ] Human-in-the-loop: agent suggests, human approves/rejects

### 3.5 Codebase RAG Pipeline

- [ ] Build chat interface backed by Qdrant retrieval + LLM generation
- [ ] Context window management: retrieve top-k relevant AST nodes and documentation
- [ ] Support architectural queries ("show me all database write paths") across 100M+ line codebases
- [ ] Conversation history with summarization for multi-turn queries

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

- [ ] Replace ActivityPub stubs with production ForgeFed implementation
- [ ] Implement: federated issues, pull requests, stars, forks, follows
- [ ] Inbox/outbox processing with side-effect idempotency
- [ ] Cross-instance identity resolution (WebFinger)
- [ ] Signature verification on federated payloads (HTTP Signatures, LD Signatures)

### 4.2 Multi-Master DAG Sync

- [ ] Implement DAG-based synchronization for Git object and metadata replication
- [ ] Conflict resolution strategy: last-write-wins for metadata, merge for Git refs
- [ ] Incremental sync with checkpointing (resume after network partition)
- [ ] Bandwidth optimization: delta compression for inter-node transfers
- [ ] Partition tolerance: cluster continues serving reads during network splits

### 4.3 Geo-Distributed Edge Caching

- [ ] Deploy edge cache nodes at geographic points of presence
- [ ] Cache hot Git objects, packfiles, and LFS+ chunks
- [ ] Cache invalidation on push events via Redis PubSub broadcast
- [ ] Hit rate target: >95% for read-heavy repository access patterns

### 4.4 Virtual File System (Production FUSE)

- [ ] Replace FUSE simulator with real kernel-mounted filesystem via `fuser`
- [ ] On-demand block fetching: read triggers gRPC call to core, blocks cached locally
- [ ] Support sparse checkout: only requested subdirectories materialized locally
- [ ] Write-through: local writes propagated to server via gRPC
- [ ] Performance target: cold read latency <500ms, warm read <10ms

### 4.5 Horizontal Scaling Patterns

- [ ] Stateless API layer: session externalized to Redis, no local state
- [ ] Git engine sharding by repository prefix
- [ ] Event bus partitioning by topic (repo-level isolation)
- [ ] Load balancer configuration (L4 for git, L7 for API)
- [ ] Autoscaling policies for API pods, runner pods, and inference pods

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

- [ ] Implement append-only audit log for all state-mutating operations
- [ ] Log fields: actor, action, resource, timestamp, IP, user-agent, outcome
- [ ] Tamper-evident log storage (hash chaining, Merkle tree verification)
- [ ] Log export API (JSON, CSV, SIEM-compatible formats)
- [ ] Retention policy enforcement with configurable periods

### 5.2 FIPS 140-2 Cryptographic Modules

- [ ] Replace ring with FIPS-validated cryptographic library (or ring in FIPS mode)
- [ ] Implement FIPS-compliant TLS configuration (no fallback to non-FIPS ciphers)
- [ ] Add cryptographic module self-test on startup
- [ ] Document FIPS compliance boundary and validation certificate

### 5.3 ISO 27001 Compliance

- [ ] Implement asset inventory for all deployed components
- [ ] Risk register and treatment plan templates
- [ ] Access review automation (periodic certification of user permissions)
- [ ] Incident response workflow integration
- [ ] Configuration management database (CMDB) sync

### 5.4 Advanced Policy Engine (RBAC/ABAC)

- [ ] Replace prototype RBAC with production policy engine
- [ ] Implement ABAC with attributes: user role, organization, IP range, time-of-day, device posture
- [ ] Policy language: CEL (Common Expression Language) or Open Policy Agent integration
- [ ] Policy versioning and audit trail
- [ ] Geofenced repository access (e.g., "source code only accessible from corporate IP")

### 5.5 HSM Integration

- [ ] PKCS#11 interface for Hardware Security Module connectivity
- [ ] Store signing keys (commit signing, artifact signing) in HSM
- [ ] HSM-backed CA for internal TLS certificates
- [ ] Failover: software key fallback when HSM unavailable (with audit alert)

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

- [ ] Helm charts for all components (core, runner, brain, VFS daemon, Qdrant)
- [ ] Production values files with resource sizing and replica counts
- [ ] Upgrade strategy: rolling updates with zero-downtime migrations
- [ ] Horizontal Pod Autoscaler configuration for all deployable units
- [ ] NetworkPolicy definitions for zero-trust intra-cluster communication

### 6.2 Observability Stack

- [ ] OpenTelemetry instrumentation across all crates (traces, metrics, logs)
- [ ] Prometheus scrape configuration with custom dashboards
- [ ] Grafana dashboards: API latency, git operations, CI throughput, AI inference, federation sync
- [ ] Alert rules for SLO violations (error budget burn rate)
- [ ] Distributed trace correlation from HTTP request through event bus to sandbox

### 6.3 Performance Optimization

- [ ] API p99 latency target: <200ms for read operations
- [ ] Git clone (1M-line repo) target: <10s over LAN
- [ ] Pipeline scheduling latency: <2s from trigger to sandbox start
- [ ] Memory profiling and optimization: target <512MB RSS per API pod under normal load
- [ ] Database query optimization: all queries <50ms at P99

### 6.4 Scale Validation

- [ ] Load testing: 10,000+ concurrent users (gRPC and HTTP mixed)
- [ ] Repository scale: 1,000+ repositories with 100M+ total lines of code
- [ ] CI throughput: 500+ concurrent pipeline runs
- [ ] Federation: 5+ nodes with 100ms inter-node latency simulation
- [ ] Sustained load test: 72-hour continuous operation with <1% error rate

### 6.5 Documentation and Release

- [ ] Operator guide: installation, configuration, upgrade, backup/restore
- [ ] API reference: OpenAPI 3.1 specification for REST, protobuf for gRPC
- [ ] Architecture decision records (ADRs) for all major design choices
- [ ] Contributing guide: development setup, coding standards, PR process
- [ ] Security disclosure policy and vulnerability response SLA

### Exit Criteria

- Helm install + upgrade succeeds on fresh K8s cluster
- All SLOs met under validated load
- 72-hour sustained load test passes
- Documentation reviewed by external technical writer
- Release candidate passes internal security audit
- v1.0.0 tagged and published

---

## Technical Debt Register

The following stub and placeholder implementations must be replaced before their respective phases can begin.

| Component | Current Implementation | Required Replacement | Target Phase |
|---|---|---|---|
| AST Parser | Regex-based pattern matching | tree-sitter runtime with incremental parsing | Phase 3 |
| Vector Database | In-memory HashMap with cosine similarity | Qdrant with hybrid search (dense + sparse) | Phase 3 |
| gRPC Client | Stub client returning hardcoded responses | Real gRPC connection to core service | Phase 1 |
| Pipeline Execution | `tokio::time::sleep` delay | Rootless Podman sandbox via K8s operator | Phase 2 |
| Git Operations | Skeleton implementations with no real I/O | gitoxide-backed clone/fetch/push | Phase 1 |
| Federation Engine | ActivityPub JSON generation (no network) | Production ForgeFed with HTTP delivery | Phase 4 |
| FUSE Filesystem | In-memory HashMap simulating filesystem | `fuser` kernel-mounted FUSE daemon | Phase 4 |
| Authentication | JWT generation and validation only | OIDC/SAML IdP integration + MFA | Phase 1 |
| Database | sqlx driver compiled, no migrations applied | CockroachDB with full migration suite | Phase 1 |
| Event Bus | Redis PubSub architecture defined, not wired | Real event propagation with WebSocket fan-out | Phase 1 |

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
| Prototype | v0.1.0 -> v0.1.1 | Complete | 5 crates, 773 tests, 85.6% coverage, hardened CI/CD, architecture proven |
| 1 -- Foundation Hardening | v0.2.0 | 1-3 | Real DB, SSH git, production auth, WebSocket events |
| 2 -- CI/CD and Storage | v0.4.0 | 3-6 | K8s operator, rootless execution, FastCDC dedup, SLSA |
| 3 -- AI Integration | v0.6.0 | 6-9 | Tree-sitter, Qdrant, local inference, PR review agent |
| 4 -- Federation and Scale | v0.8.0 | 9-12 | ForgeFed, DAG sync, FUSE, horizontal scaling |
| 5 -- Enterprise | v0.9.0 | 12-18 | SOC2, FIPS, ABAC policy engine, HSM |
| 6 -- Production | v1.0.0 | 18-24 | Helm, observability, 10K users, documentation |

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

*Last updated: 2026-05-31*
*Document owner: CivitForge core team*
*Latest audit: Phase 7.4 -- DashMap deadlock fix, clippy cleanup, version synchronization*
