-- Migration 417: Audit Trail v14

CREATE TABLE IF NOT EXISTS audit_trail_v14 (
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

CREATE INDEX IF NOT EXISTS idx_audit_trail_v14_event_type ON audit_trail_v14(event_type);
CREATE INDEX IF NOT EXISTS idx_audit_trail_v14_resource ON audit_trail_v14(resource_type, resource_id);
CREATE INDEX IF NOT EXISTS idx_audit_trail_v14_actor ON audit_trail_v14(actor_id);
CREATE INDEX IF NOT EXISTS idx_audit_trail_v14_session ON audit_trail_v14(session_id);
CREATE INDEX IF NOT EXISTS idx_audit_trail_v14_risk ON audit_trail_v14(risk_score);
CREATE INDEX IF NOT EXISTS idx_audit_trail_v14_compliance ON audit_trail_v14(compliance_status);
CREATE INDEX IF NOT EXISTS idx_audit_trail_v14_created ON audit_trail_v14(created_at);
