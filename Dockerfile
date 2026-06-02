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

WORKDIR /app

# Cache dependencies by copying Cargo files first
COPY Cargo.toml Cargo.lock rust-toolchain.toml ./
COPY crates/civit-shared/ crates/civit-shared/
COPY crates/civit-pipeline/ crates/civit-pipeline/
COPY crates/civit-core/ crates/civit-core/
COPY crates/civit-brain/ crates/civit-brain/
COPY crates/civit-crypto/ crates/civit-crypto/
COPY crates/civit-runner/ crates/civit-runner/
COPY crates/civit-vfs/ crates/civit-vfs/

# Build all workspace binaries
RUN cargo build --release --locked \
    -p civit-core -p civit-brain -p civit-runner -p civit-vfs

# ---------------------------------------------------------------------------
# Stage 2: Runtime (wolfi-base per Evergreen ADR-007)
# ---------------------------------------------------------------------------
FROM cgr.dev/chainguard/wolfi-base:latest

ARG VERSION=1.0.0-rc.3

# Runtime dependencies
RUN apk add --no-cache ca-certificates git su-exec wget

# Create nonroot user and directories
RUN addgroup -g 65532 civit && \
    adduser -D -u 65532 -G civit -h /data -s /bin/sh civit && \
    mkdir -p /data /var/lib/civit/repos /var/log/civit && \
    chown -R civit:civit /data /var/lib/civit /var/log/civit

# Copy binaries
COPY --from=builder /app/target/release/civit-core  /usr/local/bin/civit-core
COPY --from=builder /app/target/release/civit-brain  /usr/local/bin/civit-brain
COPY --from=builder /app/target/release/civit-runner /usr/local/bin/civit-runner
COPY --from=builder /app/target/release/civit-vfs    /usr/local/bin/civit-vfs

WORKDIR /data
ENV CIVIT_STORAGE_PATH=/var/lib/civit/repos

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
