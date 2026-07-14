-- CivitForge Phase 134: Pipeline Environments V2
-- Migration 134
-- Adds advanced environment management with deployment branch policies.

CREATE TABLE IF NOT EXISTS pipeline_environments_v2 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    repo_id UUID NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    url TEXT,
    protected BOOLEAN NOT NULL DEFAULT false,
    auto_deploy BOOLEAN NOT NULL DEFAULT true,
    deployment_branch_policy JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(repo_id, name)
);

CREATE INDEX IF NOT EXISTS idx_pipeline_environments_v2_repo_id ON pipeline_environments_v2(repo_id);
CREATE INDEX IF NOT EXISTS idx_pipeline_environments_v2_protected ON pipeline_environments_v2(protected);
