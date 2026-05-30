# Phase 0: Requirements Engineering Report

**Document ID:** RPT-REQ-001
**Revision:** 1.0
**Date:** 2026-05-30
**Status:** Complete
**Phase:** 0 (Requirements Engineering)

---

## 1. Executive Summary

Phase 0 (Requirements Engineering) converted all 21 functional requirements from the CivitForge PRD into 69 formally specified requirements in EARS (Easy Approach to Requirements Syntax) format, augmented by 16 non-functional requirements. A full traceability matrix maps every requirement to implementing components, planned test cases, applicable compliance standards, and roadmap phases. Initial standard conflict analysis identified 6 areas of tension requiring architectural resolution decisions.

### Key Metrics

| Metric | Value |
|--------|-------|
| Functional Requirements (EARS) | 69 |
| Non-Functional Requirements | 16 |
| Requirement Areas (REQ-*) | 5 (VCS, LFS, CI, AI, FED) |
| Standards Mapped | 7 primary, 6 conditional |
| Components Specified | 5 (CivitCore, CivitData, CivitRunner, CivitBrain, CivitVFS) |
| Test Cases Planned | 69 |
| Requirements with Standard Traceability | 25 (36%) |
| Standard Conflicts Identified | 6 |

---

## 2. Requirements Elicitation Summary

### 2.1 Source Documents

| Document | Version | Requirements Extracted |
|----------|---------|----------------------|
| requirements.md (PRD) | 1.0 | 21 functional, ~10 non-functional |
| tech_requirements.md (TRD) | 1.0.0 | Technical constraints, performance targets |
| architecture.md | 1.0 | Component boundaries, data flows |
| roadmap.md | 1.0 | Phase allocation, priority ordering |
| README.md | 1.0 | Feature overview, license terms |

### 2.2 EARS Conversion Statistics

| EARS Pattern | Count | Percentage |
|--------------|------|-----------|
| Ubiquitous (shall) | 34 | 49% |
| Event-Driven (when) | 12 | 17% |
| Unwanted Behaviour (if...then) | 11 | 16% |
| State-Driven (while) | 5 | 7% |
| Optional Feature (where) | 1 | 1% |
| Performance (measurable shall) | 6 | 9% |

### 2.3 Requirement Expansion

The original 21 PRD requirements were decomposed and expanded into 69 EARS requirements. The expansion rationale:

| Original Requirement | Expanded To | Expansion Reason |
|--------------------|-----------|------------------|
| REQ-1.1 (VFS) | REQ-VCS-001, VCS-002 | Separated functional mount requirement from performance SLA |
| REQ-1.2 (Git Backend) | REQ-VCS-003, VCS-004, VCS-005, VCS-009, VCS-010 | Split into: library choice, performance, safety, SSH, auth failure handling |
| REQ-1.4 (Build Graph) | REQ-VCS-007, VCS-008 | Separated graph storage from event-driven CI trigger |
| REQ-2.1 (Block Dedup) | REQ-LFS-001, LFS-002, LFS-003, LFS-004, LFS-008 | Split into: algorithm, storage, delta, efficiency, streaming |
| REQ-2.3 (Data Gravity) | REQ-LFS-006, LFS-007 | Separated CSI mount capability from performance SLA |
| REQ-3.1 (Rootless) | REQ-CI-001, CI-002 | Separated rootless execution from violation handling |
| REQ-3.2 (K8s) | REQ-CI-003, CI-004, CI-011 | Split into: operator, event trigger, scheduling throughput |
| REQ-3.3 (Hermetic) | REQ-CI-005, CI-006 | Separated isolation policy from state enforcement |
| REQ-3.4 (Provenance) | REQ-CI-007, CI-008 | Separated SBOM/signing from SLSA attestation generation |
| REQ-3.5 (CDE) | REQ-CI-009, CI-010 | Separated CDE provisioning from active session sync |
| REQ-4.1 (Air-Gap) | REQ-AI-001, AI-002 | Separated air-gap deployment from outbound connection blocking |
| REQ-4.2 (RAG) | REQ-AI-003, AI-004, AI-005 | Split into: indexing, event-driven update, performance |
| REQ-4.3 (Agents) | REQ-AI-008, AI-009, AI-010, AI-011 | Split into: capability, PR trigger, failure handling, sandbox limits |
| REQ-5.1 (Multi-Master) | REQ-FED-001 through FED-007 | Full expansion: read/write, latency, conflict, DAG, partition, sync protocol |
| REQ-5.2 (Edge Cache) | REQ-FED-008, FED-009, FED-010 | Split into: caching, miss handling, performance |
| (New - implicit) | REQ-FED-011, FED-012, FED-013 | Added: mTLS identity, ForgeFed protocol, message validation |

---

## 3. Traceability & Coverage Analysis

### 3.1 Component Coverage

| Component | Requirements Assigned | Percentage |
|-----------|----------------------|-----------|
| CivitCore | 28 | 41% |
| CivitData | 13 | 19% |
| CivitRunner | 19 | 28% |
| CivitBrain | 17 | 25% |
| All (NFR) | 16 | Cross-cutting |

### 3.2 Standard Coverage

| Standard | Requirements Traceable | Compliance Status |
|----------|----------------------|-------------------|
| SLSA Level 4 | REQ-CI-007, CI-008, CI-002, NFR-SEC-001 | Mapped; implementation pending Phase 2 |
| ISO/IEC 27001 | 8 requirements mapped | Mapped; implementation across all phases |
| NIST SP 800-53 | 12 requirements mapped | Mapped; implementation across all phases |
| FIPS 140-2 | REQ-VCS-009, NFR-SEC-002, NFR-SEC-003 | Mapped; requires boring crate evaluation |
| OWASP Top 10 | 6 requirements mapped | Mapped; implementation across all phases |
| ISO 26262 (HFT analog) | REQ-CI-006, REQ-FED-004, FED-007 | Mapped; conditional on HFT deployment |
| SOC 2 Type II | NFR-COMP-001, REQ-AI-001 | Mapped; evidence generation begins Phase 4 |

### 3.3 Test Coverage

All 69 requirements have at least one planned test case. Test strategy distribution:

| Test Type | Count | Percentage |
|-----------|-------|-----------|
| Unit | 8 | 12% |
| Integration | 34 | 49% |
| E2E | 24 | 35% |
| Performance/Benchmark | 15 | 22% |
| Security | 14 | 20% |
| Chaos (Partition/Failure) | 6 | 9% |
| Property-Based | 2 | 3% |
| Load | 5 | 7% |

Note: Test types are not mutually exclusive — many requirements map to multiple test types.

---

## 4. Standard Conflict Analysis

Six areas of tension between standards and requirements were identified (see `.specs/00_requirements/standard_conflicts.md`):

| Conflict ID | Tension | Severity | Resolution Strategy |
|-------------|---------|----------|-------------------|
| SC-001 | SLSA L4 hermetic builds vs. air-gap dependency fetching | High | Staged builds: hermetic inner layer, air-gap-approved registry mirror |
| SC-002 | Rootless Podman isolation vs. build performance (Seccomp overhead) | Medium | Configurable Seccomp profiles; benchmark and profile per workload |
| SC-003 | Federation eventual consistency vs. FINRA real-time audit requirements | High | WORM audit log replicated synchronously; repository state async |
| SC-004 | AI agent sandbox isolation vs. AI agent utility (needs code execution) | Medium | Capability-based sandbox: read repo + execute in isolated Podman pod |
| SC-005 | FIPS 140-2 crypto vs. Rust crypto ecosystem (ring vs. boring) | High | Gate FIPS compliance behind feature flag; use boring crate for FIPS deployments |
| SC-006 | VFS on-demand fetch latency vs. offline/air-gap operation | Medium | Pre-fetch manifests in air-gap transfers; background eager loading per policy |

---

## 5. Phase Allocation

| Phase | Requirements | Components Active | Dependencies |
|-------|-------------|-------------------|-------------|
| Phase 1 (v0.1-v0.3) | 24 REQ + 6 NFR | CivitCore, CivitData | None (foundation) |
| Phase 2 (v0.4-v0.6) | 19 REQ + 3 NFR | CivitRunner, CivitData | Phase 1 complete |
| Phase 3 (v0.7-v0.9) | 11 REQ + 2 NFR | CivitBrain | Phase 1, Phase 2 CI |
| Phase 4 (v1.0.0) | 15 REQ + 5 NFR | All | All prior phases |

---

## 6. Open Issues & Decisions Deferred

| Issue ID | Description | Decision Needed By | Owner |
|----------|-------------|--------------------|-------| 
| OID-001 | FIPS 140-2 vs FIPS 140-3 applicability — FIPS 140-3 is now the active standard | Phase 1 start | Security Architecture |
| OID-002 | Final model selection for AI tiering (Llama-3-8B confirmed; DeepSeek-Coder-33B vs alternatives) | Phase 3 start | CivitBrain WG |
| OID-003 | CockroachDB vs TiDB final selection (TRD lists both; CockroachDB preferred) | Phase 1 start | Data Architecture |
| OID-004 | Boring crate maintenance status and FIPS certification timeline | Phase 1 start | Security Architecture |
| OID-005 | ForgeFed protocol version alignment — specification is evolving | Phase 4 start | Federation WG |

---

## 7. Phase 0 Completion Checklist

- [x] All PRD functional requirements converted to EARS format
- [x] Non-functional requirements formalized with measurable criteria
- [x] Traceability matrix established (REQ -> Component -> Test -> Standard)
- [x] Standard conflict analysis documented
- [x] Requirements allocated to roadmap phases
- [x] Open issues and deferred decisions catalogued
- [x] Domain analysis completed (Phase -1 artifact reference)
- [x] Specification files committed to `.specs/00_requirements/`
- [x] Reports committed to `.reports/`

---

## 8. Phase 1 Entry Criteria

Phase 1 (Rust Core & VFS Foundation) entry is granted based on:

- [x] All Phase 1 requirements (REQ-VCS-001 through VCS-010, REQ-VCS-007, VCS-008, NFR-SEC-001 through SEC-004, NFR-PERF-001, PERF-003, NFR-DEPLOY-001, DEPLOY-002) are formally specified and traceable
- [x] Component boundaries for CivitCore and CivitData are defined
- [x] Technical risks with Phase 1 impact have mitigation strategies
- [x] Standard conflicts affecting Phase 1 (SC-005: FIPS crypto) have resolution direction
- [x] Open issues blocking Phase 1 (OID-001, OID-003, OID-004) have owners and deadlines
