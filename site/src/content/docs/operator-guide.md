---
title: Operator Guide
description: Deployment, Docker Compose, Helm, monitoring, backup, and troubleshooting for CivitForge operators.
---

## Prerequisites

- Docker 24+ and Docker Compose v2+ (for Docker deployment)
- Kubernetes 1.28+ (for Helm deployment)
- PostgreSQL 17+
- Redis 7+
- Rust 1.88+ (for source builds)

## Docker Compose Deployment

### Quick start

```bash
git clone https://github.com/WyattAu/CivitForge.git
cd CivitForge
docker compose up -d
```

Wait approximately 15 seconds for migrations to complete, then verify:

```bash
curl http://localhost:9091/healthz
# Expected: OK
```

### Services

| Service | Port | Description |
|---------|------|-------------|
| civitforge | 9091 | HTTP API server |
| runner | 8088 | CI/CD pipeline daemon |
| postgres | 5432 | PostgreSQL database |
| redis | 6379 | Redis cache |

Git SSH is available on port 2222. gRPC/VFS is on port 9090.

### Volumes

| Volume | Purpose |
|--------|---------|
| `postgres-data` | PostgreSQL data directory |
| `redis-data` | Redis persistence |
| `civit-data` | Git repository storage |
| `civit-logs` | Application logs |
| `runner-work` | CI/CD workspace |

### Production configuration

Override defaults via environment variables:

```yaml
environment:
  DATABASE_URL: postgres://civit:${PG_PASSWORD}@postgres:5432/civit
  JWT_SECRET: ${JWT_SECRET}  # Minimum 32 characters
  CIVIT_HOST: "0.0.0.0"
  CIVIT_PORT: "8080"
  REDIS_URL: redis://:${REDIS_PASSWORD}@redis:6379
  JWT_EXPIRY_HOURS: "24"
  CIVIT_STORAGE_PATH: /var/lib/civit/repos
  FEDERATION_ENABLED: "true"
  FEDERATION_INSTANCE_ID: "my-forge-01"
  FEDERATION_INSTANCE_DOMAIN: forge.example.com
  RUST_LOG: civit_core=info,tower_http=info
  CORS_ALLOWED_ORIGINS: https://forge.example.com
  RATE_LIMIT_MAX_REQUESTS: "100"
  RATE_LIMIT_WINDOW_SECS: "60"
```

### Health checks

Each service has a health check configured. The server depends on PostgreSQL
and Redis being healthy before starting:

```yaml
healthcheck:
  test: ["CMD", "wget", "-qO-", "http://localhost:8080/healthz"]
  interval: 30s
  timeout: 5s
  retries: 3
  start_period: 15s
```

## Helm Deployment

The Helm chart is in `deploy/helm/civitforge/`.

### Installation

```bash
helm install civitforge deploy/helm/civitforge \
  --set secrets.jwtSecret=${JWT_SECRET} \
  --set secrets.databaseUrl=postgres://civit:${PG_PASSWORD}@postgres:5432/civitforge \
  --set secrets.redisPassword=${REDIS_PASSWORD} \
  --set ingress.hosts[0].host=forge.example.com \
  --set ingress.tls[0].hosts[0]=forge.example.com
```

### Chart components

| Component | Default Replicas | Description |
|-----------|-----------------|-------------|
| api | 3 | HTTP API server |
| runner | 2 | CI/CD pipeline daemon |
| brain | 1 | AI/ML services |
| vfs | 2 | gRPC filesystem |
| postgres | 1 | PostgreSQL (Bitnami) |
| redis | 1 | Redis (Bitnami) |

### Values reference

| Value | Default | Description |
|-------|---------|-------------|
| `api.replicas` | 3 | API server replicas |
| `api.resources.requests.cpu` | 250m | CPU request |
| `api.resources.requests.memory` | 256Mi | Memory request |
| `api.resources.limits.cpu` | 1 | CPU limit |
| `api.resources.limits.memory` | 512Mi | Memory limit |
| `runner.replicas` | 2 | Runner replicas |
| `runner.sandboxRuntime` | podman | Container runtime |
| `ingress.enabled` | true | Enable ingress |
| `ingress.className` | nginx | Ingress class |
| `hpa.enabled` | true | Enable HPA |
| `hpa.api.minReplicas` | 3 | Minimum API replicas |
| `hpa.api.maxReplicas` | 10 | Maximum API replicas |
| `networkPolicy.enabled` | true | Enable network policy |
| `serviceMonitor.enabled` | true | Enable Prometheus metrics |

### Horizontal Pod Autoscaler

HPA scales based on CPU utilization:

```yaml
hpa:
  api:
    minReplicas: 3
    maxReplicas: 10
    targetCPUUtilization: 70
  runner:
    minReplicas: 2
    maxReplicas: 8
    targetCPUUtilization: 75
```

### Network policies

When `networkPolicy.enabled` is true, a default-deny policy is applied.
Only explicitly allowed traffic flows between components.

## Source Build

### Requirements

- Rust 1.88+
- protobuf-compiler
- Node.js 22+ and pnpm 11+ (for UI)

### Build

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install protobuf
sudo apt-get install -y protobuf-compiler

# Build workspace
cargo build --workspace --release

# Build UI
cd crates/civit-ui
pnpm install --frozen-lockfile
pnpm exec tailwindcss -i input.css -o assets/tailwind.css --minify
trunk build --release
```

### Run

```bash
docker compose up -d postgres redis

DATABASE_URL=postgres://civit:civit-dev-secure-pw-2026@localhost:5432/civit \
REDIS_URL=redis://:civit-redis-dev-2026@localhost:6379 \
JWT_SECRET=dev-secret-key-32bytes-minimum \
./target/release/civit-core
```

## Monitoring

### Health endpoints

| Endpoint | Description |
|----------|-------------|
| `GET /healthz` | Liveness check |
| `GET /readyz` | Readiness check |

### Prometheus metrics

When `serviceMonitor.enabled` is true in Helm, CivitForge exposes
Prometheus metrics on the `/metrics` endpoint.

Key metrics:

| Metric | Type | Description |
|--------|------|-------------|
| `civit_http_requests_total` | counter | Total HTTP requests |
| `civit_http_request_duration_seconds` | histogram | Request latency |
| `civit_git_operations_total` | counter | Git operations |
| `civit_pipeline_runs_total` | counter | Pipeline executions |
| `civit_pipeline_duration_seconds` | histogram | Pipeline duration |
| `civit_federation_delivery_total` | counter | Federation deliveries |
| `civit_federation_sync_lag_seconds` | gauge | Replication lag |
| `civit_db_pool_connections` | gauge | Active DB connections |

### Structured logging

CivitForge uses `tracing` with structured JSON output:

```json
{
  "timestamp": "2026-06-19T12:00:00Z",
  "level": "info",
  "message": "Pipeline completed",
  "pipeline_id": "abc-123",
  "repo": "alice/my-project",
  "duration_ms": 45000
}
```

Configure verbosity via `RUST_LOG`:

```bash
RUST_LOG=civit_core=info,civit_ci=debug,civit_runner=trace
```

## Backup and Recovery

### PostgreSQL backup

```bash
# Dump
pg_dump -U civit -d civit -Fc > backup_$(date +%Y%m%d_%H%M%S).dump

# Restore
pg_restore -U civit -d civit -c backup.dump
```

### Automated backups

Add a cron job for automated backups:

```bash
0 2 * * * pg_dump -U civit -d civit -Fc > /backups/civit_$(date +\%Y\%m\%d).dump
```

### Redis backup

```bash
redis-cli -a password BGSAVE
```

The RDB file is at `/data/dump.rdb` inside the Redis container.

### Repository storage backup

```bash
tar czf repos_$(date +%Y%m%d).tar.gz /var/lib/civit/repos
```

## Troubleshooting

### Server fails to start

**Symptom:** `civit-server` exits immediately after starting.

**Common causes:**

1. `DATABASE_URL` is empty or invalid
   ```
   Error: DATABASE_URL required
   ```
   Fix: Set `DATABASE_URL` in the environment.

2. `JWT_SECRET` is too short
   ```
   Error: JWT_SECRET must be at least 32 characters (256 bits)
   ```
   Fix: Use a longer secret: `JWT_SECRET=$(openssl rand -hex 32)`

3. PostgreSQL is not ready
   ```
   Error: connection refused
   ```
   Fix: Wait for PostgreSQL health check to pass, or check `docker compose logs postgres`.

### Database migration fails

**Symptom:** Server logs show migration errors.

Fix:
```bash
# Check migration status
DATABASE_URL=postgres://civit:password@localhost:5432/civit \
  sqlx migrate info --source crates/civit-db/src/migrations

# Run pending migrations
DATABASE_URL=postgres://civit:password@localhost:5432/civit \
  sqlx migrate run --source crates/civit-db/src/migrations
```

### Runner cannot connect to server

**Symptom:** Runner logs show connection refused to `civitforge:8080`.

Fix:
- Verify `CIVIT_RUNNER_API_URL` matches the server's container name
- Check that the server is healthy: `docker compose ps`
- Verify network connectivity: `docker compose exec runner wget -qO- http://civitforge:8080/healthz`

### Pipeline execution fails

**Symptom:** Pipeline steps fail with exit code 127 (command not found).

Fix:
- Verify the container image exists and is pullable
- Check Podman socket: `docker compose exec runner ls -la /run/podman/podman.sock`
- For K8s mode, verify the runner has permissions to create pods

### Federation not working

**Symptom:** Activities are not delivered to remote instances.

Fix:
- Verify `FEDERATION_ENABLED=true`
- Check that `FEDERATION_INSTANCE_ID` and `FEDERATION_INSTANCE_DOMAIN` are set
- Verify DNS resolution for the remote instance
- Check federation logs: `docker compose logs civitforge | grep federation`
- Verify HTTP signatures are valid

### High memory usage

**Symptom:** Server or runner consumes excessive memory.

Fix:
- Reduce `CARGO_BUILD_JOBS` if building from source
- Limit pipeline concurrency: `concurrency.max_parallel: 4`
- Increase container memory limits in Helm values
- Check for memory leaks in federation delivery queue

### Slow API responses

**Symptom:** API requests take > 1 second.

Fix:
- Check database connection pool: `DB_POOL_SIZE` may be too low
- Verify Redis connectivity
- Check for slow queries in PostgreSQL logs
- Enable query logging: `RUST_LOG=sqlx=trace`

## Security hardening

### Checklist

- [ ] Set a strong `JWT_SECRET` (minimum 32 characters, random)
- [ ] Use TLS for PostgreSQL connections (`sslmode=require`)
- [ ] Enable Redis authentication
- [ ] Configure CORS origins explicitly
- [ ] Set rate limits
- [ ] Enable network policies in Kubernetes
- [ ] Use non-root containers
- [ ] Enable audit logging
- [ ] Rotate secrets regularly
- [ ] Monitor for security advisories (`cargo audit`)

### Running as non-root

The Docker images run as non-root by default. To verify:

```bash
docker compose exec civitforge whoami
# Expected: civit (or uid 1000)
```

### Network isolation

In production, isolate services:

```
Internet -> Ingress (nginx) -> API (ClusterIP)
                             -> Runner (ClusterIP, no external access)
                             -> VFS (ClusterIP, no external access)
PostgreSQL -> Internal network only
Redis -> Internal network only
```
