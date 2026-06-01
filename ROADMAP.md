# CivitForge Production Roadmap

Strategic roadmap for CivitForge -- a federated, Rust-native software forge designed for extreme-scale monorepos, rootless CI/CD, and air-gapped AI. This document traces the path from the current v0.4.0-beta prototype to full production deployment.

This is a living document. Timelines are calibrated to a full-time core team of 3-5 engineers with periodic contributor sprints.

---

## Current State: v0.4.0-beta (Prototype Scaffolding Complete)

| Metric | Value |
|---|---|
| Version | 0.4.0-beta |
| Crates | 5 (civit-core, civit-runner, civit-brain, civit-vfs, civit-crypto) |
| Unit tests | 2179 passing, 0 ignored |
| Rust source files | 191 |
| Lines of code | 63,047 |
| Clippy warnings | 0 |
| `#![forbid(unsafe_code)]` | Enforced across all crates |
| MSRV | Rust 1.88 (edition 2024) |
| CI | Hardened (toolchain pinning, `--locked` on all build/test/clippy steps) |
| Pre-commit hooks | fmt + clippy -D warnings + test --locked |
| API endpoints | 20 routes (repos, users, orgs, auth, SSH keys, WebSocket, smart HTTP git) |

### Honest Capability Assessment

The codebase splits into three tiers. This assessment is based on a line-by-line audit of all 191 source files conducted 2026-06-01.

**Tier 1 -- Production-Ready (genuinely works end-to-end, ~22,500 LOC, 36% of codebase)**

| Component | Evidence | LOC (approx) |
|---|---|---|
| Database layer | sqlx PostgreSQL with 7+ tables, 34 methods, circuit breaker, 6 migration SQL files, real queries | 1,600 |
| Git operations | gitoxide (gix): bare repo init, commit walking, ref reading, smart HTTP (info/refs, upload-pack with BFS packfile, receive-pack) | 1,240 |
| SSH daemon | russh: Ed25519 host key, pubkey auth, rate limiting, git command routing (feature-gated `ssh-server`) | 1,310 |
| Auth: JWT | jsonwebtoken encode/decode, middleware extractor, role mapping | 125 |
| Auth: TOTP | HMAC-SHA1 per RFC 6238, otpauth:// URI generation, backup codes | 530 |
| Auth: RBAC | Role hierarchy, permission mapping, conditional policy checks | 310 |
| Auth: sessions | SHA-256 token hashing, refresh rotation, revocation, device tracking (in-memory) | 900 |
| API endpoints | 20 routes with real DB-backed handlers, input validation, auth middleware | 1,590 |
| Secret scanner | 15+ regex rules (AWS, GitHub, Google, Slack, Stripe, private keys, DB URLs) | 380 |
| License scanner | 50+ SPDX license database, Cargo.toml/package.json/go.mod detection, compliance checks | 540 |
| mTLS CA | rcgen X.509 CA creation, cert issuance with SANs, SHA-256 fingerprints | 195 |
| HMAC/Hash | SHA-256/512, HMAC-SHA256 using sha2/hmac crates | 390 |
| Policy engine | CAS-style allow/deny with Subject/Action/Resource/Condition/Effect | 430 |
| Event bus + WebSocket | DashMap pub/sub, bounded log, replay, Axum WebSocket broadcast | 1,010 |
| FastCDC | Content-defined chunking with Buzhash rolling hash, SHA-256 digests | 275 |
| OCI distribution | Real reqwest HTTP to OCI spec endpoints (manifest push/pull, blob upload) | 340 |
| Helm charts | Real Kubernetes manifests: Deployments, ConfigMaps, Secrets, HPA, NetworkPolicy, PVCs | 847 |
| Config | Environment-based AppConfig with validation | 680 |

**Tier 2 -- Structurally Complete, Partially Functional (~15,500 LOC, 25% of codebase)**

| Component | What Works | What Doesn't | LOC |
|---|---|---|---|
| OIDC auth | HTTP discovery to IdP, code exchange, userinfo fetch | `insecure_disable_signature_validation()` on id_token -- does NOT verify RS256 against JWKS | 630 |
| SAML auth | Valid AuthnRequest XML generation, response parsing | `is_valid_signature()` returns false unconditionally -- XML-DSig unimplemented | 370 |
| WebAuthn | Correct challenge/options JSON generation | `verify_registration()` and `verify_authentication()` ignore the authenticator response entirely | 530 |
| Podman service | Constructs real HTTP requests to Podman REST API v4.6.0 | Cannot reach Unix socket -- reqwest lacks Unix transport; falls back to localhost HTTP which Podman doesn't serve | 660 |
| Pipeline engine | Step sequencing, condition evaluation, failure handling, status tracking | `execute_step()` is `tokio::time::sleep(50ms)` + `Ok(())` -- never runs commands | 260 |
| CEL policy engine | Complete expression parser with full type system (Bool, Int, Double, String, List, Map, Timestamp) | No evaluator -- can parse but cannot execute `user.role == "admin"` | 1,140 |
| SLSA provenance | Spec-compliant SLSA v1 attestation generation, in-toto envelope | Self-generated (not from real build system), unsigned, no Sigstore integration | 530 |
| Diff analyzer | Parses unified diff format, checks SQL/command/path-traversal/SSRF patterns | StubReviewAgent returns hardcoded template findings | 910 |
| Telemetry | Prometheus exposition format, W3C trace context, in-process span processor | No OTLP export to Jaeger/Tempo/Grafana -- all in-process only | 1,700 |
| ForgeFed | ActivityPub data model, Ed25519 signature verification, idempotency tracking | No HTTP delivery to remote instances, in-memory inbox/outbox VecDeque | 2,080 |
| Vector DB | QdrantVectorDbAdapter with real HTTP calls to Qdrant REST API | Primary VectorDbClient is brute-force DashMap cosine similarity; RAG uses the DashMap version | 1,035 |
| LLM inference | InferenceService with real HTTP POST to OpenAI-compatible endpoints, SSE streaming | StubLlmProvider returns `[STUB]` strings; no model management or GPU allocation | 970 |

**Tier 3 -- Scaffolding/Stub (~25,000 LOC, 39% of codebase)**

| Component | Implementation | Honest Classification | LOC |
|---|---|---|---|
| AST parser | Hand-rolled tokenizer + bracket matcher named "TreeSitterParser" -- no tree-sitter crate, no grammar files, no .wasm | STUB (misleading name) | 2,220 |
| Embeddings | Deterministic byte-to-f32 conversion using DefaultHasher + positional encoding; model_name hardcoded to "text-embedding-ada-002" but no model is called | STUB | 200 |
| RAG pipeline | End-to-end wiring chains fake embeddings + DashMap vector DB + keyword-overlap mock retriever | STUB (end-to-end of nothing) | 815 |
| K8s operator | In-memory reconciler with DashMap state machine; kube crate used only for CustomResource derive macros | STUB | 490 |
| FUSE filesystem | In-memory HashMap<u64, Vec<u8>> VFS with libc error codes; no mount() syscall, no fusermount | STUB | 815 |
| Webhooks | Endpoint CRUD, event matching, delivery tracking, retry scheduling in Mutex<Vec<WebhookDelivery>>; no HTTP POST dispatcher | STUB | 360 |
| Notifications | Send/get/mark_read/preferences in Mutex<Vec<Notification>>; no SMTP, no Slack API, no email | STUB | 420 |
| HSM | Software-only ring fallback; PKCS#11 connect() bails; RSA/ECDSA verify always returns true; hardcoded HMAC key | STUB | 680 |
| Edge cache | DashMap + HashMap with real zstd compression and SHA-256 ETags; no Redis; `redis` crate declared but unused | STUB | 760 |
| Vuln scanner | Proper data models (Vulnerability, VulnScanReport); only implementation is StubVulnScanner returning hardcoded results | STUB | 310 |

### Dead Dependencies

| Crate | Declared In | Actually Used |
|---|---|---|
| `redis` | civit-core/Cargo.toml | Never imported -- edge cache is DashMap |
| `tonic` | civit-vfs/Cargo.toml | Never imported -- no gRPC server exists |
| `prost` | civit-vfs/Cargo.toml | Never imported -- no protobuf definitions |
| `kube` (runtime/client features) | civit-runner/Cargo.toml | Only `kube::CustomResource` derive in crds.rs -- no runtime cluster API |

### Technology Stack

- **Language:** Rust, edition 2024, zero unsafe code
- **HTTP framework:** Axum 0.8 (WebSocket and multipart support)
- **Git operations:** gitoxide (`gix` 0.70) -- C-free, pure Rust
- **Database driver:** sqlx 0.8 (PostgreSQL backend)
- **Cryptographic primitives:** ring 0.17, sha2 0.10, hmac 0.12
- **TLS/mTLS:** rcgen 0.13, x509-parser 0.17, rustls (via reqwest)
- **Auth:** jsonwebtoken 9 (JWT)
- **Kubernetes types:** kube-rs 0.98, k8s-openapi v1_30 (type definitions only)
- **Serialization:** serde 1, serde_json 1

---

## Dependency DAG and Phase Ordering

The codebase has natural dependency layers. Each layer must be solid before building atop it.

```
Layer 0: Platform (config, DB, health, telemetry, error types)
    |
Layer 1: Core Services (auth, RBAC, sessions, git, SSH, API, secrets/license scanning)
    |
Layer 2: Execution (Podman Unix socket, pipeline step execution, K8s operator cluster API)
    |
Layer 3: Intelligence (real AST via tree-sitter, real embeddings via candle-rs, real vector DB)
    |
Layer 4: Distribution (ForgeFed HTTP delivery, webhook dispatch, FUSE kernel mount, edge cache Redis)
    |
Layer 5: Enterprise (HSM PKCS#11, OIDC/SAML signature verification, SLSA Sigstore, FIPS, vuln scanning)
    |
Layer 6: Scale (horizontal scaling, geo-distributed edge, FUSE write-through, multi-master sync)
```

Phases below are ordered by this DAG. Each phase has dedicated workload estimates for engineering hours (single engineer, full-time equivalent).

---

## Phase 1: Execution Layer Hardening (Weeks 1-6, ~240 engineer-hours)

**Target version:** v0.5.0
**Goal:** Make the CI/CD execution layer functional. Pipelines must actually run commands inside containers. The K8s operator must talk to a real cluster.

**Dependency:** Builds on Layer 0 + Layer 1 (already production-ready).

### 1.1 Podman Unix Socket Transport (40 hours)

- [ ] Add `hyperlocal` (or equivalent Unix socket transport) to civit-runner dependencies
- [ ] Replace `reqwest::Client` with Unix socket client in `PodmanService::build_client()`
- [ ] Verify container create/start/wait/delete against real Podman daemon
- [ ] Add integration test: create alpine container, exec `echo hello`, verify stdout
- [ ] Add error mapping: Podman API error codes to domain error types
- [ ] Gate behind feature flag `podman-runtime` for CI environments without Podman

### 1.2 Pipeline Step Execution (60 hours)

- [ ] Replace `tokio::time::sleep(50ms)` in `execute_step()` with actual container execution
- [ ] Wire `PipelineEngine::execute_step()` to `PodmanService::run()`
- [ ] Pass step.image, step.commands, step.env to container creation
- [ ] Capture stdout/stderr from container and attach to StepStatus
- [ ] Implement step timeout enforcement (kill container on wall-clock expiry)
- [ ] Implement artifact capture (copy files from container to host)
- [ ] Add hermetic build flags: read-only rootfs, network isolation, no privileged
- [ ] Integration test: pipeline with 3 steps, verify execution order and output

### 1.3 Kubernetes Operator Runtime (80 hours)

- [ ] Replace in-memory DashMap reconciler with `kube::runtime::Controller`
- [ ] Implement `kube::runtime::Watcher<Api<PipelineRun>>` for CRD observation
- [ ] Wire reconciler to real K8s API: create Pod from PipelineRun spec, watch Pod status
- [ ] Implement `kube::runtime::Reconciler` trait for PipelineRun
- [ ] Add leader election via `kube::leader::LeaderElector`
- [ ] Implement node affinity from PipelineRun tolerations
- [ ] Status subresource updates: map Pod phase to PipelineRun status
- [ ] Integration test against kind/minikube: submit PipelineRun, verify Pod created and status updated

### 1.4 CEL Expression Evaluator (60 hours)

- [ ] Implement `CelContext` with variable bindings (string, int, float, bool, list, map)
- [ ] Implement evaluation for: Comparison (==, !=, <, >, <=, >=), Logical (&&, ||, !), Membership (in), Ternary
- [ ] Implement function calls: has(), startsWith(), endsWith(), matches(), size()
- [ ] Wire evaluator to policy engine for ABAC conditions
- [ ] Test against CEL conformance test suite subset (50+ cases)
- [ ] Update ABAC geofencing policy to use evaluated expressions instead of string matching

### Exit Criteria

- Pipeline with 3 steps executes in real Podman container, captures output
- K8s operator reconciles PipelineRun CRD to Pod creation on kind cluster
- CEL evaluator passes 50+ conformance tests
- All 2,179 existing tests continue to pass
- No clippy warnings, forbid(unsafe_code) maintained
- Dead `tonic`/`prost` dependencies removed from civit-vfs/Cargo.toml

---

## Phase 2: Intelligence Layer (Weeks 7-16, ~480 engineer-hours)

**Target version:** v0.6.0
**Goal:** Replace all stub AI components with real implementations. AST parsing, embeddings, vector search, and LLM inference must produce meaningful output.

**Dependency:** Builds on Layer 0 + Layer 1 (already production-ready). Independent of Phase 1 (can parallelize).

### 2.1 Real AST Parser via tree-sitter (120 hours)

- [ ] Remove `TreeSitterParser` name from hand-rolled parser; rename to `BracketParser` honestly
- [ ] Add `tree-sitter` and `tree-sitter-*` grammar crates (Rust, Python, TypeScript, Go, C++, Java, Kotlin, Swift, Ruby, Bash, YAML, TOML, JSON, Markdown)
- [ ] Implement `TreeSitterAstParser` using `tree-sitter::Parser::set_language()` and `tree-sitter::Parser::parse()`
- [ ] Extract function definitions, class definitions, import statements, call sites from CST nodes
- [ ] Build per-function metadata: complexity (cyclomatic), call graph edges, parameter types
- [ ] Implement incremental parsing: on push events, parse only changed files
- [ ] Persist AST nodes in structured JSON for downstream indexing
- [ ] Replace `RegexAstParser` usage with `TreeSitterAstParser` throughout
- [ ] Integration test: parse 1M-line monorepo in <10s
- [ ] **Architectural note:** tree-sitter C bindings require `unsafe` FFI. Options: (a) gate behind `#[cfg(feature = "tree-sitter")]` with `extern "C"` blocks that satisfy forbid(unsafe_code) via crate-level attribute in civit-brain only, or (b) use a pure-Rust parser generator. Decision required via ADR.

### 2.2 Real Embeddings via candle-rs (100 hours)

- [ ] Replace `EmbeddingWorker::embed_text()` (DefaultHasher byte-to-float) with candle-rs inference
- [ ] Add `candle-core`, `candle-nn`, `candle-transformers` dependencies
- [ ] Load a small embedding model (e.g., all-MiniLM-L6-v2, 384 dimensions) from local Safetensors
- [ ] Implement batched inference: embed multiple text chunks in parallel on CPU (or GPU if available)
- [ ] Cache embedding results in VectorDbClient with content-addressed keys
- [ ] Remove hardcoded "text-embedding-ada-002" model_name
- [ ] Integration test: embed 1000 code snippets, verify cosine similarity of near-duplicates >0.9

### 2.3 Vector DB Production Mode (60 hours)

- [ ] Wire RAG pipeline and all consumers to use `QdrantVectorDbAdapter` instead of `VectorDbClient`
- [ ] Implement collection management: create/delete per-repository collections
- [ ] Add access-controlled filtering: users can only search collections for repos they have read access to
- [ ] Implement hybrid search: dense (Qdrant HNSW) + sparse (BM25) with score fusion
- [ ] Add indexing pipeline: on push event, embed changed AST nodes, upsert to Qdrant
- [ ] Integration test: index a repo, search for "database write path", verify relevant results
- [ ] Remove DashMap `VectorDbClient` or demote to test-only fixture

### 2.4 LLM Inference Integration (120 hours)

- [ ] Remove `StubLlmProvider` from production code paths
- [ ] Wire `InferenceService` (real OpenAI-compatible HTTP client) as default provider
- [ ] Implement model management: upload, version, serve models from local storage
- [ ] Add inference endpoint to API: `POST /api/v1/ai/inference` with streaming response
- [ ] Implement token budget enforcement per repository and per user
- [ ] Add fallback chain: local vLLM server -> candle-rs CPU inference -> error
- [ ] Air-gap validation: all inference within cluster, no external API calls
- [ ] Integration test: deploy vLLM on kind cluster, submit inference request, verify response

### 2.5 Automated PR Review (80 hours)

- [ ] Remove `StubReviewAgent` from production code paths
- [ ] Wire `DiffAnalyzer` (real static analysis) as the primary review engine
- [ ] Add LLM-enhanced review: feed diff + AST context to inference service for natural language findings
- [ ] Implement severity classification: Critical (security), High (correctness), Medium (performance), Low (style)
- [ ] Add inline fix suggestions via AST manipulation (not LLM text generation)
- [ ] Implement review rules per repository (configurable severity thresholds, skip patterns)
- [ ] Human-in-the-loop: review comments posted as "suggested" status, require human approval
- [ ] Integration test: create PR with SQL injection pattern, verify review comment posted

### Exit Criteria

- tree-sitter parses changed files within 10s of push for a 1M-line monorepo
- Embeddings produce semantically meaningful vectors (verified by cosine similarity tests)
- Vector search returns relevant results for semantic code queries
- LLM inference returns non-stub responses at >=10 tokens/second on single GPU
- PR review agent posts actionable comments (not template findings)
- Zero external network dependencies for AI pipeline (air-gap verified)
- All existing tests pass, no clippy warnings

---

## Phase 3: Distribution and Federation (Weeks 17-24, ~320 engineer-hours)

**Target version:** v0.7.0
**Goal:** Make federated delivery, webhook dispatch, and notification channels functional. Components that generate output must actually deliver it.

**Dependency:** Builds on Layer 0 + Layer 1. Independent of Phases 1 and 2.

### 3.1 ForgeFed HTTP Delivery (100 hours)

- [ ] Implement outbound HTTP POST to remote ActivityPub inboxes (signed with Ed25519 HTTP Signatures)
- [ ] Implement WebFinger lookup via HTTP GET to remote instances (remove DashMap cache-only approach)
- [ ] Implement inbox processing loop: poll remote inboxes, verify signatures, process activities
- [ ] Add exponential backoff with jitter for failed deliveries
- [ ] Implement actor resolution: fetch remote actor profiles, cache with TTL
- [ ] Integration test: two CivitForge instances, create federated PR, verify delivery

### 3.2 Webhook Delivery (60 hours)

- [ ] Implement `WebhookDispatcher`: async loop that picks up Pending deliveries and POSTs to endpoint URLs
- [ ] Use `reqwest::Client` with configurable timeouts and TLS settings
- [ ] Implement retry with exponential backoff (use existing retry scheduling logic)
- [ ] Add delivery signature: HMAC-SHA256 of payload with per-endpoint secret
- [ ] Add delivery logging to audit trail
- [ ] Integration test: register webhook, trigger event, verify HTTP POST received

### 3.3 Notification Channels (80 hours)

- [ ] Implement SMTP email delivery via `lettre` crate
- [ ] Implement Slack/Mattermost webhook delivery via HTTP POST
- [ ] Implement in-app notification delivery (already works via EventBus)
- [ ] Wire `NotificationService::send()` to branch on channel type
- [ ] Apply `NotificationPreferences` filtering per user
- [ ] Add rate limiting per user per channel
- [ ] Integration test: send notification via email, verify SMTP delivery

### 3.4 Edge Cache with Redis (80 hours)

- [ ] Replace DashMap storage in `EdgeCacheManager` with `redis` client (the dependency already exists)
- [ ] Implement Redis-backed cache entries with TTL
- [ ] Implement Redis Pub/Sub for cache invalidation broadcast
- [ ] Keep zstd compression and SHA-256 ETags (these are real)
- [ ] Add cache warming on repository push events
- [ ] Integration test: put entry via Redis, retrieve from different node, verify decompression

### Exit Criteria

- Federated PR creation works between two CivitForge instances
- Webhook delivery verified with HMAC signature
- Notification delivery works for email and Slack channels
- Edge cache uses Redis backend, cache invalidation broadcasts across nodes
- All existing tests pass

---

## Phase 4: Filesystem and Scale (Weeks 25-34, ~400 engineer-hours)

**Target version:** v0.8.0
**Goal:** Kernel-mounted FUSE filesystem, horizontal scaling, and multi-node operation.

**Dependency:** Builds on Phases 1-3. FUSE depends on gRPC transport (currently dead dependency).

### 4.1 gRPC Transport Layer (60 hours)

- [ ] Define protobuf service definitions for VFS remote operations (read_block, write_block, stat, readdir)
- [ ] Generate Rust server/client from protobuf using tonic/prost (dependencies already declared)
- [ ] Implement tonic server in civit-vfs
- [ ] Implement tonic client in civit-core for VFS access
- [ ] Add mTLS between gRPC client and server
- [ ] Integration test: gRPC client reads block from server

### 4.2 FUSE Kernel Mount (120 hours)

- [ ] Add `fuser` (libfuse3 Rust bindings) dependency to civit-vfs
- [ ] Replace in-memory HashMap VFS with `fuser::FuseExt` trait implementation
- [ ] Wire `lookup`, `getattr`, `read`, `readdir`, `create`, `unlink` to gRPC remote block provider
- [ ] Implement on-demand block fetching with local caching
- [ ] Support sparse checkout: only requested subdirectories materialized locally
- [ ] Implement write-through: local writes propagated to server via gRPC
- [ ] Add graceful unmount and cleanup
- [ ] Integration test: mount repository, read file, verify content matches remote
- [ ] **Architectural note:** `fuser` uses `unsafe` internally. Same ADR decision as tree-sitter applies.

### 4.3 Multi-Master DAG Sync (100 hours)

- [ ] Implement DAG-based replication for Git object and metadata
- [ ] Conflict resolution: last-write-wins for metadata, Git merge for refs
- [ ] Implement incremental sync with checkpointing (resume after partition)
- [ ] Add delta compression for inter-node transfers
- [ ] Implement partition tolerance: serve reads during network splits, reconcile on heal
- [ ] Integration test: 3-node cluster, push to node 1, verify propagation to nodes 2 and 3

### 4.4 Horizontal Scaling (120 hours)

- [ ] Externalize all in-memory state to Redis: sessions, event bus, presence tracking
- [ ] Implement API layer statelessness verification (no local mutable state)
- [ ] Add Git engine sharding by repository prefix
- [ ] Implement event bus partitioning by topic (repo-level isolation)
- [ ] Configure L4 load balancer for git protocol, L7 for API
- [ ] Implement autoscaling policies for API pods, runner pods, inference pods
- [ ] Integration test: 3 API pods, verify session survives pod termination

### Exit Criteria

- FUSE mount provides <500ms cold read for 100GB repository
- Multi-master sync converges within 5s of partition healing
- API layer verified stateless: session survives pod restart
- 3-node cluster operates with zero data loss under network partition
- All existing tests pass

---

## Phase 5: Enterprise Hardening (Weeks 35-44, ~320 engineer-hours)

**Target version:** v0.9.0
**Goal:** Fix all partial implementations in the auth and crypto stacks. Add real HSM support, vulnerability scanning, and compliance artifacts.

**Dependency:** Builds on Layers 0-4. Independent of Phases 2 and 4 for most items.

### 5.1 OIDC Signature Verification (40 hours)

- [ ] Remove `insecure_disable_signature_validation()` from OIDC id_token validation
- [ ] Implement JWKS fetching from IdP discovery document
- [ ] Verify RS256/ES256 signature of id_token against JWKS public keys
- [ ] Cache JWKS with rotation detection
- [ ] Add nonce validation against session nonce
- [ ] Integration test: Keycloak login flow with signature verification

### 5.2 SAML Signature Verification (60 hours)

- [ ] Replace `is_valid_signature()` (always-false) with real XML-DSIG validation
- [ ] Add `xmldsig` or `xmlsec` crate dependency
- [ ] Extract Signature element, canonicalize, verify against X.509 certificate
- [ ] Validate assertions: conditions (NotBefore/NotOnOrAfter), audience, recipient
- [ ] Integration test: Okta SAML login with signature verification

### 5.3 WebAuthn Cryptographic Verification (60 hours)

- [ ] Add `cbor` crate dependency for CBOR parsing of authenticator data
- [ ] Implement `verify_registration()`: parse attestation data, verify client data, check ceremony
- [ ] Implement `verify_authentication()`: parse authenticator data, verify signature, check challenge
- [ ] Support ES-256 and RS256 attestation formats
- [ ] Integration test: register and authenticate with YubiKey via WebAuthn

### 5.4 HSM PKCS#11 Integration (80 hours)

- [ ] Replace software-only ring fallback with real PKCS#11 library loading
- [ ] Add `pkcs11` or `cryptoki` crate dependency
- [ ] Implement `C_Initialize`, `C_OpenSession`, `C_FindObjectsInit`, `C_Sign` calls
- [ ] Store signing keys in HSM, remove key material from process memory
- [ ] Implement HSM-backed CA for internal TLS certificates
- [ ] Keep software fallback (without `Ok(true)` always-verify) with audit alert
- [ ] Integration test: sign artifact with HSM key, verify signature

### 5.5 Vulnerability Scanning (40 hours)

- [ ] Replace `StubVulnScanner` with OSV API client
- [ ] Query OSV database by dependency name + version
- [ ] Parse and classify CVEs by severity (Critical/High/Medium/Low)
- [ ] Generate VulnScanReport with remediation suggestions
- [ ] Add to CI pipeline: fail build on Critical CVEs
- [ ] Integration test: scan project dependencies, verify known CVE detected

### 5.6 SLSA Sigstore Integration (40 hours)

- [ ] Replace empty signatures array in SLSA in-toto envelope with real Sigstore signing
- [ ] Add `sigstore` crate dependency
- [ ] Sign provenance attestations with Sigstore Fulcio + Rekor transparency log
- [ ] Verify provenance against Sigstore during deployment gates
- [ ] Integration test: generate provenance, sign, verify via Rekor

### Exit Criteria

- OIDC login verifies RS256 signature against JWKS
- SAML login verifies XML-DSIG
- WebAuthn registration and authentication verify cryptographic signatures
- HSM signs artifacts without key material leaving the module
- Vuln scanner detects known CVEs from OSV database
- SLSA provenance signed with Sigstore
- All existing tests pass

---

## Phase 6: Production Release (Weeks 45-54, ~320 engineer-hours)

**Target version:** v1.0.0
**Goal:** General availability. Performance validation, documentation, scale testing, and release engineering.

**Dependency:** All prior phases must be substantially complete.

### 6.1 OTLP Telemetry Export (60 hours)

- [ ] Replace in-process span processor with `opentelemetry-otlp` gRPC exporter
- [ ] Export traces to Jaeger or Tempo
- [ ] Export metrics to Prometheus via OTLP or direct scrape
- [ ] Implement distributed trace correlation: HTTP -> event bus -> sandbox
- [ ] Add Grafana dashboards for all components
- [ ] Configure alert rules for SLO violations (error budget burn rate)

### 6.2 Performance Optimization (80 hours)

- [ ] API p99 latency: <200ms for read operations (profiling + query optimization)
- [ ] Git clone (1M-line repo): <10s over LAN (packfile optimization)
- [ ] Pipeline scheduling: <2s from trigger to sandbox start
- [ ] Memory profiling: <512MB RSS per API pod under normal load
- [ ] Database query optimization: all queries <50ms at P99
- [ ] Add connection pooling tuning, prepared statements, query plan analysis

### 6.3 Scale Validation (80 hours)

- [ ] Load testing: 10,000+ concurrent users (HTTP + gRPC mixed) using wrk/ghz
- [ ] Repository scale: 1,000+ repos with 100M+ total lines
- [ ] CI throughput: 500+ concurrent pipeline runs
- [ ] Federation: 5+ nodes with 100ms inter-node latency simulation
- [ ] Sustained load test: 72-hour continuous operation with <1% error rate
- [ ] Document load test results in `.reports/`

### 6.4 Documentation and Release (100 hours)

- [ ] Operator guide: installation, configuration, upgrade, backup/restore
- [ ] API reference: OpenAPI 3.1 specification for all REST endpoints
- [ ] Architecture decision records for all major design choices
- [ ] Contributing guide: development setup, coding standards, PR process
- [ ] Security disclosure policy and vulnerability response SLA
- [ ] Generate migration guide from v0.4.0-beta to v1.0.0
- [ ] External technical writer review

### Exit Criteria

- Helm install + upgrade succeeds on fresh K8s cluster
- All SLOs met under validated load
- 72-hour sustained load test passes with <1% error rate
- Documentation reviewed by external technical writer
- Release candidate passes internal security audit
- v1.0.0 tagged and published

---

## Workload Summary

| Phase | Duration | Engineer-Hours | Key Deliverable |
|---|---|---|---|
| 1 -- Execution Layer | 6 weeks | 240 | Podman Unix socket, pipeline execution, K8s operator runtime, CEL evaluator |
| 2 -- Intelligence | 10 weeks | 480 | tree-sitter AST, candle-rs embeddings, Qdrant vector DB, LLM inference, PR review |
| 3 -- Distribution | 8 weeks | 320 | ForgeFed HTTP, webhook dispatch, notification channels, Redis edge cache |
| 4 -- Filesystem + Scale | 10 weeks | 400 | gRPC transport, FUSE mount, DAG sync, horizontal scaling |
| 5 -- Enterprise | 10 weeks | 320 | OIDC/SAML/WebAuthn verification, HSM, vuln scanning, SLSA Sigstore |
| 6 -- Production | 10 weeks | 320 | OTLP export, performance tuning, scale validation, documentation |
| **Total** | **54 weeks** | **2,080** | **v1.0.0 general availability** |

Phases 1, 2, and 3 can partially overlap (Phase 1 and Phase 2 are independent; Phase 3 depends only on Layer 1). With parallelization across 3-5 engineers, wall-clock time can be reduced to approximately 36-40 weeks.

---

## Technical Debt Register

| Component | Current State | Required Fix | Phase | Hours |
|---|---|---|---|---|
| Podman transport | HTTP client cannot reach Unix socket | Add hyperlocal or Unix socket transport | 1.1 | 40 |
| Pipeline execution | `tokio::sleep(50ms)` stub | Wire to Podman container execution | 1.2 | 60 |
| K8s operator | In-memory DashMap reconciler | Replace with kube-rs runtime Controller | 1.3 | 80 |
| CEL evaluator | Parser only, no execution engine | Implement expression evaluation | 1.4 | 60 |
| AST parser | Hand-rolled tokenizer named "tree-sitter" | Real tree-sitter or pure-Rust parser | 2.1 | 120 |
| Embeddings | DefaultHasher byte-to-f32 | candle-rs model inference | 2.2 | 100 |
| Vector DB | DashMap brute-force cosine similarity | Wire to Qdrant adapter | 2.3 | 60 |
| LLM inference | StubLlmProvider returns "[STUB]" | Wire to real inference service | 2.4 | 120 |
| PR review | StubReviewAgent template comments | Wire DiffAnalyzer + LLM-enhanced review | 2.5 | 80 |
| ForgeFed | In-memory VecDeque inbox/outbox | HTTP delivery + WebFinger lookup | 3.1 | 100 |
| Webhooks | No HTTP POST dispatcher | Async delivery loop | 3.2 | 60 |
| Notifications | In-memory Vec, no channel dispatch | SMTP + Slack + preference filtering | 3.3 | 80 |
| Edge cache | DashMap, redis crate unused | Wire to Redis backend | 3.4 | 80 |
| gRPC | tonic/prost declared but unused | Implement VFS gRPC service | 4.1 | 60 |
| FUSE | In-memory HashMap VFS | fuser kernel mount + gRPC backend | 4.2 | 120 |
| DAG sync | No implementation | Multi-master replication | 4.3 | 100 |
| Scaling | In-memory state everywhere | Externalize to Redis, stateless API | 4.4 | 120 |
| OIDC | Signature validation disabled | JWKS fetch + RS256 verify | 5.1 | 40 |
| SAML | Signature always returns false | XML-DSIG validation | 5.2 | 60 |
| WebAuthn | Response ignored | CBOR parsing + crypto verification | 5.3 | 60 |
| HSM | Software-only, verify always true | PKCS#11 library loading | 5.4 | 80 |
| Vuln scanner | StubVulnScanner hardcoded | OSV API client | 5.5 | 40 |
| SLSA | Self-generated, unsigned | Sigstore signing | 5.6 | 40 |
| Telemetry | In-process only | OTLP gRPC export | 6.1 | 60 |
| Dead deps | redis, tonic, prost unused | Remove or actually use | Various | 8 |

---

## Risk Matrix

| ID | Risk | Probability | Impact | Mitigation Strategy |
|---|---|---|---|---|
| R1 | tree-sitter requires `unsafe` FFI, conflicting with `#![forbid(unsafe_code)]` | High | High | ADR decision: (a) crate-level allow for civit-brain only, or (b) pure-Rust parser. Evaluated in Phase 2 week 1. |
| R2 | candle-rs model loading requires significant GPU memory for usable inference latency | Medium | High | Support CPU-only with quantization (INT4/INT8). Document minimum hardware requirements. |
| R3 | fuser (libfuse3) Linux kernel compatibility across distributions | Medium | Medium | Target mainline kernel 6.x LTS. Test Ubuntu, RHEL, Arch. HTTP VFS fallback for unsupported platforms. |
| R4 | OIDC/SAML signature verification requires deep crypto library knowledge | Medium | Medium | Use established crates: `openidconnect` for OIDC, `xmldsig` for SAML. Avoid hand-rolled crypto. |
| R5 | Multi-master DAG sync encounters unresolvable conflicts in edge cases | Low | Critical | Implement manual resolution interface. Extensive chaos engineering testing. |
| R6 | Podman Unix socket transport crate (`hyperlocal`) unmaintained | Medium | Medium | Evaluate alternatives: `tower-http` Unix connector, custom hyper service. Fallback: Podman TCP socket. |
| R7 | OSV API rate limiting affects CI pipeline performance | Low | Low | Cache OSV results locally. Batch queries. Implement fallback to offline advisory database. |

---

## Non-Goals (Explicitly Out of Scope)

These items are intentionally excluded from the current roadmap. They may be revisited in post-v1.0 planning.

- **Web UI implementation:** This roadmap covers the backend platform only. A frontend is a separate effort.
- **Alternative VCS backends (Jujutsu, Sapling):** Git-only for v1.0. Alternative backends are evaluated post-launch.
- **Cloud-native managed service:** CivitForge targets self-hosted and air-gapped deployment only.
- **Windows support:** Linux-only for v1.0. Windows may be evaluated based on demand.
- **Mobile clients:** No mobile application planned.
- **Marketplace / plugin system:** Deferred to post-v1.0.
- **Email/password authentication:** OIDC/SAML only. Email/password is not planned.
- **FIPS 140-2 certification:** ring in FIPS mode as interim measure. Formal certification is a separate compliance project.

---

*Milestone summary removed. All previous milestones claimed "Complete" status for components that are scaffolding. See Honest Capability Assessment above.*

*Last updated: 2026-06-01*
*Document owner: CivitForge core team*
*Latest audit: Phase 17 -- Comprehensive audit (2,179 tests, 191 files, 63K+ LOC) with per-module classification*
