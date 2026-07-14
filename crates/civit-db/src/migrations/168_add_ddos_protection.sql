CREATE TABLE IF NOT EXISTS ddos_protection (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL UNIQUE,
    enabled BOOLEAN NOT NULL DEFAULT true,
    threshold_rps INTEGER NOT NULL DEFAULT 1000,
    threshold_bps BIGINT NOT NULL DEFAULT 1000000000,
    action TEXT NOT NULL DEFAULT 'block',
    duration_seconds INTEGER NOT NULL DEFAULT 300,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS ddos_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    protection_id UUID NOT NULL REFERENCES ddos_protection(id) ON DELETE CASCADE,
    source_ip INET NOT NULL,
    request_rate DOUBLE PRECISION NOT NULL,
    action_taken TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_ddos_protection_name ON ddos_protection(name);
CREATE INDEX IF NOT EXISTS idx_ddos_events_protection_id ON ddos_events(protection_id);
CREATE INDEX IF NOT EXISTS idx_ddos_events_created_at ON ddos_events(created_at);
