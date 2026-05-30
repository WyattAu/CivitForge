# Requirements Specification (EARS Format)

**Document ID:** REQ-SPEC-001
**Revision:** 1.0
**Date:** 2026-05-30
**Source:** requirements.md (PRD v1.0)
**Standard:** ISO/IEC/IEEE 29148:2018 (EARS subset)
**Language:** EN (primary)

---

## EARS Syntax Reference

| Pattern | Template | Usage |
|---------|----------|-------|
| Ubiquitous | The system shall `[action]` | System-wide behavioral requirements |
| Event-Driven | When `[event]`, the system shall `[action]` | Triggered responses |
| Unwanted Behaviour | If `[condition]`, then the system shall `[action]` | Error handling, safety |
| State-Driven | While `[state]`, the system shall `[action]` | Mode-dependent behavior |
| Optional Feature | Where `[feature]` is supported, the system shall `[action]` | Configurable/capability-gated |

---

## REQ-VCS: Version Control & Monorepo Engine

### REQ-VCS-001 (VFS - Ubiquitous)
The system shall provide a Virtual File System client that allows a developer to mount a repository of up to 10TB in size locally, fetching files on-demand via gRPC only when accessed.

### REQ-VCS-002 (VFS - Performance)
The system shall complete the initial mount of a VFS client for a repository exceeding 1TB within 30 seconds, returning a populated directory tree with file metadata but deferring blob retrieval until file open.

### REQ-VCS-003 (Git Backend - Ubiquitous)
The system shall use gitoxide as its primary Git implementation for all repository read and write operations, avoiding libgit2 C-bindings in all critical paths.

### REQ-VCS-004 (Git Backend - Performance)
The system shall parallelize packfile generation and delta compression using rayon, achieving pack operations at a rate no less than 2x faster than equivalent C-Git operations on identical hardware.

### REQ-VCS-005 (Git Backend - Memory Safety)
The system shall enforce `#![forbid(unsafe_code)]` in all authentication, authorization, and cryptographic code modules within CivitCore.

### REQ-VCS-006 (Alternative VCS - Optional)
Where pluggable VCS backends are supported, the system shall accept Sapling or Jujutsu (jj) as alternative repository formats, mapping their internal object models to the CivitForge API.

### REQ-VCS-007 (Build Graph - Ubiquitous)
The system shall parse and store build dependency graphs from Bazel, Buck2, and Nix build files, enabling CI trigger precision to the level of affected targets only.

### REQ-VCS-008 (Build Graph - Event-Driven)
When a push event modifies files within a known build target dependency graph, the system shall compute the set of transitively affected targets and trigger CI pipelines only for those targets.

### REQ-VCS-009 (SSH - Ubiquitous)
The system shall provide a custom SSH server (via russh) supporting Ed25519 key authentication and hardware token forwarding (FIDO2/WebAuthn) for Git-over-SSH operations.

### REQ-VCS-010 (SSH - Unwanted Behaviour)
If an SSH connection attempt fails OIDC identity verification, then the system shall reject the connection, log the attempt to the WORM audit trail, and increment the source IP's failed authentication counter.

---

## REQ-LFS: Big Data & Large File Management (LFS+)

### REQ-LFS-001 (Chunking - Ubiquitous)
The system shall replace standard Git LFS with a native content-defined chunking engine using FastCDC, splitting files into variable-size blocks between 4MB and 64MB.

### REQ-LFS-002 (Deduplication - Ubiquitous)
The system shall store only unique content-defined chunks in the S3-compatible object storage backend, reconstructing original files from a manifest map stored in the Git repository.

### REQ-LFS-003 (Deduplication - Event-Driven)
When a file is updated and pushed, the system shall identify and transmit only the changed chunks (delta) between the previous and current version, not the complete file.

### REQ-LFS-004 (Deduplication - Performance)
The system shall achieve a deduplication ratio of no less than 80% for typical ML model weight updates (where <5% of tensor weights change between versions).

### REQ-LFS-005 (OCI - Ubiquitous)
The system shall support OCI (Open Container Initiative) artifact types as first-class repository citizens, enabling storage and retrieval of container images, Helm charts, and generic OCI artifacts alongside source code.

### REQ-LFS-006 (Data Gravity - Ubiquitous)
The system shall provide a Container Storage Interface (CSI) driver allowing CI/CD runners to mount LFS+ datasets and S3-stored objects directly into Podman containers without network transfer.

### REQ-LFS-007 (Data Gravity - Performance)
The system shall mount a CSI volume containing up to 100GB of LFS+ data into a Podman sandbox in under 5 seconds from pod scheduling to first-byte access.

### REQ-LFS-008 (Streaming - Ubiquitous)
The system shall support streaming upload and download of individual files up to 500GB in size without requiring the file to be held entirely in memory on either client or server.

---

## REQ-CI: CI/CD & Secure Runner Ecosystem

### REQ-CI-001 (Rootless - Ubiquitous)
The system shall execute all containerized CI tasks using rootless Podman with user namespace mapping, ensuring the container root maps to an unprivileged host user.

### REQ-CI-002 (Rootless - Unwanted Behaviour)
If a CI task attempts to escalate privileges or execute a system call outside the permitted Seccomp profile, then the system shall terminate the container, log the violation, and mark the build as failed.

### REQ-CI-003 (K8s Operator - Ubiquitous)
The system shall implement a Kubernetes Operator (via kube-rs) that translates CI pipeline definitions into Kubernetes Job/Pod resources, managing the full lifecycle from scheduling through artifact collection.

### REQ-CI-004 (K8s Operator - Event-Driven)
When a push event is published to the Redis event bus, the system shall evaluate repository-level pipeline configuration, resolve dependency ordering, and schedule the resulting Kubernetes Jobs within 500ms.

### REQ-CI-005 (Hermetic - Ubiquitous)
The system shall enforce strict network isolation (hermetic mode) for build environments, preventing all outbound network access unless explicitly whitelisted via pipeline-level policy.

### REQ-CI-006 (Hermetic - State-Driven)
While hermetic mode is active for a build pipeline, the system shall reject all DNS resolution, HTTP/S, and TCP connection attempts not targeting the whitelowed artifact mirror endpoints.

### REQ-CI-007 (Provenance - Ubiquitous)
The system shall generate an SBOM in SPDX or CycloneDX format for every build artifact produced, and cryptographically sign both the artifact and the SBOM using Sigstore/Cosign with an ephemeral key tied to the runner's OIDC identity.

### REQ-CI-008 (Provenance - Event-Driven)
When a build completes successfully, the system shall produce a signed SLSA Level 4 provenance attestation containing the build configuration, source commit hash, SBOM digest, and builder identity, and attach it to the pipeline run record.

### REQ-CI-009 (CDE - Ubiquitous)
The system shall provide ephemeral Cloud Development Environments as Kubernetes Pods with pre-warmed VFS mounts and pre-installed dependencies, accessible via a browser-based IDE.

### REQ-CI-010 (CDE - State-Driven)
While a Cloud Development Environment pod is active, the system shall maintain a persistent VFS session and sync all file modifications bidirectionally between the developer's local workspace and the remote pod.

### REQ-CI-011 (Scheduling - Performance)
The system shall support concurrent execution of at least 50,000 CI/CD jobs per day across the Kubernetes cluster, with individual job scheduling latency under 2 seconds from trigger to pod creation.

---

## REQ-AI: Private AI & Agentic Workflows

### REQ-AI-001 (Air-Gap - Ubiquitous)
The system shall deploy the entire AI stack (AST parser, embedding worker, vector database, LLM inference server) within the client's network perimeter with zero dependency on external AI APIs (OpenAI, Anthropic, Google).

### REQ-AI-002 (Air-Gap - Unwanted Behaviour)
If the AI inference server attempts to establish a network connection to any endpoint outside the configured internal network boundary, then the system shall block the connection, log the attempt, and alert the security operations team.

### REQ-AI-003 (RAG - Ubiquitous)
The system shall continuously parse the active codebase into an Abstract Syntax Tree using tree-sitter, chunk the AST into semantic units, generate vector embeddings, and store them in Qdrant for retrieval-augmented generation.

### REQ-AI-004 (RAG - Event-Driven)
When a push event is received, the system shall incrementally parse the changed files, update the affected AST embeddings in Qdrant within 60 seconds, and invalidate stale cache entries.

### REQ-AI-005 (RAG - Performance)
The system shall support semantic search across a codebase exceeding 100 million lines of code, returning relevant context results with a p99 query latency under 100ms against Qdrant.

### REQ-AI-006 (Inference - Ubiquitous)
The system shall serve local LLM inference via vLLM deployed as a Kubernetes service, with tiered model selection: lightweight models (Llama-3-8B) for commit messages and heavy models (DeepSeek-Coder-33B) for code review.

### REQ-AI-007 (Inference - Performance)
The system shall deliver AI PR review results within 120 seconds of push completion for diffs containing up to 5,000 changed lines, using the heavy inference model tier.

### REQ-AI-008 (Agents - Ubiquitous)
The system shall provide autonomous AI agents capable of: reviewing PRs for security vulnerabilities, generating unit tests for undocumented code, and resolving dependency update requests within Podman sandboxes.

### REQ-AI-009 (Agents - Event-Driven)
When a pull request is opened or updated, the system shall dispatch an AI review agent that analyzes the diff, queries Qdrant for repository context, and posts structured review comments to the PR within the configured SLA.

### REQ-AI-010 (Agents - Unwanted Behaviour)
If an AI agent's sandbox execution produces a non-zero exit code or exceeds the configured resource quota, then the system shall terminate the sandbox, capture the stdout/stderr output, and attach it to the agent's review comment as context.

### REQ-AI-011 (Agents - State-Driven)
While an AI agent is executing code in a sandbox, the system shall enforce strict resource limits (CPU, memory, wall-clock time) and prevent the agent from accessing any repository data outside the scope of the assigned PR.

---

## REQ-FED: Geo-Distributed High Availability (Federation)

### REQ-FED-001 (Multi-Master - Ubiquitous)
The system shall support geo-distributed multi-master replication across multiple federation nodes (e.g., London, New York, Tokyo), allowing writes to any local edge node with asynchronous global eventual consistency.

### REQ-FED-002 (Multi-Master - Performance)
The system shall provide local read latency under 5ms for repository metadata queries at any federation node, regardless of global replication state.

### REQ-FED-003 (Multi-Master - Event-Driven)
When a commit is pushed to a local edge node, the system shall acknowledge the write to the developer within 200ms and initiate background asynchronous replication to all peer nodes within 5 seconds.

### REQ-FED-004 (Multi-Master - Unwanted Behaviour)
If a replication conflict is detected during DAG synchronization (e.g., divergent force-push to same ref), then the system shall reject the divergent push, notify the conflicting authors, and preserve both versions in a conflict resolution queue.

### REQ-FED-005 (DAG Sync - Ubiquitous)
The system shall implement a custom DAG (Directed Acyclic Graph) synchronization protocol where nodes broadcast Merkle-root digests of their repository state via secure WebSocket, and negotiate transfer of missing Git objects asynchronously.

### REQ-FED-006 (DAG Sync - Performance)
The system shall achieve full convergence of repository state between any two federation nodes within 60 seconds under normal network conditions for repositories up to 1TB in size.

### REQ-FED-007 (DAG Sync - State-Driven)
While a network partition exists between federation nodes, the system shall continue to accept local writes at each partitioned node and queue all replication operations for replay upon partition resolution.

### REQ-FED-008 (Edge Caching - Ubiquitous)
The system shall cache read-heavy operations (clones, build artifacts, LFS+ blobs) at edge nodes geographically proximate to the requesting developers.

### REQ-FED-009 (Edge Caching - Event-Driven)
When a cache miss occurs at an edge node for a requested Git object or LFS+ blob, the system shall fetch the object from the nearest peer node or origin, cache it locally, and serve the request transparently to the client.

### REQ-FED-010 (Edge Caching - Performance)
The system shall serve cached LFS+ chunks from edge nodes with a p99 retrieval latency under 50ms for objects up to 64MB.

### REQ-FED-011 (Identity - Ubiquitous)
The system shall issue X.509 certificates to each federation node and enforce mutual TLS (mTLS) for all inter-node communication, with automated certificate rotation without service downtime.

### REQ-FED-012 (ForgeFed - Ubiquitous)
The system shall implement the ForgeFed protocol (ActivityPub extension) for federating issues, pull requests, stars, and comments across independent CivitForge instances and compliant third-party forges.

### REQ-FED-013 (ForgeFed - Unwanted Behaviour)
If an incoming ForgeFed ActivityPub message fails message signature verification or originates from a node not in the trusted federation allowlist, then the system shall reject the message and log the attempt.

---

## Non-Functional Requirements (NFR)

### NFR-SEC-001 (Memory Safety)
The system shall enforce `#![forbid(unsafe_code)]` in CivitCore business logic. Any exception requires architectural committee approval with documented rationale and must be restricted to OS-level primitives only.

### NFR-SEC-002 (Encryption at Rest)
The system shall encrypt all Git objects and database partitions at rest using AES-256-GCM, with encryption keys managed by an external KMS (HashiCorp Vault or equivalent).

### NFR-SEC-003 (Encryption in Transit)
The system shall enforce TLS 1.3 for all external connections and mTLS for all internal service-to-service communication.

### NFR-SEC-004 (Audit Logging)
The system shall write a WORM-compliant audit record for every API call, Git clone, SSH connection, and CI pipeline execution, exported via OpenTelemetry to configured SIEM endpoints.

### NFR-SEC-005 (RBAC)
The system shall enforce Role-Based Access Control at the organization, repository, directory, and file level, with policy evaluation latency under 10ms per request.

### NFR-SEC-006 (ABAC)
The system shall support Attribute-Based Access Control policies allowing rule composition across user identity, group membership, source IP, repository classification, and time-of-day.

### NFR-PERF-001 (UI Latency)
The system shall achieve a Web UI Time-to-Interactive (TTI) under 200ms for all standard repository browsing operations (file listing, commit history, PR list) under normal load.

### NFR-PERF-002 (Concurrent Users)
The system shall support at least 10,000 concurrent developers performing push/pull operations without degradation of the p99 latency SLAs.

### NFR-PERF-003 (Monorepo Scale)
The system shall support repositories up to 10TB in total size with 50 million or more commits without degradation of VFS mount time or query performance.

### NFR-PERF-004 (API Throughput)
The system shall handle at least 100,000 authenticated API requests per second across the CivitCore gateway cluster under normal operating conditions.

### NFR-COMP-001 (SOC2)
The system shall produce auditable evidence artifacts supporting SOC 2 Type II certification across Security, Availability, Processing Integrity, Confidentiality, and Privacy trust service criteria.

### NFR-COMP-002 (ISO 27001)
The system shall be designed to support ISO/IEC 27001 certification, with all security controls mapped to applicable ISO 27001 Annex A controls.

### NFR-COMP-003 (FINRA/SEC)
The system shall maintain algorithmic trading audit trails compliant with FINRA and SEC requirements for firms operating in regulated trading environments, including immutable records of code changes to trading algorithms.

### NFR-DEPLOY-001 (Air-Gap)
The system shall be deployable in fully air-gapped environments, with all Docker images, LLM weights, and Rust binaries packaged as offline-transferable tarballs with deterministic checksums.

### NFR-DEPLOY-002 (Infrastructure as Code)
The system shall be deployed exclusively via Infrastructure as Code using Helm Charts, with no manual configuration steps required for standard deployment topologies.

### NFR-DEPLOY-003 (Hardware)
The system shall operate on application servers with 64+ CPU cores and 128GB RAM, and AI servers with a minimum of 2x NVIDIA A100 or H100 GPUs for vLLM inference.
