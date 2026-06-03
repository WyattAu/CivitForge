# CivitForge Operator Guide

Deployment, configuration, and operational procedures for CivitForge v1.1.0.

## Prerequisites

| Requirement | Minimum | Recommended |
|-------------|---------|-------------|
| CPU | 2 cores | 4+ cores |
| RAM | 4 GB | 8 GB |
| Disk | 20 GB SSD | 100 GB SSD |
| PostgreSQL | 17 | 17 |
| Redis | 7 | 7 |
| Rust (source build) | 1.88 | 1.88 |
| Podman/Docker | 3.x / 20.x | Latest |

## Quick Start (Docker Compose)

```bash
git clone https://github.com/WyattAu/CivitForge.git
cd CivitForge
docker compose up -d
```

Verify:
```bash
curl http://localhost:9091/healthz  # => OK
```

Host port mapping (from docker-compose.yml):

| Host Port | Container Port | Service |
|-----------|----------------|---------|
| 9091 | 8080 | REST API + WebSocket |
| 2222 | 2222 | Git SSH |
| 9090 | 9090 | VFS gRPC |
| 8088 | 8088 | Runner HTTP |

Default credentials (development only):

- PostgreSQL: `civit` / `civit-dev-secure-pw-2026` on port 5432
- Redis: password `civit-redis-dev-2026` on port 6379
- JWT secret: `change-me-change-me-dev-secret` (change for production)

## Quick Start (Podman)

```bash
podman kube play docker-compose.yml
```

Or individual containers:
```bash
podman run -d --name civit-postgres \
  -e POSTGRES_DB=civit -e POSTGRES_USER=civit -e POSTGRES_PASSWORD=civit \
  -p 5432:5432 postgres:17-alpine

podman run -d --name civit-redis \
  -p 6379:6379 redis:7-alpine

podman run -d --name civit \
  -e DATABASE_URL=postgres://civit:civit@host.containers.internal:5432/civit \
  -e JWT_SECRET=your-production-secret \
  -p 9091:8080 -p 2222:2222 \
  ghcr.io/wyattau/evergreenimageregistry/civitforge:latest
```

## Quick Start (Source Build)

```bash
cargo build --release --workspace
./target/release/civit-core
```

## Configuration Reference

All configuration is via environment variables.

### Required

| Variable | Description | Example |
|----------|-------------|---------|
| `DATABASE_URL` | PostgreSQL connection string (sqlx format) | `postgres://user:pass@host:5432/civit` |
| `JWT_SECRET` | JWT signing key, minimum 16 characters | Generate with `openssl rand -base64 32` |

### Optional

| Variable | Default | Description |
|----------|---------|-------------|
| `CIVIT_HOST` | `127.0.0.1` | API bind address. Use `0.0.0.0` for all interfaces. |
| `CIVIT_PORT` | `8080` | API bind port |
| `REDIS_URL` | `redis://127.0.0.1:6379` | Redis for sessions and edge cache |
| `JWT_EXPIRY_HOURS` | `24` | JWT token expiration in hours |
| `CIVIT_STORAGE_PATH` | `/var/lib/civit/repos` | Git repository storage path |
| `CIVIT_ENCRYPTION_KEY` | *(none)* | AES-256-GCM key for pipeline variable encryption |
| `FEDERATION_ENABLED` | `false` | Enable ForgeFed ActivityPub federation |
| `FEDERATION_INSTANCE_ID` | `default-instance` | Unique federation instance ID |
| `FEDERATION_INSTANCE_DOMAIN` | `localhost` | Public domain for federation |
| `RUST_LOG` | `civit_core=info,tower_http=debug` | Log filter (tracing format) |

### DATABASE_URL format

```
postgres://[USER]:[PASSWORD]@[HOST]:[PORT]/[DATABASE]
```

With connection pool parameters:
```
postgres://user:pass@host:5432/db?sslmode=disable&max_connections=20
```

### Notifications

Without SMTP or Slack configuration, notifications are logged but not sent (log-only mode). Configuration is programmatic via `NotificationChannelConfig`.

## Database Migrations

CivitForge runs migrations automatically on startup via sqlx. Migrations are numbered 001-021 (odd-numbered SQL files). To check status:

```sql
SELECT * FROM schema_migrations ORDER BY version;
```

## Ports

| Port | Service | Required |
|------|---------|-----------|
| 8080 (host 9091) | REST API + WebSocket | Yes |
| 2222 | Git SSH | No |
| 9090 | VFS gRPC | No |
| 8088 | Runner HTTP | No (if using runner) |

## Storage

| Path | Description | Persistent |
|------|-------------|-----------|
| `/var/lib/civit/repos` | Git bare repositories | Yes (critical) |
| `/var/log/civit` | Application logs | No |
| PostgreSQL data | User/org/repo/pipeline/issue/wiki/OCI metadata | Yes (critical) |
| Redis data | Session cache, edge cache, pub/sub (ephemeral) | Optional |

## Logging

CivitForge uses `tracing` with structured output. Configure via `RUST_LOG`:

```bash
RUST_LOG=civit_core=info          # Production
RUST_LOG=civit_core=debug,tower_http=trace  # Debug
```

## Health Checks

| Endpoint | Response | Purpose |
|----------|----------|---------|
| `GET /healthz` | `OK` (200) | Liveness probe |
| `GET /ready` | `OK` (200) | Readiness probe |
| `GET /api/v1/health` | `OK` (200) | API health |

Kubernetes probe example:
```yaml
livenessProbe:
  httpGet:
    path: /healthz
    port: 8080
  initialDelaySeconds: 10
  periodSeconds: 30

readinessProbe:
  httpGet:
    path: /ready
    port: 8080
  initialDelaySeconds: 5
  periodSeconds: 10
```

## Upgrading

### Docker Compose

```bash
git pull origin main
docker compose build
docker compose up -d
```

Migrations run automatically on startup.

### Kubernetes (Helm)

```bash
helm upgrade civitforge civitforge/civitforge \
  --namespace civitforge \
  --reuse-values
```

### Backup

```bash
pg_dump -Fc civit > civit-backup-$(date +%Y%m%d).dump
tar czf repos-backup-$(date +%Y%m%d).tar.gz /var/lib/civit/repos/
```

## Security Notes

- Generate JWT secret with `openssl rand -base64 32` (48 chars, above 16-char minimum)
- Use PostgreSQL SSL mode `verify-full` in production
- Enable Redis AUTH in production (`requirepass your-secret`)
- Bind to `0.0.0.0` only behind a reverse proxy; otherwise use `127.0.0.1`
- Container runs as nonroot (UID 65532) with all capabilities dropped
