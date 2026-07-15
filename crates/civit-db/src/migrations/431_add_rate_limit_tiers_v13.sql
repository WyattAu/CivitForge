-- Rate Limiting v14 (tiers v13 + alerts v10)
CREATE TABLE IF NOT EXISTS rate_limit_tiers_v13 (
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

CREATE TABLE IF NOT EXISTS rate_limit_alerts_v10 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id),
    tier_id UUID NOT NULL REFERENCES rate_limit_tiers_v13(id),
    alert_type TEXT NOT NULL,
    threshold DOUBLE PRECISION NOT NULL,
    enabled BOOLEAN NOT NULL DEFAULT true,
    last_triggered_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_rate_limit_tiers_v13_name ON rate_limit_tiers_v13(name);
CREATE INDEX IF NOT EXISTS idx_rate_limit_alerts_v10_user_id ON rate_limit_alerts_v10(user_id);
CREATE INDEX IF NOT EXISTS idx_rate_limit_alerts_v10_tier_id ON rate_limit_alerts_v10(tier_id);
CREATE INDEX IF NOT EXISTS idx_rate_limit_alerts_v10_alert_type ON rate_limit_alerts_v10(alert_type);
