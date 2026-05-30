# CivitForge Product Roadmap

Strategic milestones for the CivitForge federated, Rust-native engineering platform. This roadmap covers four phases spanning approximately 24 months.

*Note: This is a living document. Timelines and priorities are subject to change based on community feedback and enterprise partner requirements. To propose a feature, open an issue with the `enhancement` label.*

---

## Current Status: **Phase 1 (Active Development)**
Building the foundational Rust core (`CivitCore`) and the Virtual File System (VFS) client.

---

## Phase 1: Rust Foundation and VFS (v0.1.x - v0.3.x)
**Goal:** Establish a memory-safe, high-throughput Git backend and lay the groundwork for terabyte-scale monorepo support.

- [x] **Project scaffolding:** Cargo workspace setup, Axum HTTP boilerplate, repository CI/CD.
- [ ] **Gitoxide integration:** Implement core Git read/write operations (clone, fetch, push) without C bindings.
- [ ] **SSH and auth layer:** Custom SSH server (`russh`) with Ed25519 support and JWT/OIDC authentication.
- [ ] **Relational schema:** Initial CockroachDB migrations for users, repositories, and organizations.
- [ ] **Web UI v1:** Wasm or Next.js frontend with repository browsing, commit history, and issue tracking.
- [ ] **VFS prototype:** Rust `fuser` daemon for local mounting of Git trees without full working-directory checkout.

## Phase 2: Secure Execution and Big Data (v0.4.x - v0.6.x)
**Goal:** Introduce the `CivitRunner` K8s orchestration layer and replace Git-LFS with block-level deduplicating storage.

- [ ] **K8s Runner Operator:** Rust-based Kubernetes Operator consuming CI/CD events via Redis PubSub.
- [ ] **Rootless Podman sandboxes:** `crun` and rootless Podman as the default execution environment for all pipeline tasks.
- [ ] **LFS+ and FastCDC:** Content-defined chunking for deduplicating large datasets (ML tensors, tick data).
- [ ] **OCI registry:** Native storage of Docker images and OCI artifacts within the forge.
- [ ] **Hermetic build enforcement:** Network isolation for runners to guarantee reproducible builds (Bazel/Buck2 compatibility).
- [ ] **SLSA provenance:** Automatic SBOM generation and Cosign image signing for all CI/CD artifacts.

## Phase 3: CivitBrain and Local AI (v0.7.x - v0.9.x)
**Goal:** Introduce localized, perimeter-gated AI agents with full codebase context.

- [ ] **AST parsing engine:** `tree-sitter` integration for continuous AST generation on every commit.
- [ ] **Vector database integration:** Qdrant deployment and background worker for embedding AST nodes and documentation.
- [ ] **vLLM / Ollama serving:** K8s manifests for serving open-weights models (e.g., Llama-3, DeepSeek Coder) within the cluster.
- [ ] **Codebase RAG:** Chat interface for architectural queries across 100M+ lines of code with context-aware retrieval.
- [ ] **Agentic PR reviews:** Autonomous AI agents for code review, vulnerability detection, and inline fix suggestions.

## Phase 4: Geo-Scale Enterprise and Federation (v1.0.0)
**Goal:** Multi-master high availability, regulatory compliance, and ForgeFed interoperability.

- [ ] **Multi-master replication:** CockroachDB global deployment topology with < 50 ms read latency globally.
- [ ] **DAG state-sync:** Directed acyclic graph synchronization protocol for asynchronous Git object replication across nodes.
- [ ] **ForgeFed implementation:** ActivityPub/ForgeFed support for cross-instance issues, stars, and PRs.
- [ ] **Edge caching engine:** Intelligent caching of Git objects and LFS+ chunks at edge nodes.
- [ ] **Enterprise compliance:** SOC2, ISO 27001, and SEC/FINRA-compliant WORM audit logs exported via OpenTelemetry.
- [ ] **Advanced ABAC:** Attribute-Based Access Control policies (e.g., geofenced repository cloning).

---

## Horizon (Beyond 1.0)
Features under evaluation for future releases:

*   **Ephemeral cloud dev environments (CDEs):** Browser-based IDEs backed by Podman sandboxes with pre-warmed VFS mounts.
*   **Alternative VCS backends:** First-class support for Jujutsu (`jj`) or Sapling as alternatives to Git for massive monorepos.
*   **HSM integration:** Native Hardware Security Module support for cryptographic commit signing.

---

## How to Get Involved

*   **Pick up an issue:** Look for issues tagged `good first issue` or `help wanted`.
*   **Join the working groups:** Bi-weekly public Discord calls for the **Core**, **Runner**, and **AI** working groups.
*   **Draft an RFC:** Submit a Request for Comments in the discussions tab before implementing large changes.
