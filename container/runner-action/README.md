# civitforge-runner-action

CivitForge CI runner action image — extends runner-base with the Rust toolchain for building Rust projects.

## Quick Start

```bash
docker run --rm \
  -e CIVIT_SERVER_URL=http://civit:8080 \
  -e RUNNER_TOKEN=your-runner-token \
  -v /path/to/repo:/home/runner/work \
  ghcr.io/wyattau/evergreenimageregistry/civitforge-runner-action:latest
```

## Included Tools (inherited from runner-base + added)

| Tool | Version | Purpose |
|------|---------|---------|
| git | system | Repository checkout |
| Podman | 5.3.1 | Container execution |
| Rust | 1.88 | Rust compilation (stable channel) |
| Cargo | 1.88 | Rust package manager |
| gcc | system | C compilation (for Rust native deps) |
| musl-dev | system | Static linking support |
| pkg-config | system | Build configuration |
| openssl-dev | system | TLS support for Rust crates |

## Rust Toolchain

Installed via `rustup` with minimal profile to reduce image size:
- `rustc` 1.88 (stable)
- `cargo` 1.88
- `rust-std`

Add components at runtime if needed:

```bash
rustup component add clippy rustfmt
```

## Ports

| Port | Service | Description |
|------|---------|-------------|
| 8088 | Runner HTTP | Runner API / callback endpoint |
| 9101 | Metrics | Health-shim / metrics endpoint |

## Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `CIVIT_RUNNER_WORKDIR` | `/home/runner/work` | Pipeline workspace |
| `RUSTUP_HOME` | `/home/runner/.rustup` | Rustup installation directory |
| `CARGO_HOME` | `/home/runner/.cargo` | Cargo cache directory |
| `APP_UID` | `65532` | Runtime user ID |
| `APP_GID` | `65532` | Runtime group ID |

## Cargo Cache

Mount a persistent volume for Cargo registry/cache to speed up builds:

```bash
docker run -v cargo-cache:/home/runner/.cargo \
  -v rustup-cache:/home/runner/.rustup \
  civitforge-runner-action:latest
```

## Image Details

| Attribute | Value |
|-----------|-------|
| Base image | `cgr.dev/chainguard/wolfi-base` (glibc) |
| Tier | community |
| Architecture | linux/amd64 only (rustup multiarch fails under QEMU) |
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
- [x] amd64 only (multiarch = false, rustup QEMU limitation)
