# civitforge-runner

CivitForge CI/CD pipeline daemon. Polls the CivitForge server for pipeline jobs and executes each step inside an ephemeral Podman container.

**This image does NOT contain language toolchains** (Rust, Go, Node, etc.). Each step's `image:` in `.civit/pipeline.yaml` specifies the container image to use — users bring their own toolchains, just like ForgeJo, GitLab CI, and GitHub Actions.

## Quick Start

```bash
docker run -d \
  --name civit-runner \
  -v /var/run/podman/podman.sock:/var/run/podman/podman.sock:ro \
  -e CIVIT_SERVER_URL=http://civitforge:8080 \
  -e RUNNER_TOKEN=your-runner-token \
  ghcr.io/wyattau/evergreenimageregistry/civitforge-runner:latest
```

## What's Inside

| Tool | Purpose |
|------|---------|
| `civit-runner` | Pipeline daemon binary (polls server, orchestrates steps) |
| Podman 5.8.2 | Container runtime (spawns step containers) |
| git | Repository checkout for workspace setup |
| make, bash | Build orchestration and script execution |

## What's NOT Inside

Toolchains for individual steps come from the user's pipeline YAML:

```yaml
# .civit/pipeline.yaml
steps:
  - name: build
    image: rust:1.88        # ← user specifies this
    commands: cargo build --release
  - name: lint
    image: node:20-alpine    # ← or this
    commands: npm run lint
  - name: test
    image: alpine:3.20       # ← or this
    commands: ./run-tests.sh
```

## Ports

| Port | Service | Description |
|------|---------|-------------|
| 8088 | Runner HTTP | API / webhook callback endpoint |
| 9101 | Metrics | Health-shim / Prometheus metrics |

## Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `CIVIT_SERVER_URL` | *(required)* | CivitForge server URL |
| `RUNNER_TOKEN` | *(required)* | Authentication token for runner registration |
| `CIVIT_RUNNER_WORKDIR` | `/home/runner/work` | Pipeline workspace root |

## Volumes

| Path | Description |
|------|-------------|
| `/home/runner/work` | Pipeline workspace (shared across steps via volume mounts) |
| `/tmp/civit-runner` | Temporary build artifacts |
| `/var/run/podman/podman.sock` | Podman socket (required for container execution) |

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
