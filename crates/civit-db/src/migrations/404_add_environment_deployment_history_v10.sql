CREATE TABLE IF NOT EXISTS environment_deployment_history_v10 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    environment_id UUID NOT NULL REFERENCES pipeline_environments(id),
    version TEXT NOT NULL,
    sha TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'deployed',
    deployed_by UUID NOT NULL REFERENCES users(id),
    rollback_of UUID REFERENCES environment_deployment_history_v10(id),
    metadata JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_environment_deployment_history_v10_env_id ON environment_deployment_history_v10(environment_id);
CREATE INDEX IF NOT EXISTS idx_environment_deployment_history_v10_status ON environment_deployment_history_v10(status);
CREATE INDEX IF NOT EXISTS idx_environment_deployment_history_v10_created_at ON environment_deployment_history_v10(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_environment_deployment_history_v10_deployed_by ON environment_deployment_history_v10(deployed_by);
CREATE INDEX IF NOT EXISTS idx_environment_deployment_history_v10_rollback_of ON environment_deployment_history_v10(rollback_of);
