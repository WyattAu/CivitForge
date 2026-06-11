# =============================================================================
# CivitForge - Legacy convenience Dockerfile (top-level)
# =============================================================================
# This is the original convenience Dockerfile at the repo root.
# For EvergreenImageRegistry-compliant builds, use container/civitforge/Dockerfile
# which follows wolfi-base, nonroot, healthcheck, and OCI label conventions.
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

# Install protobuf compiler and build dependencies for civit-vfs
RUN apt-get update && apt-get install -y protobuf-compiler pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Cache dependencies by copying Cargo files first
COPY Cargo.toml Cargo.lock rust-toolchain.toml ./
COPY crates/ crates/

# Build all workspace binaries
RUN cargo build --release --locked \
    -p civit-core -p civit-brain -p civit-runner -p civit-vfs

# Strip debug symbols and collect
RUN for bin in civit-core civit-brain civit-runner civit-vfs; do \
        strip /app/target/release/${bin} 2>/dev/null || true; \
    done

# ---------------------------------------------------------------------------
# Stage 2: Runtime (wolfi-base per Evergreen ADR-007)
# ---------------------------------------------------------------------------
FROM cgr.dev/chainguard/wolfi-base:latest

ARG VERSION=1.1.0

# Runtime dependencies
RUN apk add --no-cache ca-certificates git su-exec wget

# Create nonroot user and directories
RUN addgroup -g 65532 civit 2>/dev/null; \
    adduser -D -u 65532 -G civit -h /data -s /bin/sh civit 2>/dev/null; \
    mkdir -p /data /var/lib/civit/repos /var/log/civit /srv/civit-ui && \
    chown -R 65532:65532 /data /var/lib/civit /var/log/civit /srv/civit-ui

# Copy binaries
COPY --from=builder /app/target/release/civit-core  /usr/local/bin/civit-core
COPY --from=builder /app/target/release/civit-brain  /usr/local/bin/civit-brain
COPY --from=builder /app/target/release/civit-runner /usr/local/bin/civit-runner
COPY --from=builder /app/target/release/civit-vfs    /usr/local/bin/civit-vfs

# Copy pre-built Web UI assets (WASM + JS)
COPY --from=builder /app/crates/civit-ui/dist/ /srv/civit-ui/

WORKDIR /data
ENV CIVIT_STORAGE_PATH=/var/lib/civit/repos
ENV CIVIT_UI_DIR=/srv/civit-ui

EXPOSE 8080 2222 9090

HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
    CMD wget -qO- http://localhost:8080/healthz || exit 1

USER 65532:65532

ENTRYPOINT ["/usr/local/bin/civit-core"]
CMD []

LABEL org.opencontainers.image.title="civitforge" \
      org.opencontainers.image.description="CivitForge - federated Rust-native software forge" \
      org.opencontainers.image.version="${VERSION}" \
      org.opencontainers.image.vendor="CivitForge" \
      evergreen.base.image="wolfi" \
      evergreen.image.tier="critical" \
      evergreen.constraint.nonroot="true"

STOPSIGNAL SIGTERM
