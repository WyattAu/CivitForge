CREATE TABLE IF NOT EXISTS api_analytics_v6 (
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
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS api_analytics_correlations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    request_id UUID NOT NULL,
    parent_request_id UUID,
    correlation_type TEXT NOT NULL DEFAULT 'independent',
    trace_id TEXT,
    span_id TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS api_analytics_capacity_plans (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    endpoint TEXT NOT NULL,
    method TEXT NOT NULL,
    current_rps INTEGER NOT NULL DEFAULT 0,
    projected_rps INTEGER NOT NULL DEFAULT 0,
    capacity_limit INTEGER NOT NULL DEFAULT 0,
    utilization_percent NUMERIC(5,2) NOT NULL DEFAULT 0,
    last_calculated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(endpoint, method)
);

CREATE INDEX IF NOT EXISTS idx_api_analytics_v6_endpoint ON api_analytics_v6(endpoint);
CREATE INDEX IF NOT EXISTS idx_api_analytics_v6_method ON api_analytics_v6(method);
CREATE INDEX IF NOT EXISTS idx_api_analytics_v6_status ON api_analytics_v6(status_code);
CREATE INDEX IF NOT EXISTS idx_api_analytics_v6_user ON api_analytics_v6(user_id);
CREATE INDEX IF NOT EXISTS idx_api_analytics_v6_region ON api_analytics_v6(region);
CREATE INDEX IF NOT EXISTS idx_api_analytics_v6_request_id ON api_analytics_v6(request_id);
CREATE INDEX IF NOT EXISTS idx_api_analytics_v6_created ON api_analytics_v6(created_at);
CREATE INDEX IF NOT EXISTS idx_api_analytics_correlations_request ON api_analytics_correlations(request_id);
CREATE INDEX IF NOT EXISTS idx_api_analytics_capacity_endpoint ON api_analytics_capacity_plans(endpoint);
