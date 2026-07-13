CREATE TABLE IF NOT EXISTS api_analytics (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    endpoint TEXT NOT NULL,
    method TEXT NOT NULL,
    status_code INTEGER NOT NULL,
    response_time_ms INTEGER NOT NULL,
    user_id UUID,
    ip_address INET,
    user_agent TEXT,
    request_size_bytes INTEGER NOT NULL DEFAULT 0,
    response_size_bytes INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_api_analytics_created_at ON api_analytics(created_at);
CREATE INDEX IF NOT EXISTS idx_api_analytics_endpoint ON api_analytics(endpoint);
CREATE INDEX IF NOT EXISTS idx_api_analytics_user_id ON api_analytics(user_id);
CREATE INDEX IF NOT EXISTS idx_api_analytics_status_code ON api_analytics(status_code);

CREATE TABLE IF NOT EXISTS api_usage_summary (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    period_start TIMESTAMPTZ NOT NULL,
    period_end TIMESTAMPTZ NOT NULL,
    total_requests INTEGER NOT NULL DEFAULT 0,
    total_errors INTEGER NOT NULL DEFAULT 0,
    avg_response_time_ms DOUBLE PRECISION NOT NULL DEFAULT 0,
    p95_response_time_ms DOUBLE PRECISION NOT NULL DEFAULT 0,
    unique_users INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_api_usage_summary_period ON api_usage_summary(period_start, period_end);