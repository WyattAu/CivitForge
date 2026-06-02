# =============================================================================
# CivitForge - Federated Rust-native Software Forge
# EvergreenImageRegistry-compliant container image
# =============================================================================
# Tier: critical
# Base: wolfi (cgr.dev/chainguard/wolfi-base)
# Multi-arch: linux/amd64, linux/arm64
# =============================================================================

ARG TARGETARCH

# ---------------------------------------------------------------------------
# Stage 1: Rust builder (debian:bookworm as permitted downloader/builder)
# ---------------------------------------------------------------------------
FROM debian:bookworm AS builder

ARG TARGETARCH
ARG RUST_VERSION=1.88

RUN apt-get update && \
    apt-get install -y --no-install-recommends \
        build-essential \
        curl \
        ca-certificates \
        pkg-config \
        libssl-dev \
        protobuf-compiler \
        git \
    && rm -rf /var/lib/apt/lists/*

# Install Rust toolchain
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y \
        --default-toolchain ${RUST_VERSION} \
        --profile minimal \
        --target x86_64-unknown-linux-gnu
ENV PATH="/root/.cargo/bin:${PATH}"

# Create placeholder to satisfy BuildKit COPY eval (Evergreen P3)
RUN mkdir -p /out/usr/local/bin
RUN touch /out/usr/local/bin/civitforge

# Copy source
WORKDIR /src
COPY Cargo.toml Cargo.lock rust-toolchain.toml ./
COPY crates/civit-shared/ crates/civit-shared/
COPY crates/civit-pipeline/ crates/civit-pipeline/
COPY crates/civit-core/ crates/civit-core/
COPY crates/civit-brain/ crates/civit-brain/
COPY crates/civit-crypto/ crates/civit-crypto/
COPY crates/civit-runner/ crates/civit-runner/
COPY crates/civit-vfs/ crates/civit-vfs/
COPY crates/civit-ui/ crates/civit-ui/

# Build release binaries
RUN cargo build --release --locked \
    -p civit-core -p civit-brain -p civit-runner -p civit-vfs

# Strip debug symbols and copy to /out
RUN for bin in civit-core civit-brain civit-runner civit-vfs; do \
        strip /src/target/release/${bin} 2>/dev/null || true; \
        cp /src/target/release/${bin} /out/usr/local/bin/${bin}; \
    done && \
    test -f /out/usr/local/bin/civit-core && \
    test -f /out/usr/local/bin/civit-brain && \
    test -f /out/usr/local/bin/civit-runner && \
    test -f /out/usr/local/bin/civit-vfs

# ---------------------------------------------------------------------------
# Stage 2: Entrypoint script builder (debian for shell script creation)
# ---------------------------------------------------------------------------
FROM debian:bookworm AS entrypoint-builder

ARG APP_UID=65532
ARG APP_GID=65532

RUN mkdir -p /out/etc && touch /out/etc/entrypoint.sh
RUN printf '#!/bin/sh\n\
set -e\n\
\n\
# Configurable runtime UID override (Evergreen C028)\n\
CURRENT_UID=$(id -u)\n\
DESIRED_UID=${APP_UID:-65532}\n\
\n\
if [ "$CURRENT_UID" != "$DESIRED_UID" ]; then\n\
    echo "Switching from UID $CURRENT_UID to UID $DESIRED_UID"\n\
    addgroup -g ${APP_GID:-65532} civit 2>/dev/null || true\n\
    adduser -D -u "$DESIRED_UID" -G civit -h /data civit 2>/dev/null || true\n\
    chown -R "$DESIRED_UID:${APP_GID:-65532}" /data /var/log/civit 2>/dev/null || true\n\
    exec su-exec "$DESIRED_UID" /usr/local/bin/civit-core "$@"\n\
else\n\
    exec /usr/local/bin/civit-core "$@"\n\
fi\n' > /out/etc/entrypoint.sh && \
    chmod +x /out/etc/entrypoint.sh && \
    test -f /out/etc/entrypoint.sh

# ---------------------------------------------------------------------------
# Stage 3: Final image (wolfi-base per Evergreen ADR-007)
# ---------------------------------------------------------------------------
FROM cgr.dev/chainguard/wolfi-base:latest

ARG APP_UID=65532
ARG APP_GID=65532
ARG VERSION=1.1.0

# Install runtime dependencies
RUN apk add --no-cache \
    ca-certificates \
    git \
    su-exec \
    wget

# Use existing nonroot user (Evergreen: wolfi-base ships 65532:65532)
# Set home to /data for our application
RUN mkdir -p /data /var/log/civit /var/lib/civit/repos && \
    chown -R 65532:65532 /data /var/log/civit /var/lib/civit

# Copy binaries from builder
COPY --from=builder /out/usr/local/bin/civit-core   /usr/local/bin/civit-core
COPY --from=builder /out/usr/local/bin/civit-brain   /usr/local/bin/civit-brain
COPY --from=builder /out/usr/local/bin/civit-runner  /usr/local/bin/civit-runner
COPY --from=builder /out/usr/local/bin/civit-vfs     /usr/local/bin/civit-vfs

# Copy entrypoint script
COPY --from=entrypoint-builder /out/etc/entrypoint.sh /etc/entrypoint.sh

WORKDIR /data
ENV CIVIT_STORAGE_PATH=/var/lib/civit/repos
ENV RUST_LOG=civit_core=info,tower_http=debug

# Ports: 8080=API, 2222=SSH, 9090=gRPC/VFS, 9101=metrics health-shim
EXPOSE 8080 2222 9090 9101

# Health check (Evergreen: wolfi has wget, NOT curl)
HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
    CMD wget -qO- http://localhost:8080/healthz || exit 1

USER 65532:65532

ENTRYPOINT ["/etc/entrypoint.sh"]
CMD []

# --- OCI Standard Labels ---
LABEL org.opencontainers.image.title="civitforge" \
      org.opencontainers.image.description="CivitForge - federated Rust-native software forge for extreme-scale monorepos" \
      org.opencontainers.image.version="${VERSION}" \
      org.opencontainers.image.vendor="CivitForge" \
      org.opencontainers.image.source="https://github.com/WyattAu/CivitForge" \
      org.opencontainers.image.licenses="AGPL-3.0-or-later"

# --- Evergreen Metadata Labels ---
LABEL evergreen.image.tier="critical" \
      evergreen.base.image="wolfi" \
      evergreen.constraint.nonroot="true" \
      evergreen.constraint.wolfi="true" \
      evergreen.health.type="http" \
      evergreen.image.category="source-control"

# --- Security Hardening Labels ---
LABEL evergreen.security.cap-drop="ALL" \
      evergreen.security.no-new-privileges="true" \
      evergreen.security.read-only-rootfs="true" \
      evergreen.security.seccomp="runtime-default"

STOPSIGNAL SIGTERM
