-- API Analytics v15
CREATE TABLE IF NOT EXISTS api_analytics_v15 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    endpoint TEXT NOT NULL,
    method TEXT NOT NULL,
    status_code INTEGER NOT NULL,
    response_time_ms INTEGER NOT NULL,
    user_id UUID,
    request_size_bytes INTEGER NOT NULL DEFAULT 0,
    response_size_bytes INTEGER NOT NULL DEFAULT 0,
    cache_hit BOOLEAN NOT NULL DEFAULT false,
    region TEXT NOT NULL DEFAULT 'us-east-1',
    user_agent TEXT,
    request_id UUID,
    cost_cents INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_api_analytics_v15_endpoint ON api_analytics_v15(endpoint);
CREATE INDEX IF NOT EXISTS idx_api_analytics_v15_method ON api_analytics_v15(method);
CREATE INDEX IF NOT EXISTS idx_api_analytics_v15_status_code ON api_analytics_v15(status_code);
CREATE INDEX IF NOT EXISTS idx_api_analytics_v15_user_id ON api_analytics_v15(user_id);
CREATE INDEX IF NOT EXISTS idx_api_analytics_v15_region ON api_analytics_v15(region);
CREATE INDEX IF NOT EXISTS idx_api_analytics_v15_created_at ON api_analytics_v15(created_at);
CREATE INDEX IF NOT EXISTS idx_api_analytics_v15_cost_cents ON api_analytics_v15(cost_cents);
