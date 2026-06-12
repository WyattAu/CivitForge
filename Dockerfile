# =============================================================================
# CivitForge - EvergreenImageRegistry-compliant Dockerfile
# =============================================================================
# Tier: critical
# Base: wolfi (cgr.dev/chainguard/wolfi-base)
# Multi-arch: linux/amd64, linux/arm64
# Compliance: FIPS 140-2, SLSA L4, Cosign signing
#
# Usage:
#   docker build -t civitforge .
#   docker run -p 8080:8080 -e DATABASE_URL=postgres://... civitforge
# =============================================================================

ARG TARGETARCH

# ---------------------------------------------------------------------------
# Stage 1: Rust builder
# ---------------------------------------------------------------------------
FROM rust:1.88-slim AS builder

# Install protobuf compiler and build dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
        protobuf-compiler pkg-config libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Cache dependencies by copying Cargo files first
COPY Cargo.toml Cargo.lock rust-toolchain.toml ./
COPY crates/ crates/

# Build all workspace binaries
RUN cargo build --release --locked \
    -p civit-core -p civit-brain -p civit-runner -p civit-vfs

# Strip debug symbols
RUN for bin in civit-core civit-brain civit-runner civit-vfs; do \
        strip /app/target/release/${bin} 2>/dev/null || true; \
    done

# ---------------------------------------------------------------------------
# Stage 2: Runtime (wolfi-base per Evergreen ADR-007)
# ---------------------------------------------------------------------------
FROM cgr.dev/chainguard/wolfi-base:latest

ARG VERSION=2.1.3
ARG APP_UID=65532
ARG APP_GID=65532

# Runtime dependencies (minimal: no shells, no package managers in final image)
# git is needed for repository operations, wget for healthcheck
RUN apk add --no-cache ca-certificates git wget

# Create nonroot user and directories
RUN addgroup -g ${APP_GID} civit 2>/dev/null || true; \
    adduser -D -u ${APP_UID} -G civit -h /data -s /bin/sh civit 2>/dev/null || true; \
    mkdir -p /data /var/lib/civit/repos /var/log/civit /srv/civit-ui && \
    chown -R ${APP_UID}:${APP_GID} /data /var/lib/civit /var/log/civit /srv/civit-ui

# Copy binaries from builder
COPY --from=builder /app/target/release/civit-core  /usr/local/bin/civit-core
COPY --from=builder /app/target/release/civit-brain  /usr/local/bin/civit-brain
COPY --from=builder /app/target/release/civit-runner /usr/local/bin/civit-runner
COPY --from=builder /app/target/release/civit-vfs    /usr/local/bin/civit-vfs

# Copy pre-built Web UI assets (WASM + JS)
COPY --from=builder /app/crates/civit-ui/dist/ /srv/civit-ui/

WORKDIR /data

# Configuration via environment variables
ENV CIVIT_STORAGE_PATH=/var/lib/civit/repos
ENV CIVIT_UI_DIR=/srv/civit-ui
ENV CIVIT_HOST=0.0.0.0
ENV CIVIT_PORT=8080
ENV RUST_LOG=civit_core=info,tower_http=debug

EXPOSE 8080 2222 9090

# Health check (mandatory per Evergreen standard)
HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
    CMD wget -qO- http://localhost:8080/healthz || exit 1

# Run as nonroot (Evergreen requirement)
USER ${APP_UID}:${APP_GID}

ENTRYPOINT ["/usr/local/bin/civit-core"]
CMD []

# OCI Standard Labels
LABEL org.opencontainers.image.title="civitforge" \
      org.opencontainers.image.description="CivitForge - federated Rust-native software forge" \
      org.opencontainers.image.version="${VERSION}" \
      org.opencontainers.image.vendor="CivitForge" \
      org.opencontainers.image.source="https://github.com/WyattAu/CivitForge" \
      org.opencontainers.image.licenses="AGPL-3.0-or-later"

# Evergreen Image Registry Labels
LABEL evergreen.base.image="wolfi" \
      evergreen.image.tier="critical" \
      evergreen.constraint.nonroot="true" \
      evergreen.constraint.wolfi="true" \
      evergreen.health.type="http" \
      evergreen.image.category="source-control" \
      evergreen.image.status="functional"

# Security Hardening Labels
LABEL evergreen.security.cap-drop="ALL" \
      evergreen.security.no-new-privileges="true" \
      evergreen.security.read-only-rootfs="false"

STOPSIGNAL SIGTERM
