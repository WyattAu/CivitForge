CREATE TABLE IF NOT EXISTS deployment_strategy_configs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    strategy_id UUID NOT NULL REFERENCES deployment_strategies(id) ON DELETE CASCADE,
    config_key TEXT NOT NULL,
    config_value JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(strategy_id, config_key)
);

CREATE TABLE IF NOT EXISTS deployment_strategy_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    strategy_id UUID NOT NULL REFERENCES deployment_strategies(id),
    action TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'success',
    details JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);