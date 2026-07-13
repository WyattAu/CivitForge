-- Migration 078: Enhance Deployment Protections
-- Adds allowed branches and deployment locks.

ALTER TABLE deployment_protections ADD COLUMN IF NOT EXISTS allowed_branches TEXT[] NOT NULL DEFAULT '{}';

CREATE TABLE IF NOT EXISTS deployment_locks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    environment_id UUID NOT NULL REFERENCES pipeline_environments(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id),
    reason TEXT NOT NULL DEFAULT '',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_deployment_locks_env_id ON deployment_locks(environment_id);
CREATE INDEX IF NOT EXISTS idx_deployment_locks_user_id ON deployment_locks(user_id);
