# civitforge

CivitForge server image. Federated, Rust-native software forge.

## Quick Start

```bash
docker run -d \
  --name civit \
  -p 9091:8080 \
  -e DATABASE_URL=postgres://civit:civit@db:5432/civit \
  -e JWT_SECRET=your-secret-key-at-least-16-chars \
  -v civit-data:/var/lib/civit/repos \
  ghcr.io/wyattau/evergreenimageregistry/civitforge:latest
```

Verify:
```bash
curl http://localhost:9091/healthz
curl http://localhost:9091/api/v1/health
```

## Ports

| Container Port | Service |
|----------------|---------|
| 8080 | HTTP REST API + WebSocket |
| 2222 | Git SSH (optional) |
| 9090 | VFS gRPC |

## Environment Variables

### Required

| Variable | Description | Example |
|----------|-------------|---------|
| `DATABASE_URL` | PostgreSQL connection string | `postgres://user:pass@host:5432/db` |
| `JWT_SECRET` | JWT signing secret (>=16 chars) | `openssl rand -base64 32` |

### Optional

| Variable | Default | Description |
|----------|---------|-------------|
| `CIVIT_HOST` | `127.0.0.1` | Bind address |
| `CIVIT_PORT` | `8080` | Bind port |
| `REDIS_URL` | `redis://127.0.0.1:6379` | Redis for sessions + edge cache |
| `JWT_EXPIRY_HOURS` | `24` | Token expiration |
| `CIVIT_STORAGE_PATH` | `/var/lib/civit/repos` | Git repository storage |
| `CIVIT_ENCRYPTION_KEY` | *(none)* | AES-256-GCM key for pipeline variable encryption |
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

Supports `APP_UID`/`APP_GID` for runtime user override. Default: `65532:65532` (nonroot, OpenShift SCC compatible).

## Security

- Runs as nonroot user (UID 65532)
- All Linux capabilities dropped
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
