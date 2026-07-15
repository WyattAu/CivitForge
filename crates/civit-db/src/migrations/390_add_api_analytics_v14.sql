CREATE TABLE IF NOT EXISTS api_analytics_v14 (
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

CREATE INDEX IF NOT EXISTS idx_api_analytics_v14_endpoint ON api_analytics_v14(endpoint);
CREATE INDEX IF NOT EXISTS idx_api_analytics_v14_method ON api_analytics_v14(method);
CREATE INDEX IF NOT EXISTS idx_api_analytics_v14_status ON api_analytics_v14(status_code);
CREATE INDEX IF NOT EXISTS idx_api_analytics_v14_user ON api_analytics_v14(user_id);
CREATE INDEX IF NOT EXISTS idx_api_analytics_v14_region ON api_analytics_v14(region);
CREATE INDEX IF NOT EXISTS idx_api_analytics_v14_created ON api_analytics_v14(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_api_analytics_v14_cache ON api_analytics_v14(cache_hit);
