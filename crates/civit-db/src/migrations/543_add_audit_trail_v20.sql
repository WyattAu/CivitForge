CREATE TABLE IF NOT EXISTS audit_trail_v20 (
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

CREATE INDEX IF NOT EXISTS idx_audit_trail_v20_event_type ON audit_trail_v20(event_type);
CREATE INDEX IF NOT EXISTS idx_audit_trail_v20_resource ON audit_trail_v20(resource_type, resource_id);
CREATE INDEX IF NOT EXISTS idx_audit_trail_v20_actor ON audit_trail_v20(actor_id);
CREATE INDEX IF NOT EXISTS idx_audit_trail_v20_compliance ON audit_trail_v20(compliance_status);
CREATE INDEX IF NOT EXISTS idx_audit_trail_v20_risk ON audit_trail_v20(risk_score);
CREATE INDEX IF NOT EXISTS idx_audit_trail_v20_created ON audit_trail_v20(created_at);
