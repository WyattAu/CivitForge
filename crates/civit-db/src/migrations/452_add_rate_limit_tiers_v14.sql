-- Rate Limiting v15 (tiers v14 + alerts v11)
CREATE TABLE IF NOT EXISTS rate_limit_tiers_v14 (
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

CREATE TABLE IF NOT EXISTS rate_limit_alerts_v11 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id),
    tier_id UUID NOT NULL REFERENCES rate_limit_tiers_v14(id),
    alert_type TEXT NOT NULL,
    threshold DOUBLE PRECISION NOT NULL,
    enabled BOOLEAN NOT NULL DEFAULT true,
    last_triggered_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_rate_limit_tiers_v14_name ON rate_limit_tiers_v14(name);
CREATE INDEX IF NOT EXISTS idx_rate_limit_alerts_v11_user_id ON rate_limit_alerts_v11(user_id);
CREATE INDEX IF NOT EXISTS idx_rate_limit_alerts_v11_tier_id ON rate_limit_alerts_v11(tier_id);
CREATE INDEX IF NOT EXISTS idx_rate_limit_alerts_v11_alert_type ON rate_limit_alerts_v11(alert_type);
