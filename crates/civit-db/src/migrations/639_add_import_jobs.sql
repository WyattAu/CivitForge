-- Import job tracking (Migration 639)
-- Tracks forge migration jobs: per-repo git clones + metadata sync, with
-- post-clone verification. Foundation for the Migration API v1 (ADR-0006).

CREATE TABLE IF NOT EXISTS import_jobs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    forge TEXT NOT NULL CHECK (forge IN ('github', 'gitlab', 'forgejo', 'gitea', 'url')),
    source_url TEXT NOT NULL,
    dest_owner TEXT NOT NULL,
    dest_name TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'queued'
        CHECK (status IN ('queued', 'cloning', 'verifying', 'completed', 'failed')),
    error TEXT,
    -- Post-clone verification artifacts
    commit_count BIGINT,
    verified_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_import_jobs_user_created ON import_jobs(user_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_import_jobs_status ON import_jobs(status) WHERE status IN ('queued', 'cloning', 'verifying');
