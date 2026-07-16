-- Pipeline Environments v24: Deployment history v19
CREATE TABLE IF NOT EXISTS environment_deployment_history_v19 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    environment_id UUID NOT NULL REFERENCES pipeline_environments(id),
    version TEXT NOT NULL,
    sha TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'deployed',
    deployed_by UUID NOT NULL REFERENCES users(id),
    rollback_of UUID REFERENCES environment_deployment_history_v19(id),
    metadata JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_env_deploy_history_v19_env ON environment_deployment_history_v19(environment_id);
CREATE INDEX IF NOT EXISTS idx_env_deploy_history_v19_status ON environment_deployment_history_v19(status);
CREATE INDEX IF NOT EXISTS idx_env_deploy_history_v19_deployed_by ON environment_deployment_history_v19(deployed_by);
CREATE INDEX IF NOT EXISTS idx_env_deploy_history_v19_rollback ON environment_deployment_history_v19(rollback_of);
CREATE INDEX IF NOT EXISTS idx_env_deploy_history_v19_created ON environment_deployment_history_v19(created_at DESC);
