# CivitForge Master Traceability Matrix

**Version:** 0.1.0 | **Date:** 2026-05-30 | **Phase:** Phase 2 Architecture

---

## Yellow Papers → Blue Papers (Forward Traceability)

| Yellow Paper | Blue Paper(s) | Trace Type |
|---|---|---|
| YP-VERSION-CONTROL-GIT-001 (Git Object Model & DAG Consistency) | BP-CORE-API-001 §4, §5, §7 (Git Engine, Smart HTTP, gitoxide) | Requirements → Design |
| YP-VERSION-CONTROL-GIT-001 | BP-VFS-001 §4, §5, §7 (On-demand fetch, gRPC protocol) | Requirements → Design |
| YP-VERSION-CONTROL-GIT-001 | BP-BRAIN-001 §4 (Incremental indexing on push) | Requirements → Design |
| YP-VERSION-CONTROL-GIT-001 | proof_git_dag.lean (DAG acyclicity, Merkle uniqueness) | Requirements → Formal Proof |
| YP-STORAGE-CHUNKING-001 (FastCDC Content-Defined Chunking) | BP-RUNNER-001 §4, §7 (CSI mounts, LFS+ integration) | Requirements → Design |
| YP-STORAGE-CHUNKING-001 | BP-VFS-001 §5 (Placeholder hydration, chunk fetch) | Requirements → Design |
| YP-STORAGE-CHUNKING-001 | proof_chunking.lean (Bounded chunks, determinism) | Requirements → Formal Proof |
| YP-NETWORK-FEDERATION-001 (ForgeFed DAG Sync) | BP-CORE-API-001 §5, §7 (Federation engine, ActivityPub) | Requirements → Design |
| YP-NETWORK-FEDERATION-001 | BP-CRYPTO-001 §7 (mTLS for inter-node) | Requirements → Design |
| YP-NETWORK-FEDERATION-001 | proof_dag_sync.lean (Convergence, causal ordering) | Requirements → Formal Proof |
| YP-SECURITY-RBAC-001 (RBAC/ABAC) | BP-CORE-API-001 §4, §5, §7 (Auth middleware, RBAC evaluation) | Requirements → Design |
| YP-SECURITY-RBAC-001 | BP-RUNNER-001 §7 (Sandbox security context) | Requirements → Design |
| YP-SECURITY-RBAC-001 | BP-CRYPTO-001 §7 (SBOM, Cosign, certificate management) | Requirements → Design |
| YP-SECURITY-RBAC-001 | proof_rbac.lean (Deny-override, termination, inheritance) | Requirements → Formal Proof |
| YP-AI-RAG-001 (RAG over AST) | BP-BRAIN-001 §4, §5, §7 (tree-sitter, embeddings, RAG retrieval) | Requirements → Design |
| YP-AI-RAG-001 | proof_rag.lean (Cosine bounds, top-k distinctness) | Requirements → Formal Proof |

---

## Blue Papers → Yellow Papers (Reverse Traceability)

| Blue Paper | Yellow Paper(s) | Rationale |
|---|---|---|
| BP-CORE-API-001 | YP-VERSION-CONTROL-GIT-001, YP-SECURITY-RBAC-001, YP-NETWORK-FEDERATION-001 | Core API implements Git engine, auth, and federation |
| BP-RUNNER-001 | YP-STORAGE-CHUNKING-001, YP-SECURITY-RBAC-001 | Runner uses chunked storage and sandbox security |
| BP-BRAIN-001 | YP-AI-RAG-001, YP-VERSION-CONTROL-GIT-001 | Brain implements RAG pipeline triggered by Git events |
| BP-VFS-001 | YP-VERSION-CONTROL-GIT-001, YP-STORAGE-CHUNKING-001 | VFS provides on-demand access to Git objects and chunks |
| BP-CRYPTO-001 | YP-SECURITY-RBAC-001, YP-NETWORK-FEDERATION-001 | Crypto provides signing and mTLS for security and federation |

---

## Blue Papers → Interface Contracts

| Blue Paper | Interface Contract | Endpoints/Services |
|---|---|---|
| BP-CORE-API-001 | IC-CORE-API-001 | REST (auth, repos, PRs, issues, orgs), gRPC (VFS), SSH |
| BP-CORE-API-001 | IC-FEDERATION-001 | ActivityPub, gRPC FederationSyncService |
| BP-RUNNER-001 | IC-RUNNER-001 | gRPC PipelineService, K8s PipelineRun CRD, REST management |
| BP-BRAIN-001 | IC-BRAIN-001 | gRPC AIService, REST (chat, search, review) |
| BP-VFS-001 | IC-VFS-001 | FUSE callbacks, gRPC VFSFetchService |
| BP-CRYPTO-001 | IC-FEDERATION-001 | mTLS configuration, certificate rotation |

---

## Blue Papers → Formal Proofs

| Blue Paper | Proof File | Properties Proven |
|---|---|---|
| BP-CORE-API-001 | proof_git_dag.lean | DAG acyclicity, Merkle tree uniqueness, reachability transitivity, content-addressable uniqueness |
| BP-RUNNER-001 | proof_chunking.lean | FastCDC bounded chunks, determinism, delta-chunking preservation |
| BP-RUNNER-001 | proof_rbac.lean (sandbox security) | RBAC deny-override |
| BP-BRAIN-001 | proof_rag.lean | Cosine similarity ∈ [-1,1], top-k distinctness, dimensionality bounds |
| BP-CRYPTO-001 | proof_rbac.lean (certificate) | Policy termination |
| BP-CORE-API-001 | proof_dag_sync.lean | Eventual convergence, no data loss, causal ordering |

---

## Product Requirements → Blue Papers

| PRD Requirement | Blue Paper Section | Status |
|---|---|---|
| REQ-1.1: VFS for 5TB+ monorepos | BP-VFS-001 §1, §7 | Addressed |
| REQ-1.2: Rust Git backend (gitoxide) | BP-CORE-API-001 §3, §7 | Addressed |
| REQ-2.1: Block-level deduplication | BP-RUNNER-001 §7 (FastCDC), BP-VFS-001 §7 | Addressed |
| REQ-2.3: Data gravity mounting | BP-RUNNER-001 §7 (CSI) | Addressed |
| REQ-3.1: Rootless Podman execution | BP-RUNNER-001 §3, §7 | Addressed |
| REQ-3.2: K8s orchestration | BP-RUNNER-001 §3, §7 (kube-rs) | Addressed |
| REQ-3.4: SBOM + Cosign | BP-CRYPTO-001 §3, §7 | Addressed |
| REQ-4.1: Air-gapped AI | BP-BRAIN-001 §1, §8 (vLLM local) | Addressed |
| REQ-4.2: Codebase RAG | BP-BRAIN-001 §1, §7 (tree-sitter, Qdrant) | Addressed |
| REQ-4.3: Autonomous agents | BP-BRAIN-001 §7 (ReAct agent loop) | Addressed |
| REQ-5.1: Multi-master replication | BP-CORE-API-001 §7 (Federation engine) | Addressed |

---

## Technical Requirements → Blue Papers

| TRD Section | Blue Paper | Status |
|---|---|---|
| §2.1: Axum + tokio | BP-CORE-API-001 §3 | Addressed |
| §2.2: gitoxide (pure Rust) | BP-CORE-API-001 §3 | Addressed |
| §2.3: ForgeFed DAG sync | BP-CORE-API-001 §7, IC-FEDERATION-001 | Addressed |
| §3.1: CockroachDB | BP-CORE-API-001 §6, BP-RUNNER-001 §6 | Addressed |
| §3.2: FastCDC + S3 | BP-RUNNER-001 §7, BP-CRYPTO-001 §7 | Addressed |
| §3.3: Qdrant | BP-BRAIN-001 §6 | Addressed |
| §4.1: kube-rs Operator | BP-RUNNER-001 §3, §7 | Addressed |
| §4.2: Rootless Podman | BP-RUNNER-001 §3, §7 | Addressed |
| §4.3: Cosign/Sigstore | BP-CRYPTO-001 §3, §7 | Addressed |
| §5.1: tree-sitter AST | BP-BRAIN-001 §3, §7 | Addressed |
| §5.2: vLLM serving | BP-BRAIN-001 §8 | Addressed |
| §6: REST/gRPC/SSH APIs | BP-CORE-API-001 §5, IC-CORE-API-001 | Addressed |
| §7: Zero-trust auth | BP-CORE-API-001 §7, BP-CRYPTO-001 §7 | Addressed |

---

## Compliance → Blue Papers

| Standard | Blue Paper(s) | Section |
|---|---|---|
| SLSA L3/L4 | BP-RUNNER-001, BP-CRYPTO-001 | §11 Compliance Matrix |
| SOC2 CC6/CC7/CC8 | BP-CORE-API-001, BP-RUNNER-001, BP-BRAIN-001 | §11 Compliance Matrix |
| ISO 27001 | BP-CORE-API-001, BP-RUNNER-001, BP-BRAIN-001, BP-CRYPTO-001 | §11 Compliance Matrix |
| NIST SP 800-53 | BP-CORE-API-001, BP-RUNNER-001, BP-CRYPTO-001 | §11 Compliance Matrix |
| NIST SP 800-218 (SSDF) | BP-CRYPTO-001 | §11 Compliance Matrix |
| NTIA Minimum Elements | BP-CRYPTO-001 | §11 Compliance Matrix |
| EO 14028 | BP-CRYPTO-001 | §11 Compliance Matrix |
| FINRA 4530/4512 | BP-CORE-API-001, BP-BRAIN-001 | §11 Compliance Matrix |
| EU AI Act | BP-BRAIN-001 | §11 Compliance Matrix |

---

## Gap Analysis

| Area | Coverage | Gaps |
|---|---|---|
| Version Control | Full | None identified |
| Storage/Chunking | Full | None identified |
| Federation | Full | None identified |
| Security/RBAC | Full | None identified |
| AI/RAG | Full | None identified |
| VFS | Full | None identified |
| Supply Chain (SBOM/Cosign) | Full | None identified |
| CI/CD Orchestration | Full | None identified |
| Deployment (K8s) | Full | None identified |
| Compliance | Substantial | Penetration testing planned Phase 4 |
| Performance Benchmarks | Partial | Load testing blocked on staging env |
