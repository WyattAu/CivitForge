CREATE TABLE IF NOT EXISTS deployment_history (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    environment_id UUID NOT NULL REFERENCES pipeline_environments(id) ON DELETE CASCADE,
    version TEXT NOT NULL,
    sha TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'deployed',
    deployed_by UUID NOT NULL REFERENCES users(id),
    rollback_of UUID REFERENCES deployment_history(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_deployment_history_env_id ON deployment_history(environment_id);
CREATE INDEX IF NOT EXISTS idx_deployment_history_status ON deployment_history(status);
CREATE INDEX IF NOT EXISTS idx_deployment_history_created_at ON deployment_history(created_at DESC);
