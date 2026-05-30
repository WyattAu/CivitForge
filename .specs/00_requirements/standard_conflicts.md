# Standard & Requirement Conflicts

**Document ID:** SC-001
**Revision:** 1.0
**Date:** 2026-05-30
**Status:** Active
**Purpose:** Document identified tensions between standards, requirements, and architectural constraints

---

## Conflict Register

| ID | Tension | Severity | Affected Requirements | Status |
|----|---------|----------|----------------------|--------|
| SC-001 | SLSA L4 hermetic builds vs. air-gap dependency fetching | High | REQ-CI-005, REQ-CI-006, NFR-DEPLOY-001 | Resolution Proposed |
| SC-002 | Rootless Podman isolation vs. build performance | Medium | REQ-CI-001, REQ-CI-002, NFR-PERF-004 | Under Analysis |
| SC-003 | Federation eventual consistency vs. FINRA real-time audit | High | REQ-FED-001, REQ-FED-007, NFR-COMP-003 | Resolution Proposed |
| SC-004 | AI sandbox isolation vs. AI agent utility | Medium | REQ-AI-008, REQ-AI-010, REQ-AI-011 | Resolution Proposed |
| SC-005 | FIPS 140-2 crypto vs. Rust crypto ecosystem | High | REQ-VCS-009, NFR-SEC-002, NFR-SEC-003 | Under Analysis |
| SC-006 | VFS on-demand fetch vs. air-gap offline operation | Medium | REQ-VCS-001, REQ-VCS-002, NFR-DEPLOY-001 | Resolution Proposed |

---

## SC-001: SLSA Level 4 Hermetic Builds vs. Air-Gap Dependency Fetching

**Severity:** High
**Conflict Type:** Security vs. Operational

### Tension

SLSA Level 4 requires **hermetic builds** — build environments with zero network access to guarantee reproducibility (REQ-CI-005, REQ-CI-006). This prevents supply chain injection during builds. However, CivitForge targets **air-gapped deployments** (NFR-DEPLOY-001) where dependency registries are internal mirrors that may themselves need network access to periodically sync with upstream.

### Impact

- A fully hermetic build cannot pull dependencies during build time, even from internal mirrors
- Air-gapped environments must pre-position all dependencies, but the build environment's network isolation prevents even accessing the internal mirror
- SLSA provenance requires verifiable dependency sources, which complicates offline dependency management

### Resolution Strategy

Implement **two-tier dependency management**:

1. **Tier 1 — Air-Gap Registry Sync (Pre-Build):** A separate, non-hermetic sync process periodically (or on-demand) pulls approved dependencies from upstream into the internal registry mirror. This process runs outside the build pipeline and is itself SLSA-attested.
2. **Tier 2 — Hermetic Build with Local Mirror:** The build environment is configured with a `sources.list` / `cargo.toml` pointing to the local mirror. The hermetic network policy whitelows the internal registry endpoint only. The mirror is read-only during builds.
3. **SBOM Completeness:** The SBOM generator verifies that all resolved dependencies match the pre-approved manifest checksums.

### Verification

- Hermetic build produces identical artifacts when run multiple times (reproducibility test)
- Internal registry mirrors contain only approved, checksum-verified dependencies
- SBOM attests to exact dependency versions and sources

---

## SC-002: Rootless Podman Isolation vs. Build Performance

**Severity:** Medium
**Conflict Type:** Security vs. Performance

### Tension

Rootless Podman with user namespace mapping and strict Seccomp profiles (REQ-CI-001, REQ-CI-002) introduces measurable overhead compared to privileged container execution:

- User namespace mapping adds syscall overhead for UID/GID translation
- Seccomp BPF filtering adds per-syscall latency (typically 1-5 microseconds)
- `crun` (OCI runtime) rootless mode cannot leverage cgroups v2 features available to privileged containers
- Bazel/Buck2 builds spawning thousands of parallel compile processes are syscall-heavy

### Impact

- HFT customers require fast CI turnaround for trading algorithm changes
- A 20-30% performance penalty from rootless isolation may violate SLA for time-sensitive builds
- Security policy (REQ-CI-001) mandates rootless, but performance requirement (NFR-PERF-004) demands throughput

### Resolution Strategy

Implement **configurable Seccomp profiles by workload type**:

1. **Strict Profile (default):** Full Seccomp allowlist for standard build workloads. Acceptable ~10-15% overhead for most CI jobs.
2. **Relaxed Profile (trusted builds):** For Bazel/Buck2 builds in trusted repositories with code review gates, provide a broader Seccomp allowlist that reduces syscall filtering overhead to ~2-3%.
3. **gVisor Defense-in-Depth (optional):** For maximum isolation without Seccomp overhead, offer gVisor (user-space kernel) as an alternative runtime for sensitive workloads. gVisor shifts isolation to the application kernel level with different performance characteristics.
4. **Benchmark Gate:** Every pipeline type must pass a performance baseline test. If rootless isolation exceeds the SLA threshold, the pipeline is flagged for optimization review.

### Verification

- Rootless build performance within 15% of privileged baseline for standard workloads
- No container escape possible under strict or gVisor profiles
- Performance benchmarks automated and tracked per pipeline template

---

## SC-003: Federation Eventual Consistency vs. FINRA Real-Time Audit

**Severity:** High
**Conflict Type:** Architectural (CAP theorem) vs. Regulatory

### Tension

CivitForge uses eventual consistency for federation (REQ-FED-001, REQ-FED-007) — commits push to local edge nodes and replicate asynchronously. However, FINRA Rule 3110 and SEC Rule 17a-4 require **immutable, real-time audit trails** for algorithmic trading systems (NFR-COMP-003). If a developer pushes a change to a trading algorithm at the London edge node, FINRA requires an immediate, verifiable audit record — but the federation sync to the NY compliance node may take up to 60 seconds.

### Impact

- Eventual consistency conflicts with the perception of real-time regulatory audit
- Compliance officers may argue that a 60-second replication window creates an uncontrolled period where the NY node has an incomplete audit trail
- Split-brain scenarios during network partitions could create conflicting audit records

### Resolution Strategy

Implement **synchronous audit replication with asynchronous data replication**:

1. **WORM Audit Log (Synchronous):** Audit events are written to a separate, synchronously replicated CockroachDB table (or dedicated audit stream). This uses CockroachDB's consensus protocol to guarantee audit records are written to a quorum of nodes before acknowledging the push. Audit records include cryptographic hashes linking them to the source commit.
2. **Repository State (Asynchronous):** Git objects, LFS+ blobs, and repository metadata continue with eventual consistency via DAG sync. This preserves the low-latency push experience.
3. **Partition Handling:** During network partitions, audit records from isolated nodes are queued and replayed upon reconnection with original timestamps preserved. No audit records are lost.
4. **Compliance View:** A read-only compliance endpoint queries the synchronous audit log, providing regulators with a consistent, real-time view of all operations regardless of repository state consistency.

### Verification

- Audit record acknowledged only after synchronous quorum write
- Compliance endpoint returns consistent audit trail across all nodes within 100ms
- Zero audit record loss in partition simulation tests (Chaos engineering)

---

## SC-004: AI Sandbox Isolation vs. AI Agent Utility

**Severity:** Medium
**Conflict Type:** Security vs. Functionality

### Tension

AI agents must be strictly sandboxed to prevent prompt injection attacks from executing arbitrary code on the host (REQ-AI-010, REQ-AI-011). However, the AI agent's primary utility is **writing and executing code** — generating unit tests, running fixes, and verifying behavior in sandboxes (REQ-AI-008). Overly restrictive isolation prevents the agent from being useful; insufficient isolation creates a security vulnerability.

### Impact

- A compromised or prompt-injected AI agent could exfiltrate repository data via the sandbox
- An overly restricted sandbox cannot compile, test, or execute code — rendering AI agents useless
- Resource limits must be enforced without breaking legitimate long-running test suites

### Resolution Strategy

Implement **capability-based sandbox with output control**:

1. **Sandbox Capability Set:** Each AI agent request declares required capabilities (compile, test, network-mirror-access). The sandbox is provisioned with only the declared capabilities.
2. **Output Validation:** All agent output (file writes, PR comments, API calls) passes through an output validation layer that checks for policy violations (e.g., no writes outside assigned scope, no external network calls).
3. **Deterministic Replay:** Agent sandbox executions are recorded (stdin, stdout, stderr, file changes) and can be replayed for audit. The recording is cryptographically signed and stored as part of the agent's review evidence.
4. **Progressive Trust:** AI agents start with restricted capabilities. Successful, policy-compliant executions earn progressive trust (wider scope, longer timeouts) up to configured maximums. Failed policy checks reset the trust level.

### Verification

- Agent cannot access files outside PR scope (tested with adversarial prompts)
- Agent cannot initiate outbound network connections (tested with network monitoring)
- Agent execution replay produces identical results (determinism test)
- Trust level progression is logged and auditable

---

## SC-005: FIPS 140-2 Crypto vs. Rust Crypto Ecosystem

**Severity:** High
**Conflict Type:** Compliance vs. Technology Choice

### Tension

FIPS 140-2 requires using validated cryptographic modules. The Rust ecosystem primarily uses `ring` for cryptography, which is **not FIPS 140-2 validated**. The `boring` crate wraps BoringSSL (Google's FIPS-validated fork of OpenSSL), but introduces C-bindings — which contradicts CivitForge's `#![forbid(unsafe_code)]` policy (NFR-SEC-001) and the choice of gitoxide over libgit2 to avoid C-bindings.

### Impact

- Federal and Defense deployments require FIPS-validated crypto
- Using `boring` crate introduces C code into the build, conflicting with the pure-Rust security narrative
- `ring` cannot be used in FIPS-compliant environments
- Some CI environments (HFT) may not require FIPS but benefit from it

### Resolution Strategy

Implement **feature-gated cryptographic backend selection**:

1. **Default (Non-FIPS):** Use `ring` for all cryptographic operations. Pure Rust, no C-bindings, `#![forbid(unsafe_code)]` maintained. Suitable for most commercial and open-source deployments.
2. **FIPS Feature Flag (`--features fips`):** When enabled, swap cryptographic operations to use the `boring` crate (BoringSSL). This feature flag:
   - Enables `unsafe` blocks in a dedicated `civit-crypto` crate (isolated from CivitCore business logic)
   - Uses FIPS-validated BoringSSL module
   - Is available only for builds targeting FIPS-compliant environments
3. **Architecture Isolation:** All FIPS-related C-bindings are confined to `civit-crypto` crate with a trait-based abstraction layer. CivitCore depends on `civit-crypto`'s public trait interface, never directly on `boring` or `ring`.
4. **Future Path:** Monitor the RustCrypto ecosystem for a FIPS-validated pure-Rust crypto module. If one becomes available, migrate and deprecate the `boring` backend.

### Verification

- Default build contains zero `unsafe` blocks in CivitCore and zero C-dependencies
- FIPS build passes FIPS module self-test on module initialization
- Cryptographic operations produce identical results across both backends (interoperability test)
- Feature-gated builds are tested in CI (matrix: default + fips)

---

## SC-006: VFS On-Demand Fetch vs. Air-Gap Offline Operation

**Severity:** Medium
**Conflict Type:** Performance vs. Operational

### Tension

The VFS client fetches Git objects on-demand via gRPC from the server (REQ-VCS-001). This design assumes a network connection between the developer's workstation and the forge. Air-gapped deployments (NFR-DEPLOY-001) may have workstations with no network access to the forge, or the forge itself may be disconnected from the internet.

### Impact

- Developers in air-gapped environments cannot use VFS if it requires continuous server connectivity
- Large monorepo full clones defeat the purpose of VFS (which exists to avoid 10TB downloads)
- Air-gap transfers (tarballs) must be designed to support partial repository data

### Resolution Strategy

Implement **VFS with pre-positioned local cache and policy-based eager loading**:

1. **Pre-Positioned Cache:** During air-gap transfer, the forge packages not just the full repository but also a **VFS metadata cache** — containing all tree objects, commit metadata, and file path listings. This allows VFS to mount and browse the full directory tree offline.
2. **Eager Loading Policy:** Administrators configure per-directory eager loading rules (e.g., "preload all `*.rs` files in `src/core/`" or "preload files modified in the last 30 days"). During network-available periods, the VFS client downloads these files into a local LRU cache.
3. **Offline Mode:** When the forge is unreachable, VFS operates in read-only mode from the local cache. Write operations (commits) are queued locally and pushed when connectivity is restored.
4. **Transfer Optimization:** Air-gap tarballs use the same FastCDC LFS+ chunking, enabling incremental delta transfers rather than full 10TB transfers for each update cycle.

### Verification

- VFS mount completes within 30 seconds using pre-positioned metadata cache (no network)
- Offline mode allows code browsing and search from local cache
- Queued offline commits replay successfully upon reconnection
- Incremental air-gap transfers contain only changed chunks (delta test)

---

## Conflict Interaction Matrix

| Conflict | SC-001 | SC-002 | SC-003 | SC-004 | SC-005 | SC-006 |
|---------|--------|--------|--------|--------|--------|--------|
| SC-001 (SLSA vs Air-Gap) | — | Independent | **Related** — both affect air-gapped deployments | Independent | Independent | **Related** — both affect air-gapped deployments |
| SC-002 (Rootless vs Perf) | Independent | — | Independent | **Related** — both affect sandbox performance | Independent | Independent |
| SC-003 (Consistency vs FINRA) | **Related** | Independent | — | Independent | Independent | Independent |
| SC-004 (AI Sandbox vs Utility) | Independent | **Related** | Independent | — | Independent | Independent |
| SC-005 (FIPS vs Rust) | Independent | Independent | Independent | Independent | — | Independent |
| SC-006 (VFS vs Air-Gap) | **Related** | Independent | Independent | Independent | Independent | — |

### Related Conflict Groups

1. **Air-Gap Group (SC-001 + SC-003 + SC-006):** All three conflicts involve tension between network-free operation and various requirements (build hermeticity, audit consistency, VFS on-demand). Resolution must be coordinated to avoid contradicting strategies.
2. **Sandbox Group (SC-002 + SC-004):** Both involve trade-offs between isolation overhead and functional capability in containerized environments. Resolution strategies should share a common configurable-isolation framework.
