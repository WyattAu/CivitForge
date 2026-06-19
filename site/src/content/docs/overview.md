---
title: CivitForge
description: Federated, Rust-native software forge for large-scale monorepos.
---

## CivitForge

Federated, Rust-native software forge for large-scale monorepos. Provides Git
hosting, CI/CD pipeline execution with rootless Podman, OCI container registry,
issue tracking with Kanban boards, wiki, code search, and an air-gapped AI
subsystem.

## Key Features

**Rust-Native**

13-workspace Cargo crate, edition 2024, `#![forbid(unsafe_code)]` enforced.
Zero C FFI dependencies in the default build.

**Federated**

ForgeFed ActivityPub protocol support. Cross-instance issue tracking, PR
review, and code search.

**Air-Gapped AI**

Local LLM inference (vLLM/Ollama), AST parsing (19 languages), RAG pipeline,
vector DB. No external API dependencies.

**Rootless CI**

Pipeline execution in rootless Podman sandboxes. Kubernetes operator
integration via kube-rs.

## Quick Start

```bash
docker compose up -d
curl http://localhost:9091/healthz
```

Services exposed on host ports: 9091 (API), 2222 (Git SSH), 9090 (VFS gRPC).

## Architecture

| Crate | Role |
|-------|------|
| `civit-core` | Axum HTTP server, authentication, gitoxide Git engine, SSH daemon |
| `civit-db` | Database layer: migrations, models, connection pool |
| `civit-git` | Git operations: archive, blame, diff, commit graph |
| `civit-auth` | Authentication: JWT, LDAP, personal access tokens, SSH keys |
| `civit-ci` | CI/CD: pipeline execution, DAG scheduling, secrets |
| `civit-runner` | CI/CD pipeline execution, Kubernetes operator, rootless Podman |
| `civit-brain` | AI agent workflows, AST parsing, RAG pipeline, vector DB |
| `civit-vfs` | gRPC filesystem server (tonic/prost) |
| `civit-crypto` | CEL evaluator, HMAC/SHA, OIDC, SAML, WebAuthn, SLSA |
| `civit-shard` | Database sharding with consistent hashing |
| `civit-pipeline` | YAML pipeline spec parsing and validation |
| `civit-shared` | Shared API types for backend-frontend type sharing |

## Test Coverage

- 3,600+ tests passing across the workspace
- Zero clippy warnings (`-D warnings` enforced in CI)
- Playwright GUI traversal (23 routes, 92/92 design checks pass)
- Pre-commit hook: fmt, clippy, test, emoji scan
