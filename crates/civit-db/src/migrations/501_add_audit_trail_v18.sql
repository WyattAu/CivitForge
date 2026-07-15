CREATE TABLE IF NOT EXISTS audit_trail_v18 (
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

CREATE INDEX IF NOT EXISTS idx_audit_trail_v18_event_type ON audit_trail_v18(event_type);
CREATE INDEX IF NOT EXISTS idx_audit_trail_v18_resource_type ON audit_trail_v18(resource_type);
CREATE INDEX IF NOT EXISTS idx_audit_trail_v18_resource_id ON audit_trail_v18(resource_id);
CREATE INDEX IF NOT EXISTS idx_audit_trail_v18_actor_id ON audit_trail_v18(actor_id);
CREATE INDEX IF NOT EXISTS idx_audit_trail_v18_action ON audit_trail_v18(action);
CREATE INDEX IF NOT EXISTS idx_audit_trail_v18_request_id ON audit_trail_v18(request_id);
CREATE INDEX IF NOT EXISTS idx_audit_trail_v18_session_id ON audit_trail_v18(session_id);
CREATE INDEX IF NOT EXISTS idx_audit_trail_v18_risk_score ON audit_trail_v18(risk_score);
CREATE INDEX IF NOT EXISTS idx_audit_trail_v18_compliance_status ON audit_trail_v18(compliance_status);
CREATE INDEX IF NOT EXISTS idx_audit_trail_v18_created_at ON audit_trail_v18(created_at);
