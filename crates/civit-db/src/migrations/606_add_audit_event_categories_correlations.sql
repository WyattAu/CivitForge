CREATE TABLE IF NOT EXISTS audit_event_categories_v20 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL UNIQUE,
    description TEXT NOT NULL DEFAULT '',
    severity TEXT NOT NULL DEFAULT 'info',
    retention_days INTEGER NOT NULL DEFAULT 365,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS audit_event_correlations_v20 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    event_id UUID NOT NULL REFERENCES audit_events(id) ON DELETE CASCADE,
    correlated_event_id UUID NOT NULL REFERENCES audit_events(id),
    correlation_type TEXT NOT NULL DEFAULT 'related',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(event_id, correlated_event_id)
);
