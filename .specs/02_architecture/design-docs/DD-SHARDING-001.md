# Database Sharding Architecture

**Document ID:** DD-SHARDING-001
**Status:** Proposed
**Target Version:** v3.0.0
**Author:** Autonomous Engineering

---

## 1. Executive Summary

### Problem

The CivitForge monorepo currently operates with a single PostgreSQL instance
as its sole data store. All entities (users, organizations, repositories,
issues, pull requests, pipelines, sessions, and audit events) reside in this
single instance, accessed through a unified `DatabasePool` with circuit breaker
protection. This architecture imposes hard constraints:

- **Storage ceiling**: A single PostgreSQL instance cannot scale beyond
  approximately 10TB without significant performance degradation in both
  read and write operations. Index maintenance, vacuum operations, and WAL
  management become prohibitively expensive beyond this threshold.

- **Concurrent connection limit**: PostgreSQL supports a practical maximum of
  approximately 10K concurrent connections before connection pool exhaustion
  and context-switch overhead degrade throughput. CivitForge's current
  `DatabasePool` (`pool.rs`) exposes a single `PgPool` to the entire
  application, making this a hard ceiling on concurrent developer capacity.

- **Write throughput bottleneck**: PostgreSQL uses a single-writer model per
  tuple. High-contention tables (repositories, pull requests, issues) serialize
  writes, limiting write throughput to a single node's capacity.

- **Single point of failure**: Loss of the primary database instance causes
  total service outage. The existing circuit breaker pattern (`pool.rs:48-71`)
  provides graceful degradation but cannot mask total unavailability.

### Solution

Repository-based hash partitioning with consistent hashing, distributing data
across multiple independent PostgreSQL instances. Each shard owns a subset of
repositories and all associated entities (issues, PRs, pipelines, comments,
releases) co-located via foreign key relationships. A new `civit-shard` crate
provides shard routing, consistent hashing ring management, and cross-shard
query coordination.

### Target

- **Storage**: 100TB+ aggregate across shards
- **Concurrent developers**: 100K+ (each shard handles ~25% of load at 4
  shards, scaling linearly with shard count)
- **Write throughput**: 4x+ improvement at 4 shards (linear scaling for
  shard-local writes)
- **Availability**: No single shard failure causes total service outage

---

## 2. Shard Key Selection

### 2.1 Primary Shard Key: Repository Owner + Name Hash

The primary shard key is a deterministic hash of `(owner_id, repository_name)`.
This choice is justified by:

**Co-location guarantee**: All data for a repository (issues in `issues`,
pull requests in `pull_requests`, PR comments in `pr_comments`, pipelines in
`pipelines`, releases in `releases`, branch protection in
`branch_protection_rules`, timeline events in `pr_timeline`, status checks in
`pr_status_checks`) references `repo_id` via foreign keys. By sharding on
`repository_id` (derived from owner+name), all related data lands on the same
shard. This eliminates cross-shard joins for the vast majority of queries.

**Write distribution**: Repository-centric operations (creating issues, merging
PRs, running pipelines) dominate write load. Distributing by repository spreads
write contention across shards.

**Lookup efficiency**: The `get_repo_by_owner_name` method (`repository.rs:270-279`)
already performs owner+name lookup. The shard router can intercept this path
without additional index overhead.

**Hash function**: The shard key hash uses `xxHash3` for speed (non-cryptographic)
combined with a consistent hashing ring for minimal redistribution on shard
addition/removal.

```rust
pub fn shard_key(owner_id: Uuid, repo_name: &str) -> u64 {
    let mut hasher = xxhash_rust::xxh3::Xxh3::new();
    hasher.update(owner_id.as_bytes());
    hasher.update(repo_name.as_bytes());
    hasher.finish()
}
```

### 2.2 Secondary Shard Key: User ID

User-centric queries (session validation in `session.rs:77-96`, access token
validation in `repository.rs:912-929`, SSH key lookup in
`repository.rs:1135-1141`) cannot be routed via repository shard key. Two
strategies handle this:

**Replicated user table**: The `users` table is replicated to all shards via
logical replication. User writes (create, update, profile changes) propagate
asynchronously from the coordinator to all shards with a bounded replication
lag of <1 second. This enables:

- Session validation: `SELECT * FROM sessions WHERE token_hash = $1` runs on
  any shard (sessions are user-local, not repo-local).
- Access token validation: `SELECT user_id FROM access_tokens WHERE token_hash = $1`
  runs on any shard.
- SSH key lookup: `SELECT * FROM ssh_keys WHERE fingerprint = $1` runs on any
  shard.

**User-owned data**: User profile data, SSH keys, WebAuthn credentials, and
sessions are stored on the shard where the user's primary shard is assigned.
The user's primary shard is determined by `hash(user_id) mod N_SHARDS`.

### 2.3 Shard Routing Function

The routing layer uses a consistent hashing ring with virtual nodes (vnodes)
to ensure even distribution:

```rust
pub struct ConsistentHashRing {
    ring: BTreeMap<u64, ShardId>,
    vnodes_per_shard: u32,
}

impl ConsistentHashRing {
    pub fn get_shard(&self, key: u64) -> ShardId {
        self.ring
            .range(key..)
            .next()
            .or_else(|| self.ring.iter().next())
            .map(|(_, shard)| *shard)
            .expect("ring is non-empty")
    }
}
```

With `vnodes_per_shard = 256`, a 4-shard ring has 1024 virtual nodes,
achieving <5% standard deviation in shard sizes for uniform workloads.

---

## 3. Shard Topology

### 3.1 Initial Configuration: 4 Shards

The initial deployment uses 4 shards, each targeting approximately 25% of
total data:

| Shard | Role | Connection Pool | Target Size |
|-------|------|-----------------|-------------|
| 0 | Primary (legacy) | 200 connections | 25TB |
| 1 | Read-write replica | 200 connections | 25TB |
| 2 | Read-write replica | 200 connections | 25TB |
| 3 | Read-write replica | 200 connections | 25TB |

Shard 0 starts as the existing primary instance during migration. After
cutover, it becomes a regular shard.

### 3.2 Growth Plan

| Phase | Shards | Aggregate Capacity | Trigger |
|-------|--------|-------------------|---------|
| Initial | 4 | 100TB, 100K developers | Migration complete |
| Growth | 8 | 200TB, 200K developers | Any shard >80% capacity |
| Scale | 16 | 400TB, 400K developers | Rebalancing frequency >weekly |
| Maximum | 32 | 800TB, 800K developers | Infrastructure limit |

Each shard addition requires one new PostgreSQL instance provisioned with
the full schema, consistent ring update, and background data migration.

### 3.3 Per-Shard Architecture

Each shard runs an independent PostgreSQL instance with:

- Full schema (all tables from the current database)
- Local connection pool managed by a `ShardPool` wrapper
- Read replica(s) for read scaling within the shard
- Independent backup and recovery configuration
- Dedicated monitoring and alerting

```
Shard N
  |
  +-- Primary PostgreSQL (read-write)
  |
  +-- Replica 1 (read-only)
  |
  +-- Replica 2 (read-only)
```

---

## 4. Data Distribution

### 4.1 Shard Mapping Table

The central coordinator database (a lightweight PostgreSQL instance separate
from any shard) stores the shard assignment mapping:

```sql
CREATE TABLE shard_assignments (
    shard_key_hash BIGINT PRIMARY KEY,
    shard_id INTEGER NOT NULL,
    assigned_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    migrated_at TIMESTAMPTZ,
    status TEXT NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'migrating', 'active', 'archived'))
);

CREATE INDEX idx_shard_assignments_shard_id ON shard_assignments(shard_id);
CREATE INDEX idx_shard_assignments_status ON shard_assignments(status);
```

The coordinator also stores shard metadata:

```sql
CREATE TABLE shards (
    id INTEGER PRIMARY KEY,
    write_url TEXT NOT NULL,
    read_urls TEXT[] NOT NULL DEFAULT '{}',
    status TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'draining', 'offline')),
    max_connections INTEGER NOT NULL DEFAULT 200,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

### 4.2 Shard Assignment Algorithm

Assignment uses consistent hashing to minimize redistribution:

1. Compute `shard_key_hash = xxHash3(owner_id, repo_name)`
2. Look up `shard_id` in the consistent hash ring
3. Insert/update `shard_assignments` row with the computed hash and shard_id
4. The `status` field tracks migration state:
   - `pending`: Not yet migrated to target shard
   - `migrating`: Data copy in progress
   - `active`: Reads and writes routed to assigned shard
   - `archived`: Old data cleaned up

### 4.3 Rebalancing Strategy

When a new shard is added, approximately 1/N of all keys redistribute. The
rebalancing process:

1. **Provision**: Deploy new PostgreSQL instance with full schema
2. **Ring update**: Add new shard to the consistent hash ring (in-memory)
3. **Identify**: Compute which keys now map to the new shard
4. **Background copy**: For each affected repository:
   a. Begin dual-write (write to both old and new shard)
   b. Copy historical data from old shard to new shard
   c. Verify row counts and checksums
   d. Switch reads to new shard
   e. Remove dual-write for this repository
5. **Cleanup**: Remove old shard data for migrated repositories

This is an online operation with no downtime. The dual-write phase ensures
data consistency during the copy window. Estimated time: 2-4 hours for
10TB of data to migrate, with <100ms additional write latency during
dual-write.

---

## 5. Query Routing

### 5.1 Shard-Aware Query Routing Layer

The `civit-shard` crate provides a `ShardRouter` that wraps the existing
`DatabasePool` interface:

```rust
pub struct ShardRouter {
    coordinator: PgPool,
    ring: Arc<ConsistentHashRing>,
    shards: HashMap<ShardId, ShardPool>,
    user_shard_cache: DashCache<Uuid, ShardId>,
}

pub struct ShardPool {
    id: ShardId,
    write_pool: PgPool,
    read_pools: Vec<PgPool>,
    circuit_breakers: Vec<CircuitBreaker>,
}
```

The router intercepts all database operations and routes them to the correct
shard based on the shard key. For operations that do not have a natural shard
key (e.g., global searches), the router uses scatter-gather.

### 5.2 Routing Rules

| Operation | Shard Key | Routing |
|-----------|-----------|---------|
| `get_repo(id)` | `repo_id` lookup table | Route to owning shard |
| `get_repo_by_owner_name(owner, name)` | Direct | Compute shard from owner+name hash |
| `create_issue(repo_id, ...)` | `repo_id` | Route to owning shard |
| `list_issues(repo_id, ...)` | `repo_id` | Route to owning shard |
| `create_pr(repo_id, ...)` | `repo_id` | Route to owning shard |
| `merge_pr(id, ...)` | `pr_id` -> `repo_id` | Route to owning shard |
| `create_user(...)` | `user_id` | Route to user's shard, replicate to all |
| `get_user_by_id(id)` | `user_id` | Any shard (replicated) |
| `validate_access_token(hash)` | `token_hash` | Any shard (replicated) |
| `validate_session(token)` | `token_hash` | Any shard (replicated) |
| `list_all_repos()` | None | Scatter-gather across all shards |
| `admin_list_repos(...)` | None | Scatter-gather with pagination merge |
| `audit_event_stats()` | None | Scatter-gather with aggregation |

### 5.3 Cross-Shard Query Handling

**Global queries** (no shard key available) use scatter-gather:

```rust
impl ShardRouter {
    pub async fn scatter_gather<F, T>(&self, f: F) -> Result<Vec<T>>
    where
        F: Fn(&PgPool) -> Fut,
        Fut: Future<Output = Result<Vec<T>>>,
    {
        let mut results = Vec::new();
        let futures: Vec<_> = self.shards.values()
            .filter(|s| s.is_healthy())
            .map(|shard| f(&shard.read_pool))
            .collect();

        for result in futures::future::join_all(futures).await {
            match result {
                Ok(mut rows) => results.append(&mut rows),
                Err(e) => log::warn!("shard query failed: {}", e),
            }
        }
        Ok(results)
    }
}
```

**Pagination merging**: For paginated scatter-gather queries (e.g.,
`admin_list_repos` with search), the router:
1. Issues `SELECT ... ORDER BY created_at DESC LIMIT (limit + offset)` to
   each shard
2. Merges results from all shards
3. Sorts the merged result by `created_at DESC`
4. Applies the final `LIMIT` and `OFFSET`
5. Returns the page

This is O(shards * (limit + offset)) per query, which is acceptable for
admin operations that are infrequent.

**Cross-shard joins**: Queries that join across shards (e.g., "all issues
assigned to user X across all repositories") require scatter-gather with
application-level join. The router fans out the query to all shards, each
shard returns its local results, and the router merges them in memory.

---

## 6. Migration Strategy

### Phase 1: Dual-Write (v3.0.0-alpha)

**Objective**: Begin writing to both the original primary and new shards
without affecting reads.

**Mechanics**:
1. Deploy the `civit-shard` crate with `MigrationMode::DualWrite`
2. The shard router writes to the original primary for all operations
3. A background worker reads the WAL (Write-Ahead Log) of the original
   primary and writes changed rows to the appropriate shard
4. The `shard_assignments` table tracks which repositories have been
   fully synced

**Validation**:
- Compare row counts between primary and each shard
- Verify checksums for migrated repositories
- Monitor replication lag (target: <1 second)

**Rollback**: Disable dual-write, continue using original primary.

### Phase 2: Read-from-Shards (v3.0.0-beta)

**Objective**: Route reads to shards while maintaining write fallback.

**Mechanics**:
1. Set `MigrationMode::ReadFromShards`
2. Reads for migrated repositories route to their assigned shard
3. Reads for non-migrated repositories fall back to the original primary
4. Writes continue to both original primary and shard (dual-write)

**Validation**:
- Compare read results between original primary and shards
- Monitor latency delta (shard reads should be within 10% of primary)
- Verify data consistency with periodic checksum validation

**Rollback**: Revert to `MigrationMode::DualWrite` or `MigrationMode::Direct`.

### Phase 3: Cutover (v3.1.0)

**Objective**: Route all traffic through the shard router.

**Mechanics**:
1. Set `MigrationMode::Cutover`
2. All reads and writes route through the shard router
3. The original primary becomes Shard 0 (no longer special)
4. Remove the dual-write background worker
5. Each repository lives on exactly one shard

**Validation**:
- End-to-end latency within 5% of pre-sharding baseline
- No data inconsistencies detected by checksum validation
- All queries routing to correct shards (verify via query logging)

**Rollback**: Revert to `MigrationMode::ReadFromShards` and re-enable dual-write.

### Phase 4: Decommission (v3.2.0)

**Objective**: Remove legacy infrastructure and optimize.

**Mechanics**:
1. Remove the coordinator database's dependency on the original primary
2. Archive the original primary data (keep for 90 days for safety)
3. Decommission the original primary instance
4. Optimize shard router performance (remove dual-write overhead)
5. Enable per-shard read replicas for read scaling

**Validation**:
- All shards healthy with <1% error rate
- No references to the original primary in codebase
- Archive verified with checksum comparison

---

## 7. Consistency Model

### 7.1 Strong Consistency Within a Shard

All operations within a single shard execute against a single PostgreSQL
instance (with read replicas). This provides:

- **Serializability**: Transactions within a shard are serialized by
  PostgreSQL's MVCC with SERIALIZABLE isolation level
- **Foreign key integrity**: All `repo_id` references resolve within the
  same shard
- **Unique constraints**: `(repo_id, number)` on issues and PRs, `(repo_id,
  tag_name)` on releases, `(pr_id, context, commit_sha)` on status checks
  are enforced locally

### 7.2 Eventual Consistency Across Shards

Cross-shard operations use eventual consistency:

- **User replication**: User data replicates to all shards with <1 second
  lag. During the replication window, a newly created user may not be
  visible on all shards.
- **Global counters**: Stars, watchers, and other counters are updated
  locally on the owning shard. Global aggregates (total stars across all
  repos) are computed via scatter-gather and may reflect stale data.
- **Activity feed**: Activity events are written to the owning shard's
  local table. Global activity feeds require scatter-gather.

### 7.3 Conflict Resolution for Cross-Shard Operations

| Scenario | Strategy | Guarantee |
|----------|----------|-----------|
| User created on shard A, validated on shard B | Replicated user table | Eventually consistent (lag <1s) |
| Two shards update user profile simultaneously | Last-writer-wins with timestamp | Convergent (one update wins) |
| Repository transferred between shards | Two-phase migration with locking | Strong consistency (migration is atomic) |
| Star count across shards | Local increment, global aggregation | Eventual consistency (lag <1s) |
| Audit event logging | Write locally, no global ordering | Eventual consistency (per-shard ordering) |

---

## 8. Implementation Plan

### 8.1 New Crate: civit-shard

**Location**: `crates/civit-shard/`

**Modules**:
- `ring.rs`: Consistent hash ring with virtual nodes
- `router.rs`: Shard-aware query routing
- `coordinator.rs`: Shard mapping management
- `migrator.rs`: Online data migration
- `scatter.rs`: Scatter-gather for global queries

**Dependencies**: `sqlx`, `xxhash-rust`, `dashmap`, `tokio`

### 8.2 Modified Crates

**civit-db**:
- `pool.rs`: Extend `DatabasePool` to support multiple pools (one per shard)
- `repository.rs`: Add shard-aware variants of all methods
- New: `ShardDatabasePool` struct that wraps multiple `DatabasePool` instances

**civit-core**:
- Add shard router initialization in application startup
- Inject `ShardRouter` into request handlers
- Update all database call sites to use shard-aware routing

### 8.3 New Migration

```sql
-- Migration: shard_assignments
CREATE TABLE shard_assignments (
    shard_key_hash BIGINT PRIMARY KEY,
    shard_id INTEGER NOT NULL REFERENCES shards(id),
    assigned_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    migrated_at TIMESTAMPTZ,
    status TEXT NOT NULL DEFAULT 'pending'
        CHECK (status IN ('pending', 'migrating', 'active', 'archived'))
);

CREATE TABLE shards (
    id INTEGER PRIMARY KEY,
    write_url TEXT NOT NULL,
    read_urls TEXT[] NOT NULL DEFAULT '{}',
    status TEXT NOT NULL DEFAULT 'active'
        CHECK (status IN ('active', 'draining', 'offline')),
    max_connections INTEGER NOT NULL DEFAULT 200,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Insert initial shard configuration
INSERT INTO shards (id, write_url, read_urls) VALUES
    (0, '${SHARD_0_URL}', '{}'),
    (1, '${SHARD_1_URL}', '{}'),
    (2, '${SHARD_2_URL}', '{}'),
    (3, '${SHARD_3_URL}', '{}');
```

### 8.4 Configuration

Environment variables:

```bash
# Shard configuration
SHARD_COUNT=4
SHARD_0_URL=postgres://civit:pass@shard0:5432/civit
SHARD_1_URL=postgres://civit:pass@shard1:5432/civit
SHARD_2_URL=postgres://civit:pass@shard2:5432/civit
SHARD_3_URL=postgres://civit:pass@shard3:5432/civit

# Read replicas (optional, comma-separated per shard)
SHARD_0_READ_URLS=postgres://civit:pass@shard0-replica1:5432/civit,postgres://civit:pass@shard0-replica2:5432/civit

# Migration mode
SHARD_MIGRATION_MODE=direct  # direct | dual_write | read_from_shards | cutover

# Coordinator database (for shard mapping)
COORDINATOR_URL=postgres://civit:pass@coordinator:5432/civit_shard_metadata
```

---

## 9. Risk Assessment

### 9.1 Data Loss During Migration

**Risk**: Data loss during the dual-write or migration phases if a shard
becomes unavailable mid-migration.

**Mitigation**:
- Dual-write ensures all data exists on both old and new locations until
  migration is verified
- Checksum validation runs after each repository migration
- Original primary retains all data until Phase 4 decommission (90-day
  retention)
- Migration is resumable: if a shard goes down mid-migration, the
  `shard_assignments.status` field tracks which repositories were fully
  migrated

**Residual risk**: Low (<0.01% data loss probability with checksum validation)

### 9.2 Performance Degradation During Rebalancing

**Risk**: Rebalancing reads/writes from one shard to another adds latency.

**Mitigation**:
- Rebalancing occurs in the background with no user-facing downtime
- Dual-write phase adds ~10ms write latency (one additional round-trip)
- Read migration is instantaneous (just a routing change)
- Rebalancing can be paused/resumed without data corruption
- Rate limiting prevents rebalancing from saturating network/disk

**Residual risk**: Medium (5-10% latency increase during active rebalancing,
duration: 2-4 hours per shard addition)

### 9.3 Cross-Shard Query Complexity

**Risk**: Scatter-gather queries are slower than single-shard queries and
scale linearly with shard count.

**Mitigation**:
- Most queries (95%+) are shard-local and unaffected
- Global queries (admin dashboards, search, activity feeds) are infrequent
- Pagination merging is O(shards * page_size), acceptable for N < 32 shards
- Future optimization: dedicated read-only aggregation shards for global
  queries

**Residual risk**: Low (cross-shard queries are a small fraction of total
query volume)

### 9.4 Consistency Gaps During Replication

**Risk**: User data replication lag causes authentication failures or
inconsistent reads.

**Mitigation**:
- User replication uses synchronous logical replication with <1 second lag
- Authentication operations (session, access token validation) include
  retry logic for transient replication lag
- Critical operations (user creation) write to the user's primary shard
  first, ensuring immediate availability for subsequent reads on that shard

**Residual risk**: Low (replication lag <1 second, retry logic handles
transient gaps)

---

## 10. Success Metrics

| Metric | Current (Single DB) | Target (Sharded) | Measurement |
|--------|---------------------|------------------|-------------|
| Shard count | N/A | 4 (initial) | `SELECT COUNT(*) FROM shards` |
| Data distribution uniformity | N/A | <5% std deviation | Per-shard row counts |
| Shard-local query latency | ~50ms p99 | <50ms p99 | Application metrics |
| Cross-shard query latency | N/A | <200ms p99 | Scatter-gather timing |
| Rebalancing speed | N/A | >10TB/hour | Migration throughput |
| Concurrent developers | ~10K max | 100K+ | Connection pool utilization |
| Storage capacity | ~10TB | 100TB+ | Aggregate `pg_database_size` |
| Write throughput | ~5K inserts/s | 20K+ inserts/s | Application metrics |
| Migration data loss | N/A | 0 bytes | Checksum validation |

---

## Appendix A: Query Routing Examples

### Example 1: Create Issue (Shard-Local)

```
Input: repo_id = "abc-123", title = "Bug fix"

1. ShardRouter receives create_issue request
2. Lookup shard for repo_id "abc-123" via consistent hash ring
3. Route to Shard 2 (hash("abc-123") mod 4 = 2)
4. Execute: INSERT INTO issues (repo_id, title, ...) VALUES ($1, $2, ...)
   on Shard 2's write pool
5. Return created issue
```

### Example 2: Global Search (Scatter-Gather)

```
Input: search = "authentication", limit = 20

1. ShardRouter receives admin_list_repos(search="authentication", limit=20)
2. Fan out to all 4 shards:
   - Shard 0: SELECT * FROM repositories WHERE name ILIKE '%authentication%'
              ORDER BY created_at DESC LIMIT 30
   - Shard 1: (same query)
   - Shard 2: (same query)
   - Shard 3: (same query)
3. Merge results from all shards (total: ~120 rows)
4. Sort merged results by created_at DESC
5. Apply LIMIT 20, OFFSET 0
6. Return 20 results
```

### Example 3: User Authentication (Replicated)

```
Input: token_hash = "sha256:abc..."

1. ShardRouter receives validate_access_token request
2. Token hash does not contain shard key information
3. Route to any healthy shard (Shard 0 by default)
4. Execute: SELECT user_id, expires_at FROM access_tokens
            WHERE token_hash = $1
            on Shard 0's read pool
5. Return user_id if valid, error if expired/not found
```

---

## Appendix B: Environment Configuration Matrix

| Variable | Default | Description |
|----------|---------|-------------|
| `SHARD_COUNT` | `1` | Number of shards (1 = no sharding) |
| `SHARD_N_URL` | Required | PostgreSQL connection URL for shard N |
| `SHARD_N_READ_URLS` | `{}` | Comma-separated read replica URLs |
| `SHARD_MIGRATION_MODE` | `direct` | Migration phase control |
| `COORDINATOR_URL` | Required | Shard metadata database URL |
| `SHARD_VNODES_PER_SHARD` | `256` | Virtual nodes per shard on hash ring |
| `SHARD_REBALANCE_RATE_LIMIT` | `100` | Max repositories to migrate per minute |
| `SHARD_REPLICATION_LAG_THRESHOLD` | `1000` | Max replication lag in ms before warning |
