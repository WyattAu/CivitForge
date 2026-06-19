---
title: Database Layer
description: Migrations, schema, connection pooling, and session management for CivitForge.
---

## Overview

CivitForge uses PostgreSQL 17 as its primary datastore, accessed via `sqlx`
0.8 with compile-time query checking. The `civit-db` crate provides the
database abstraction layer, including migrations, models, connection pooling,
and session management.

## Migrations

Migrations are stored in `crates/civit-db/src/migrations/` as numbered SQL
files. Rollback scripts are in the `down/` subdirectory.

### Migration numbering

Migrations follow a three-digit zero-padded convention: `001_initial_schema.sql`
through `058_webauthn.sql`. Each migration has a corresponding rollback in
`down/001_initial_schema_down.sql`.

### Running migrations

Migrations run automatically at server startup via `sqlx::migrate!()`. The
`schema_migrations` table tracks applied versions:

```sql
CREATE TABLE IF NOT EXISTS schema_migrations (
    version BIGINT PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    applied_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

To run migrations manually:

```bash
DATABASE_URL=postgres://civit:password@localhost:5432/civit \
  sqlx migrate run --source crates/civit-db/src/migrations
```

To rollback the last migration:

```bash
DATABASE_URL=postgres://civit:password@localhost:5432/civit \
  sqlx migrate revert --source crates/civit-db/src/migrations
```

### Migration list

| Migration | Description |
|-----------|-------------|
| 001 | Initial schema: users, organizations, repositories, issues, PRs, pipelines, tokens, audit events |
| 003 | SSH keys, branches, pipeline steps, events |
| 005 | Auth identity tables |
| 007 | Permissions tables |
| 009 | CI/CD pipeline tables |
| 011 | OCI registry tables |
| 013 | Issue tracking tables |
| 015 | Wiki tables |
| 017 | Code search tables |
| 019 | Wiki content snapshots |
| 021 | Full-text search indexes |
| 025 | Activity federation tables |
| 029 | Password hash column |
| 031 | Pull request tracking |
| 033 | Star/watch counts |
| 035 | Login attempt tracking |
| 036 | Email verification |
| 037 | Mentions and cross-references |
| 038 | Repository secrets and pipeline caches |
| 039 | Secret scanning and SLSA tables |
| 040 | Boards (Kanban) |
| 041 | Webhooks |
| 042 | Webhook deliveries |
| 043 | Pipeline schedules |
| 044 | Repository collaborators |
| 045 | Deploy keys |
| 046 | Notifications |
| 047 | User banned status |
| 048 | Repository archived status and topics |
| 050 | Issue and PR feature enhancements |
| 051 | Environments and deployments |
| 052 | User profiles |
| 053 | OpenID Connect |
| 054 | Merge queue |
| 055 | Site settings |
| 056 | OIDC admin configuration |
| 057 | CODEOWNERS and reviews |
| 058 | WebAuthn registration |

## Schema

### Core tables

#### users

```sql
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    username VARCHAR(255) NOT NULL UNIQUE,
    email VARCHAR(255) NOT NULL UNIQUE,
    display_name VARCHAR(255) NOT NULL DEFAULT '',
    bio TEXT NOT NULL DEFAULT '',
    role VARCHAR(50) NOT NULL DEFAULT 'guest',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

Roles: `guest`, `user`, `maintainer`, `admin`, `superadmin`.

#### organizations

```sql
CREATE TABLE organizations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL UNIQUE,
    display_name VARCHAR(255) NOT NULL DEFAULT '',
    description TEXT NOT NULL DEFAULT '',
    visibility VARCHAR(50) NOT NULL DEFAULT 'private',
    owner_id UUID NOT NULL REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

#### repositories

```sql
CREATE TABLE repositories (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    owner_id UUID NOT NULL REFERENCES users(id),
    org_id UUID REFERENCES organizations(id),
    visibility VARCHAR(50) NOT NULL DEFAULT 'private',
    default_branch VARCHAR(255) NOT NULL DEFAULT 'main',
    is_fork BOOLEAN NOT NULL DEFAULT false,
    parent_repo_id UUID REFERENCES repositories(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(owner_id, name)
);
```

#### issues

```sql
CREATE TABLE issues (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    repo_id UUID NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    number SERIAL NOT NULL,
    title VARCHAR(1024) NOT NULL,
    body TEXT NOT NULL DEFAULT '',
    status VARCHAR(50) NOT NULL DEFAULT 'open',
    author_id UUID NOT NULL REFERENCES users(id),
    assignee_id UUID REFERENCES users(id),
    labels JSONB NOT NULL DEFAULT '[]',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    closed_at TIMESTAMPTZ,
    UNIQUE(repo_id, number)
);
```

#### pull_requests

```sql
CREATE TABLE pull_requests (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    repo_id UUID NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    number SERIAL NOT NULL,
    title VARCHAR(1024) NOT NULL,
    body TEXT NOT NULL DEFAULT '',
    status VARCHAR(50) NOT NULL DEFAULT 'open',
    author_id UUID NOT NULL REFERENCES users(id),
    source_branch VARCHAR(255) NOT NULL,
    target_branch VARCHAR(255) NOT NULL,
    merge_commit_id VARCHAR(64),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    merged_at TIMESTAMPTZ,
    closed_at TIMESTAMPTZ,
    UNIQUE(repo_id, number)
);
```

#### pipelines

```sql
CREATE TABLE pipelines (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    repo_id UUID NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    commit_sha VARCHAR(64) NOT NULL,
    status VARCHAR(50) NOT NULL DEFAULT 'pending',
    trigger VARCHAR(50) NOT NULL DEFAULT 'push',
    yaml_path VARCHAR(512) NOT NULL DEFAULT '.civit/pipeline.yaml',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    started_at TIMESTAMPTZ,
    finished_at TIMESTAMPTZ
);
```

Pipeline statuses: `pending`, `running`, `success`, `failure`, `cancelled`,
`skipped`.

#### access_tokens

```sql
CREATE TABLE access_tokens (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    token_hash VARCHAR(128) NOT NULL UNIQUE,
    scopes JSONB NOT NULL DEFAULT '["read"]',
    expires_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_used_at TIMESTAMPTZ
);
```

Tokens are stored as bcrypt hashes. The plaintext token is only returned at
creation time.

#### audit_events

```sql
CREATE TABLE audit_events (
    id BIGSERIAL PRIMARY KEY,
    actor_id UUID NOT NULL,
    action VARCHAR(100) NOT NULL,
    resource_type VARCHAR(100) NOT NULL,
    resource_id UUID,
    ip_address VARCHAR(45),
    user_agent TEXT,
    outcome VARCHAR(20) NOT NULL DEFAULT 'success',
    details JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

### Indexes

```sql
CREATE INDEX idx_repositories_owner ON repositories(owner_id);
CREATE INDEX idx_repositories_org ON repositories(org_id);
CREATE INDEX idx_issues_repo ON issues(repo_id);
CREATE INDEX idx_issues_author ON issues(author_id);
CREATE INDEX idx_issues_status ON issues(status);
CREATE INDEX idx_prs_repo ON pull_requests(repo_id);
CREATE INDEX idx_pipelines_repo ON pipelines(repo_id);
CREATE INDEX idx_pipelines_status ON pipelines(status);
CREATE INDEX idx_tokens_user ON access_tokens(user_id);
CREATE INDEX idx_tokens_hash ON access_tokens(token_hash);
CREATE INDEX idx_audit_actor ON audit_events(actor_id);
CREATE INDEX idx_audit_created ON audit_events(created_at);
CREATE INDEX idx_audit_resource ON audit_events(resource_type, resource_id);
```

## Connection Pooling

CivitForge uses `sqlx::PgPool` with configurable pool sizes. The pool is
initialized at server startup and shared across all request handlers.

### Configuration

| Variable | Type | Default | Description |
|----------|------|---------|-------------|
| `DATABASE_URL` | string | -- | PostgreSQL connection string |
| `DB_POOL_SIZE` | u32 | 20 | Maximum connections in the pool |
| `DB_POOL_TIMEOUT` | u32 | 30 | Seconds to wait for a connection |

### Pool behavior

- Connections are lazily established on first use.
- Idle connections are closed after the timeout.
- The pool uses `sqlx::postgres::PgPoolOptions` under the hood.
- Health checks run via `pg_isready` in Docker healthcheck.

### Connection string format

```
postgres://USER:PASSWORD@HOST:PORT/DATABASE?options
```

Common options:
- `sslmode=require` -- enforce TLS
- `connect_timeout=10` -- connection timeout in seconds
- `pool_max_size=20` -- max pool connections

## Session Management

Sessions are managed via JWT tokens stored in Redis. The `civit-db` crate
provides session CRUD operations.

### Session flow

1. User authenticates (local, LDAP, OIDC).
2. Server generates a JWT with user claims.
3. JWT is returned to the client in the response body.
4. Client includes JWT in `Authorization: Bearer <token>` header.
5. Server validates JWT signature and checks session in Redis.

### Session storage

Sessions are stored in Redis with the following structure:

```
session:<user_id>:<token_hash> = <session_json>
TTL = JWT_EXPIRY_HOURS * 3600
```

### Token rotation

Personal access tokens (PATs) support rotation. When a PAT is used, the
`last_used_at` timestamp is updated. Old tokens can be revoked via the API.

## Redis Usage

Redis serves multiple purposes beyond session storage:

| Use Case | Key Pattern | TTL |
|----------|-------------|-----|
| Session cache | `session:<user_id>:<token_hash>` | JWT expiry |
| Rate limiting | `ratelimit:<ip>` | Window duration |
| Pub/Sub events | `events:<channel>` | None |
| Pipeline locks | `pipeline:<pipeline_id>:lock` | 300s |
| Federation inbox | `federation:inbox:<instance_id>` | 3600s |

## Backup and Recovery

### Database backup

```bash
pg_dump -U civit -d civit -Fc > backup_$(date +%Y%m%d_%H%M%S).dump
```

### Restore

```bash
pg_restore -U civit -d civit -c backup_20260619_120000.dump
```

### Redis backup

```bash
redis-cli -a password BGSAVE
```

The RDB file is persisted to `/data/dump.rdb` inside the Redis container.
