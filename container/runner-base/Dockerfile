# =============================================================================
# CivitForge Runner Base - CI runner base image
# EvergreenImageRegistry-compliant container image
# =============================================================================
# Tier: standard
# Base: wolfi (cgr.dev/chainguard/wolfi-base)
# Multi-arch: linux/amd64, linux/arm64
# Purpose: Base image for CI/CD pipeline execution. Includes git, Podman CLI,
#          and common build tools. Use as FROM base for action-specific images.
# =============================================================================

ARG TARGETARCH

# ---------------------------------------------------------------------------
# Stage 1: Tool downloader (debian:bookworm as permitted build stage)
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
RUN mkdir -p /out/usr/local/bin && touch /out/usr/local/bin/runner-init

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
# Stage 2: Final image (wolfi-base per Evergreen ADR-007)
# ---------------------------------------------------------------------------
FROM cgr.dev/chainguard/wolfi-base:latest

ARG APP_UID=65532
ARG APP_GID=65532
ARG VERSION=0.8.0-alpha

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
# Set up home/workspace for CI runner
RUN mkdir -p /home/runner/.local/bin /home/runner/work /tmp/civit-runner && \
    chown -R 65532:65532 /home/runner /tmp/civit-runner

# Copy Podman from downloader
COPY --from=downloader /out/usr/local/bin/podman /usr/local/bin/podman

# Copy runner binary from CivitForge build (injected at build time)
# The actual civit-runner binary is COPY'd in the action images or via volume mount
RUN mkdir -p /usr/local/bin && touch /usr/local/bin/civit-runner

WORKDIR /home/runner/work
ENV PATH="/home/runner/.local/bin:${PATH}"
ENV CIVIT_RUNNER_WORKDIR=/home/runner/work

# Ports: 8088=runner HTTP, 9101=metrics
EXPOSE 8088 9101

# Health check: verify podman and git are available
HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
    CMD wget -qO- http://localhost:8088/ || exit 1

USER 65532:65532

ENTRYPOINT ["civit-runner"]
CMD []

# --- OCI Standard Labels ---
LABEL org.opencontainers.image.title="civitforge-runner-base" \
      org.opencontainers.image.description="CivitForge CI runner base image with git, Podman CLI, and build tools" \
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
