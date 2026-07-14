CREATE TABLE IF NOT EXISTS api_analytics_v2 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    endpoint TEXT NOT NULL,
    method TEXT NOT NULL,
    status_code INTEGER NOT NULL,
    response_time_ms INTEGER NOT NULL,
    user_id UUID,
    request_size_bytes INTEGER NOT NULL DEFAULT 0,
    response_size_bytes INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_api_analytics_v2_endpoint ON api_analytics_v2(endpoint);
CREATE INDEX IF NOT EXISTS idx_api_analytics_v2_method ON api_analytics_v2(method);
CREATE INDEX IF NOT EXISTS idx_api_analytics_v2_status_code ON api_analytics_v2(status_code);
CREATE INDEX IF NOT EXISTS idx_api_analytics_v2_user_id ON api_analytics_v2(user_id);
CREATE INDEX IF NOT EXISTS idx_api_analytics_v2_created_at ON api_analytics_v2(created_at);
