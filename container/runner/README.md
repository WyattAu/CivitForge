# civitforge-runner

CivitForge CI/CD pipeline daemon. Polls the CivitForge server for pipeline jobs and executes each step inside an ephemeral Podman container.

This image does not contain language toolchains (Rust, Go, Node, etc.). Each step's `image:` in `.civit/pipeline.yaml` specifies the container image to use.

## Quick Start

```bash
docker run -d \
  --name civit-runner \
  -v /var/run/podman/podman.sock:/var/run/podman/podman.sock:ro \
  -e CIVIT_SERVER_URL=http://civitforge:8080 \
  -e RUNNER_TOKEN=your-runner-token \
  ghcr.io/wyattau/evergreenimageregistry/civitforge-runner:latest
```

## Contents

| Tool | Purpose |
|------|---------|
| `civit-runner` | Pipeline daemon binary |
| Podman 5.8.2 | Container runtime |
| git | Repository checkout |
| make, bash | Build orchestration |

## Ports

| Port | Service |
|------|---------|
| 8088 | Runner HTTP API |
| 9101 | Metrics |

## Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `CIVIT_SERVER_URL` | *(required)* | CivitForge server URL |
| `RUNNER_TOKEN` | *(required)* | Authentication token for runner registration |
| `CIVIT_RUNNER_WORKDIR` | `/home/runner/work` | Pipeline workspace root |

## Volumes

| Path | Description |
|------|-------------|
| `/home/runner/work` | Pipeline workspace (shared across steps) |
| `/tmp/civit-runner` | Temporary build artifacts |
| `/var/run/podman/podman.sock` | Podman socket (required) |

## Image Details

| Attribute | Value |
|-----------|-------|
| Base image | `cgr.dev/chainguard/wolfi-base` (glibc) |
| Tier | standard |
| Architecture | linux/amd64, linux/arm64 |
| Licenses | AGPL-3.0-or-later |
