# Technical Requirements Document (TRD): CivitForge
**Version:** 1.0.0 | **Language:** Rust | **Target Scale:** Enterprise / VFS / LFS+

## 1. System Architecture Overview
CivitForge is structured as a federated modular monolith for its core API, with an event-driven microservice architecture for runners and AI components.

The system partitions into four logical domains:
1.  **CivitCore:** Git/HTTP API layer (Rust).
2.  **CivitData:** Distributed storage and state layer (CockroachDB, S3, Qdrant).
3.  **CivitRunner:** K8s/Podman orchestration layer (Rust).
4.  **CivitBrain:** Localized AI/LLM inference and RAG indexing layer.

---

## 2. CivitCore: Application and Git Layer

### 2.1. Web Framework and Async Runtime
*   **Runtime:** `tokio` for high-throughput asynchronous I/O.
*   **HTTP/API framework:** `axum` (deep tokio integration, high-performance routing).
*   **Memory safety constraint:** Global `#![forbid(unsafe_code)]` in all business logic and authentication modules. C bindings are restricted to OS-level primitives exclusively.

### 2.2. High-Performance Git Backend
*   **Library:** `gitoxide` (pure Rust Git implementation). `libgit2` is explicitly excluded to avoid C-binding memory-leak vectors and threading bottlenecks.
*   **VFS protocol:**
    *   RPC server supporting GVFS/Scalar and EdenFS protocols.
    *   Git objects fetched on-demand via gRPC.
    *   Local FUSE daemon (`fuser` crate) for edge-client mounting.
*   **Monorepo optimization:** Packfile generation and delta compression parallelized via `rayon`.

### 2.3. Federation and Synchronization Engine
*   **Protocol:** ActivityPub extended via ForgeFed standards for metadata (issues, PRs, comments).
*   **Replication algorithm:** Inter-node Git synchronization uses a custom DAG state-sync protocol. Nodes broadcast Merkle-root updates via secure WebSockets; missing objects are negotiated and transferred asynchronously to avoid blocking pushes.
*   **Node identity:** Each federated node holds an X.509 certificate. All inter-node communication is enforced via mTLS.

---

## 3. CivitData: Storage and State Layer

### 3.1. Relational Metadata
*   **Engine:** CockroachDB.
*   **Rationale:** PostgreSQL wire compatibility with native multi-master geo-replication. Sub-millisecond local reads for UI rendering at each geographic site, with globally consistent transaction ordering.

### 3.2. Blob and Object Storage (LFS+ Engine)
*   **Engine:** S3-compatible object store (MinIO for on-prem, AWS S3 for cloud).
*   **Chunking algorithm:** FastCDC (Fast Content-Defined Chunking) implemented in Rust replaces standard Git-LFS.
    *   *Mechanism:* Large datasets and tensor weights are split into deduplicated chunks (4 MB -- 64 MB range).
    *   *Storage:* Only unique chunks are persisted in S3. A Rust-based manifest map for file reconstruction is stored in Git.

### 3.3. Vector Database (AI Memory)
*   **Engine:** Qdrant (Rust-native, distributed deployment support).
*   **Data stored:** Codebase AST embeddings, documentation vectors, and historical PR resolution context.

---

## 4. CivitRunner: CI/CD Orchestration

### 4.1. Orchestrator Component
*   **Implementation:** Kubernetes Operator written in Rust using `kube-rs`.
*   **Trigger mechanism:** Event-driven via Redis/DragonflyDB PubSub. CivitCore publishes events to Redis on commit; the Operator consumes events and schedules K8s Pods.

### 4.2. Secure Execution Sandbox (Podman)
*   **Container runtime:** `crun` (low-memory OCI runtime).
*   **Isolation:** K8s Pods mount builder images and execute tasks via rootless Podman.
    *   User namespaces map container UID 0 to an unprivileged host user.
    *   Strict Seccomp profiles restrict the system call surface.
*   **Data gravity mounts:** LFS chunks and ML datasets are mounted into the Podman container via K8s CSI, bypassing network-based data transfer.

### 4.3. Supply Chain Security (SLSA Level 4)
*   **Cryptographic signing:** Native Sigstore/Cosign integration.
*   On build completion, the runner generates an SBOM (SPDX/CycloneDX via Rust crates) and signs both the image and the SBOM using an ephemeral key bound to the runner's OIDC identity.

---

## 5. CivitBrain: Localized AI Integration

### 5.1. Codebase Parsing and RAG
*   **AST parser:** `tree-sitter` (Rust bindings) to parse the codebase into an AST on every push.
*   **Embedding generator:** An async Rust worker batch-processes AST nodes into semantic text chunks, sends them to a local embedding model (e.g., `nomic-embed-text`), and persists the vectors in Qdrant.

### 5.2. Inference Serving
*   **Server:** vLLM deployed as a K8s service.
*   **Model tiering:**
    *   *Lightweight tasks* (commit message generation): `Llama-3-8B-Instruct`.
    *   *Heavy tasks* (code review, test generation): `DeepSeek-Coder-33B` (open-weights, locally hosted).
*   **Agent sandbox:** When the AI agent writes code or tests a fix, it requests a CivitRunner Podman sandbox, executes the code, captures stdout/stderr, and iteratively refines its pull request.

---

## 6. API and Interfaces

*   **GraphQL / REST:** User and frontend interactions (issues, PRs, admin controls).
*   **gRPC:** Internal microservice communication (CivitCore <-> CivitRunner <-> CivitBrain).
*   **SSH:** Custom SSH server (`russh` crate) for Git over SSH with Ed25519 key support and FIDO2 hardware token forwarding.

---

## 7. Security Architecture and Compliance

*   **Authentication:** Zero-trust model. Identity via SAML/OIDC (Okta, Keycloak). Native WebAuthn/Passkey support.
*   **Encryption at rest:** All Git objects and DB partitions encrypted via AES-256-GCM. Keys managed by an external KMS (HashiCorp Vault).
*   **Audit logging:** WORM-compliant audit trails for every API call, Git clone, and UI access, exported to Splunk/Datadog via OpenTelemetry.
*   **RBAC and ABAC:** Attribute-Based Access Control supporting policies such as restricting repository clone access by team membership and source IP geofence.

---

## 8. Deployment Model

*   CivitForge is distributed via Helm Charts.
*   **Air-gapped support:** All container images, LLM weights, and Rust binaries are packaged into tarballs for offline transfer into classified or air-gapped environments.
*   **Hardware requirements (per enterprise node):**
    *   *App servers (Rust):* 64+ cores for parallel Git packing, 128 GB RAM.
    *   *AI servers:* Minimum 2x NVIDIA A100 or H100 GPUs for vLLM inference.

---
