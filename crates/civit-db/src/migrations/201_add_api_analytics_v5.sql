CREATE TABLE IF NOT EXISTS api_analytics_v5 (
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
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_api_analytics_v5_endpoint ON api_analytics_v5(endpoint);
CREATE INDEX IF NOT EXISTS idx_api_analytics_v5_user ON api_analytics_v5(user_id);
CREATE INDEX IF NOT EXISTS idx_api_analytics_v5_created ON api_analytics_v5(created_at);
CREATE INDEX IF NOT EXISTS idx_api_analytics_v5_region ON api_analytics_v5(region);
CREATE INDEX IF NOT EXISTS idx_api_analytics_v5_status ON api_analytics_v5(status_code);
