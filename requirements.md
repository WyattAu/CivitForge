# Product Requirements Document (PRD): CivitForge
**Version:** 1.0 | **Target:** Enterprise, HFT, Tier-1 Tech, Defense

## 1. Executive Summary
CivitForge is a Rust-native software engineering platform designed for extreme-scale monorepos, regulated environments (HFT, defense), and ML/AI workflows. It provides parallelized Git processing, rootless Podman/Kubernetes CI/CD orchestration, firewall-gated AI agents, and a distributed multi-master architecture under a zero-trust security model.

## 2. Target Audience and Personas
*   **High-Frequency Trading (HFT) firms:** Require IP protection, air-gapped operations, rootless CI/CD to prevent container escapes, and management of large historical data (tick data).
*   **Google-scale technology companies:** Manage monorepos exceeding 1 TB. Require virtual file systems, distributed build caching (Bazel/Buck2), and extreme concurrency.
*   **Machine learning and AI labs:** Require native large-file support (tensor weights, datasets) and data gravity (bringing compute to storage rather than copying data to compute).

---

## 3. Functional Requirements

### 3.1. Version Control and Monorepo Engine
*   **REQ-1.1: Virtual File System (VFS):** The forge must support a native VFS client (EdenFS or Scalar protocol) allowing developers to mount a 5 TB+ monorepo locally in seconds. Files are fetched on-demand when opened or compiled.
*   **REQ-1.2: Native Rust Git backend:** Utilize `gitoxide` to parallelize object packing, delta compression, and indexing, bypassing the C-Git bottleneck.
*   **REQ-1.3: Alternative VCS support:** Provide pluggable backends for monorepo-friendly VCS formats (Sapling, Jujutsu).
*   **REQ-1.4: Build graph awareness:** Native integration with Bazel, Buck2, and Nix. The forge must parse target dependency graphs to optimize code browsing and CI trigger precision.

### 3.2. Big Data and Large File Management (LFS+)
*   **REQ-2.1: Block-level deduplication:** Replace standard Git LFS with a native content-defined chunking engine. If a 50 GB model weight is modified by 1 MB, only the changed 1 MB chunk is stored and transmitted.
*   **REQ-2.2: OCI artifact support:** Treat large binaries, datasets, and container images as native repository objects using OCI standards.
*   **REQ-2.3: Data gravity mounting:** Allow CI/CD runners to mount large datasets via K8s CSI without `git clone` or HTTP transfer of bulk data at build time.

### 3.3. CI/CD and Secure Runner Ecosystem
*   **REQ-3.1: Rootless Podman execution:** All containerized CI tasks must run via Podman in rootless mode, eliminating container-escape attack vectors.
*   **REQ-3.2: Native Kubernetes orchestration:** The forge operates as a K8s controller. CI pipelines translate directly into K8s Jobs.
*   **REQ-3.3: Hermetic and ephemeral environments:** Enforce network isolation for builds to guarantee reproducibility (required for HFT compliance and Bazel caching).
*   **REQ-3.4: Secure image provenance:** Built-in SBOM generation and cryptographic signing (Sigstore/Cosign) for every artifact and runner image.
*   **REQ-3.5: Ephemeral CDEs:** Browser-based IDEs backed by K8s Pods with pre-warmed VFS mounts and pre-installed dependencies.

### 3.4. Private AI and Agentic Workflows
*   **REQ-4.1: Air-gapped AI deployment:** The AI stack must be deployable entirely within the client's perimeter with zero dependency on external APIs.
*   **REQ-4.2: Codebase RAG:** The forge must parse the monorepo into an AST, vectorize it into a local vector database, and support accurate retrieval-augmented generation across 100M+ lines of code.
*   **REQ-4.3: Autonomous agents:** AI agents that review PRs for security flaws, generate unit tests, and resolve dependency updates within ephemeral Podman sandboxes.

### 3.5. Geo-Distributed High Availability (Federation)
*   **REQ-5.1: Multi-master replication:** Geo-distributed nodes (e.g., London, NY, Tokyo). Commits target the local edge node for zero-latency writes, then converge globally via asynchronous replication.
*   **REQ-5.2: Edge caching:** Read-heavy operations (clones, build artifacts, LFS blobs) must be cached at edge nodes collocated with developers.

---

## 4. Non-Functional Requirements

*   **Security:** Memory-safe core (Rust `#[forbid(unsafe_code)]` in cryptographic and authentication paths). Strict multi-tenant RBAC at directory and file granularity.
*   **Performance:**
    *   Web UI time-to-interactive (TTI) < 200 ms.
    *   Support concurrent PR operations from 10,000+ developers.
    *   Handle monorepositories up to 10 TB with tens of millions of commits.
*   **Compliance:** Out-of-the-box compliance reporting for SOC2, ISO 27001, and financial regulatory standards (FINRA/SEC algorithmic audit trails).

---

## 5. Technical Implementation and Architecture Stack

### 5.1. Application Tier
*   **Core backend API:** Rust (`Axum` for HTTP/gRPC routing, `Tokio` for async concurrency).
*   **Git interactions:** `gitoxide` (pure Rust Git implementation) for thread-safe, high-throughput repository operations.
*   **Frontend:** WebAssembly compiled from Rust (`Leptos` or `Dioxus`), or a React/Next.js frontend.

### 5.2. Data and Storage Tier
*   **Relational metadata:** CockroachDB or TiDB for geo-distributed, strongly consistent, multi-master SQL storage (issues, PR metadata, users).
*   **Object storage:** MinIO or Ceph (S3-compatible) for Git blob storage, LFS chunks, and build artifacts.
*   **Vector database:** Qdrant or Milvus for codebase vector embeddings and AI RAG retrieval.
*   **Caching and queues:** Redis or DragonflyDB for pub/sub messaging and CI/CD task queues.

### 5.3. Infrastructure and Orchestration Tier
*   **Runner engine:** Rust-based Kubernetes Operator.
*   **Container runtime:** Podman and `crun` (rootless, daemonless container execution).
*   **AI serving:** vLLM or Ollama integrated into the K8s cluster to serve open-weights models (e.g., Llama 3, DeepSeek Coder) internally.

---

## 6. Implementation Phasing

*   **Phase 1: Rust Core and VFS (Months 1-6)**
    *   Build the core Rust web server and authentication system.
    *   Implement Git HTTP/SSH handlers using `gitoxide`.
    *   Develop the VFS client and server for monorepo support.
*   **Phase 2: Podman CI/CD and Artifacts (Months 7-12)**
    *   Build the K8s runner operator.
    *   Implement rootless Podman execution for pipelines.
    *   Build the content-defined chunking and deduplication storage engine.
*   **Phase 3: AI Brain (Months 13-18)**
    *   Integrate AST parsing and the vector database.
    *   Deploy localized LLM serving.
    *   Launch AI PR reviews and codebase chat.
*   **Phase 4: Geo-Scale Enterprise (Months 19-24)**
    *   Implement CockroachDB multi-master replication.
    *   Build edge-caching for global deployments.
    *   Finalize SOC2 and financial compliance audit logging.
