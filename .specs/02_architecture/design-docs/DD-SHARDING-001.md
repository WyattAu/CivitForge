# Database Sharding Architecture

**Document ID:** DD-SHARDING-001
**Status:** Proposed
**Target Version:** v3.0.0
**Author:** Autonomous Engineering

---

## 1. Problem Statement

CivitForge stores all repository metadata (users, repos, issues, PRs, pipelines,
wiki) in a single PostgreSQL instance. At scale (10K+ users, 10TB monorepos),
a single database becomes:

- A write throughput bottleneck (PostgreSQL single-writer per tuple)
- A single point of failure
- A storage ceiling (practical max ~20TB per instance with acceptable latency)

## 2. Sharding Strategy

### 2.1 Shard Key: repository_id

Sharding by `repository_id` (UUID) provides:

- **Locality**: all data for a repository (issues, PRs, pipelines, wiki) lives
  on the same shard, enabling joins without cross-shard coordination.
- **Even distribution**: UUID v4 provides uniform hash distribution.
- **No resharding for new repos**: new repos map to any shard.

### 2.2 Hash Partitioning

```
shard_id = hash(repository_id) mod N_SHARDS
```

Using consistent hashing (rendezvous hashing) to minimize data movement during
shard addition/removal.

### 2.3 Lookup Service

A lightweight shard-router service (embedded in civit-core) maintains:

```rust
pub struct ShardRouter {
    shards: Vec<ShardConfig>,
    ring: ConsistentHashRing<ShardId>,
}

pub struct ShardConfig {
    id: ShardId,
    write_pool: PgPool,
    read_replicas: Vec<PgPool>,
    healthy: AtomicBool,
}

impl ShardRouter {
    pub fn shard_for_repo(&self, repo_id: Uuid) -> &ShardConfig {
        self.ring.get(&repo_id).expect("ring non-empty")
    }

    pub async fn execute<F, T>(&self, repo_id: Uuid, f: F) -> Result<T>
    where
        F: AsyncFn(&PgPool) -> Result<T>,
    {
        let shard = self.shard_for_repo(repo_id);
        let pool = if shard.healthy.load(Relaxed) {
            &shard.write_pool
        } else {
            return Err(anyhow!("shard {} is unhealthy", shard.id));
        };
        f(pool).await
    }
}
```

### 2.4 Cross-Shard Queries

Global tables (users, organizations) are replicated to all shards via logical
replication. Queries that span repositories use a scatter-gather pattern:

```rust
pub async fn list_all_repos(router: &ShardRouter) -> Vec<Repository> {
    let mut results = Vec::new();
    // Fan out to all healthy shards in parallel.
    let futures: Vec<_> = router.all_shards()
        .map(|shard| async move {
            shard.query("SELECT * FROM repositories").await.unwrap_or_default()
        })
        .collect();
    for f in futures::future::join_all(futures).await {
        results.extend(f);
    }
    results
}
```

## 3. Migration Path

### Phase 1: Dual-Write (v3.0.0)
- Shard router reads from the primary, writes to both primary and shard(s).
- Background sync backfills historical data.

### Phase 2: Read from Shards (v3.1.0)
- Reads for sharded repos go to their designated shard.
- Non-sharded repos still read from primary.

### Phase 3: Cutover (v3.2.0)
- All reads and writes go through shard router.
- Primary becomes shard 0.

### Phase 4: Decommission (v3.3.0)
- Remove dual-write. Each repo lives on exactly one shard.

## 4. Operational Concerns

### 4.1 Shard Rebalancing

Adding a shard requires:
1. Provision new PostgreSQL instance
2. Add to consistent hash ring
3. Backfill data for repos that now hash to the new shard
4. Verify data integrity
5. Switch reads/writes for migrated repos

Estimated time: 2-4 hours per shard addition (online, no downtime).

### 4.2 Failure Handling

- **Shard unavailable**: Router marks shard unhealthy, returns 503 for affected
  repos. Read replicas serve reads if write primary is down.
- **Split-brain**: PostgreSQL streaming replication + PgBouncer prevents this.
  WAL archiving enables point-in-time recovery.

### 4.3 Schema Migrations

Migrations apply to ALL shards. The migration runner iterates:

```rust
for shard in router.all_shards() {
    migration_manager.apply(shard.write_pool()).await?;
}
```

If a migration fails on one shard, the runner rolls back all shards.

## 5. Configuration

```toml
# civitforge.toml
[sharding]
enabled = true
shard_count = 4
default_shard = 0

[[sharding.shards]]
id = 0
write_url = "postgres://civit:pass@shard0:5432/civit"
read_urls = ["postgres://civit:pass@shard0-replica:5432/civit"]

[[sharding.shards]]
id = 1
write_url = "postgres://civit:pass@shard1:5432/civit"
read_urls = ["postgres://civit:pass@shard1-replica:5432/civit"]
```

## 6. Testing Strategy

- **Unit**: ShardRouter hash distribution, failover logic
- **Integration**: Multi-shard queries, cross-shard joins
- **Chaos**: Kill a shard, verify graceful degradation
- **Load**: 10K repos across 4 shards, measure throughput
