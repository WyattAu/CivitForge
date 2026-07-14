CREATE TABLE IF NOT EXISTS environment_deployment_history_v2 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    environment_id UUID NOT NULL REFERENCES pipeline_environments(id) ON DELETE CASCADE,
    version TEXT NOT NULL,
    sha TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'deployed',
    deployed_by UUID NOT NULL REFERENCES users(id),
    rollback_of UUID REFERENCES environment_deployment_history_v2(id),
    metadata JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_deployment_history_v2_env_id ON environment_deployment_history_v2(environment_id);
CREATE INDEX IF NOT EXISTS idx_deployment_history_v2_deployed_by ON environment_deployment_history_v2(deployed_by);
CREATE INDEX IF NOT EXISTS idx_deployment_history_v2_status ON environment_deployment_history_v2(status);
CREATE INDEX IF NOT EXISTS idx_deployment_history_v2_created_at ON environment_deployment_history_v2(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_deployment_history_v2_rollback_of ON environment_deployment_history_v2(rollback_of);
