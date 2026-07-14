CREATE TABLE IF NOT EXISTS environment_deployment_history_v5 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    environment_id UUID NOT NULL REFERENCES pipeline_environments(id),
    version TEXT NOT NULL,
    sha TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'deployed',
    deployed_by UUID NOT NULL REFERENCES users(id),
    rollback_of UUID REFERENCES environment_deployment_history_v5(id),
    metadata JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS deployment_comparison_v5 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    from_deployment_id UUID NOT NULL REFERENCES environment_deployment_history_v5(id),
    to_deployment_id UUID NOT NULL REFERENCES environment_deployment_history_v5(id),
    diff_summary JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(from_deployment_id, to_deployment_id)
);

CREATE TABLE IF NOT EXISTS deployment_analytics_v5 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    environment_id UUID NOT NULL REFERENCES pipeline_environments(id) ON DELETE CASCADE,
    period_start TIMESTAMPTZ NOT NULL,
    total_deployments INTEGER NOT NULL DEFAULT 0,
    successful_deployments INTEGER NOT NULL DEFAULT 0,
    failed_deployments INTEGER NOT NULL DEFAULT 0,
    avg_deploy_time_ms BIGINT NOT NULL DEFAULT 0,
    rollback_count INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_deployment_history_v5_env_id ON environment_deployment_history_v5(environment_id);
CREATE INDEX IF NOT EXISTS idx_deployment_history_v5_deployed_by ON environment_deployment_history_v5(deployed_by);
CREATE INDEX IF NOT EXISTS idx_deployment_history_v5_status ON environment_deployment_history_v5(status);
CREATE INDEX IF NOT EXISTS idx_deployment_history_v5_created_at ON environment_deployment_history_v5(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_deployment_history_v5_rollback_of ON environment_deployment_history_v5(rollback_of);
CREATE INDEX IF NOT EXISTS idx_deployment_comparison_v5_from ON deployment_comparison_v5(from_deployment_id);
CREATE INDEX IF NOT EXISTS idx_deployment_comparison_v5_to ON deployment_comparison_v5(to_deployment_id);
CREATE INDEX IF NOT EXISTS idx_deployment_analytics_v5_env_id ON deployment_analytics_v5(environment_id);
CREATE INDEX IF NOT EXISTS idx_deployment_analytics_v5_period ON deployment_analytics_v5(period_start DESC);
