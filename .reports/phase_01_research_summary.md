# Phase 1 Research Summary Report

**CivitForge Project** | **Phase 1: The Rust Core & VFS** | **Date: 2026-05-30**

---

## Overview

Phase 1 research establishes the theoretical foundations for the CivitForge software forge. This report summarizes the five Yellow Papers produced, their key findings, and the validated design decisions that will inform the Phase 1 implementation.

---

## Yellow Papers Produced

### 1. YP-VERSION-CONTROL-GIT-001: Git Object Model & DAG Consistency

**Key Results:**
- Formalized the Git object model as a Merkle DAG with SHA-256 content addressing (replacing Git's SHA-1 for collision resistance at TB+ scale)
- Proved reachability correctness ($O(n + |E|)$ BFS), merge conflict detection bounds, and pack integrity guarantees
- Designed ALG-PACK-001 for parallel packfile generation with $O(N \cdot W \cdot \bar{s} / P)$ wall-clock time, supporting 64+ threads
- Established domain constraints: 10TB max repo, 10M+ commits, 50M objects per pack, <1ms object lookup (p99)

**Impact on Phase 1:** Confirms `gitoxide` as the correct choice for pure-Rust VCS. Parallel pack generation justifies the 64+ core hardware requirement. SHA-256 migration path is validated.

### 2. YP-STORAGE-CHUNKING-001: FastCDC Content-Defined Chunking

**Key Results:**
- Formalized the Gear rolling hash with $O(1)$ window-advance and full mathematical derivation of chunk size distribution (shifted geometric, bounded by $[M_s, M_a]$)
- Proved dedup ratio bounds: for $k$-byte modification in $L$-byte file, $\text{DR} \approx 1 - (2w + k) / L$
- Designed ALG-CDC-001 with $O(L)$ time complexity and >2 GB/s throughput per core
- Established constraints: 4MB-64MB chunks, 64-bit Gear hash, >90% dedup for 1% modification

**Impact on Phase 1:** FastCDC chosen over Rabin CDC for 2-3x throughput improvement (jump-table optimization). 16MB default target chunk size balances dedup ratio against metadata overhead. Block-level dedup is validated for ML model weights and HFT tick data use cases.

### 3. YP-NETWORK-FEDERATION-001: ForgeFed DAG Synchronization Protocol

**Key Results:**
- Modeled federation as a distributed state machine with Lamport-timestamp causal ordering
- Proved convergence bounds: $(k-1) \cdot \delta$ under synchronous network; $O(\delta + \lambda \cdot \Delta_t \cdot \log n_{\text{ops}})$ post-partition
- Designed ALG-SYNC-001 with Merkle-root-guided divergence detection ($O(1)$ initial comparison) and $O(m \log m)$ causal ordering
- Confirmed AP tradeoff: availability during partition, convergence after reconnection with deny-override for conflicting policy updates

**Impact on Phase 1:** Federation is deferred to Phase 4 but the Merkle-root data structure should be embedded in the VCS layer from Phase 1. The ActivityPub/ForgeFed message format is specified and can be prototyped.

### 4. YP-SECURITY-RBAC-001: RBAC/ABAC Access Control Model

**Key Results:**
- Formalized hybrid RBAC/ABAC with deny-override semantics (Axiom 2) and default-deny (Axiom 1)
- Proved $O(n \cdot k)$ policy evaluation complexity, reducible to $O(m \cdot k)$ with resource indexing
- Proved permission inheritance soundness through role hierarchy (depth ≤10)
- Designed ALG-POLICY-001 with <1ms p99 evaluation target, supporting attribute conditions (IP, time, MFA, device)

**Impact on Phase 1:** Policy engine must be implemented in Phase 1 (authentication/authorization is foundational). The $O(m \cdot k)$ indexed evaluation confirms feasibility for 10,000+ policies. SAML/OIDC integration is required from day one.

### 5. YP-AI-RAG-001: RAG over Abstract Syntax Trees

**Key Results:**
- Formalized AST-based code chunking using `tree-sitter` with function/class/module granularity
- Proved HNSW recall@10 >0.999 for $M=16$, $ef=100$, $N_v=10^8$ (well above 0.95 target)
- Proved index update complexity $O(b \cdot M \cdot \log N_v)$ for batch insertions
- Designed ALG-RAG-001 with ~61ms per-file indexing and ~17ms query latency

**Impact on Phase 1:** RAG is deferred to Phase 3 but the AST parsing infrastructure (tree-sitter integration) can be prototyped in Phase 1. The vector index schema should be designed alongside the CockroachDB schema in Phase 1.

---

## Cross-Paper Design Decisions

| Decision | Rationale | Papers |
|---|---|---|
| SHA-256 over SHA-1 for object hashing | Collision resistance at TB+ scale | YP-GIT-001 |
| FastCDC over classic Rabin CDC | 2-3x throughput improvement via jump-table | YP-CHUNKING-001 |
| AP over CP for federation | Availability during partitions for geo-distributed teams | YP-FED-001 |
| Deny-override over deny-first-match | Stronger security guarantee; simpler policy authoring | YP-RBAC-001 |
| AST chunking over line-based chunking | Structural semantics preserved for retrieval quality | YP-RAG-001 |
| Qdrant for vector storage | Rust-native; consistent with project language strategy | YP-RAG-001 |
| CockroachDB for metadata | Geo-distributed SQL; multi-master by design | YP-FED-001, YP-RBAC-001 |
| Merkle roots embedded in VCS layer | Enables both reachability proofs and federation sync | YP-GIT-001, YP-FED-001 |

---

## Test Coverage Summary

| Vector Set | Count | Categories | Yellow Paper |
|---|---|---|---|
| Git operations | 12 | 8 nominal, 2 boundary, 2 adversarial | YP-GIT-001 |
| FastCDC chunking | 8 | 4 nominal, 3 boundary, 1 adversarial | YP-CHUNKING-001 |
| DAG federation | 6 | 3 nominal, 1 boundary, 2 adversarial | YP-FED-001 |
| RBAC evaluation | 8 | 5 nominal, 3 boundary | YP-RBAC-001 |
| RAG pipeline | 10 | 6 nominal, 3 boundary, 1 adversarial | YP-RAG-001 |
| **Total** | **44** | | |

---

## Validated Architecture Decisions

1. **`#![forbid(unsafe_code)]` in business logic** — Confirmed by the formal correctness requirements across all papers. Memory safety is non-negotiable.
2. **`gitoxide` over `libgit2`** — Confirmed by YP-GIT-001. Pure-Rust enables lock-free concurrent pack generation (ALG-PACK-001) and eliminates C-binding memory leak risks.
3. **64+ core servers** — Justified by parallel pack generation (YP-GIT-001), FastCDC multi-core scaling (YP-CHUNKING-001), and concurrent policy evaluation (YP-RBAC-001).
4. **Phase-gated delivery** — Research confirms the phasing strategy: VCS core in Phase 1, chunking in Phase 2, RAG in Phase 3, federation in Phase 4.

---

## Open Research Questions

1. **SHA-256 vs BLAKE3 for object hashing** — BLAKE3 offers 5-10x faster hashing than SHA-256. Consider benchmarking and potentially adopting BLAKE3 as a non-standard extension with SHA-256 compatibility mode.
2. **Adaptive chunk sizes** — Current FastCDC parameters are static. Research into workload-adaptive chunk sizing (e.g., adjusting $\mu$ based on file type and modification patterns).
3. **CRDT-based metadata sync** — Consider Conflict-Free Replicated Data Types (CRDTs) as an alternative to Lamport-timestamp + deny-override for issue/PR metadata, potentially reducing conflict rates.
4. **Hierarchical neural code embeddings** — Investigate whether a two-stage embedding (intra-function + inter-function) improves retrieval recall over single-stage AST chunking.

---

## Next Steps (Phase 1 Implementation)

1. Implement core `gitoxide`-based VCS operations (clone, fetch, push)
2. Build `Axum` API gateway with OIDC authentication
3. Implement ALG-POLICY-001 policy engine
4. Design CockroachDB schema for Users, Repositories, Organizations, Policies
5. Prototype `tree-sitter` integration for AST parsing (future-proofing for Phase 3)
6. Implement SSH server using `russh` with Ed25519 key support
7. Build Web UI v1 for repository browsing
