CREATE TABLE IF NOT EXISTS environment_deployment_history_v6 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    environment_id UUID NOT NULL REFERENCES pipeline_environments(id),
    version TEXT NOT NULL,
    sha TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'deployed',
    deployed_by UUID NOT NULL REFERENCES users(id),
    rollback_of UUID REFERENCES environment_deployment_history_v6(id),
    metadata JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS deployment_comparison_v6 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    from_deployment_id UUID NOT NULL REFERENCES environment_deployment_history_v6(id),
    to_deployment_id UUID NOT NULL REFERENCES environment_deployment_history_v6(id),
    diff_summary JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(from_deployment_id, to_deployment_id)
);

CREATE TABLE IF NOT EXISTS deployment_analytics_v6 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    environment_id UUID NOT NULL REFERENCES pipeline_environments(id),
    period_start TIMESTAMPTZ NOT NULL,
    total_deployments INTEGER NOT NULL DEFAULT 0,
    successful_deployments INTEGER NOT NULL DEFAULT 0,
    failed_deployments INTEGER NOT NULL DEFAULT 0,
    avg_deploy_time_ms BIGINT NOT NULL DEFAULT 0,
    rollback_count INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
