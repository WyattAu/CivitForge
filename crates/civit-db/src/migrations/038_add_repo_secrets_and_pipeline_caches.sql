-- Migration 038: Add repo_secrets and pipeline_caches tables.
-- Repo secrets: AES-256-GCM encrypted CI/CD secrets per repository.
-- Pipeline caches: cached build artifacts per repository.

-- -----------------------------------------------------------------------
-- Repository secrets (encrypted at rest)
-- -----------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS repo_secrets (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    repo_id     UUID NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    name        VARCHAR(255) NOT NULL,
    value_enc   BYTEA NOT NULL,      -- AES-256-GCM encrypted value
    nonce       BYTEA NOT NULL,      -- GCM nonce (12 bytes)
    created_by  UUID REFERENCES users(id),
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    UNIQUE(repo_id, name)
);

CREATE INDEX idx_repo_secrets_repo ON repo_secrets (repo_id);

-- -----------------------------------------------------------------------
-- Pipeline caches (build artifact caching)
-- -----------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS pipeline_caches (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    repo_id     UUID NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    key         VARCHAR(512) NOT NULL,
    path        TEXT NOT NULL DEFAULT '',
    size_bytes  BIGINT NOT NULL DEFAULT 0,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at  TIMESTAMPTZ,

    UNIQUE(repo_id, key)
);

CREATE INDEX idx_pipeline_caches_repo ON pipeline_caches (repo_id);
CREATE INDEX idx_pipeline_caches_key ON pipeline_caches (repo_id, key);
CREATE INDEX idx_pipeline_caches_expires ON pipeline_caches (expires_at);
