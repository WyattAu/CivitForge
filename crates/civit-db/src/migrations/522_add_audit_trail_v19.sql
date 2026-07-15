CREATE TABLE IF NOT EXISTS audit_trail_v19 (
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

CREATE INDEX IF NOT EXISTS idx_audit_trail_v19_event_type ON audit_trail_v19(event_type);
CREATE INDEX IF NOT EXISTS idx_audit_trail_v19_resource_type ON audit_trail_v19(resource_type);
CREATE INDEX IF NOT EXISTS idx_audit_trail_v19_resource_id ON audit_trail_v19(resource_id);
CREATE INDEX IF NOT EXISTS idx_audit_trail_v19_actor_id ON audit_trail_v19(actor_id);
CREATE INDEX IF NOT EXISTS idx_audit_trail_v19_action ON audit_trail_v19(action);
CREATE INDEX IF NOT EXISTS idx_audit_trail_v19_request_id ON audit_trail_v19(request_id);
CREATE INDEX IF NOT EXISTS idx_audit_trail_v19_session_id ON audit_trail_v19(session_id);
CREATE INDEX IF NOT EXISTS idx_audit_trail_v19_risk_score ON audit_trail_v19(risk_score);
CREATE INDEX IF NOT EXISTS idx_audit_trail_v19_compliance_status ON audit_trail_v19(compliance_status);
CREATE INDEX IF NOT EXISTS idx_audit_trail_v19_created_at ON audit_trail_v19(created_at);