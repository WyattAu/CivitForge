CREATE TABLE IF NOT EXISTS rate_limit_policies (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL UNIQUE,
    rate_limit INTEGER NOT NULL,
    window_seconds INTEGER NOT NULL,
    burst_size INTEGER NOT NULL DEFAULT 0,
    enabled BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS rate_limit_buckets (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    key TEXT NOT NULL,
    tokens INTEGER NOT NULL,
    last_refill TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(key)
);

CREATE INDEX IF NOT EXISTS idx_rate_limit_policies_name ON rate_limit_policies(name);
CREATE INDEX IF NOT EXISTS idx_rate_limit_policies_enabled ON rate_limit_policies(enabled);
CREATE INDEX IF NOT EXISTS idx_rate_limit_buckets_key ON rate_limit_buckets(key);
CREATE INDEX IF NOT EXISTS idx_rate_limit_buckets_last_refill ON rate_limit_buckets(last_refill);
