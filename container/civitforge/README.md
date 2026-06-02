# civitforge

CivitForge — federated, Rust-native software forge for extreme-scale monorepos.

## Quick Start

```bash
docker run -d \
  --name civit \
  -p 8080:8080 \
  -e DATABASE_URL=postgres://civit:civit@db:5432/civit \
  -e JWT_SECRET=your-secret-key-at-least-16-chars \
  -v civit-data:/var/lib/civit/repos \
  ghcr.io/wyattau/evergreenimageregistry/civitforge:latest
```

With docker-compose:

```bash
curl -sSL https://raw.githubusercontent.com/WyattAu/CivitForge/main/docker-compose.yml | docker compose -f - up -d
```

Verify:

```bash
curl http://localhost:8080/healthz  # => OK
curl http://localhost:8080/api/v1/health  # => OK
```

## Ports

| Port | Service | Description |
|------|---------|-------------|
| 8080 | API | HTTP REST API + WebSocket |
| 2222 | SSH | Git SSH operations (optional) |
| 9090 | gRPC | VFS remote operations |
| 9101 | Metrics | Health-shim / metrics endpoint |

## Environment Variables

### Required

| Variable | Description | Example |
|----------|-------------|---------|
| `DATABASE_URL` | PostgreSQL connection string | `postgres://user:pass@host:5432/db` |
| `JWT_SECRET` | JWT signing secret (>=16 chars) | `change-me-change-me` |

### Optional

| Variable | Default | Description |
|----------|---------|-------------|
| `CIVIT_HOST` | `127.0.0.1` | Bind address |
| `CIVIT_PORT` | `8080` | Bind port |
| `REDIS_URL` | `redis://127.0.0.1:6379` | Redis for sessions + edge cache |
| `JWT_EXPIRY_HOURS` | `24` | Token expiration |
| `CIVIT_STORAGE_PATH` | `/var/lib/civit/repos` | Git repository storage |
| `FEDERATION_ENABLED` | `false` | Enable ForgeFed federation |
| `FEDERATION_INSTANCE_ID` | `default-instance` | Federation instance identifier |
| `FEDERATION_INSTANCE_DOMAIN` | `localhost` | Federation public domain |
| `RUST_LOG` | `civit_core=info,tower_http=debug` | Log level filter |

## Volumes

| Path | Description |
|------|-------------|
| `/var/lib/civit/repos` | Git repository storage (persistent) |
| `/var/log/civit` | Application logs |
| `/data` | Working directory |

## Runtime UID Override

The image supports `APP_UID`/`APP_GID` for runtime user override:

```bash
docker run -e APP_UID=1000 -e APP_GID=1000 civitforge:latest
```

Default: `65532:65532` (nonroot, OpenShift SCC compatible).

## Security

- Runs as nonroot user (UID 65532)
- All Linux capabilities dropped (`cap-drop=ALL`)
- No new privileges
- Read-only rootfs (data volumes are writable)
- seccomp runtime-default profile

## Image Details

| Attribute | Value |
|-----------|-------|
| Base image | `cgr.dev/chainguard/wolfi-base` (glibc) |
| Tier | critical |
| Architecture | linux/amd64, linux/arm64 |
| Licenses | AGPL-3.0-or-later |
| Source | [github.com/WyattAu/CivitForge](https://github.com/WyattAu/CivitForge) |

## Conformance Checklist

- [x] Approved base image (wolfi, per ADR-007)
- [x] Nonroot USER 65532:65532
- [x] HEALTHCHECK with standard parameters
- [x] OCI standard labels
- [x] Evergreen metadata labels
- [x] Security hardening labels
- [x] STOPSIGNAL SIGTERM
- [x] Multi-arch (amd64 + arm64)
