CREATE TABLE IF NOT EXISTS rate_limit_tiers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL UNIQUE,
    description TEXT NOT NULL DEFAULT '',
    rate_limit INTEGER NOT NULL,
    burst_limit INTEGER NOT NULL,
    monthly_quota INTEGER,
    price_cents INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_rate_limit_tiers_name ON rate_limit_tiers(name);
