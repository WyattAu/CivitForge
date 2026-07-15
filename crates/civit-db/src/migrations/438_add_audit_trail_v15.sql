-- CivitForge Phase 438: Audit Trail V15
-- Migration 438
-- Adds compliance status tracking, risk scoring, anomaly detection, and forensics analysis to audit trail.

CREATE TABLE IF NOT EXISTS audit_trail_v15 (
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

CREATE INDEX idx_audit_trail_v15_event_type ON audit_trail_v15(event_type);
CREATE INDEX idx_audit_trail_v15_resource_type ON audit_trail_v15(resource_type);
CREATE INDEX idx_audit_trail_v15_resource_id ON audit_trail_v15(resource_id);
CREATE INDEX idx_audit_trail_v15_actor_id ON audit_trail_v15(actor_id);
CREATE INDEX idx_audit_trail_v15_action ON audit_trail_v15(action);
CREATE INDEX idx_audit_trail_v15_session_id ON audit_trail_v15(session_id);
CREATE INDEX idx_audit_trail_v15_risk_score ON audit_trail_v15(risk_score);
CREATE INDEX idx_audit_trail_v15_compliance_status ON audit_trail_v15(compliance_status);
CREATE INDEX idx_audit_trail_v15_created_at ON audit_trail_v15(created_at);
