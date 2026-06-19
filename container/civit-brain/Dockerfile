# =============================================================================
# CivitForge Brain - Search indexing and code intelligence daemon
# EvergreenImageRegistry-compliant container image
# =============================================================================
# Tier: standard
# Base: wolfi (cgr.dev/chainguard/wolfi-base)
# Multi-arch: linux/amd64, linux/arm64
# Purpose: Background daemon for code search indexing, cross-referencing,
#          and code intelligence features.
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
    && rm -rf /var/lib/apt/lists/*

# Map Docker TARGETARCH to Rust target triple
RUN case "$TARGETARCH" in \
        amd64) RUST_TARGET="x86_64-unknown-linux-gnu" ;; \
        arm64) RUST_TARGET="aarch64-unknown-linux-gnu" ;; \
        *) echo "Unsupported arch: $TARGETARCH"; exit 1 ;; \
    esac && \
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y \
        --default-toolchain ${RUST_VERSION} \
        --profile minimal \
        --target ${RUST_TARGET}
ENV PATH="/root/.cargo/bin:${PATH}"

# Create placeholder to satisfy BuildKit COPY eval (Evergreen P3)
RUN mkdir -p /out/usr/local/bin
RUN touch /out/usr/local/bin/civitforge

# Copy source (entire workspace — all inter-crate deps must be present)
WORKDIR /src
COPY Cargo.toml Cargo.lock rust-toolchain.toml ./
COPY crates/ crates/

# Build only civit-brain
RUN cargo build --release --locked -p civit-brain

# Strip debug symbols and copy to /out
RUN strip /src/target/release/civit-brain 2>/dev/null || true && \
    cp /src/target/release/civit-brain /out/usr/local/bin/civit-brain && \
    test -f /out/usr/local/bin/civit-brain

# ---------------------------------------------------------------------------
# Stage 2: Final image (wolfi-base per Evergreen ADR-007)
# ---------------------------------------------------------------------------
FROM cgr.dev/chainguard/wolfi-base:latest

ARG APP_UID=65532
ARG APP_GID=65532
ARG VERSION=2.2.0

# Install runtime dependencies
RUN apk add --no-cache \
    ca-certificates \
    git \
    su-exec \
    wget

# Use existing nonroot user (wolfi-base ships 65532:65532)
RUN mkdir -p /data /var/log/civit && \
    chown -R 65532:65532 /data /var/log/civit

# Copy civit-brain binary from builder
COPY --from=builder /out/usr/local/bin/civit-brain /usr/local/bin/civit-brain

WORKDIR /data
ENV CIVIT_STORAGE_PATH=/var/lib/civit/repos
ENV RUST_LOG=civit_brain=info

# Port: 8082=brain HTTP API
EXPOSE 8082

# Health check (Evergreen: wolfi has wget, NOT curl)
HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
    CMD wget -qO- http://localhost:8082/healthz || exit 1

USER 65532:65532

ENTRYPOINT ["civit-brain"]
CMD []

# --- OCI Standard Labels ---
LABEL org.opencontainers.image.title="civit-brain" \
      org.opencontainers.image.description="CivitForge Brain - search indexing and code intelligence daemon" \
      org.opencontainers.image.version="${VERSION}" \
      org.opencontainers.image.vendor="CivitForge" \
      org.opencontainers.image.source="https://github.com/WyattAu/CivitForge" \
      org.opencontainers.image.licenses="AGPL-3.0-or-later"

# --- Evergreen Metadata Labels ---
LABEL evergreen.image.tier="standard" \
      evergreen.base.image="wolfi" \
      evergreen.constraint.nonroot="true" \
      evergreen.constraint.wolfi="true" \
      evergreen.health.type="http" \
      evergreen.image.category="search" \
      evergreen.image.status="functional"

# --- Security Hardening Labels ---
LABEL evergreen.security.cap-drop="ALL" \
      evergreen.security.no-new-privileges="true" \
      evergreen.security.read-only-rootfs="true" \
      evergreen.security.seccomp="runtime-default"

STOPSIGNAL SIGTERM
