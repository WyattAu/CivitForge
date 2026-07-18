# Shard Key Strategy for CivitForge

This document defines the sharding strategy for horizontal database scaling.

## Shard Key Selection

CivitForge uses a composite shard key derived from organization and repository identifiers.

### Primary Shard Key: `repository_id`

All repository-scoped data (commits, branches, PRs, issues, releases) is sharded by `repository_id`. This ensures that all data for a single repository resides on the same shard, enabling efficient local joins and transactions.

```
Shard = Hash(repository_id) % shard_count
```

### Secondary Shard Key: `organization_id`

Organization-level data (settings, members, billing) is sharded by `organization_id`. This keeps organization metadata co-located and avoids cross-shard queries for org operations.

```
Shard = Hash(organization_id) % shard_count
```

### Key Selection Rationale

| Entity | Shard Key | Reason |
|--------|-----------|--------|
| Repository | `repository_id` | All repo operations are self-contained |
| Pull Request | `repository_id` | PRs belong to a repo |
| Issue | `repository_id` | Issues belong to a repo |
| Commit | `repository_id` | Commits belong to a repo |
| Release | `repository_id` | Releases belong to a repo |
| Organization | `organization_id` | Org data is org-scoped |
| User | `organization_id` | User memberships are org-scoped |
| Pipeline | `repository_id` | Pipelines run on repos |
| Webhook | `repository_id` | Webhooks fire on repo events |

## Consistent Hashing Algorithm

CivitForge uses SHA-256 consistent hashing with virtual nodes (vnodes) for uniform distribution.

### Parameters

- **Hash function**: SHA-256, truncated to 64 bits
- **Vnodes per shard**: 256 (default)
- **Ring size**: `shard_count * 256` virtual positions

### Virtual Node Placement

Each physical shard occupies 256 positions on the hash ring. Virtual node keys are generated as:

```
VNode key = SHA256("{shard_id}#{vnode_index}")
```

### Distribution Properties

With 256 vnodes per shard:
- **Uniformity**: Each shard receives approximately `1/N` of all keys (within 15% tolerance)
- **Minimal disruption**: Adding or removing a shard reassigns only `~1/N` of keys
- **Deterministic**: Same key always maps to the same shard

### Example

```rust
// With 4 shards and 256 vnodes each:
// - Ring has 1024 virtual positions
// - 10,000 keys distributed ~2500 per shard
// - Adding shard-4 reassigns ~20% of keys (1/5)
```

## Resharding Procedure

### Phase 1: Preparation

1. Provision the new shard database instance
2. Run schema migrations on the new shard
3. Verify connectivity and health

### Phase 2: Dual-Write

1. Update the shard map to include the new shard
2. Enable dual-write mode:
   - Writes go to both old and new shard assignments
   - Reads continue from old shard assignments
3. Start background migration of existing data

### Phase 3: Read Migration

1. Verify data consistency between old and new shards
2. Route reads for migrated repositories to new shards
3. Monitor for discrepancies

### Phase 4: Cutover

1. Stop dual-write for migrated repositories
2. Route all traffic through the new shard map
3. Verify no data loss

### Phase 5: Decommission

1. Archive old shard data
2. Remove old shard from the ring
3. Update monitoring and alerts

### Rollback Procedure

At any phase, rollback by:
1. Reverting the shard map to the previous state
2. Re-routing traffic to the original shards
3. Verifying data consistency

## Cross-Shard Queries

Some queries inherently span multiple shards. CivitForge handles these through application-level scatter-gather.

### Global Queries

- **User dashboard**: Queries repos across all shards for a user
- **Organization analytics**: Aggregates data across all org repos
- **Search**: Full-text search across all repositories
- **Admin metrics**: System-wide statistics

### Scatter-Gather Pattern

```
1. Query all shards in parallel
2. Collect partial results
3. Merge and sort at application layer
4. Apply pagination/limit after merge
```

### Performance Considerations

- **Parallel execution**: All shard queries execute concurrently
- **Result streaming**: Partial results are streamed as they arrive
- **Timeout handling**: Individual shard timeouts don't block the entire query
- **Partial failure tolerance**: Degraded results returned if some shards are unavailable

### Denormalization for Hot Paths

For frequently-accessed cross-shard data, maintain denormalized copies:

- **User repo count**: Cached in user profile (Redis)
- **Organization member count**: Cached in org metadata
- **Global search index**: Elasticsearch with cross-shard data

## Shard Health Monitoring

### Health Check Metrics

Each shard exposes health metrics collected by the `ShardHealth` monitor:

| Metric | Threshold | Action |
|--------|-----------|--------|
| Connection latency | > 100ms | Warning |
| Query latency (p99) | > 500ms | Warning |
| Error rate | > 1% | Alert, route away |
| Replication lag | > 10s | Warning |
| Disk usage | > 80% | Warning |
| Disk usage | > 95% | Alert, route away |
| Connection pool utilization | > 90% | Warning |

### Health Check Flow

```
Every 10 seconds:
  1. Execute "SELECT 1" on each shard
  2. Measure response latency
  3. Check replication lag (if replica)
  4. Check disk usage
  5. Update shard health status
  6. If unhealthy, remove from routing ring
  7. Emit metrics and alerts
```

### Automatic Recovery

When a shard becomes healthy again:
1. Re-add to the routing ring
2. Resume traffic gradually (weighted routing)
3. Verify data consistency
4. Restore full traffic

### Monitoring Dashboard

Key dashboard panels:
- Shard query latency (p50, p95, p99)
- Shard error rates
- Shard connection pool usage
- Replication lag per shard
- Key distribution across shards
- Resharding progress (during migrations)
