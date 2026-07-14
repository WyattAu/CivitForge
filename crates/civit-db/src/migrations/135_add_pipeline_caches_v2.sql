-- CivitForge Phase 135: Pipeline Caches V2
-- Migration 135
-- Adds advanced caching with hit tracking and statistics.

CREATE TABLE IF NOT EXISTS pipeline_caches_v2 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    repo_id UUID NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    key TEXT NOT NULL,
    path TEXT NOT NULL,
    size_bytes BIGINT NOT NULL DEFAULT 0,
    hit_count INTEGER NOT NULL DEFAULT 0,
    last_hit_at TIMESTAMPTZ,
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(repo_id, key)
);

CREATE INDEX IF NOT EXISTS idx_pipeline_caches_v2_repo_id ON pipeline_caches_v2(repo_id);
CREATE INDEX IF NOT EXISTS idx_pipeline_caches_v2_key ON pipeline_caches_v2(repo_id, key);
CREATE INDEX IF NOT EXISTS idx_pipeline_caches_v2_expires ON pipeline_caches_v2(expires_at);
CREATE INDEX IF NOT EXISTS idx_pipeline_caches_v2_hit_count ON pipeline_caches_v2(hit_count DESC);
