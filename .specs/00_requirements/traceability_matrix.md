# Traceability Matrix

**Document ID:** TM-001
**Revision:** 1.0
**Date:** 2026-05-30
**Scope:** Phase 0 (Requirements Engineering) — Initial Matrix

---

## Matrix Legend

| Column | Meaning |
|--------|---------|
| REQ-ID | Requirement identifier (from requirements.md EARS spec) |
| PRD-REF | Source requirement in original PRD |
| COMPONENT | Primary implementing component |
| SUB-COMPONENT | Specific module/crate responsible |
| TEST-TYPE | Test strategy (Unit, Integration, E2E, Property, Chaos) |
| TEST-ID | Planned test case identifier |
| STANDARD | Applicable compliance standard |
| PHASE | Roadmap phase for implementation |
| PRIORITY | Critical / High / Medium / Low |

---

## REQ-VCS: Version Control & Monorepo Engine

| REQ-ID | PRD-REF | COMPONENT | SUB-COMPONENT | TEST-TYPE | TEST-ID | STANDARD | PHASE | PRIORITY |
|--------|---------|-----------|---------------|-----------|---------|----------|-------|----------|
| REQ-VCS-001 | REQ-1.1 | CivitCore | civit-core (VFS gRPC) | Integration, E2E | T-VFS-001 | — | 1 | Critical |
| REQ-VCS-002 | REQ-1.1 | CivitCore | civit-vfs (FUSE daemon) | E2E, Performance | T-VFS-002 | — | 1 | High |
| REQ-VCS-003 | REQ-1.2 | CivitCore | civit-core (git engine) | Unit, Integration | T-GIT-001 | — | 1 | Critical |
| REQ-VCS-004 | REQ-1.2 | CivitCore | civit-core (pack engine) | Performance, Benchmark | T-GIT-002 | — | 1 | High |
| REQ-VCS-005 | REQ-1.2 | CivitCore | civit-core (crypto/auth) | Static Analysis (cargo-geiger) | T-GIT-003 | NIST SP 800-53 SC-12 | 1 | Critical |
| REQ-VCS-006 | REQ-1.3 | CivitCore | civit-core (VCS adapter) | Integration | T-VCS-003 | — | 4 | Low |
| REQ-VCS-007 | REQ-1.4 | CivitCore | civit-core (build graph) | Unit, Integration | T-BG-001 | — | 1 | High |
| REQ-VCS-008 | REQ-1.4 | CivitCore | civit-core (CI trigger) | Integration, E2E | T-BG-002 | — | 1 | High |
| REQ-VCS-009 | REQ-1.2 | CivitCore | civit-core (SSH server) | Integration, Security | T-SSH-001 | FIPS 140-2 | 1 | Critical |
| REQ-VCS-010 | REQ-1.2 | CivitCore | civit-core (auth) | E2E, Security | T-SSH-002 | NIST SP 800-53 AU-2, OWASP A07 | 1 | Critical |

## REQ-LFS: Big Data & Large File Management

| REQ-ID | PRD-REF | COMPONENT | SUB-COMPONENT | TEST-TYPE | TEST-ID | STANDARD | PHASE | PRIORITY |
|--------|---------|-----------|---------------|-----------|---------|----------|-------|----------|
| REQ-LFS-001 | REQ-2.1 | CivitData | civit-data (chunk engine) | Unit, Property | T-LFS-001 | — | 2 | Critical |
| REQ-LFS-002 | REQ-2.1 | CivitData | civit-data (S3 storage) | Integration | T-LFS-002 | ISO 27001 A.10.1 | 2 | Critical |
| REQ-LFS-003 | REQ-2.1 | CivitData | civit-data (delta engine) | Unit, Integration | T-LFS-003 | — | 2 | Critical |
| REQ-LFS-004 | REQ-2.1 | CivitData | civit-data (chunk engine) | Performance | T-LFS-004 | — | 2 | High |
| REQ-LFS-005 | REQ-2.2 | CivitCore | civit-core (OCI registry) | Integration, E2E | T-OCI-001 | OCI Distribution Spec | 2 | High |
| REQ-LFS-006 | REQ-2.3 | CivitRunner | civit-runner (CSI driver) | Integration, E2E | T-CSI-001 | K8s CSI Spec | 2 | Critical |
| REQ-LFS-007 | REQ-2.3 | CivitRunner | civit-runner (CSI driver) | Performance | T-CSI-002 | — | 2 | High |
| REQ-LFS-008 | REQ-2.1 | CivitData | civit-data (stream engine) | E2E, Performance | T-LFS-005 | — | 2 | High |

## REQ-CI: CI/CD & Secure Runner Ecosystem

| REQ-ID | PRD-REF | COMPONENT | SUB-COMPONENT | TEST-TYPE | TEST-ID | STANDARD | PHASE | PRIORITY |
|--------|---------|-----------|---------------|-----------|---------|----------|-------|----------|
| REQ-CI-001 | REQ-3.1 | CivitRunner | civit-runner (Podman sandbox) | Integration, Security | T-CI-001 | NIST SP 800-53 SC-7, OWASP A04 | 2 | Critical |
| REQ-CI-002 | REQ-3.1 | CivitRunner | civit-runner (Podman sandbox) | E2E, Security, Chaos | T-CI-002 | SLSA Level 4, NIST SP 800-53 AC-4 | 2 | Critical |
| REQ-CI-003 | REQ-3.2 | CivitRunner | civit-runner (K8s operator) | Integration, E2E | T-CI-003 | — | 2 | Critical |
| REQ-CI-004 | REQ-3.2 | CivitRunner | civit-runner (event consumer) | Integration, Performance | T-CI-004 | — | 2 | High |
| REQ-CI-005 | REQ-3.3 | CivitRunner | civit-runner (network policy) | Integration, Security | T-CI-005 | NIST SP 800-53 SC-7 | 2 | High |
| REQ-CI-006 | REQ-3.3 | CivitRunner | civit-runner (network policy) | E2E, Chaos | T-CI-006 | ISO 26262 (HFT analog) | 2 | High |
| REQ-CI-007 | REQ-3.4 | CivitRunner | civit-runner (SBOM/signer) | Integration | T-CI-007 | SLSA Level 4, NIST SP 800-53 SI-7 | 2 | Critical |
| REQ-CI-008 | REQ-3.4 | CivitRunner | civit-runner (provenance) | Integration, E2E | T-CI-008 | SLSA Level 4 | 2 | Critical |
| REQ-CI-009 | REQ-3.5 | CivitRunner | civit-runner (CDE operator) | E2E | T-CI-009 | — | 4 | Medium |
| REQ-CI-010 | REQ-3.5 | CivitRunner | civit-runner (CDE sync) | Integration | T-CI-010 | — | 4 | Medium |
| REQ-CI-011 | REQ-3.2 | CivitRunner | civit-runner (K8s scheduler) | Performance, Load | T-CI-011 | — | 2 | High |

## REQ-AI: Private AI & Agentic Workflows

| REQ-ID | PRD-REF | COMPONENT | SUB-COMPONENT | TEST-TYPE | TEST-ID | STANDARD | PHASE | PRIORITY |
|--------|---------|-----------|---------------|-----------|---------|----------|-------|----------|
| REQ-AI-001 | REQ-4.1 | CivitBrain | civit-brain (all) | Integration, E2E | T-AI-001 | NIST SP 800-53 SC-7, SOC2 | 3 | Critical |
| REQ-AI-002 | REQ-4.1 | CivitBrain | civit-brain (network guard) | Security, Chaos | T-AI-002 | NIST SP 800-53 SC-7, OWASP A05 | 3 | Critical |
| REQ-AI-003 | REQ-4.2 | CivitBrain | civit-brain (AST parser) | Unit, Integration | T-AI-003 | — | 3 | Critical |
| REQ-AI-004 | REQ-4.2 | CivitBrain | civit-brain (embedding worker) | Integration, Performance | T-AI-004 | — | 3 | High |
| REQ-AI-005 | REQ-4.2 | CivitBrain | civit-brain (RAG engine) | Performance, Benchmark | T-AI-005 | — | 3 | High |
| REQ-AI-006 | REQ-4.1 | CivitBrain | civit-brain (vLLM server) | Integration, E2E | T-AI-006 | — | 3 | Critical |
| REQ-AI-007 | REQ-4.2 | CivitBrain | civit-brain (review agent) | Performance, E2E | T-AI-007 | — | 3 | High |
| REQ-AI-008 | REQ-4.3 | CivitBrain | civit-brain (agent framework) | Integration, E2E | T-AI-008 | — | 3 | High |
| REQ-AI-009 | REQ-4.3 | CivitBrain | civit-brain (PR review agent) | E2E | T-AI-009 | — | 3 | High |
| REQ-AI-010 | REQ-4.3 | CivitBrain | civit-brain (sandbox guard) | Integration, Security | T-AI-010 | NIST SP 800-53 AC-4 | 3 | High |
| REQ-AI-011 | REQ-4.3 | CivitBrain | civit-brain (sandbox monitor) | E2E, Chaos | T-AI-011 | OWASP A08 | 3 | High |

## REQ-FED: Geo-Distributed High Availability (Federation)

| REQ-ID | PRD-REF | COMPONENT | SUB-COMPONENT | TEST-TYPE | TEST-ID | STANDARD | PHASE | PRIORITY |
|--------|---------|-----------|---------------|-----------|---------|----------|-------|----------|
| REQ-FED-001 | REQ-5.1 | CivitCore | civit-core (federation engine) | Integration, E2E | T-FED-001 | ISO 27001 A.12.1 | 4 | Critical |
| REQ-FED-002 | REQ-5.2 | CivitData | CockroachDB | Performance | T-FED-002 | — | 4 | High |
| REQ-FED-003 | REQ-5.1 | CivitCore | civit-core (federation engine) | Performance, E2E | T-FED-003 | — | 4 | High |
| REQ-FED-004 | REQ-5.1 | CivitCore | civit-core (conflict resolver) | Integration, Chaos | T-FED-004 | ISO 26262 (HFT analog) | 4 | Critical |
| REQ-FED-005 | REQ-5.1 | CivitCore | civit-core (DAG sync) | Integration, E2E | T-FED-005 | — | 4 | Critical |
| REQ-FED-006 | REQ-5.1 | CivitCore | civit-core (DAG sync) | Performance, Load | T-FED-006 | — | 4 | High |
| REQ-FED-007 | REQ-5.1 | CivitCore | civit-core (DAG sync) | Chaos, Network Partition | T-FED-007 | ISO 26262 (HFT analog) | 4 | Critical |
| REQ-FED-008 | REQ-5.2 | CivitData | civit-data (edge cache) | Integration, Performance | T-FED-008 | — | 4 | High |
| REQ-FED-009 | REQ-5.2 | CivitData | civit-data (edge cache) | Integration, E2E | T-FED-009 | — | 4 | Medium |
| REQ-FED-010 | REQ-5.2 | CivitData | civit-data (edge cache) | Performance | T-FED-010 | — | 4 | High |
| REQ-FED-011 | REQ-5.1 | CivitCore | civit-core (mTLS manager) | Integration, Security, Chaos | T-FED-011 | NIST SP 800-53 SC-12, SC-13 | 4 | Critical |
| REQ-FED-012 | REQ-5.1 | CivitCore | civit-core (ForgeFed engine) | Integration, E2E | T-FED-012 | ForgeFed Spec, ActivityPub | 4 | High |
| REQ-FED-013 | REQ-5.1 | CivitCore | civit-core (ForgeFed engine) | Security, E2E | T-FED-013 | OWASP A08, NIST SP 800-53 SI-4 | 4 | Critical |

## Non-Functional Requirements

| REQ-ID | COMPONENT | TEST-TYPE | TEST-ID | STANDARD | PHASE | PRIORITY |
|--------|-----------|-----------|---------|----------|-------|----------|
| NFR-SEC-001 | CivitCore | Static Analysis (cargo-geiger, clippy) | T-NFR-001 | NIST SP 800-53 | 1 | Critical |
| NFR-SEC-002 | CivitData | Integration, Security | T-NFR-002 | FIPS 140-2, ISO 27001 A.10.1 | 1 | Critical |
| NFR-SEC-003 | All | Security, Integration | T-NFR-003 | FIPS 140-2, NIST SP 800-53 SC-8 | 1 | Critical |
| NFR-SEC-004 | CivitCore | Integration, E2E | T-NFR-004 | SOC2, ISO 27001 A.12.4, NIST AU-3 | 1 | Critical |
| NFR-SEC-005 | CivitCore | Performance, Load | T-NFR-005 | NIST SP 800-53 AC-3 | 1 | High |
| NFR-SEC-006 | CivitCore | Integration, Security | T-NFR-006 | NIST SP 800-53 AC-4 | 4 | High |
| NFR-PERF-001 | CivitCore | Performance, Benchmark | T-NFR-007 | — | 1 | High |
| NFR-PERF-002 | CivitCore | Load, Chaos | T-NFR-008 | — | 4 | High |
| NFR-PERF-003 | CivitCore, CivitData | Performance, E2E | T-NFR-009 | — | 1 | High |
| NFR-PERF-004 | CivitCore | Load, Benchmark | T-NFR-010 | — | 2 | High |
| NFR-COMP-001 | All | Audit, Review | T-NFR-011 | SOC2 Type II | 4 | Critical |
| NFR-COMP-002 | All | Audit, Review | T-NFR-012 | ISO 27001 | 4 | High |
| NFR-COMP-003 | CivitCore | Integration, E2E | T-NFR-013 | FINRA/SEC | 4 | Critical |
| NFR-DEPLOY-001 | All | Integration, E2E | T-NFR-014 | — | 1 | High |
| NFR-DEPLOY-002 | All | Integration, E2E | T-NFR-015 | — | 1 | High |
| NFR-DEPLOY-003 | CivitBrain | Integration | T-NFR-016 | — | 3 | Medium |

---

## Coverage Summary

| Category | Total Requirements | Mapped to Component | Mapped to Test | Mapped to Standard | Coverage % |
|----------|-------------------|--------------------:|---------------:|-------------------:|-----------:|
| REQ-VCS | 10 | 10 | 10 | 3 | 100% |
| REQ-LFS | 8 | 8 | 8 | 1 | 100% |
| REQ-CI | 11 | 11 | 11 | 4 | 100% |
| REQ-AI | 11 | 11 | 11 | 3 | 100% |
| REQ-FED | 13 | 13 | 13 | 5 | 100% |
| NFR | 16 | 16 | 16 | 9 | 100% |
| **Total** | **69** | **69** | **69** | **25** | **100%** |

---

## Phase Distribution

| Phase | Requirements Assigned | Components Active |
|-------|----------------------|-------------------|
| Phase 1 (v0.1-v0.3) | 24 | CivitCore, CivitData |
| Phase 2 (v0.4-v0.6) | 19 | CivitRunner, CivitData |
| Phase 3 (v0.7-v0.9) | 11 | CivitBrain |
| Phase 4 (v1.0.0) | 15 | All |
| Cross-Phase (NFR) | 16 | All |
