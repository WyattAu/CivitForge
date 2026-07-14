CREATE TABLE IF NOT EXISTS rate_limit_tiers_v2 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL UNIQUE,
    description TEXT NOT NULL DEFAULT '',
    rate_limit INTEGER NOT NULL,
    burst_limit INTEGER NOT NULL,
    monthly_quota INTEGER,
    price_cents INTEGER NOT NULL DEFAULT 0,
    features JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS rate_limit_usage_v2 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id),
    tier_id UUID NOT NULL REFERENCES rate_limit_tiers_v2(id),
    period_start TIMESTAMPTZ NOT NULL,
    usage_count INTEGER NOT NULL DEFAULT 0,
    UNIQUE(user_id, tier_id, period_start)
);

CREATE INDEX IF NOT EXISTS idx_rate_limit_tiers_v2_name ON rate_limit_tiers_v2(name);
CREATE INDEX IF NOT EXISTS idx_rate_limit_usage_v2_user ON rate_limit_usage_v2(user_id);
CREATE INDEX IF NOT EXISTS idx_rate_limit_usage_v2_tier ON rate_limit_usage_v2(tier_id);
CREATE INDEX IF NOT EXISTS idx_rate_limit_usage_v2_period ON rate_limit_usage_v2(period_start);
