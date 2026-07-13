-- CivitForge Phase 65: Pipeline Environments
-- Migration 065
-- Adds deployment environments and protection rules.

CREATE TABLE IF NOT EXISTS pipeline_environments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    repo_id UUID NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    url TEXT,
    protected BOOLEAN NOT NULL DEFAULT false,
    auto_deploy BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(repo_id, name)
);

CREATE TABLE IF NOT EXISTS deployment_protections (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    environment_id UUID NOT NULL REFERENCES pipeline_environments(id) ON DELETE CASCADE,
    required_approvals INTEGER NOT NULL DEFAULT 1,
    wait_timer INTEGER NOT NULL DEFAULT 0,
    allow_admin_override BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS environment_deployments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    environment_id UUID NOT NULL REFERENCES pipeline_environments(id) ON DELETE CASCADE,
    pipeline_run_id UUID REFERENCES pipeline_runs(id) ON DELETE SET NULL,
    sha VARCHAR(64) NOT NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'pending',
    creator_id UUID REFERENCES users(id) ON DELETE SET NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_pipeline_environments_repo_id ON pipeline_environments(repo_id);
CREATE INDEX IF NOT EXISTS idx_deployment_protections_env_id ON deployment_protections(environment_id);
CREATE INDEX IF NOT EXISTS idx_environment_deployments_env_id ON environment_deployments(environment_id);
CREATE INDEX IF NOT EXISTS idx_environment_deployments_status ON environment_deployments(status);
