CREATE TABLE IF NOT EXISTS environment_deployment_history_v12 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    environment_id UUID NOT NULL REFERENCES pipeline_environments(id),
    version TEXT NOT NULL,
    sha TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'deployed',
    deployed_by UUID NOT NULL REFERENCES users(id),
    rollback_of UUID REFERENCES environment_deployment_history_v12(id),
    metadata JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_environment_deployment_history_v12_env_id ON environment_deployment_history_v12(environment_id);
CREATE INDEX IF NOT EXISTS idx_environment_deployment_history_v12_status ON environment_deployment_history_v12(status);
CREATE INDEX IF NOT EXISTS idx_environment_deployment_history_v12_created_at ON environment_deployment_history_v12(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_environment_deployment_history_v12_deployed_by ON environment_deployment_history_v12(deployed_by);
CREATE INDEX IF NOT EXISTS idx_environment_deployment_history_v12_rollback_of ON environment_deployment_history_v12(rollback_of);

CREATE TABLE IF NOT EXISTS deployment_comparison_v12 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    from_deployment_id UUID NOT NULL REFERENCES environment_deployment_history_v12(id),
    to_deployment_id UUID NOT NULL REFERENCES environment_deployment_history_v12(id),
    diff_summary JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(from_deployment_id, to_deployment_id)
);

CREATE INDEX IF NOT EXISTS idx_deployment_comparison_v12_from ON deployment_comparison_v12(from_deployment_id);
CREATE INDEX IF NOT EXISTS idx_deployment_comparison_v12_to ON deployment_comparison_v12(to_deployment_id);

CREATE TABLE IF NOT EXISTS deployment_analytics_v12 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    environment_id UUID NOT NULL REFERENCES pipeline_environments(id),
    period_start TIMESTAMPTZ NOT NULL,
    total_deployments INTEGER NOT NULL DEFAULT 0,
    successful_deployments INTEGER NOT NULL DEFAULT 0,
    failed_deployments INTEGER NOT NULL DEFAULT 0,
    avg_deploy_time_ms BIGINT NOT NULL DEFAULT 0,
    rollback_count INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(environment_id, period_start)
);

CREATE INDEX IF NOT EXISTS idx_deployment_analytics_v12_env_id ON deployment_analytics_v12(environment_id);
