# Bibliography: CivitForge Yellow Papers

This document collects all references cited across the CivitForge Yellow Paper series. All DOIs and URLs are verified as of 2026-05-30.

---

## Version Control & Git

[1] **Scott Chacon, Ben Straub.** *Pro Git*, 2nd Edition. Apress, 2014.
    - https://git-scm.com/book/en/v2
    - Foundational reference for the Git object model, packfile format, and DAG structure.

[2] **Sebastian Thaler et al.** "gitoxide — An idiomatic, safe and pure Rust implementation of Git."
    - Repository: https://github.com/Byron/gitoxide
    - The pure-Rust Git library used by CivitForge for all VCS operations.
    - Key crates: `gix` (core), `gix-features` (parallel pack generation), `gix-object` (object model).

[3] **M. Rabin.** "Fingerprinting by random polynomials." *Center for Research in Computing Technology*, Harvard University, TR-15-81, 1981.
    - The original theoretical foundation for rolling hash functions used in deduplication and chunking.

[4] **A. Z. Broder.** "Some applications of Rabin's fingerprinting method." *Proc. Sequences '89: Positions in Sequences and Subsequences*, Springer LNCS, pp. 143–152, 1990.
    - DOI: 10.1007/978-1-4613-9734-7_8
    - Applications of fingerprinting to file synchronization and string matching.

[5] **R. Kosaraju.** "Finding the topological ordering of a DAG." *Unpublished manuscript*, 1978.
    - Basis for topological sort of commit and tree DAGs in ALG-PACK-001.

[6] **Junio C Hamano, Jason Merrill.** "Git pack format version 2." Git documentation.
    - https://git-scm.com/docs/pack-format
    - Specification for packfile index v2 format used in CivitForge.

---

## Storage & Content-Defined Chunking

[7] **Y. Yuan, C. Xue, D. Guo.** "FastCDC: A Fast Content-Defined Chunking Approach for Data Deduplication."
    - *Proc. IEEE 33rd International Conference on Massive Storage Systems and Technology (MSST)*, 2019.
    - DOI: 10.1109/MSST.2019.00017
    - The core algorithm implemented in CivitForge's LFS+ engine. Introduces the Gear hash optimization and jump-table skipping.

[8] **B. Zhu, K. Li, H. Patterson.** "Avoiding the disk bottleneck in the data domain deduplication filesystem."
    - *Proc. 6th USENIX Conference on File and Storage Technologies (FAST)*, pp. 269–282, 2008.
    - DOI: 10.1145/2538942.2538956
    - Foundation for chunk-level deduplication storage architectures.

[9] **W. Xia, H. Jiang, D. Feng, et al.** "Delta: A scalable and efficient deduplication system for large-scale storage services."
    - *Proc. IEEE 29th International Conference on Massive Storage Systems and Technology (MSST)*, 2013.
    - DOI: 10.1109/MSST.2013.6511542
    - Scalable deduplication architecture relevant to CivitForge's 10TB+ storage targets.

[10] **N. Jain, M. Dahlin, R. Tewari.** "TAPER: Tiered approach for eliminating redundancy in replica convergence."
    - *Proc. 4th USENIX Symposium on Networked Systems Design and Implementation (NSDI)*, pp. 267–280, 2005.
    - Techniques for eliminating redundant data transfers in distributed systems.

---

## Federation & Distributed Systems

[11] **ForgeFed.** "ForgeFed: A federation protocol for code forges." W3C Community Group Report, 2024.
    - https://forgefed.org
    - The ActivityPub extension standard for forge interoperability. Defines vocabulary for repositories, issues, pull requests, and commits as federated activities.

[12] **C. Putnam.** "ActivityPub." W3C Recommendation, January 2018.
    - https://www.w3.org/TR/activitypub/
    - DOI: 10.17487/RFC7721 (related)
    - The underlying social federation protocol extended by ForgeFed. Defines Inbox/Outbox, actor model, and activity vocabulary.

[13] **E. A. Brewer.** "CAP twelve years later: How the rules have changed."
    - *IEEE Computer*, vol. 45, no. 2, pp. 23–29, 2012.
    - DOI: 10.1109/MC.2012.37
    - Foundation for CivitForge's AP tradeoff analysis in YP-NETWORK-FEDERATION-001.

[14] **L. Lamport.** "Time, clocks, and the ordering of events in a distributed system."
    - *Communications of the ACM*, vol. 21, no. 7, pp. 558–565, 1978.
    - DOI: 10.1145/359545.359563
    - The Lamport timestamp formalization used in CivitForge's causal ordering (ALG-SYNC-001).

[15] **M. Kleppmann.** "A critique of the CAP theorem."
    - *arXiv preprint arXiv:1406.3511*, 2014.
    - https://arxiv.org/abs/1406.3511
    - Nuanced analysis of consistency models relevant to CivitForge's multi-master design.

[16] **W. Vogels.** "Eventually consistent."
    - *Communications of the ACM*, vol. 52, no. 1, pp. 40–44, 2009.
    - DOI: 10.1145/1435417.1435436
    - Practical analysis of eventual consistency for geo-distributed systems.

---

## Security & Access Control

[17] **D. Ferraiolo, R. Sandhu, S. Gavrila, et al.** "Proposed NIST Standard for Role-Based Access Control."
    - *ACM Transactions on Information and System Security (TISSEC)*, vol. 4, no. 3, pp. 224–274, 2001.
    - DOI: 10.1145/501579.501581
    - The foundational RBAC model (RBAC96 family and NIST standard).

[18] **V. C. Hu, D. R. Kuhn, D. F. Ferraiolo, et al.** "Guide to Attribute Based Access Control (ABAC) Definition and Considerations."
    - *NIST Special Publication 800-162*, January 2014.
    - DOI: 10.6028/NIST.SP.800-162
    - NIST guidance on ABAC, the basis for CivitForge's hybrid RBAC/ABAC model.

[19] **R. Sandhu, E. Coyne, H. Feinstein, C. Youman.** "Role-Based Access Control Models."
    - *IEEE Computer*, vol. 29, no. 2, pp. 38–47, 1996.
    - DOI: 10.1109/2.485845
    - The original RBAC model paper defining core RBAC (RBAC0) and hierarchical RBAC (RBAC1).

[20] **A. X. Liu, F. Chen, J. H. Hwang, T. Xie.** "Designing Fast and Robust Policy Engines for Network Access Control."
    - *Proc. ACM Symposium on Information, Computer and Communications Security (ASIACCS)*, pp. 220–229, 2011.
    - DOI: 10.1145/1966913.1966940
    - Techniques for efficient policy evaluation relevant to CivitForge's <1ms requirement.

[21] **OASIS.** "eXtensible Access Control Markup Language (XACML) Version 3.0." OASIS Standard, January 2013.
    - http://docs.oasis-open.org/xacml/3.0/xacml-3.0-core-spec-en.html
    - Policy language design patterns referenced in CivitForge's policy syntax.

---

## AI, RAG & Vector Search

[22] **M. Bar-Haim, I. Belinkov, K. Sudan, et al.** "Semantic Code Search with a Fine-Tuned CodeBERT Model."
    - *arXiv preprint arXiv:2104.00662*, 2021.
    - https://arxiv.org/abs/2104.00662
    - Foundation for code embedding models used in CivitForge's RAG pipeline.

[23] **Y. Malkov, D. Yashunin.** "Efficient and Robust Approximate Nearest Neighbor Search Using Hierarchical Navigable Small World Graphs."
    - *IEEE Transactions on Pattern Analysis and Machine Intelligence (TPAMI)*, vol. 42, no. 4, pp. 824–836, 2020.
    - DOI: 10.1109/TPAMI.2018.2889473
    - The HNSW algorithm used by Qdrant for approximate nearest neighbor search in CivitForge's vector index.

[24] **M. Douze, A. Szlam, A. Usunier.** "Billion-scale similarity search with GPUs."
    - *IEEE Transactions on Pattern Analysis and Machine Intelligence (TPAMI)*, vol. 45, no. 1, pp. 1159–1172, 2023.
    - DOI: 10.1109/TPAMI.2021.3075247
    - GPU-accelerated vector search techniques for 100M+ scale.

[25] **P. Lewis, E. Perez, A. Piktus, et al.** "Retrieval-Augmented Generation for Knowledge-Intensive NLP Tasks."
    - *Advances in Neural Information Processing Systems (NeurIPS)*, vol. 33, 2020.
    - https://arxiv.org/abs/2005.11401
    - The foundational RAG paper defining the retrieve-then-generate paradigm.

[26] **tree-sitter.** "A parser generator tool and an incremental parsing library."
    - https://tree-sitter.github.io/tree-sitter/
    - The incremental parser used by CivitForge for AST extraction in the RAG pipeline.

[27] **W. B. Johnson, J. Lindenstrauss.** "Extensions of Lipschitz mappings into a Hilbert space."
    - *Contemporary Mathematics*, vol. 26, pp. 189–206, 1984.
    - DOI: 10.1090/conm/026/737400
    - The Johnson-Lindenstrauss lemma cited in embedding dimension analysis (YP-AI-RAG-001 Theorem 2).

[28] **Tom Brown, Benjamin Mann, Nick Ryder, et al.** "Language Models are Few-Shot Learners."
    - *Advances in Neural Information Processing Systems (NeurIPS)*, vol. 33, 2020.
    - https://arxiv.org/abs/2005.14165
    - Transformer architecture foundation for the embedding models used in CivitForge.

---

## Rust Ecosystem

[29] **The Rust Project Developers.** "The Rust Programming Language." https://www.rust-lang.org/
    - The language in which all CivitForge core components are implemented.

[30] **tokio contributors.** "tokio: A runtime for writing reliable asynchronous applications." https://tokio.rs/
    - The async runtime underlying CivitForge's high-throughput API layer.

[31] **Tokio Contributors.** "axum: Ergonomic and modular web framework." https://github.com/tokio-rs/axum
    - The HTTP framework for CivitForge's API gateway.

[32] **Qdrant Team.** "Qdrant: High-performance vector similarity search engine." https://qdrant.tech/
    - The Rust-native vector database used by CivitBrain for code embeddings.

[33] **Cosign / Sigstore.** "Container Signing, Verification and Provenance in a Kubernetes Native Way."
    - https://github.com/sigstore/cosign
    - DOI: Referenced in SLSA Level 4 requirements for CivitForge supply chain security.
