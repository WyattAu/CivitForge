# CivitForge Operator Guide

Deployment, configuration, and operational procedures for CivitForge v0.8.0-alpha.

## Prerequisites

| Requirement | Minimum | Recommended |
|-------------|---------|-------------|
| CPU | 2 cores | 4+ cores |
| RAM | 4 GB | 8 GB |
| Disk | 20 GB SSD | 100 GB SSD |
| PostgreSQL | 15+ | 17 |
| Redis | 6+ | 7 |
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
curl http://localhost:8080/healthz  # => OK
```

Default credentials:
- **PostgreSQL:** `civit:civit` on `localhost:5432`
- **Redis:** no auth on `localhost:6379`
- **JWT Secret:** `change-me-change-me-dev-secret` (CHANGE FOR PRODUCTION)

## Quick Start (Podman)

```bash
podman kube play docker-compose.yml
```

Or individual containers:
```bash
# PostgreSQL
podman run -d --name civit-postgres \
  -e POSTGRES_DB=civit -e POSTGRES_USER=civit -e POSTGRES_PASSWORD=civit \
  -p 5432:5432 postgres:17-alpine

# Redis
podman run -d --name civit-redis \
  -p 6379:6379 redis:7-alpine

# CivitForge
podman run -d --name civit \
  -e DATABASE_URL=postgres://civit:civit@host.containers.internal:5432/civit \
  -e JWT_SECRET=your-production-secret \
  -p 8080:8080 -p 2222:2222 \
  ghcr.io/wyattau/evergreenimageregistry/civitforge:latest
```

## Quick Start (Source Build)

```bash
git clone https://github.com/WyattAu/CivitForge.git
cd CivitForge
cargo build --release --workspace
./target/release/civit-core
```

## Configuration Reference

All configuration is via environment variables. Required variables must be set; optional variables have sensible defaults.

### Required

| Variable | Description | Example |
|----------|-------------|---------|
| `DATABASE_URL` | PostgreSQL connection string (sqlx format) | `postgres://user:pass@host:5432/civit` |
| `JWT_SECRET` | JWT signing key, minimum 16 characters | `your-secret-key-at-least-16-chars` |

### Optional

| Variable | Default | Description |
|----------|---------|-------------|
| `CIVIT_HOST` | `127.0.0.1` | API bind address. Use `0.0.0.0` for all interfaces. |
| `CIVIT_PORT` | `8080` | API bind port |
| `REDIS_URL` | `redis://127.0.0.1:6379` | Redis for sessions and edge cache |
| `JWT_EXPIRY_HOURS` | `24` | JWT token expiration in hours |
| `CIVIT_STORAGE_PATH` | `/var/lib/civit/repos` | Git repository storage path |
| `FEDERATION_ENABLED` | `false` | Enable ForgeFed activitypub federation |
| `FEDERATION_INSTANCE_ID` | `default-instance` | Unique federation instance ID |
| `FEDERATION_INSTANCE_DOMAIN` | `localhost` | Public domain for federation |
| `RUST_LOG` | `civit_core=info,tower_http=debug` | Log filter (env_logger format) |

### DATABASE_URL Format

```
postgres://[USER]:[PASSWORD]@[HOST]:[PORT]/[DATABASE]
```

With connection pool parameters:
```
postgres://user:pass@host:5432/db?sslmode=disable&max_connections=20
```

### SMTP Configuration (Notifications)

SMTP is configured programmatically via `NotificationChannelConfig`. For environment-based setup:

```rust
NotificationChannelConfig {
    smtp: Some(SmtpConfig {
        host: "smtp.gmail.com",
        port: 587,
        username: "user@example.com",
        password: "app-password",
        from_address: "noreply@example.com",
        use_tls: true,
    }),
    ..Default::default()
}
```

Without SMTP config, email notifications are logged but not sent (log-only mode).

### Slack Configuration

```rust
NotificationChannelConfig {
    slack: Some(SlackConfig {
        bot_token: "xoxb-your-bot-token",
        default_channel: "#alerts",
    }),
    ..Default::default()
}
```

## Database Migrations

CivitForge runs migrations automatically on startup. The migration manager:

1. Checks `schema_migrations` table for current version
2. Applies all pending migrations in order
3. Records each migration in `schema_migrations`

Migrations are located in `civit-core/src/db/migrations/`.

To run migrations manually:
```sql
SELECT * FROM schema_migrations ORDER BY version;
```

## Ports

| Port | Service | Required |
|------|---------|-----------|
| 8080 | REST API + WebSocket | Yes |
| 2222 | Git SSH access | No |
| 9090 | VFS gRPC | No |
| 9101 | Metrics/health-shim | No |

## Storage

| Path | Description | Persistent |
|------|-------------|-----------|
| `/var/lib/civit/repos` | Git bare repositories | Yes (critical) |
| `/var/log/civit` | Application logs | No |
| PostgreSQL data | User/org/repo metadata | Yes (critical) |
| Redis data | Session cache (ephemeral) | Optional |

## Logging

CivitForge uses `tracing` with structured output. Configure via `RUST_LOG`:

```bash
# Production
RUST_LOG=civit_core=info

# Debug
RUST_LOG=civit_core=debug,tower_http=trace

# Trace all
RUST_LOG=trace
```

## Health Checks

| Endpoint | Response | Purpose |
|----------|----------|---------|
| `GET /healthz` | `OK` (200) | Liveness (always responds) |
| `GET /ready` | `OK` (200) | Readiness (same as healthz) |
| `GET /api/v1/health` | `OK` (200) | API health |

For container orchestrators:
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

Migrations run automatically on startup. No manual steps needed.

### Rolling Upgrade (Kubernetes)

```yaml
spec:
  strategy:
    type: RollingUpdate
    rollingUpdate:
      maxSurge: 1
      maxUnavailable: 0
```

### Backup

```bash
# Database backup
pg_dump -Fc civit > civit-backup-$(date +%Y%m%d).dump

# Repository backup (filesystem)
tar czf repos-backup-$(date +%Y%m%d).tar.gz /var/lib/civit/repos/
```

## Troubleshooting

### Server won't start

1. Check `DATABASE_URL` is set and PostgreSQL is reachable
2. Check `JWT_SECRET` is at least 16 characters
3. Check `CIVIT_STORAGE_PATH` directory exists and is writable
4. Check port 8080 is not already in use

### Migration failures

1. Check PostgreSQL version compatibility
2. Check `schema_migrations` table for partial migrations
3. Manually fix and update the version number

### High memory usage

1. Check `DATABASE_URL` pool size (reduce `max_connections`)
2. Check Redis connection limits
3. Profile with `civit-bench` to identify hot paths

### Performance issues

```bash
# Run benchmark
cargo run --release -p civit-core --bin civit-bench

# Run scale test (60s)
cargo run --release -p civit-core --bin civit-scale -- http://localhost:8080 60
```

## Security Notes

- **JWT Secret:** Generate with `openssl rand -base64 32` (48 chars, well above 16 minimum)
- **PostgreSQL:** Use SSL mode `verify-full` in production
- **Redis:** Enable AUTH in production (`requirepass your-secret`)
- **Network:** Bind to `0.0.0.0` only behind a reverse proxy; otherwise use `127.0.0.1`
- **Container:** Image runs as nonroot (UID 65532) with all capabilities dropped
