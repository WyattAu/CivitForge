---
title: Quick Start
description: Get CivitForge running in under 5 minutes.
---

## Prerequisites

- Docker 24+ and Docker Compose v2+
- Git
- Curl (for health checks)

## Docker Compose (Recommended)

```bash
git clone https://github.com/WyattAu/CivitForge.git
cd CivitForge
docker compose up -d
```

This starts PostgreSQL 17, Redis 7, and the CivitForge server. Wait ~15 seconds
for migrations to complete, then verify:

```bash
curl http://localhost:9091/healthz
# Expected: OK
```

## Access the UI

Open `http://localhost:9091` in your browser. The WASM UI loads automatically.

## Default Credentials (Development Only)

| Service | Credentials |
|---------|-------------|
| PostgreSQL | `civit` / `civit-dev-secure-pw-2026` (port 5432) |
| Redis | password `civit-redis-dev-2026` (port 6379) |

## API Authentication

Register a user and obtain a JWT:

```bash
curl -X POST http://localhost:9091/api/v1/auth/register \
  -H 'Content-Type: application/json' \
  -d '{"username":"alice","email":"alice@example.com","password":"SecurePass123!"}'
```

Use the returned `token` in the `Authorization: Bearer <token>` header for
subsequent requests.

## Source Build

```bash
# Install Rust 1.88+
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install protobuf compiler
sudo apt-get install -y protobuf-compiler

# Build
cargo build --workspace --release

# Start dependencies
docker compose up -d postgres redis

# Run
DATABASE_URL=postgres://civit:civit-dev-secure-pw-2026@localhost:5432/civit \
REDIS_URL=redis://:civit-redis-dev-2026@localhost:6379 \
JWT_SECRET=dev-secret-key-32bytes-minimum \
./target/release/civit-core
```
