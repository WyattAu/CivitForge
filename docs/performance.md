# Performance Targets and Optimization Guide

## SLO Targets

| Metric | Target | Measurement |
|--------|--------|-------------|
| API p99 latency (read) | < 200ms | Histogram, P99 |
| API p99 latency (write) | < 500ms | Histogram, P99 |
| Git clone (1M-line repo) | < 10s over LAN | End-to-end timing |
| Pipeline scheduling latency | < 2s (trigger to sandbox start) | Event timestamp delta |
| API pod RSS memory | < 512MB | cgroup memory.current |
| DB query P99 latency | < 50ms | pg_stat_statements |
| WebSocket event delivery | < 100ms | Client-side timing |

## Connection Pooling

### PostgreSQL (sqlx)

- `max_connections`: 20 per API pod
- `idle_timeout`: 30s
- `max_lifetime`: 30min
- `acquire_timeout`: 5s

### Redis

- `pool_size`: 10 per API pod
- `connection_timeout`: 2s
- `response_timeout`: 5s
- Command pipelining enabled for batch operations

## Cache Strategy

### L1: In-memory (DashMap)

- Repository metadata cache: TTL 60s, max 10k entries
- User session cache: TTL 300s, max 50k entries
- Federation instance cache: TTL 300s, max 1k entries

### L2: Redis

- Hot repository tree listings: TTL 30s
- Pipeline artifact manifests: TTL 600s
- Rate limit counters: Sliding window, 60s window
- WebSocket fan-out buffer: TTL 5s

### Cache Invalidation

- Write-through for repository mutations
- Event-driven invalidation via EventBus
- Version-based cache keys for immutable content

## Database Optimization

### Index Recommendations

```sql
CREATE INDEX idx_repos_owner ON repositories (owner_id);
CREATE INDEX idx_repos_updated ON repositories (updated_at DESC);
CREATE INDEX idx_repos_name_lower ON repositories (LOWER(name));
CREATE INDEX idx_users_username ON users (LOWER(username));
CREATE INDEX idx_users_email ON users (LOWER(email));
CREATE INDEX idx_pipelines_repo ON pipelines (repository_id, created_at DESC);
CREATE INDEX idx_pipelines_status ON pipelines (status);
CREATE INDEX idx_fed_instances_domain ON federation_instances (domain);
```

### Query Optimization

- Always use `LIMIT` clauses on list endpoints
- Prefer `SELECT id, ...` over `SELECT *`
- Use `EXPLAIN ANALYZE` on slow queries
- Cursor-based pagination over offset-based

## Memory Profiling

### Per-Pod Budget

| Component | Budget |
|-----------|--------|
| API server | 512MB RSS |
| Runner | 1GB RSS |
| Brain | 256MB RSS |
| VFS | 1GB RSS |

## HTTP Performance

- Keep-alive connections: 120s idle timeout
- Response compression: gzip for payloads > 1KB
- WebSocket: binary framing for event payloads

## Git Operations

- Shallow clone support for CI: `--depth=1`
- Upload pack caching for popular repos
- Ref advertisement optimization for large repos
- Deltapack compression for network transfers
