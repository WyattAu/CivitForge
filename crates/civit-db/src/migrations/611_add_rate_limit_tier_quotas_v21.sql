CREATE TABLE IF NOT EXISTS rate_limit_tier_quotas_v21 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tier TEXT NOT NULL,
    requests_per_second INTEGER NOT NULL DEFAULT 10,
    requests_per_day INTEGER NOT NULL DEFAULT 10000,
    burst_size INTEGER NOT NULL DEFAULT 50,
    enabled BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(tier)
);

CREATE TABLE IF NOT EXISTS rate_limit_usage_analytics_v21 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id),
    tier TEXT NOT NULL,
    requests_used INTEGER NOT NULL DEFAULT 0,
    period_start TIMESTAMPTZ NOT NULL,
    period_end TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_rate_limit_tier_quotas_v21_tier ON rate_limit_tier_quotas_v21(tier);
CREATE INDEX IF NOT EXISTS idx_rate_limit_usage_analytics_v21_user ON rate_limit_usage_analytics_v21(user_id);
CREATE INDEX IF NOT EXISTS idx_rate_limit_usage_analytics_v21_tier ON rate_limit_usage_analytics_v21(tier);
CREATE INDEX IF NOT EXISTS idx_rate_limit_usage_analytics_v21_period ON rate_limit_usage_analytics_v21(period_start, period_end);
