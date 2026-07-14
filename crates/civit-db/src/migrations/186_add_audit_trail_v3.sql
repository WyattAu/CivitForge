-- CivitForge Phase 186: Audit Trail V3
-- Migration 186
-- Enhances audit trail with geographic tracking, enhanced session tracking, and forensics support.

CREATE TABLE IF NOT EXISTS audit_trail_v3 (
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
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_audit_trail_v3_event_type ON audit_trail_v3(event_type);
CREATE INDEX idx_audit_trail_v3_resource_type ON audit_trail_v3(resource_type);
CREATE INDEX idx_audit_trail_v3_resource_id ON audit_trail_v3(resource_id);
CREATE INDEX idx_audit_trail_v3_actor_id ON audit_trail_v3(actor_id);
CREATE INDEX idx_audit_trail_v3_action ON audit_trail_v3(action);
CREATE INDEX idx_audit_trail_v3_request_id ON audit_trail_v3(request_id);
CREATE INDEX idx_audit_trail_v3_session_id ON audit_trail_v3(session_id);
CREATE INDEX idx_audit_trail_v3_created_at ON audit_trail_v3(created_at);
CREATE INDEX idx_audit_trail_v3_compliance ON audit_trail_v3(event_type, created_at);
CREATE INDEX idx_audit_trail_v3_forensics ON audit_trail_v3(actor_id, created_at, event_type);
