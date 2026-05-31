FROM rust:1.88-slim AS builder

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY civit-core ./civit-core
COPY civit-brain ./civit-brain
COPY civit-crypto ./civit-crypto
COPY civit-runner ./civit-runner
COPY civit-vfs ./civit-vfs

RUN cargo build --release

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/civit-core /usr/local/bin/civit-core

EXPOSE 8080

CMD ["civit-core"]
