CREATE TABLE IF NOT EXISTS rate_limit_tiers_v5 (
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

CREATE TABLE IF NOT EXISTS rate_limit_alerts_v2 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id),
    tier_id UUID NOT NULL REFERENCES rate_limit_tiers_v5(id),
    alert_type TEXT NOT NULL,
    threshold DOUBLE PRECISION NOT NULL,
    enabled BOOLEAN NOT NULL DEFAULT true,
    last_triggered_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_rate_limit_tiers_v5_name ON rate_limit_tiers_v5(name);
CREATE INDEX IF NOT EXISTS idx_rate_limit_alerts_v2_user ON rate_limit_alerts_v2(user_id);
CREATE INDEX IF NOT EXISTS idx_rate_limit_alerts_v2_tier ON rate_limit_alerts_v2(tier_id);
CREATE INDEX IF NOT EXISTS idx_rate_limit_alerts_v2_type ON rate_limit_alerts_v2(alert_type);
CREATE INDEX IF NOT EXISTS idx_rate_limit_alerts_v2_enabled ON rate_limit_alerts_v2(enabled);
