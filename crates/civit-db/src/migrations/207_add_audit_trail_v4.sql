-- CivitForge Phase 207: Audit Trail V4
-- Migration 207
-- Enhances audit trail with risk scoring, anomaly detection, compliance reporting, and forensics.

CREATE TABLE IF NOT EXISTS audit_trail_v4 (
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
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_audit_trail_v4_event_type ON audit_trail_v4(event_type);
CREATE INDEX idx_audit_trail_v4_resource_type ON audit_trail_v4(resource_type);
CREATE INDEX idx_audit_trail_v4_resource_id ON audit_trail_v4(resource_id);
CREATE INDEX idx_audit_trail_v4_actor_id ON audit_trail_v4(actor_id);
CREATE INDEX idx_audit_trail_v4_action ON audit_trail_v4(action);
CREATE INDEX idx_audit_trail_v4_request_id ON audit_trail_v4(request_id);
CREATE INDEX idx_audit_trail_v4_session_id ON audit_trail_v4(session_id);
CREATE INDEX idx_audit_trail_v4_risk_score ON audit_trail_v4(risk_score);
CREATE INDEX idx_audit_trail_v4_created_at ON audit_trail_v4(created_at);
CREATE INDEX idx_audit_trail_v4_compliance ON audit_trail_v4(event_type, created_at);
CREATE INDEX idx_audit_trail_v4_forensics ON audit_trail_v4(actor_id, created_at, event_type);
CREATE INDEX idx_audit_trail_v4_risk_analysis ON audit_trail_v4(risk_score, event_type, created_at);
