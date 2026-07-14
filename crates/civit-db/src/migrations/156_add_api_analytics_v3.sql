CREATE TABLE IF NOT EXISTS api_analytics_v3 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    endpoint TEXT NOT NULL,
    method TEXT NOT NULL,
    status_code INTEGER NOT NULL,
    response_time_ms INTEGER NOT NULL,
    user_id UUID,
    request_size_bytes INTEGER NOT NULL DEFAULT 0,
    response_size_bytes INTEGER NOT NULL DEFAULT 0,
    cache_hit BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_api_analytics_v3_endpoint ON api_analytics_v3(endpoint);
CREATE INDEX IF NOT EXISTS idx_api_analytics_v3_method ON api_analytics_v3(method);
CREATE INDEX IF NOT EXISTS idx_api_analytics_v3_status_code ON api_analytics_v3(status_code);
CREATE INDEX IF NOT EXISTS idx_api_analytics_v3_user_id ON api_analytics_v3(user_id);
CREATE INDEX IF NOT EXISTS idx_api_analytics_v3_created_at ON api_analytics_v3(created_at);
CREATE INDEX IF NOT EXISTS idx_api_analytics_v3_cache_hit ON api_analytics_v3(cache_hit);
