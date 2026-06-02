# civitforge-runner-base

CivitForge CI runner base image — provides git, Podman CLI, and common build tools for pipeline execution.

This image is intended as a `FROM` base for action-specific runner images. See `container/runner-action/` for the Rust action image.

## Quick Start

```bash
docker run --rm \
  -e CIVIT_SERVER_URL=http://civit:8080 \
  -e RUNNER_TOKEN=your-runner-token \
  ghcr.io/wyattau/evergreenimageregistry/civitforge-runner-base:latest
```

## Included Tools

| Tool | Version | Purpose |
|------|---------|---------|
| git | system | Repository checkout |
| Podman | 5.3.1 | Container execution (OCI, rootless-compatible) |
| make | system | Build orchestration |
| bash | system | Script execution |
| wget | system | Health checks, downloads |
| su-exec | system | Nonroot execution |

## Ports

| Port | Service | Description |
|------|---------|-------------|
| 8088 | Runner HTTP | Runner API / callback endpoint |
| 9101 | Metrics | Health-shim / metrics endpoint |

## Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `CIVIT_RUNNER_WORKDIR` | `/home/runner/work` | Pipeline workspace |
| `APP_UID` | `65532` | Runtime user ID |
| `APP_GID` | `65532` | Runtime group ID |

## Volumes

| Path | Description |
|------|-------------|
| `/home/runner/work` | Pipeline workspace (mount pipeline source here) |
| `/tmp/civit-runner` | Temporary build artifacts |

## Image Details

| Attribute | Value |
|-----------|-------|
| Base image | `cgr.dev/chainguard/wolfi-base` (glibc) |
| Tier | standard |
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
