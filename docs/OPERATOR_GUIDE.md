# CivitForge Operator Guide

Deployment, configuration, and operational procedures for CivitForge v1.1.0.

## System Requirements

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

## Quick Start (Helm)

```bash
helm install civitforge deploy/helm/civitforge \
  --namespace civitforge --create-namespace \
  --set postgresql.host=your-pg-host \
  --set redis.host=your-redis-host \
  --set jwtSecret=$(openssl rand -base64 32)
```

## Configuration Reference

All configuration is via environment variables.

### Required

| Variable | Description | Example |
|----------|-------------|---------|
| `DATABASE_URL` | PostgreSQL connection string (sqlx format) | `postgres://user:pass@host:5432/civit` |
| `JWT_SECRET` | JWT signing key, minimum 32 characters | Generate with `openssl rand -base64 32` |

### Core

| Variable | Default | Description |
|----------|---------|-------------|
| `CIVIT_HOST` | `127.0.0.1` | API bind address. Use `0.0.0.0` for all interfaces. |
| `CIVIT_PORT` | `8080` | API bind port |
| `REDIS_URL` | `redis://127.0.0.1:6379` | Redis for sessions and edge cache |
| `JWT_EXPIRY_HOURS` | `24` | JWT token expiration in hours |
| `CIVIT_STORAGE_PATH` | `/var/lib/civit/repos` | Git repository storage path |
| `CIVIT_ENCRYPTION_KEY` | *(none)* | AES-256-GCM key for pipeline variable encryption |
| `CIVIT_DEBUG` | `false` | Enable debug mode |
| `UI_ASSETS_PATH` | `./crates/civit-ui/dist` | Path to compiled Leptos UI assets |

### TLS

| Variable | Default | Description |
|----------|---------|-------------|
| `TLS_CERT_PATH` | *(none)* | Path to TLS certificate (PEM). Enables HTTPS when set. |
| `TLS_KEY_PATH` | *(none)* | Path to TLS private key (PEM) |

### Federation

| Variable | Default | Description |
|----------|---------|-------------|
| `FEDERATION_ENABLED` | `false` | Enable ForgeFed ActivityPub federation |
| `FEDERATION_INSTANCE_ID` | `default-instance` | Unique federation instance ID |
| `FEDERATION_INSTANCE_DOMAIN` | `localhost` | Public domain for federation |

### LDAP

| Variable | Default | Description |
|----------|---------|-------------|
| `LDAP_ENABLED` | `false` | Enable LDAP authentication backend |
| `LDAP_URL` | `ldap://localhost:389` | LDAP server URL |
| `LDAP_BIND_DN` | *(empty)* | Bind DN for LDAP authentication |
| `LDAP_BIND_PASSWORD` | *(empty)* | Bind password for LDAP authentication |
| `LDAP_USER_SEARCH_BASE` | `ou=users` | Base DN for user searches |
| `LDAP_USER_FILTER` | `(uid={})` | User search filter (use `{}` as username placeholder) |
| `LDAP_GROUP_SEARCH_BASE` | `ou=groups` | Base DN for group searches |
| `LDAP_GROUP_FILTER` | `(memberUid={})` | Group search filter |
| `LDAP_MAX_CONNECTIONS` | `10` | Max LDAP connection pool size |
| `LDAP_TLS_CA_PATH` | *(none)* | Path to CA certificate for LDAPS |
| `LDAP_CONNECTION_TIMEOUT_SECS` | `10` | LDAP connection timeout |

### Security

| Variable | Default | Description |
|----------|---------|-------------|
| `LOGIN_MAX_ATTEMPTS` | `5` | Max failed login attempts before lockout |
| `LOGIN_LOCKOUT_SECS` | `900` | Lockout duration in seconds (15 min) |
| `PASSWORD_MIN_LENGTH` | `8` | Minimum password length |
| `PASSWORD_MAX_LENGTH` | `128` | Maximum password length |
| `PASSWORD_REQUIRE_UPPERCASE` | `true` | Require uppercase letter in password |
| `PASSWORD_REQUIRE_LOWERCASE` | `true` | Require lowercase letter in password |
| `PASSWORD_REQUIRE_DIGIT` | `true` | Require digit in password |
| `PASSWORD_REQUIRE_SPECIAL` | `true` | Require special character in password |

### Rate Limiting & CORS

| Variable | Default | Description |
|----------|---------|-------------|
| `CORS_ALLOWED_ORIGINS` | *(empty)* | Comma-separated allowed origins for CORS |
| `RATE_LIMIT_MAX_REQUESTS` | *(none)* | Max requests per window per IP |
| `RATE_LIMIT_WINDOW_SECS` | *(none)* | Rate limit window duration |

### Logging

| Variable | Default | Description |
|----------|---------|-------------|
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

CivitForge runs migrations automatically on startup via sqlx. Migrations are numbered 001-049 (odd-numbered SQL files, 32 total). Rollback migrations are available in `crates/civit-db/src/migrations/down/`.

To check migration status:
```sql
SELECT * FROM schema_migrations ORDER BY version;
```

To manually apply migrations (if auto-migrate is disabled):
```bash
cargo sqlx migrate run --source crates/civit-db/src/migrations
```

To rollback the last migration:
```bash
cargo sqlx migrate revert --source crates/civit-db/src/migrations
```

## Ports

| Port | Service | Required |
|------|---------|----------|
| 8080 (host 9091) | REST API + WebSocket | Yes |
| 2222 | Git SSH | No |
| 9090 | VFS gRPC | No |
| 8088 | Runner HTTP | No (if using runner) |

## Storage

| Path | Description | Persistent |
|------|-------------|------------|
| `/var/lib/civit/repos` | Git bare repositories | Yes (critical) |
| `/var/log/civit` | Application logs | No |
| PostgreSQL data | User/org/repo/pipeline/issue/wiki/OCI metadata | Yes (critical) |
| Redis data | Session cache, edge cache, pub/sub (ephemeral) | Optional |

## Monitoring

### Health Checks

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

### Logging

CivitForge uses `tracing` with structured output. Configure via `RUST_LOG`:

```bash
RUST_LOG=civit_core=info          # Production
RUST_LOG=civit_core=debug,tower_http=trace  # Debug
```

### Metrics

OTLP exporter is built-in (no external SDK dependency). Configure your OTLP collector endpoint to receive traces and metrics.

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
helm upgrade civitforge deploy/helm/civitforge \
  --namespace civitforge \
  --reuse-values
```

### Source Build

```bash
git pull origin main
cargo build --release --workspace
# Stop the running instance, replace binary, restart
```

## Backup and Recovery

### Backup

```bash
# Database
pg_dump -Fc civit > civit-backup-$(date +%Y%m%d).dump

# Git repositories
tar czf repos-backup-$(date +%Y%m%d).tar.gz /var/lib/civit/repos/
```

### Recovery

```bash
# Restore database
pg_restore -Fc -d civit civit-backup-YYYYMMDD.dump

# Restore repositories
tar xzf repos-backup-YYYYMMDD.tar.gz -C /
```

## Troubleshooting

### Common Issues

**Database connection refused**
- Verify PostgreSQL is running: `pg_isready -h localhost -p 5432`
- Check `DATABASE_URL` format: `postgres://user:pass@host:port/dbname`
- Ensure the database exists and the user has permissions

**Redis connection refused**
- Verify Redis is running: `redis-cli ping`
- Check `REDIS_URL` format: `redis://:password@host:port`

**JWT validation errors**
- `JWT_SECRET` must be at least 32 characters
- Regenerate with: `openssl rand -base64 32`

**LDAP authentication failures**
- Verify LDAP server is reachable: `ldapsearch -x -H ldap://localhost:389 -b "ou=users" -D "bind_dn" -w "password"`
- Check `LDAP_BIND_DN` and `LDAP_BIND_PASSWORD` are correct
- For LDAPS, ensure `LDAP_TLS_CA_PATH` points to a valid CA certificate

**Port 8080 already in use**
- Change `CIVIT_PORT` or stop the conflicting service
- In Docker, check port mappings: `docker compose ps`

**Migration errors**
- Check current migration state: `SELECT * FROM schema_migrations;`
- Ensure `DATABASE_URL` user has CREATE/ALTER permissions
- Rollback if needed: `cargo sqlx migrate revert`

**Git SSH not working**
- Ensure port 2222 is exposed in your container/network
- Verify Ed25519 host key is generated on first start
- Check client SSH config: `Host forge.example.com Port 2222`

## Security Notes

- Generate JWT secret with `openssl rand -base64 32` (48 chars, above 32-char minimum)
- Use PostgreSQL SSL mode `verify-full` in production
- Enable Redis AUTH in production (`requirepass your-secret`)
- Bind to `0.0.0.0` only behind a reverse proxy; otherwise use `127.0.0.1`
- Container runs as nonroot (UID 65532) with all capabilities dropped
- LDAP connections should use LDAPS (port 636) with TLS CA verification in production
