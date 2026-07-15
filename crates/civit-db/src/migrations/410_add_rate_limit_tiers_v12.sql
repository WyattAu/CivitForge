-- Rate Limiting v13 (tiers v12 + alerts v9)
CREATE TABLE IF NOT EXISTS rate_limit_tiers_v12 (
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

CREATE TABLE IF NOT EXISTS rate_limit_alerts_v9 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id),
    tier_id UUID NOT NULL REFERENCES rate_limit_tiers_v12(id),
    alert_type TEXT NOT NULL,
    threshold DOUBLE PRECISION NOT NULL,
    enabled BOOLEAN NOT NULL DEFAULT true,
    last_triggered_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_rate_limit_tiers_v12_name ON rate_limit_tiers_v12(name);
CREATE INDEX IF NOT EXISTS idx_rate_limit_alerts_v9_user_id ON rate_limit_alerts_v9(user_id);
CREATE INDEX IF NOT EXISTS idx_rate_limit_alerts_v9_tier_id ON rate_limit_alerts_v9(tier_id);
CREATE INDEX IF NOT EXISTS idx_rate_limit_alerts_v9_alert_type ON rate_limit_alerts_v9(alert_type);
