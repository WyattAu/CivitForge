CREATE TABLE IF NOT EXISTS audit_trail_v7 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    event_type TEXT NOT NULL,
    resource_type TEXT NOT NULL,
    resource_id UUID NOT NULL,
    actor_id UUID REFERENCES users(id),
    action TEXT NOT NULL,
    details JSONB NOT NULL DEFAULT '{}',
    ip_address INET,
    user_agent TEXT,
    request_id UUID,
    session_id TEXT,
    geo_location JSONB,
    risk_score INTEGER NOT NULL DEFAULT 0,
    compliance_status TEXT NOT NULL DEFAULT 'compliant',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
