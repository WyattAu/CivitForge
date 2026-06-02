# =============================================================================
# CivitForge Runner Action - Rust build action image
# EvergreenImageRegistry-compliant container image
# =============================================================================
# Tier: community
# Base: civitforge-runner-base (FROM container/runner-base)
# Multi-arch: linux/amd64 only (Go/rustup multiarch fails under QEMU)
# Purpose: Per-action runner for Rust CI pipelines. Extends runner-base with
#          Rust toolchain, cargo, and common build dependencies.
# =============================================================================

ARG TARGETARCH

# ---------------------------------------------------------------------------
# Stage 1: Rust toolchain downloader (debian:bookworm as permitted build stage)
# ---------------------------------------------------------------------------
FROM debian:bookworm AS rust-downloader

ARG TARGETARCH

RUN apt-get update && \
    apt-get install -y --no-install-recommends \
        curl \
        ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create placeholder for BuildKit COPY eval (Evergreen P3)
RUN mkdir -p /out && touch /out/rustup-init

# Download rustup installer
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
        -o /out/rustup-init && \
    chmod +x /out/rustup-init && \
    test -s /out/rustup-init

# ---------------------------------------------------------------------------
# Stage 2: Final image (extends civitforge-runner-base)
# ---------------------------------------------------------------------------
# NOTE: In production, this FROM would reference the published base image:
#   FROM ghcr.io/wyattau/evergreenimageregistry/civitforge-runner-base:latest
# For local/dev builds, we reference the local build context:
FROM cgr.dev/chainguard/wolfi-base:latest

ARG APP_UID=65532
ARG APP_GID=65532
ARG VERSION=0.8.0-alpha

# Install all base tools (duplicated from runner-base for standalone build)
# In production, this entire stage is replaced by FROM runner-base
RUN apk add --no-cache \
    ca-certificates \
    git \
    openssh-client \
    make \
    wget \
    su-exec \
    shadow \
    bash \
    coreutils \
    build-base \
    openssl-dev

# Use existing nonroot user (wolfi-base ships 65532:65532)
# Set up home/workspace for CI runner with Rust toolchain dirs
RUN mkdir -p /home/runner/.local/bin /home/runner/work /home/runner/.cargo \
    && chown -R 65532:65532 /home/runner

# Install Rust toolchain as nonroot (wolfi ships user 65532:65532)
COPY --from=rust-downloader /out/rustup-init /tmp/rustup-init
RUN su-exec 65532:65532 /bin/sh -c ' \
        HOME=/home/runner \
        RUSTUP_HOME=/home/runner/.rustup \
        CARGO_HOME=/home/runner/.cargo \
        /tmp/rustup-init -y \
            --default-toolchain 1.88 \
            --profile minimal \
            --no-modify-path' && \
    rm -f /tmp/rustup-init

ENV PATH="/home/runner/.cargo/bin:/home/runner/.local/bin:${PATH}"
ENV RUSTUP_HOME=/home/runner/.rustup
ENV CARGO_HOME=/home/runner/.cargo
ENV CIVIT_RUNNER_WORKDIR=/home/runner/work

WORKDIR /home/runner/work

# Ports: 8088=runner HTTP, 9101=metrics
EXPOSE 8088 9101

# Health check
HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
    CMD wget -qO- http://localhost:8088/ || exit 1

USER 65532:65532

ENTRYPOINT ["civit-runner"]
CMD []

# --- OCI Standard Labels ---
LABEL org.opencontainers.image.title="civitforge-runner-action" \
      org.opencontainers.image.description="CivitForge CI runner action image for Rust pipelines" \
      org.opencontainers.image.version="${VERSION}" \
      org.opencontainers.image.vendor="CivitForge" \
      org.opencontainers.image.source="https://github.com/WyattAu/CivitForge" \
      org.opencontainers.image.licenses="AGPL-3.0-or-later"

# --- Evergreen Metadata Labels ---
LABEL evergreen.image.tier="community" \
      evergreen.base.image="wolfi" \
      evergreen.constraint.nonroot="true" \
      evergreen.constraint.wolfi="true" \
      evergreen.health.type="http" \
      evergreen.image.category="ci" \
      evergreen.image.status="functional"

# --- Security Hardening Labels ---
LABEL evergreen.security.cap-drop="ALL" \
      evergreen.security.no-new-privileges="true" \
      evergreen.security.read-only-rootfs="true" \
      evergreen.security.seccomp="runtime-default"

STOPSIGNAL SIGTERM
