# Horizontal Scaling Guide for CivitForge

This guide covers scaling CivitForge horizontally across multiple service instances with stateless design, load balancing, session management, connection pooling, and caching.

## Stateless Service Design

### Core Principle

CivitForge services must be stateless — no request-local state stored in-memory that isn't shared across instances. All state lives in external stores (PostgreSQL, Redis, S3).

### What Can Be In-Memory

- Request-scoped data (parsed request body, auth context)
- Non-persistent caches with TTL (rate limit counters, feature flags)
- Configuration loaded at startup
- Compiled templates, static assets

### What Must Be External

- User sessions (Redis)
- Feature flag state (Redis/PostgreSQL)
- Rate limit counters (Redis)
- Job queues (Redis/PostgreSQL)
- Webhook delivery state (PostgreSQL)
- Git repository storage (S3 or shared filesystem)

### Health Check Endpoint

Each instance exposes a health check for load balancers:

```
GET /api/v1/health
{
  "status": "healthy",
  "version": "3.2.0",
  "uptime_seconds": 12345,
  "database": "connected",
  "redis": "connected"
}
```

### Graceful Shutdown

```rust
// On SIGTERM/SIGINT:
// 1. Stop accepting new connections
// 2. Wait for in-flight requests to complete (max 30s)
// 3. Close database pools
// 4. Close Redis connections
// 5. Exit
```

## Load Balancer Configuration

### NGINX

```nginx
upstream civitforge {
    least_conn;
    server 10.0.1.1:8080 weight=5;
    server 10.0.1.2:8080 weight=5;
    server 10.0.1.3:8080 weight=5;

    keepalive 32;
}

server {
    listen 443 ssl http2;
    server_name civitforge.example.com;

    ssl_certificate /etc/ssl/certs/civitforge.pem;
    ssl_certificate_key /etc/ssl/private/civitforge.key;

    location / {
        proxy_pass http://civitforge;
        proxy_http_version 1.1;
        proxy_set_header Connection "";
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;

        # Health check
        proxy_connect_timeout 5s;
        proxy_read_timeout 30s;
        proxy_send_timeout 30s;

        # Circuit breaker: fail fast if backend is down
        proxy_next_upstream error timeout http_502 http_503;
        proxy_next_upstream_tries 3;
    }

    # Dedicated health check endpoint
    location /api/v1/health {
        proxy_pass http://civitforge;
        proxy_connect_timeout 2s;
        proxy_read_timeout 5s;
    }

    # Git smart HTTP (long timeouts)
    location ~ ^/(.+)/(.+)/(info/refs|git-upload-pack|git-receive-pack)$ {
        proxy_pass http://civitforge;
        proxy_http_version 1.1;
        proxy_set_header Connection "";
        proxy_read_timeout 300s;
        proxy_send_timeout 300s;
        client_max_body_size 10g;
    }
}
```

### HAProxy

```haproxy
global
    maxconn 10000

defaults
    mode http
    timeout connect 5s
    timeout client 30s
    timeout server 30s
    option httplog
    option dontlognull
    option http-server-close
    option forwardfor

frontend http_front
    bind *:80
    bind *:443 ssl crt /etc/ssl/civitforge.pem
    redirect scheme https if !{ ssl_fc }
    default_backend civitforge_back

backend civitforge_back
    balance leastconn
    option httpchk GET /api/v1/health
    http-check expect status 200

    server civit1 10.0.1.1:8080 check weight 5 maxconn 200
    server civit2 10.0.1.2:8080 check weight 5 maxconn 200
    server civit3 10.0.1.3:8080 check weight 5 maxconn 200

    # Sticky sessions (if needed for uploads)
    # cookie SERVERID insert indirect nocache
```

### Kubernetes Ingress

```yaml
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: civitforge
  annotations:
    nginx.ingress.kubernetes.io/upstream-hash-by: "$request_uri"
    nginx.ingress.kubernetes.io/proxy-connect-timeout: "5"
    nginx.ingress.kubernetes.io/proxy-read-timeout: "30"
    nginx.ingress.kubernetes.io/proxy-body-size: "10g"
spec:
  rules:
    - host: civitforge.example.com
      http:
        paths:
          - path: /
            pathType: Prefix
            backend:
              service:
                name: civitforge
                port:
                  number: 8080
---
apiVersion: apps/v1
kind: Deployment
metadata:
  name: civitforge
spec:
  replicas: 3
  selector:
    matchLabels:
      app: civitforge
  template:
    spec:
      containers:
        - name: civitforge
          image: civitforge:latest
          ports:
            - containerPort: 8080
          readinessProbe:
            httpGet:
              path: /api/v1/health
              port: 8080
            initialDelaySeconds: 5
            periodSeconds: 10
          livenessProbe:
            httpGet:
              path: /api/v1/health
              port: 8080
            initialDelaySeconds: 15
            periodSeconds: 20
          resources:
            requests:
              cpu: 500m
              memory: 512Mi
            limits:
              cpu: 2000m
              memory: 2Gi
```

## Session Management (Redis)

### Redis Configuration

```toml
# civitforge.toml
[session]
driver = "redis"
redis_url = "redis://redis-cluster:6379"
key_prefix = "civitforge:session:"
ttl_seconds = 86400  # 24 hours
```

### Session Store Interface

Sessions are stored in Redis with automatic expiration:

```
SET civitforge:session:{session_id} {session_json} EX 86400
GET civitforge:session:{session_id}
DEL civitforge:session:{session_id}
```

### Session Data Structure

```json
{
  "user_id": "usr_abc123",
  "org_id": "org_xyz789",
  "roles": ["admin", "write"],
  "created_at": "2026-01-15T10:30:00Z",
  "last_active_at": "2026-01-15T14:22:00Z",
  "ip_address": "192.168.1.100",
  "user_agent": "Mozilla/5.0..."
}
```

### Session Rotation

On each authenticated request:
1. Read session from Redis
2. If `last_active_at` > 30 minutes ago, rotate session ID
3. Update `last_active_at` timestamp
4. Extend Redis TTL

### Connection Pooling for Redis

```rust
// Redis connection pool config
let redis_pool = redis::aio::ConnectionPool::builder()
    .max_connections(50)
    .min_idle(10)
    .build(redis_client);
```

## Database Connection Pooling

### Per-Instance Pool Configuration

Each service instance maintains its own connection pool:

```toml
[database]
max_connections = 20        # Per instance
min_idle = 5
connect_timeout_secs = 5
idle_timeout_secs = 300
max_lifetime_secs = 1800
```

### Pool Sizing Formula

```
pool_size = (concurrent_requests * avg_query_duration_seconds) * safety_factor
```

For a typical deployment:
- 3 instances, each with 20 connections = 60 total connections
- PostgreSQL `max_connections` should be >= 60 + headroom (e.g., 100)

### Connection Pool Monitoring

Track per-instance metrics:
- `db_pool_active_connections` — connections currently in use
- `db_pool_idle_connections` — connections in the pool
- `db_pool_wait_time_seconds` — time waiting for a connection
- `db_pool_checkout_errors` — failed connection acquisitions

### PgBouncer (Optional)

For very high connection counts, use PgBouncer as a connection pooler:

```ini
[databases]
civitforge = host=127.0.0.1 port=5432 dbname=civitforge

[pgbouncer]
pool_mode = transaction
max_client_conn = 1000
default_pool_size = 50
min_pool_size = 10
reserve_pool_size = 5
```

## Cache Strategy (Redis + In-Memory)

### Two-Tier Cache Architecture

```
Request → In-Memory Cache (per instance) → Redis Cache (shared) → PostgreSQL
```

### In-Memory Cache (Local)

Short-lived, instance-local cache for hot data:

```rust
use std::time::Duration;

// Feature flags, rate limit counters, hot config
let local_cache = moka::future::Cache::builder()
    .max_capacity(10_000)
    .time_to_live(Duration::from_secs(30))
    .build();
```

Use cases:
- Feature flags (refresh every 30s)
- Rate limit counters (reset every 60s)
- Repository metadata (refresh every 5min)

### Redis Cache (Shared)

Cross-instance shared cache for session data and expensive queries:

```
Key patterns:
  civitforge:session:{id}          → Session JSON (TTL: 24h)
  civitforge:repo:meta:{repo_id}   → Repository metadata (TTL: 5min)
  civitforge:user:perms:{user_id}  → Permission set (TTL: 10min)
  civitforge:ratelimit:{key}       → Rate limit counter (TTL: 60s)
  civitforge:feature:{flag_name}   → Feature flag value (TTL: 30s)
```

### Cache Invalidation

```
Write path:
  1. Write to PostgreSQL
  2. Delete from Redis cache
  3. Broadcast invalidation to all instances via Redis pub/sub

Read path:
  1. Check in-memory cache → hit? return
  2. Check Redis cache → hit? populate in-memory, return
  3. Query PostgreSQL → populate Redis + in-memory, return
```

### Cache Stampede Prevention

Use Redis locks to prevent cache stampede:

```rust
async fn get_or_load<T: Serialize>(key: &str, loader: impl Future<Output = T>) -> T {
    // Try Redis first
    if let Some(cached) = redis.get(key).await {
        return cached;
    }

    // Acquire lock to prevent stampede
    let lock_key = format!("lock:{key}");
    if redis.set_nx(&lock_key, "1", 10).await {
        let value = loader.await;
        redis.set(key, &value, 300).await;
        redis.del(&lock_key).await;
        value
    } else {
        // Another instance is loading; wait and retry
        tokio::time::sleep(Duration::from_millis(100)).await;
        redis.get(key).await.expect("cache should be populated")
    }
}
```

### Monitoring Cache Performance

Key metrics to track:
- `cache_hit_ratio` — target > 90%
- `cache_miss_rate` — target < 10%
- `cache_eviction_count` — memory pressure indicator
- `redis_memory_used_bytes` — Redis memory usage
- `redis_connected_clients` — Redis connection count

## Deployment Topology

### Small Scale (1-2 instances)

```
                    ┌─────────────┐
                    │    NGINX    │
                    └──────┬──────┘
                           │
              ┌────────────┼────────────┐
              │                         │
         ┌────┴────┐              ┌─────┴────┐
         │ Civit1  │              │ Civit2   │
         └────┬────┘              └────┬─────┘
              │                         │
              └────────┬────────────────┘
                       │
              ┌────────┴────────┐
              │   PostgreSQL    │
              │   (primary)     │
              └─────────────────┘
```

### Medium Scale (3-6 instances)

```
                    ┌─────────────┐
                    │  CloudFlare │
                    └──────┬──────┘
                           │
                    ┌──────┴──────┐
                    │  NGINX /    │
                    │  HAProxy    │
                    └──────┬──────┘
                           │
         ┌─────────┬───────┼───────┬─────────┐
         │         │       │       │         │
      ┌──┴──┐  ┌──┴──┐ ┌──┴──┐ ┌──┴──┐  ┌──┴──┐
      │ C1  │  │ C2  │ │ C3  │ │ C4  │  │ C5  │
      └──┬──┘  └──┬──┘ └──┬──┘ └──┬──┘  └──┬──┘
         │         │       │       │         │
         └─────────┴───┬───┴───────┴─────────┘
                       │
              ┌────────┴────────┐
              │  Redis Cluster  │
              └────────┬────────┘
                       │
         ┌─────────────┼─────────────┐
         │             │             │
    ┌────┴────┐  ┌─────┴────┐  ┌────┴────┐
    │ Postgres│  │ Postgres │  │ Postgres│
    │ primary │  │ replica1 │  │ replica2│
    └─────────┘  └──────────┘  └─────────┘
```

### Large Scale (7+ instances, sharded)

```
See docs/SHARD_STRATEGY.md for database sharding topology.
Service layer scales independently with stateless instances.
```

## Scaling Checklist

- [ ] All services are stateless (no local state)
- [ ] Sessions stored in Redis
- [ ] Database connection pools sized per instance
- [ ] Load balancer health checks configured
- [ ] Graceful shutdown handling implemented
- [ ] Two-tier caching (local + Redis) deployed
- [ ] Cache invalidation via Redis pub/sub working
- [ ] Prometheus metrics exposed at `/api/v1/metrics`
- [ ] Auto-scaling rules configured (CPU/memory/connection count)
- [ ] Redis cluster or sentinel for HA
- [ ] PostgreSQL replication for read scaling
- [ ] Circuit breakers enabled for external calls
- [ ] Rate limiting applied at load balancer or application level
