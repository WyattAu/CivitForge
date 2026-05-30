# Phase -1: Context Discovery Report

**Document ID:** RPT-CD-001
**Revision:** 1.0
**Date:** 2026-05-30
**Status:** Complete
**Phase:** -1 (Pre-Requirements / Domain Exploration)

---

## 1. Executive Summary

Phase -1 (Context Discovery) established the foundational understanding of the CivitForge domain, identified applicable regulatory standards, catalogued technical risks, and defined the scope boundary for requirements engineering. The domain is confirmed as **Distributed Software Forge** operating at extreme scale (10TB monorepos, 10K+ concurrent users, geo-distributed) with stringent security requirements spanning supply chain integrity (SLSA Level 4), information security (ISO 27001), federal cryptography (FIPS 140-2), and financial regulatory compliance (FINRA/SEC).

---

## 2. Domain Identification

### 2.1 Primary Domain

**Distributed Software Configuration Management & Platform Engineering**

CivitForge combines five traditionally separate product categories into a single federated system:

| Category | CivitForge Component | Traditional Equivalents |
|----------|---------------------|------------------------|
| Git Hosting | CivitCore (gitoxide) | GitHub, GitLab, Bitbucket |
| CI/CD Orchestration | CivitRunner (K8s/Podman) | Jenkins, CircleCI, GitHub Actions |
| Large File Storage | CivitData (FastCDC LFS+) | Git LFS, Artifactory, DVC |
| AI Code Intelligence | CivitBrain (tree-sitter + vLLM) | GitHub Copilot, Sourcegraph Cody |
| Federation/HA | CivitCore (ForgeFed/DAG) | Gerrit multi-site, GitLab Geo |

### 2.2 Domain Complexity Assessment

The domain is classified as **Critical Complexity** based on the following dimensions:

| Dimension | Rating | Justification |
|-----------|--------|---------------|
| Concurrency | Extreme | 10K+ simultaneous developers, async event-driven architecture |
| Distributed State | Critical | Multi-master geo-replication with eventual consistency |
| Security Surface | Critical | Zero-trust, air-gap, SLSA L4, FIPS 140-2, HFT IP protection |
| Scale | Extreme | 10TB repos, 50M+ commits, 500GB individual files |
| Real-time Requirements | High | <200ms TTI, <120s AI review SLA, <5ms local reads |
| Regulatory Burden | Critical | ISO 27001, SOC2, NIST 800-53, FIPS 140-2, FINRA/SEC, OWASP |

---

## 3. Standards Identification

### 3.1 Mandatory Standards

The following standards are **non-negotiable** for CivitForge's target market (HFT firms, Defense, Tier-1 tech):

| Standard | Domain | Rationale | Applicability |
|----------|--------|-----------|---------------|
| SLSA Level 4 | Supply Chain | Highest provenance guarantee; required by HFT compliance | All artifact-producing pipelines |
| ISO/IEC 27001:2022 | Information Security | Baseline ISMS for enterprise deployment | Architecture-wide |
| NIST SP 800-53 Rev 5 | Security Controls | Federal security control framework; Defense contracts | Architecture-wide |
| FIPS 140-2 | Cryptography | Required for US Federal and many Defense deployments | TLS, encryption, signing modules |
| OWASP Top 10 (2021) | Application Security | Baseline web application security | CivitCore API, Web UI |

### 3.2 Conditional Standards

| Standard | Domain | Condition |
|----------|--------|-----------|
| ISO 26262 | Functional Safety | Applicable only for HFT customer deployments (ASIL analog mapping) |
| SOC 2 Type II | Service Organization | Required for SaaS/hosted CivitForge deployments |
| FINRA Rule 3110 / SEC Rule 17a-4 | Financial Regulation | Required for HFT/regulated trading firm deployments |
| PCI DSS | Payment Card Data | Not applicable unless forge processes payment data (not in scope) |

### 3.3 Reference Standards (Informed By)

| Standard | Domain | Usage |
|----------|--------|-------|
| ISO/IEC/IEEE 29148:2018 | Requirements Engineering | EARS format for requirements specification |
| OCI Distribution Spec | Container Images | OCI artifact storage format |
| ActivityPub W3C REC | Federation | ForgeFed protocol foundation |
| Kubernetes CSI Spec | Storage | Container Storage Interface driver implementation |
| SPDX / CycloneDX | Supply Chain | SBOM generation format |
| OpenTelemetry | Observability | Audit trail and telemetry export |

---

## 4. Risk Assessment

### 4.1 Architectural Risks

| ID | Risk | Severity | Mitigation Strategy |
|----|------|----------|-------------------|
| AR-001 | gitoxide may not achieve feature parity with libgit2 for edge-case Git operations by Phase 1 deadline | High | Define explicit gitoxide feature gate matrix; maintain libgit2 fallback adapter (disabled by default) |
| AR-002 | FastCDC chunk deduplication overhead may violate latency SLAs on 10TB repos under concurrent writes | High | Benchmark FastCDC against Btrfs-style fixed-block chunking; implement adaptive chunk sizing |
| AR-003 | Qdrant single-node performance may degrade beyond 200M vectors, requiring distributed deployment earlier than planned | Medium | Plan for Qdrant distributed cluster from Phase 3 inception; define sharding strategy by repository |
| AR-004 | Multi-master CockroachDB write amplification may impact push latency at 10K+ concurrent writes | Medium | Implement write batching in CivitCore; use CockroachDB regional tables for partitioned writes |
| AR-005 | ForgeFed DAG sync protocol may encounter split-brain scenarios during extended network partitions | Critical | Implement vector clocks with last-writer-wins conflict resolution; require manual resolution for same-ref conflicts |

### 4.2 Security Risks

| ID | Risk | Severity | Mitigation Strategy |
|----|------|----------|-------------------|
| SR-001 | Rust crate supply chain compromise (malicious dependency injection) | Critical | Implement cargo-deny, cargo-audit in CI; pin exact dependency versions; maintain private registry mirror for air-gapped deployments |
| SR-002 | Rootless Podman container escape via kernel vulnerability | Critical | Monitor crun/CRI-O CVEs; implement additional layer: gVisor application kernel for defense-in-depth |
| SR-003 | AI prompt injection via malicious PR content (e.g., encoded instructions in code) | High | Sanitize all code input to AI pipeline; restrict AI agent output channels; implement output validation |
| SR-004 | Federation node compromise enabling cross-organization data access | Critical | Per-organization encryption keys; node-level attestation before federation; least-privilege federation tokens |

### 4.3 Operational Risks

| ID | Risk | Severity | Mitigation Strategy |
|----|------|----------|-------------------|
| OR-001 | vLLM inference latency exceeds 120s SLA for PR review on large diffs | Medium | Implement model tiering with fast-path for diffs >5K lines; pre-warm GPU context |
| OR-002 | VFS FUSE daemon memory pressure on developer workstations with 10TB repos | Medium | Implement LRU eviction for VFS metadata; benchmark with configurable memory ceilings |
| OR-003 | Helm chart complexity exceeds operational team capability | Medium | Provide observability stack (Prometheus/Grafana dashboards) with every Helm release; automated upgrade canary tests |

---

## 5. Scope Boundaries

### 5.1 In Scope (Phase 0-4)

- Git hosting via gitoxide with VFS (EdenFS/Scalar protocol)
- Block-level LFS+ with FastCDC deduplication
- Rootless Podman CI/CD on Kubernetes
- Air-gapped AI (tree-sitter + Qdrant + vLLM)
- ForgeFed federation with DAG sync
- SLSA Level 4 provenance
- SOC2/ISO 27001/FINRA audit logging

### 5.2 Out of Scope (This Release Cycle)

- Native Windows VFS client (Linux/macOS only for Phase 1-4)
- Mobile application
- Marketplace/plugin ecosystem
- Billing/metering for SaaS hosting
- ML model training pipeline (inference only)
- Alternative VCS (Sapling/Jujutsu) as primary backend

### 5.3 Deferred to Horizon (Post-1.0)

- Ephemeral Cloud Dev Environments (browser-based IDEs)
- HSM native integration for commit signing
- Full Jujutsu/Sapling first-class support
- Bazel/Buck2 remote execution service (BES/BEF)
- Cross-platform VFS for Windows

---

## 6. Key Findings & Decisions

| Decision ID | Decision | Rationale |
|-------------|----------|-----------|
| KD-001 | Adopt EARS (Easy Approach to Requirements Syntax) for all functional requirements | Provides unambiguous, testable requirement syntax; aligns with ISO 29148 |
| KD-002 | Target SLSA Level 4 rather than Level 3 | HFT and Defense customers require highest provenance guarantee; Level 4 differentiates from competitors |
| KD-003 | Use CockroachDB over PostgreSQL | Geo-distributed multi-master requires native replication; CockroachDB provides PostgreSQL compatibility with distributed consensus |
| KD-004 | Use FastCDC over fixed-block chunking for LFS+ | Content-defined chunking provides superior deduplication for ML model weights where small changes produce large file diffs |
| KD-005 | Restrict unsafe code via `#![forbid(unsafe_code)]` in CivitCore | Memory safety is a primary market differentiator; allows HFT/Defense compliance claims |
| KD-006 | Implement custom DAG sync rather than using existing replication | Standard Git replication lacks Merkle-root negotiation and is synchronous; custom protocol enables async edge-friendly sync |
| KD-007 | Phase AI (CivitBrain) as Phase 3 rather than earlier | AI depends on stable VCS, storage, and CI/CD foundations; premature AI integration would block core forge functionality |

---

## 7. Next Phase Entry Criteria

Phase 0 (Requirements Engineering) entry is granted based on:

- [x] Domain boundaries defined and documented
- [x] Applicable standards identified and catalogued
- [x] Technical risks assessed with mitigation strategies
- [x] Scope boundaries established (in-scope, out-of-scope, deferred)
- [x] Key architectural decisions documented with rationale
- [x] Stakeholder analysis complete
- [x] Source materials (PRD, TRD, Architecture, Roadmap) reviewed and understood
