# =============================================================================
# CivitForge Runner - CI/CD pipeline daemon
# EvergreenImageRegistry-compliant container image
# =============================================================================
# Tier: standard
# Base: wolfi (cgr.dev/chainguard/wolfi-base)
# Multi-arch: linux/amd64, linux/arm64
# Purpose: Long-lived daemon that polls the CivitForge server for CI
#          pipeline jobs and executes them. Each step runs in an
#          ephemeral container spawned via Podman. The toolchain for
#          each step comes from the user's pipeline YAML `image:` field
#          (e.g. rust:1.88, node:20-alpine), not from this image.
#
#          Analogue: forgejo-runner, gitlab-runner.
#          See: https://docs.forgejo.org/latest/admin/actions/
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
    && rm -rf /var/lib/apt/lists/*

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y \
        --default-toolchain ${RUST_VERSION} \
        --profile minimal \
        --target x86_64-unknown-linux-gnu
ENV PATH="/root/.cargo/bin:${PATH}"

WORKDIR /src
COPY Cargo.toml Cargo.lock rust-toolchain.toml ./
COPY crates/ crates/

RUN cargo build --release --locked -p civit-runner
RUN strip /src/target/release/civit-runner 2>/dev/null || true
RUN cp /src/target/release/civit-runner /civit-runner

# ---------------------------------------------------------------------------
# Stage 2: Tool downloader (debian:bookworm as permitted build stage)
# ---------------------------------------------------------------------------
FROM debian:bookworm AS downloader

ARG TARGETARCH

RUN apt-get update && \
    apt-get install -y --no-install-recommends \
        curl \
        ca-certificates \
        binutils \
    && rm -rf /var/lib/apt/lists/*

# Create placeholder for BuildKit COPY eval (Evergreen P3)
RUN mkdir -p /out/usr/local/bin && touch /out/usr/local/bin/civit-runner

# Download Podman CLI binary (remote static, amd64/arm64)
ARG TARGETARCH
RUN PODMAN_ARCH=${TARGETARCH:-amd64} && \
    curl --retry 3 --retry-delay 5 -fsSL \
        "https://github.com/containers/podman/releases/download/v5.8.2/podman-remote-static-linux_${PODMAN_ARCH}.tar.gz" \
    -o /tmp/podman.tar.gz && \
    tar -xzf /tmp/podman.tar.gz -C /tmp && \
    find /tmp/bin -name "podman-remote*" -exec cp {} /out/usr/local/bin/podman \; && \
    strip /out/usr/local/bin/podman && \
    chmod +x /out/usr/local/bin/podman && \
    test -s /out/usr/local/bin/podman && \
    rm -rf /tmp/podman*

# ---------------------------------------------------------------------------
# Stage 3: Final image (wolfi-base per Evergreen ADR-007)
# ---------------------------------------------------------------------------
FROM cgr.dev/chainguard/wolfi-base:latest

ARG APP_UID=65532
ARG APP_GID=65532
ARG VERSION=1.1.0

# Install runtime tools via wolfi apk
RUN apk add --no-cache \
    ca-certificates \
    git \
    openssh-client \
    make \
    wget \
    su-exec \
    shadow \
    bash \
    coreutils

# Use existing nonroot user (wolfi-base ships 65532:65532)
# Set up home/workspace for the CI daemon
RUN mkdir -p /home/runner/.local/bin /home/runner/work /tmp/civit-runner && \
    chown -R 65532:65532 /home/runner /tmp/civit-runner

# Copy Podman from downloader
COPY --from=downloader /out/usr/local/bin/podman /usr/local/bin/podman

# Copy civit-runner binary from builder
COPY --from=builder /civit-runner /usr/local/bin/civit-runner

WORKDIR /home/runner/work
ENV PATH="/home/runner/.local/bin:${PATH}"
ENV CIVIT_RUNNER_WORKDIR=/home/runner/work

# Ports: 8088=runner HTTP, 9101=metrics
EXPOSE 8088 9101

# Health check: verify runner daemon is responsive
HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
    CMD wget -qO- http://localhost:8088/ || exit 1

USER 65532:65532

ENTRYPOINT ["civit-runner"]
CMD []

# --- OCI Standard Labels ---
LABEL org.opencontainers.image.title="civitforge-runner" \
      org.opencontainers.image.description="CivitForge CI/CD pipeline daemon. Polls for jobs and executes step containers via Podman. Toolchains come from user-specified pipeline images, not this image." \
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
      evergreen.image.category="ci" \
      evergreen.image.status="functional"

# --- Security Hardening Labels ---
LABEL evergreen.security.cap-drop="ALL" \
      evergreen.security.no-new-privileges="true" \
      evergreen.security.read-only-rootfs="true" \
      evergreen.security.seccomp="runtime-default"

STOPSIGNAL SIGTERM
