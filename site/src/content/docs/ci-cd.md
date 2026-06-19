---
title: CI/CD Pipeline
description: Pipeline YAML specification, runner architecture, caching, secrets, and status badges for CivitForge CI/CD.
---

## Overview

CivitForge includes a built-in CI/CD system with a YAML-based pipeline
specification. Pipelines execute in rootless Podman containers (local mode) or
Kubernetes pods (K8s operator mode). The system consists of three components:

- **civit-pipeline**: YAML parsing and validation
- **civit-ci**: Orchestration, DAG scheduling, cache management
- **civit-runner**: Execution engine, container lifecycle

## Pipeline YAML Specification

Pipeline files are stored at `.civit/pipeline.yaml` in the repository root
(configurable per pipeline via `yaml_path`).

### Minimal example

```yaml
name: build-and-test
on:
  push:
    branches: [main]
steps:
  - name: build
    image: rust:1.88
    commands:
      - cargo build --release
  - name: test
    image: rust:1.88
    commands:
      - cargo test --workspace
```

### Full specification

```yaml
name: pipeline-name
on:
  push:
    branches: [main, "release/*"]
    tags: ["v*"]
    paths: ["src/**", "Cargo.toml"]
    paths_ignore: ["docs/**", "*.md"]
  pull_request:
    branches: [main]
  schedule:
    cron: "0 2 * * 1"  # Weekly on Monday at 02:00
  manual: {}

env:
  RUST_LOG: info
  CARGO_INCREMENTAL: "0"

concurrency:
  group: ${{ github.ref }}
  cancel_in_progress: true

services:
  postgres:
    image: postgres:17-alpine
    env:
      POSTGRES_DB: test
      POSTGRES_USER: test
      POSTGRES_PASSWORD: test
    ports:
      - 5432:5432
    options: --health-cmd pg_isready --health-interval 10s

cache:
  key: cargo-${{ hashFiles('Cargo.lock') }}
  paths:
    - ~/.cargo/registry
    - ~/.cargo/git
    - target

secrets:
  - DATABASE_URL
  - API_KEY

workspace:
  directory: /workspace
  persist: true

steps:
  - name: lint
    image: rust:1.88
    commands:
      - cargo fmt --check
      - cargo clippy --workspace -- -D warnings
    when:
      if: "push.branch == 'main' || pull_request"

  - name: build
    image: rust:1.88
    commands:
      - cargo build --release
    depends_on: [lint]

  - name: test
    image: rust:1.88
    commands:
      - cargo test --workspace
    depends_on: [build]
    services: [postgres]
    env:
      DATABASE_URL: postgres://test:test@localhost:5432/test
    timeout: 600

  - name: deploy
    image: alpine:3.20
    commands:
      - echo "Deploying..."
    depends_on: [test]
    when:
      if: "push.tag && startsWith(push.tag, 'v')"
```

### Trigger configuration

#### Push triggers

```yaml
on:
  push:
    branches: [main, "release/*"]  # Glob patterns
    tags: ["v*", "release-*"]      # Tag patterns
    paths: ["src/**"]              # Only trigger on path changes
    paths_ignore: ["docs/**"]      # Exclude paths
```

Branch and tag patterns support:
- Exact match: `main`
- Prefix wildcard: `release/*`
- Suffix wildcard: `*.md`
- Double-star: `**` (matches any path)

#### Pull request triggers

```yaml
on:
  pull_request:
    branches: [main]
```

#### Schedule triggers

```yaml
on:
  schedule:
    cron: "0 2 * * 1"  # Standard 5-field cron
```

Cron format: `minute hour day-of-month month day-of-week`

#### Manual triggers

```yaml
on:
  manual: {}  # Always triggers on manual dispatch
```

### Step configuration

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `name` | string | -- | Step identifier (required) |
| `image` | string | -- | Container image (required) |
| `commands` | list | -- | Shell commands to execute |
| `depends_on` | list | `[]` | Steps that must complete first |
| `services` | list | `[]` | Service containers to start |
| `env` | map | `{}` | Environment variables |
| `timeout` | int | 3600 | Maximum execution time in seconds |
| `when` | object | -- | Conditional execution |
| `retry` | int | 0 | Number of retries on failure |

### Service containers

Services run alongside the step in the same network namespace:

```yaml
services:
  postgres:
    image: postgres:17-alpine
    env:
      POSTGRES_DB: test
    ports:
      - 5432:5432
    options: --health-cmd pg_isready --health-interval 10s
```

Services are started before the step executes and stopped after completion.
Port mappings use the container's network, not the host.

### Conditional execution

```yaml
when:
  if: "push.branch == 'main' && !push.tag"
```

Supported expressions:
- `push.branch`, `push.tag`, `push.commit`
- `pull_request.number`, `pull_request.base`
- `schedule.cron`
- Logical operators: `&&`, `||`, `!`
- String functions: `startsWith()`, `endsWith()`, `contains()`
- Comparison: `==`, `!=`, `>`, `<`

## Runner Architecture

### Local mode (Podman)

The default runner executes steps in rootless Podman containers:

```
civit-runner
    │
    ├── pipeline.rs          # Step orchestration
    ├── podman.rs            # Podman CLI transport
    │   ├── Unix socket      # /run/podman/podman.sock
    │   └── HTTP API         # podman system service
    └── sync.rs              # Multi-master DAG sync
```

Container lifecycle:
1. Pull image if not cached
2. Create container with volume mounts, env, network
3. Start service containers first
4. Execute step commands in main container
5. Capture stdout/stderr
6. Wait for completion or timeout
7. Collect exit code
8. Stop and remove containers

### Kubernetes mode

The K8s operator uses `kube-rs` to manage pipeline execution:

```
civit-runner
    │
    ├── kube_controller.rs   # Reconciler loop
    │   ├── Leader election   # Lease CRD
    │   ├── Pod creation     # Pipeline step as K8s Job
    │   └── Status sync      # Pod status -> Pipeline status
    └── redis_session.rs     # Token rotation
```

## Caching

Pipeline caching accelerates builds by persisting dependencies and build
artifacts between runs.

### Cache configuration

```yaml
cache:
  key: cargo-${{ hashFiles('Cargo.lock') }}
  paths:
    - ~/.cargo/registry
    - ~/.cargo/git
    - target
```

### Cache behavior

- Cache key is evaluated as an expression.
- `hashFiles()` computes a SHA-256 hash of matching files.
- Cache is restored at step start and saved at step end.
- Cache storage uses the `civit-storage` crate (local filesystem or S3).
- Maximum cache size: 5 GB per repository.
- Cache TTL: 7 days (configurable via `CIVIT_CACHE_TTL_SECS`).

### Cache restoration

Cache is restored in this order:
1. Exact key match
2. Prefix match (most recent)
3. No cache (cold build)

## Secrets

Secrets are injected into pipeline steps as environment variables.

### Configuration

```yaml
secrets:
  - DATABASE_URL
  - API_KEY
```

### Secret storage

Secrets are stored encrypted in the database (`repo_secrets` table) and
injected at runtime. They are never written to disk or exposed in logs.

### Secret access

```yaml
steps:
  - name: deploy
    image: alpine:3.20
    commands:
      - curl -H "Authorization: Bearer $API_KEY" https://api.example.com
    secrets: [API_KEY]
```

### Secret rotation

Secrets can be rotated via the API without restarting pipelines:

```bash
curl -X PUT http://localhost:9091/api/v1/repos/{repo_id}/secrets \
  -H 'Authorization: Bearer <token>' \
  -d '{"name":"API_KEY","value":"new-value"}'
```

## Status Badges

CivitForge generates SVG status badges for pipelines:

### Badge URL

```
GET /api/v1/repos/{owner}/{repo}/pipelines/{pipeline_id}/badge.svg
```

### Badge states

| Status | Color | Label |
|--------|-------|-------|
| `pending` | Gray | pending |
| `running` | Blue | running |
| `success` | Green | passing |
| `failure` | Red | failing |
| `cancelled` | Orange | cancelled |

### Markdown usage

```markdown
![Pipeline](https://forge.example.com/api/v1/repos/{owner}/{repo}/pipelines/{id}/badge.svg)
```

## Concurrency Control

Pipelines support concurrency groups to prevent redundant runs:

```yaml
concurrency:
  group: ${{ github.ref }}
  cancel_in_progress: true
```

When `cancel_in_progress` is true, any running pipeline in the same group is
cancelled when a new pipeline starts. This saves runner resources for
superseded commits.

## Timeout

Each step has a configurable timeout (default 3600 seconds). The pipeline
itself has a maximum timeout of 86400 seconds (24 hours).

```yaml
steps:
  - name: long-build
    image: rust:1.88
    commands:
      - cargo build --release
    timeout: 7200  # 2 hours
```

## Error handling

- Step failure: dependent steps are skipped, pipeline status set to `failure`
- Service failure: step fails immediately
- Timeout: step is killed, exit code 124
- Runner crash: pipeline remains in `running` state, recovered on runner restart
