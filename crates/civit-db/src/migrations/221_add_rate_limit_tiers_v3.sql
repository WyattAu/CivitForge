CREATE TABLE IF NOT EXISTS rate_limit_tiers_v3 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL UNIQUE,
    description TEXT NOT NULL DEFAULT '',
    rate_limit INTEGER NOT NULL,
    burst_limit INTEGER NOT NULL,
    monthly_quota INTEGER,
    price_cents INTEGER NOT NULL DEFAULT 0,
    features JSONB NOT NULL DEFAULT '{}',
    limits JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS rate_limit_overages (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id),
    tier_id UUID NOT NULL REFERENCES rate_limit_tiers_v3(id),
    period_start TIMESTAMPTZ NOT NULL,
    overage_count INTEGER NOT NULL DEFAULT 0,
    overage_cost_cents INTEGER NOT NULL DEFAULT 0,
    UNIQUE(user_id, tier_id, period_start)
);

CREATE TABLE IF NOT EXISTS rate_limit_alerts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id),
    tier_id UUID NOT NULL REFERENCES rate_limit_tiers_v3(id),
    alert_type TEXT NOT NULL,
    threshold_percent INTEGER NOT NULL,
    current_usage INTEGER NOT NULL,
    triggered_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    acknowledged BOOLEAN NOT NULL DEFAULT false
);

CREATE INDEX IF NOT EXISTS idx_rate_limit_tiers_v3_name ON rate_limit_tiers_v3(name);
CREATE INDEX IF NOT EXISTS idx_rate_limit_overages_user ON rate_limit_overages(user_id);
CREATE INDEX IF NOT EXISTS idx_rate_limit_overages_tier ON rate_limit_overages(tier_id);
CREATE INDEX IF NOT EXISTS idx_rate_limit_overages_period ON rate_limit_overages(period_start);
CREATE INDEX IF NOT EXISTS idx_rate_limit_alerts_user ON rate_limit_alerts(user_id);
