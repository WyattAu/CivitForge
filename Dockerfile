# CivitForge - Docker Desktop test image
# Simplified build for local development and testing

FROM debian:bookworm-slim AS builder

ARG TARGETARCH=amd64
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

WORKDIR /src
COPY Cargo.toml Cargo.lock rust-toolchain.toml ./
COPY crates/ crates/

# Build release binary
RUN cargo build --release --locked -p civit-core

# Final stage
FROM debian:bookworm-slim

RUN apt-get update && \
    apt-get install -y --no-install-recommends \
        ca-certificates \
        git \
        wget \
    && rm -rf /var/lib/apt/lists/*

RUN mkdir -p /data /var/lib/civit/repos && \
    chown -R 65532:65532 /data /var/lib/civit

COPY --from=builder /src/target/release/civit-core /usr/local/bin/civit-core

EXPOSE 8080 2222

HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
    CMD wget -qO- http://localhost:8080/healthz || exit 1

USER 65532:65532
WORKDIR /data

ENTRYPOINT ["/usr/local/bin/civit-core"]
