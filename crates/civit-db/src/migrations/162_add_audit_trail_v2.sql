-- CivitForge Phase 162: Audit Trail V2
-- Migration 162
-- Enhances audit trail with session tracking, request correlation, and compliance audit support.

CREATE TABLE IF NOT EXISTS audit_trail_v2 (
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
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_audit_trail_v2_event_type ON audit_trail_v2(event_type);
CREATE INDEX idx_audit_trail_v2_resource_type ON audit_trail_v2(resource_type);
CREATE INDEX idx_audit_trail_v2_resource_id ON audit_trail_v2(resource_id);
CREATE INDEX idx_audit_trail_v2_actor_id ON audit_trail_v2(actor_id);
CREATE INDEX idx_audit_trail_v2_action ON audit_trail_v2(action);
CREATE INDEX idx_audit_trail_v2_request_id ON audit_trail_v2(request_id);
CREATE INDEX idx_audit_trail_v2_session_id ON audit_trail_v2(session_id);
CREATE INDEX idx_audit_trail_v2_created_at ON audit_trail_v2(created_at);
CREATE INDEX idx_audit_trail_v2_compliance ON audit_trail_v2(event_type, created_at);
