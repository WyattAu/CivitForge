-- CivitForge Phase 480: Audit Trail V17
-- Migration 480
-- Enhances audit trail with compliance status tracking v17, risk scoring v17, anomaly detection v17, and forensics analysis v17.

CREATE TABLE IF NOT EXISTS audit_trail_v17 (
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

CREATE INDEX idx_audit_trail_v17_event_type ON audit_trail_v17(event_type);
CREATE INDEX idx_audit_trail_v17_resource_type ON audit_trail_v17(resource_type);
CREATE INDEX idx_audit_trail_v17_resource_id ON audit_trail_v17(resource_id);
CREATE INDEX idx_audit_trail_v17_actor_id ON audit_trail_v17(actor_id);
CREATE INDEX idx_audit_trail_v17_action ON audit_trail_v17(action);
CREATE INDEX idx_audit_trail_v17_request_id ON audit_trail_v17(request_id);
CREATE INDEX idx_audit_trail_v17_session_id ON audit_trail_v17(session_id);
CREATE INDEX idx_audit_trail_v17_risk_score ON audit_trail_v17(risk_score);
CREATE INDEX idx_audit_trail_v17_compliance_status ON audit_trail_v17(compliance_status);
CREATE INDEX idx_audit_trail_v17_created_at ON audit_trail_v17(created_at);
