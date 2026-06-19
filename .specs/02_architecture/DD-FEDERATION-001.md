# DD-FEDERATION-001: Multi-Region Federation Design

**Status:** Draft
**Author:** Architecture Team
**Date:** 2026-06-19
**Supersedes:** N/A
**Depends On:** DD-FEDERATION-000 (Current Single-Instance Federation)

---

## Table of Contents

1. [Executive Summary](#1-executive-summary)
2. [Federation Protocol](#2-federation-protocol)
3. [Multi-Master Replication](#3-multi-master-replication)
4. [Region Architecture](#4-region-architecture)
5. [Data Synchronization](#5-data-synchronization)
6. [Consistency Model](#6-consistency-model)
7. [Conflict Resolution](#7-conflict-resolution)
8. [Implementation Plan](#8-implementation-plan)
9. [Security Considerations](#9-security-considerations)
10. [Risk Assessment](#10-risk-assessment)
11. [Success Metrics](#11-success-metrics)

---

## 1. Executive Summary

### 1.1 Problem

Single-region deployment of CivitForge limits geographic latency and availability for globally distributed users. Operations such as repository creation, issue tracking, pull request reviews, and CI/CD pipeline triggers exhibit unacceptable latency (>200ms) for users distant from the primary deployment region. A single-region architecture also constitutes a single point of failure: regional outages, cloud provider incidents, or network partitions result in complete service unavailability.

### 1.2 Solution

Multi-region federation leveraging CRDT-based (Conflict-free Replicated Data Type) conflict resolution. Each region operates an independent CivitForge instance with a local database, participating in a multi-master replication topology. Inter-region communication uses ActivityPub as the transport layer with ForgeFed extensions for forge-specific semantics. Vector clocks provide causality tracking; per-entity CRDT types ensure automatic, deterministic conflict resolution without coordination.

### 1.3 Target

| Metric | Target |
|--------|--------|
| Local read latency | <100ms (p99) |
| Inter-region replication lag | <5s (bounded) |
| System availability | 99.99% |
| Conflict resolution time | <1s (automatic) |

### 1.4 Current State Assessment

The existing federation implementation (`crates/civit-core/src/federation/`) provides foundational infrastructure:

- **ActivityPub transport:** `activitypub.rs` defines `Actor`, `Activity`, `ActivityObject`, and `InboxHandler` with basic validation. The `process_incoming` method handles Create, Update, Delete, Follow, Accept, Reject activity types.
- **ForgeFed extensions:** `forgefed.rs` implements `ForgeFedActivity` enum covering CreateRepository, ForkRepository, StarRepository, FollowUser, CreateIssue, ReviewPullRequest, CreatePullRequest, Comment, Like, Accept, Reject, Undo. Includes `IdempotencyTracker` using `DashMap` with time-based cleanup and `ForgeFedProcessor` with SHA-256 activity ID generation.
- **Vector clocks:** `vector_clock.rs` implements generic `VectorClock<V>` with `increment`, `happened_before`, `is_concurrent`, `merge`, and `descends_from` operations.
- **Inbox/Outbox:** `inbox_outbox.rs` provides `InboxProcessor` with idempotency, retry (configurable max_retries), and `OutboxProcessor` with exponential/fixed backoff strategies and jitter support.
- **Multi-master:** `multimaster.rs` defines `ConflictStrategy` (LWW, FWW, Merge, Manual, None), `ConflictResolution` with merge logic, `IncrementalSyncEngine` with checkpoint management, `PartitionTracker`, `BandwidthOptimizer`, and `DeltaCompressor` for binary diff.
- **Replication:** `replication.rs` implements `RegionId`, `ReplicationMessage`, `ReplicationPayload` (SyncDelta, FullSnapshot, Heartbeat, PartitionNotice), `ReplicationPeer` with async health tracking, `ReplicationTransport` with heartbeat and health monitor loops.
- **Sync:** `sync.rs` provides `DagSyncEngine` for DAG-based sync topology with BFS path finding and cycle detection.
- **Delivery:** `delivery.rs` implements `FederationDeliveryService` with WebFinger resolution, Ed25519 HTTP Signatures, actor caching, and batch delivery with retry.
- **HTTP Signatures:** `http_signatures.rs` supports Ed25519, RSA-SHA256, ECDSA-P256, HMAC-SHA256 algorithms with LD signature verification.

Gaps identified: no per-entity CRDT implementations, no region coordinator, no region discovery mechanism, no sync protocol for full/incremental/repair modes, no compression for bulk transfers, and no region-level access control.

---

## 2. Federation Protocol

### 2.1 Transport Layer: ActivityPub

CivitForge uses ActivityPub (W3C Recommendation, 2018) as the federation transport layer. ActivityPub provides:

- **Actor model:** Each region instance is an `Application` actor. Users within a region are represented as `Person` actors. The existing `Actor` struct in `activitypub.rs` already defines the required fields: `id`, `type`, `preferred_username`, `inbox`, `outbox`, `public_key`, `endpoints`.
- **Activity vocabulary:** Standard activity types (Create, Update, Delete, Follow, Undo, Accept, Reject, Add, Like, Announce) are already defined in the `ActivityType` enum. Custom ForgeFed activity types are expressed through the `ForgeFedActivity` enum in `forgefed.rs`.
- **Inbox/Outbox pattern:** Each region exposes an inbox endpoint for receiving activities and an outbox for sending. The existing `InboxProcessor` and `OutboxProcessor` in `inbox_outbox.rs` implement queue management with idempotency, retry, and backoff.
- **Collection model:** Followers/Following collections per actor, public inbox for unicast delivery, shared inbox for multicast delivery to all followers.

**Protocol compliance requirements:**

| Requirement | Status |
|-------------|--------|
| JSON-LD context | To implement |
| Actor endpoints | Implemented |
| Activity validation | Implemented |
| Collection management | To implement |
| WebFinger discovery | Implemented (referenced in `webfinger.rs`) |

### 2.2 ForgeFed Extensions

ForgeFed extends ActivityPub with forge-specific semantics. The existing `ForgeFedActivity` enum in `forgefed.rs` (lines 12-70) already defines the required activity types. Extensions beyond the current implementation:

**Repository operations:**
- `CreateRepository` — includes repository metadata (name, description, visibility, default branch)
- `ForkRepository` — references source and target repository URIs
- `ArchiveRepository` — new activity type for repository archival
- `TransferRepository` — new activity type for ownership transfer

**Collaboration operations:**
- `CreateIssue` — includes title, body, labels, assignees, milestone references
- `CreatePullRequest` — includes source/target branches, diff metadata, merge strategy
- `ReviewPullRequest` — includes review state (Approved, ChangesRequested, Comment), inline comments
- `Comment` — generic commenting on any entity (issue, PR, commit, review)

**Social operations:**
- `StarRepository` — repository starrer graph
- `FollowUser` — user follow graph
- `Like` — reaction to any entity

**Moderation operations:**
- `Accept` / `Reject` — follow request acceptance
- `Undo` — reversal of any prior activity

### 2.3 Authentication: HTTP Signatures

All inter-region communication is authenticated using HTTP Signatures (draft-cavage-http-signatures). The existing implementation in `http_signatures.rs` supports:

- **Ed25519:** Primary algorithm for inter-region communication. Uses `ring::signature::Ed25519KeyPair` for signing and `UnparsedPublicKey` for verification. The `generate_ed25519_keypair()` function produces PKCS#8 DER-encoded private keys and 32-byte public keys.
- **Required signed headers:** `(request-target)`, `host`, `date`, `digest` (as configured in `HttpSigningConfig::default()`).
- **Digest computation:** SHA-256 with base64 encoding, applied to the request body. Implemented in `delivery.rs:compute_digest()`.
- **Signature expiry:** Configurable via `expires_in_secs` (default 300 seconds). Signatures exceeding this window are rejected by `verify_http_signature`.
- **LD Signatures:** For ActivityPub object-level signatures, `verify_ld_signature` validates Ed25519Signature2020 proofs on JSON-LD documents.

**Inter-region authentication flow:**

1. Region A constructs ActivityPub activity with ForgeFed-specific payload
2. Activity is serialized to JSON and SHA-256 digest is computed
3. HTTP Signature is computed over `(request-target)`, `host`, `date`, `digest` headers using Ed25519 private key
4. Request is POSTed to Region B's inbox with `Content-Type: application/activity+json`
5. Region B verifies signature against Region A's published public key
6. Activity is processed and applied with CRDT conflict resolution

---

## 3. Multi-Master Replication

### 3.1 CRDT Type Selection per Entity

Each entity type is mapped to a CRDT type that provides the appropriate convergence semantics:

| Entity | CRDT Type | Rationale |
|--------|-----------|-----------|
| Repository metadata | Last-Writer-Wins Register (LWW) | Single authoritative state; latest write wins by timestamp + region tiebreaker |
| Repository stars | OR-Set (Observed-Remove Set) | Add/remove semantics; concurrent add and remove converge to add (add-wins) |
| Issues | OR-Set | Issues can be created and closed concurrently; add-wins semantics for concurrent create |
| Pull Requests | OR-Set | Same semantics as issues |
| PR Reviews | LWW Register per reviewer | Each reviewer's review is independent; latest review per reviewer wins |
| Comments | Sequence CRDT (RGA) | Ordering matters; concurrent comments from different regions are merged preserving causal order |
| Pipelines | Add-Wins Set | Pipeline definitions are additive; deletion requires explicit removal |
| Labels | OR-Set | Labels can be added/removed concurrently |
| Milestones | LWW Register | Single authoritative state |

### 3.2 Vector Clocks for Causality Tracking

The existing `VectorClock<V>` implementation in `vector_clock.rs` provides:

- **Structure:** `HashMap<V, u64>` mapping region identifiers to monotonically increasing counters.
- **`increment(node)`:** Increments the counter for a given region on each local write.
- **`happened_before(other)`:** Returns `true` if all entries in `self` are <= `other` (and at least one is strictly less). Used to determine causal ordering.
- **`is_concurrent(other)`:** Returns `true` if neither clock happened before the other. Triggers CRDT conflict resolution.
- **`merge(other)`:** Takes element-wise maximum of both clocks. Applied after conflict resolution.
- **`descends_from(other)`:** Returns `true` if all entries in `other` are <= `self`. Used for sync protocol validation.

**Usage in replication messages:**

Each `SyncDeltaEntry` (defined in `replication.rs:47-53`) carries a `vector_clock: HashMap<RegionId, u64>`. When a delta arrives, the receiving region compares vector clocks:

1. If `remote_vc.happened_before(local_vc)` — delta is stale, skip
2. If `local_vc.happened_before(remote_vc)` — delta is causally newer, apply
3. If `local_vc.is_concurrent(remote_vc)` — concurrent writes, apply CRDT conflict resolution

### 3.3 Conflict Resolution Strategies

The existing `ConflictStrategy` enum in `multimaster.rs:37-43` defines:

```rust
pub enum ConflictStrategy {
    LastWriteWins,
    FirstWriteWins,
    Merge,
    Manual,
    None,
}
```

Per-entity strategy mapping:

| Entity | Default Strategy | Fallback | Notes |
|--------|-----------------|----------|-------|
| Repository metadata | LastWriteWins | Manual | Latest timestamp wins; manual for semantic conflicts |
| Stars | OR-Set (implicit) | — | Automatic convergence; no manual intervention needed |
| Issues | OR-Set (implicit) | — | Concurrent creates produce duplicate issues; deduplication via content hash |
| Pull Requests | OR-Set (implicit) | — | Same as issues |
| PR Reviews | LastWriteWins (per reviewer) | — | Each reviewer's review tracked independently |
| Comments | Sequence CRDT | — | No conflict; concurrent comments merged in causal order |
| Pipelines | Add-Wins Set | — | Additive only; deletion requires coordination |
| Labels | OR-Set (implicit) | — | Automatic convergence |

**Manual resolution trigger conditions:**

- Repository metadata conflict where both regions changed the same field (e.g., description) with different values
- Pull request state conflict where one region merged and another closed
- Issue assignment conflict where different users assigned to different people

Manual resolution is presented to the user via a "conflict resolution required" notification in the UI, with both versions displayed side-by-side.

---

## 4. Region Architecture

### 4.1 Region Topology

Each region operates an independent CivitForge instance with:

- **Local database:** PostgreSQL (or compatible) with full schema and indexes
- **Local git storage:** Object storage (S3-compatible or local filesystem)
- **Federation stack:** ActivityPub + ForgeFed processing, CRDT conflict resolution
- **Region coordinator:** Manages peer awareness, health monitoring, replication scheduling

### 4.2 Region Coordinator

The region coordinator is a new component responsible for:

**Peer management:**
- Maintains a registry of known regions with their endpoints, health status, and replication state
- Uses `ReplicationPeer` (from `replication.rs:62-101`) for per-peer health tracking with async `RwLock<bool>` for healthy state and `Option<DateTime<Utc>>` for last_seen
- Runs `health_monitor` (from `replication.rs:278-309`) to detect unhealthy peers based on heartbeat timeout

**Heartbeat protocol:**
- Sends periodic `ReplicationPayload::Heartbeat` messages to all healthy peers
- Interval: 10 seconds (configurable)
- Heartbeat timeout: 30 seconds (3 missed heartbeats)
- Implemented via `heartbeat_loop` (from `replication.rs:240-276`)

**Replication scheduling:**
- Drains pending entries from `OutboxProcessor` at configurable batch intervals
- Prioritizes replication messages by entity type (metadata before activity)
- Coordinates with `BandwidthOptimizer` (from `multimaster.rs:283-340`) for compression decisions

**Partition detection and recovery:**
- Uses `PartitionTracker` (from `multimaster.rs:220-281`) to record detected partitions
- Sends `ReplicationPayload::PartitionNotice` to notify other regions of partitioned peers
- On partition heal, triggers full sync for affected entities

### 4.3 Region Discovery

Two discovery mechanisms, configurable per deployment:

**DNS SRV Records:**
```
_civitforge._tcp.federation.example.com.  IN  SRV  10 60 9090 region-us-east.federation.example.com.
_civitforge._tcp.federation.example.com.  IN  SRV  10 60 9090 region-eu-west.federation.example.com.
_civitforge._tcp.federation.example.com.  IN  SRV  10 60 9090 region-ap-south.federation.example.com.
```

**Configuration file:**
```toml
[federation]
region_id = "us-east"
regions = [
  { id = "eu-west", endpoint = "https://eu-west.civitforge.internal:9090" },
  { id = "ap-south", endpoint = "https://ap-south.civitforge.internal:9090" },
]
discovery_method = "config"  # or "dns"
dns_service_name = "_civitforge._tcp.federation.example.com"
```

**Environment variables:**
- `FEDERATION_REGION_ID` — unique identifier for this region
- `FEDERATION_REGIONS` — comma-separated list of peer region endpoints
- `FEDERATION_DISCOVERY_METHOD` — `config` or `dns`

---

## 5. Data Synchronization

### 5.1 Outbox Pattern

All writes are local-first with async replication:

1. **Local write:** Activity is committed to local database with vector clock incremented for local region
2. **Outbox enqueue:** Activity is serialized to `FederatedActivity` and enqueued to `OutboxProcessor` (from `inbox_outbox.rs:246-402`) with target instance
3. **Delivery:** `FederationDeliveryService` (from `delivery.rs:136-422`) drains pending entries, resolves inbox URLs via WebFinger, signs requests with Ed25519, and POSTs to remote inboxes
4. **Retry:** Failed deliveries use exponential backoff with jitter (from `BackoffStrategy::compute_delay_with_jitter`, `inbox_outbox.rs:92-105`)

**Outbox delivery flow:**

```
Local DB Write
    |
    v
Increment Vector Clock
    |
    v
Serialize to FederatedActivity
    |
    v
Enqueue to OutboxProcessor (per target region)
    |
    v
FederationDeliveryService.deliver_batch()
    |
    +---> Resolve inbox URL (WebFinger + cache)
    +---> Compute SHA-256 digest
    +---> Sign with Ed25519 HTTP Signature
    +---> HTTP POST to remote inbox
    +---> Mark delivered / mark failed (with backoff)
```

### 5.2 Inbox Processing

Remote activities are received and processed:

1. **Receive:** `InboxProcessor::receive()` (from `inbox_outbox.rs:141-161`) adds activity to queue with idempotency key (SHA-256 hash of activity JSON)
2. **Idempotency check:** Duplicate activities are detected via `pending_idempotency` and `processed` hash sets
3. **Process:** `InboxProcessor::process_next()` invokes handler function with the activity
4. **Conflict resolution:** Handler applies CRDT merge logic based on entity type
5. **Retry:** Failed processing retries up to `max_retries` (default 3)

### 5.3 Sync Protocol

Three sync modes, managed by `IncrementalSyncEngine` (from `multimaster.rs:140-218`):

**Full Sync (new region joining):**
1. New region sends `ReplicationPayload::FullSnapshot` request to seed region
2. Seed region serializes complete database state as compressed binary blob
3. New region applies snapshot locally, initializing vector clocks
4. Subsequent operations use incremental sync

**Incremental Sync (ongoing):**
1. Each local write generates a `SyncDelta` with `old_revision`, `new_revision`, `delta_data`, and `new_data`
2. Delta is computed by `DeltaCompressor::compute_delta()` (from `multimaster.rs:345-402`) using binary diff algorithm
3. Delta is compressed by `BandwidthOptimizer::optimize_transfer()` (from `multimaster.rs:298-313`): if delta is smaller than full data and exceeds `min_delta_size`, delta is used; otherwise full data is sent
4. Delta is wrapped in `ReplicationPayload::SyncDelta` with checkpoint ID and vector clock
5. Receiving region applies delta via `DeltaCompressor::apply_delta()` (from `multimaster.rs:404-458`)

**Repair Sync (gap recovery):**
1. `PartitionTracker` detects partition heal
2. Affected regions exchange `SyncCheckpoint` records
3. Gaps identified by comparing `last_synced_revision` values
4. Missing deltas are requested and applied
5. Full sync triggered if gap exceeds configurable threshold (default: 1000 deltas)

### 5.4 Compression

Bulk transfers use zstd compression (to be implemented):

- **Delta compression:** `BandwidthOptimizer` already selects between delta and full data based on size
- **Bulk transfer compression:** Full snapshots and large delta batches are compressed with zstd level 3 (configurable via `compression_level`)
- **Minimum delta threshold:** `min_delta_size` (default 64 bytes) prevents overhead for trivial changes

---

## 6. Consistency Model

### 6.1 Intra-Region Consistency

Within a single region, all operations are serialized:

- **Database isolation level:** SERIALIZABLE (PostgreSQL)
- **Read-your-writes:** Guaranteed within a region
- **Linearizability:** All operations within a region appear totally ordered
- **Git operations:** Local git operations are not replicated; only metadata (issues, PRs, reviews) is federated

### 6.2 Inter-Region Consistency

Across regions, consistency is eventual with bounded convergence:

- **Convergence guarantee:** CRDTs ensure all regions converge to the same state given sufficient time without new writes
- **Bounded staleness:** With inter-region replication lag <5s, all regions see a consistent view within 5 seconds of any write
- **Causal consistency:** Vector clocks ensure that causally related operations are applied in causal order across regions
- **No global consensus:** Regions operate independently; no two-phase commit or Paxos/Raft coordination

### 6.3 User Experience During Inconsistency

When a user reads data that may be stale:

- **Read banner:** "Your changes will be visible in other regions shortly"
- **Optimistic UI:** Local writes are immediately visible in the originating region
- **Conflict notification:** If a concurrent edit is detected and requires manual resolution, user is notified with side-by-side diff
- **Staleness indicator:** Optional "Last synced X seconds ago" indicator for cross-region data

---

## 7. Conflict Resolution

### 7.1 Automatic Resolution

CRDTs handle the majority of conflicts automatically:

**Non-conflicting changes:**
- Different fields modified on same entity → field-level merge (existing `merge_values` in `multimaster.rs:108-119`)
- Add to set on one region, remove on another → add-wins semantics (OR-Set)
- Concurrent comments → merge in causal order (RGA sequence CRDT)

**Concurrent edits to same field:**
- Repository metadata (description, visibility) → LWW by timestamp
- Issue state (open/close) → LWW by timestamp
- PR state (open/close/merge) → LWW by timestamp with merge-wins preference

**Resolution pipeline:**

```
Incoming Delta
    |
    v
Compare Vector Clocks
    |
    +---> Causal (happened_before) → Apply directly
    +---> Concurrent (is_concurrent) → Apply CRDT merge
    +---> Stale (remote is behind) → Skip
```

### 7.2 Manual Resolution

Manual resolution is required for semantic conflicts that CRDTs cannot resolve automatically:

**Trigger conditions:**
- Both regions modified the same repository description field to different values
- One region merged a PR while another region closed it without merge
- Issue label set conflict where labels were renamed on different regions

**Resolution UI:**
1. User is notified of unresolved conflict via dashboard
2. Side-by-side diff view shows local vs. remote versions
3. User selects one version, edits manually, or combines changes
4. Resolution is serialized as a new activity with `ConflictStrategy::Manual` metadata
5. Resolution activity is replicated to all regions

### 7.3 Conflict Audit Trail

All conflict resolutions are logged via `ConflictResolution::resolution_log` (from `multimaster.rs:61-106`):

```rust
pub struct ConflictEntry {
    pub entity_id: String,
    pub local_value: serde_json::Value,
    pub remote_value: serde_json::Value,
    pub resolved_value: Option<serde_json::Value>,
    pub strategy: ConflictStrategy,
    pub timestamp: DateTime<Utc>,
}
```

This log is persisted to a dedicated `conflict_log` table for auditing and debugging.

---

## 8. Implementation Plan

### 8.1 New Crate: civit-federation-core

Create a dedicated crate for CRDT types and sync protocol:

```
crates/civit-federation-core/
  src/
    lib.rs
    crdt/
      mod.rs
      lww_register.rs      # Last-Writer-Wins Register
      or_set.rs            # Observed-Remove Set
      rga.rs               # Replicated Growable Array (sequence CRDT)
      add_wins_set.rs      # Add-Wins Set
      mv_register.rs       # Multi-Value Register (for manual resolution)
    sync/
      mod.rs
      full_sync.rs         # Full snapshot sync protocol
      incremental_sync.rs  # Delta-based incremental sync
      repair_sync.rs       # Gap detection and repair
    region/
      mod.rs
      coordinator.rs       # Region coordinator
      discovery.rs         # DNS SRV and config-based discovery
      health.rs            # Health monitoring and partition detection
```

### 8.2 Modified Crates

**civit-core:**
- Add region-aware routing: requests are routed to the nearest region based on client IP geolocation
- Add region metadata to all entity models (`region_id` field)
- Extend `InboxHandler` to handle multi-region incoming activities with vector clock validation
- Add region-aware pagination for cross-region queries

**civit-db:**
- Add `regions` table: stores region metadata (id, endpoint, public_key, status, last_seen)
- Add `federation_state` table: tracks per-peer replication state (checkpoint, lag, last_sync)
- Add `conflict_log` table: audit trail for manual conflict resolutions
- Add vector clock columns to all entity tables
- Add migration for existing data: initialize vector clocks with current timestamp

### 8.3 Database Migrations

```sql
-- regions table
CREATE TABLE regions (
    id VARCHAR(64) PRIMARY KEY,
    endpoint VARCHAR(255) NOT NULL,
    public_key_pem TEXT NOT NULL,
    status VARCHAR(32) NOT NULL DEFAULT 'active',
    last_seen TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- federation_state table
CREATE TABLE federation_state (
    id SERIAL PRIMARY KEY,
    local_region_id VARCHAR(64) NOT NULL REFERENCES regions(id),
    remote_region_id VARCHAR(64) NOT NULL REFERENCES regions(id),
    last_synced_revision VARCHAR(128),
    replication_lag_ms INTEGER,
    last_sync_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(local_region_id, remote_region_id)
);

-- conflict_log table
CREATE TABLE conflict_log (
    id SERIAL PRIMARY KEY,
    entity_type VARCHAR(64) NOT NULL,
    entity_id VARCHAR(128) NOT NULL,
    local_value JSONB NOT NULL,
    remote_value JSONB NOT NULL,
    resolved_value JSONB,
    strategy VARCHAR(32) NOT NULL,
    region_a VARCHAR(64) NOT NULL,
    region_b VARCHAR(64) NOT NULL,
    resolved_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Add vector clock columns to entity tables
ALTER TABLE repositories ADD COLUMN vector_clock JSONB NOT NULL DEFAULT '{}';
ALTER TABLE issues ADD COLUMN vector_clock JSONB NOT NULL DEFAULT '{}';
ALTER TABLE pull_requests ADD COLUMN vector_clock JSONB NOT NULL DEFAULT '{}';
ALTER TABLE comments ADD COLUMN vector_clock JSONB NOT NULL DEFAULT '{}';
ALTER TABLE pipeline_definitions ADD COLUMN vector_clock JSONB NOT NULL DEFAULT '{}';
```

### 8.4 Configuration

```toml
[federation]
enabled = true
region_id = "us-east"

[federation.discovery]
method = "config"  # "config" or "dns"
dns_service_name = "_civitforge._tcp.federation.example.com"

[federation.regions]
"eu-west" = { endpoint = "https://eu-west.civitforge.internal:9090" }
"ap-south" = { endpoint = "https://ap-south.civitforge.internal:9090" }

[federation.sync]
heartbeat_interval_secs = 10
health_timeout_secs = 30
max_sync_delta = 1000
full_sync_threshold = 10000

[federation.delivery]
max_concurrent = 10
max_attempts = 5
backoff_base_ms = 1000
backoff_max_ms = 300000
jitter_enabled = true
batch_size = 25

[federation.compression]
enabled = true
level = 3
min_delta_size = 64

[federation.security]
tls_required = true
signature_algorithm = "ed25519"
```

### 8.5 Implementation Phases

**Phase 1: CRDT Foundation (4 weeks)**
- Implement CRDT types in civit-federation-core
- Unit tests for convergence properties
- Integration with existing vector clock implementation

**Phase 2: Region Coordinator (3 weeks)**
- Implement region discovery (DNS SRV + config)
- Implement health monitoring and partition detection
- Implement heartbeat protocol

**Phase 3: Sync Protocol (4 weeks)**
- Implement full sync mode
- Implement incremental sync with delta compression
- Implement repair sync with gap detection

**Phase 4: Integration (3 weeks)**
- Modify civit-core for region-aware routing
- Modify civit-db for region metadata
- Database migrations
- Configuration integration

**Phase 5: Conflict Resolution UI (2 weeks)**
- Manual resolution interface
- Conflict audit trail
- Staleness indicators

**Phase 6: Testing and Hardening (3 weeks)**
- Chaos testing: network partitions, region failures
- Load testing: cross-region latency, throughput
- Security audit: signature verification, TLS, access control

---

## 9. Security Considerations

### 9.1 Inter-Region Authentication

All inter-region communication is authenticated using Ed25519 HTTP Signatures:

- Each region generates a unique Ed25519 keypair at startup (using `generate_ed25519_keypair()` from `http_signatures.rs:408-418`)
- Public key is published at `https://{region}/actor#main-key`
- Every outgoing request includes a signature over `(request-target)`, `host`, `date`, `digest` headers
- Incoming requests are verified against the sender's published public key
- Signature expiry enforced (default 300 seconds)

### 9.2 Transport Security

- **TLS mandatory:** All inter-region HTTP communication uses TLS 1.3
- **Certificate pinning:** Optional configuration for pinning peer TLS certificates
- **mTLS option:** For high-security deployments, mutual TLS with client certificates

### 9.3 Region-Level Access Control

- **Region allowlist:** Each region maintains an allowlist of trusted peer regions
- **Activity filtering:** Activities from unknown or untrusted regions are rejected
- **Rate limiting:** Per-region rate limiting for inbound activities
- **Geographic restrictions:** Optional configuration restricting which regions can replicate which data

### 9.4 Audit Trail

- All cross-region operations logged with source region, target region, activity type, timestamp
- Conflict resolution decisions logged in `conflict_log` table
- Federation delivery attempts logged with success/failure status
- Signature verification failures logged with source IP and reason

### 9.5 Key Management

- Ed25519 private keys stored in environment variables or secrets manager (never in config files)
- Key rotation: regions generate new keypairs and publish updated public keys
- Graceful key rotation: old signatures accepted for a configurable grace period (default 24 hours)

---

## 10. Risk Assessment

### 10.1 Split-Brain Scenarios

**Risk:** Network partition causes regions to operate independently, accepting conflicting writes.

**Mitigation:**
- Vector clocks detect concurrent writes from partitioned regions
- CRDTs ensure convergence after partition heals
- `PartitionTracker` detects partition formation and healing
- `PartitionNotice` messages notify other regions of partitioned peers
- User-visible conflict notification for semantic conflicts requiring manual resolution

**Residual risk:** If partition duration exceeds the delta retention window (`max_deltas` in `IncrementalSyncEngine`), gaps require full sync. This is bounded by configurable thresholds and automated repair.

### 10.2 Data Loss During Network Partitions

**Risk:** Writes during partition are lost if a region fails before partition heals.

**Mitigation:**
- Local database durability (PostgreSQL with synchronous commit)
- Outbox persistence: activities are persisted in outbox before delivery
- Delta retention: `IncrementalSyncEngine` retains up to `max_deltas` (default 1000) deltas in memory; persisted to database for durability
- Full sync as fallback: if delta retention is exceeded, full sync recovers all data

**Residual risk:** Complete region failure during partition results in data loss for writes that were only on the failed region. This is mitigated by the multi-region replication factor (minimum 3 regions recommended).

### 10.3 Performance Impact of Cross-Region Queries

**Risk:** Cross-region queries exhibit high latency due to network round trips.

**Mitigation:**
- Read-local-first: all reads served from local database
- Eventual consistency accepted for cross-region reads
- Staleness bounded by replication lag (<5s target)
- Optional cross-region query routing with caching for read-heavy workloads

**Residual risk:** Users querying data from a specific remote region (e.g., viewing a repository owned by a user in another region) will experience network latency. This is acceptable for the stated latency targets.

### 10.4 CRDT Convergence Bugs

**Risk:** Incorrect CRDT implementation leads to non-convergent state.

**Mitigation:**
- Formal convergence property tests for all CRDT types
- Property-based testing with random operation sequences
- Integration tests with simulated multi-region topologies
- Existing `VectorClock` implementation already has comprehensive tests

### 10.5 Scalability Limits

**Risk:** Federation traffic grows linearly with number of regions and activity volume.

**Mitigation:**
- Delta compression reduces bandwidth usage
- Batch delivery with configurable batch size
- Compression for bulk transfers
- Region-local caching for frequently accessed cross-region data
- Monitoring and alerting on replication lag and delivery queue depth

---

## 11. Success Metrics

### 11.1 Latency Metrics

| Metric | Target | Measurement |
|--------|--------|-------------|
| Local read latency | <100ms (p99) | Application-level histogram |
| Local write latency | <50ms (p99) | Database query latency |
| Inter-region replication lag | <5s (bounded) | Vector clock comparison across regions |
| Cross-region read latency | <500ms (p99) | Application-level histogram |

### 11.2 Availability Metrics

| Metric | Target | Measurement |
|--------|--------|-------------|
| System availability | 99.99% | Uptime monitoring per region |
| Region availability | 99.95% | Individual region uptime |
| Replication availability | 99.99% | Outbox delivery success rate |
| Conflict resolution availability | 100% | CRDT convergence guarantee |

### 11.3 Consistency Metrics

| Metric | Target | Measurement |
|--------|--------|-------------|
| Conflict resolution time | <1s | Time from conflict detection to resolution |
| Convergence time | <10s | Time from write to convergence across all regions |
| Manual resolution rate | <1% | Percentage of conflicts requiring manual intervention |
| Staleness window | <5s | Maximum time any region is stale |

### 11.4 Operational Metrics

| Metric | Target | Measurement |
|--------|--------|-------------|
| Delta compression ratio | >2:1 | Average delta size / full data size |
| Outbox delivery throughput | >1000 activities/sec | Per-region delivery rate |
| Memory overhead | <100MB | Additional memory for federation stack |
| Storage overhead | <5% | Additional storage for vector clocks and sync state |

### 11.5 Monitoring and Alerting

- **Prometheus metrics:** Exposed via `/metrics` endpoint
- **Grafana dashboards:** Real-time visualization of replication lag, delivery success rate, conflict rate
- **Alerting rules:**
  - Replication lag >10s for >60s
  - Outbox delivery failure rate >5% for >5min
  - Conflict rate >10/min for >5min
  - Region health check failure >3 consecutive checks

---

## Appendix A: Existing Code References

| Component | File | Key Types/Functions |
|-----------|------|---------------------|
| ActivityPub transport | `crates/civit-core/src/federation/activitypub.rs` | `Actor`, `Activity`, `InboxHandler` |
| ForgeFed extensions | `crates/civit-core/src/federation/forgefed.rs` | `ForgeFedActivity`, `ForgeFedProcessor`, `IdempotencyTracker` |
| Vector clocks | `crates/civit-core/src/federation/vector_clock.rs` | `VectorClock<V>` |
| Inbox/Outbox | `crates/civit-core/src/federation/inbox_outbox.rs` | `InboxProcessor`, `OutboxProcessor`, `BackoffStrategy` |
| Multi-master | `crates/civit-core/src/federation/multimaster.rs` | `ConflictStrategy`, `ConflictResolution`, `IncrementalSyncEngine`, `DeltaCompressor` |
| Replication | `crates/civit-core/src/federation/replication.rs` | `RegionId`, `ReplicationTransport`, `ReplicationPeer`, `ReplicationPayload` |
| Sync topology | `crates/civit-core/src/federation/sync.rs` | `DagSyncEngine`, `SyncNode`, `SyncEdge` |
| Delivery | `crates/civit-core/src/federation/delivery.rs` | `FederationDeliveryService`, `FederationDeliveryConfig` |
| HTTP Signatures | `crates/civit-core/src/federation/http_signatures.rs` | `SignatureVerifier`, `HttpSignature`, `generate_ed25519_keypair` |

## Appendix B: Glossary

| Term | Definition |
|------|-----------|
| CRDT | Conflict-free Replicated Data Type — data structure that converges automatically under concurrent updates |
| LWW | Last-Writer-Wins — conflict resolution strategy where the most recent write by timestamp wins |
| OR-Set | Observed-Remove Set — CRDT supporting add/remove with add-wins semantics |
| RGA | Replicated Growable Array — sequence CRDT for ordered collections |
| Vector Clock | Data structure tracking causal ordering of events across distributed nodes |
| Inbox/Outbox | Pattern where local writes are queued in outbox for async delivery to remote inboxes |
| ForgeFed | Extension of ActivityPub for forge-specific operations (repositories, issues, PRs) |
| HTTP Signature | Authentication mechanism using cryptographic signatures over HTTP request components |

---

*Document version: 1.0*
*Last updated: 2026-06-19*
