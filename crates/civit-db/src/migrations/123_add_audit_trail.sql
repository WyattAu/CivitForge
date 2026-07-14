-- CivitForge Phase 123: Audit Trail
-- Migration 123
-- Adds comprehensive audit trail for recording events, search, export, and compliance audit.

CREATE TABLE IF NOT EXISTS audit_trail (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    event_type TEXT NOT NULL,
    resource_type TEXT NOT NULL,
    resource_id UUID NOT NULL,
    actor_id UUID REFERENCES users(id),
    action TEXT NOT NULL,
    details JSONB NOT NULL DEFAULT '{}',
    ip_address INET,
    user_agent TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_audit_trail_event_type ON audit_trail(event_type);
CREATE INDEX idx_audit_trail_resource_type ON audit_trail(resource_type);
CREATE INDEX idx_audit_trail_resource_id ON audit_trail(resource_id);
CREATE INDEX idx_audit_trail_actor_id ON audit_trail(actor_id);
CREATE INDEX idx_audit_trail_action ON audit_trail(action);
CREATE INDEX idx_audit_trail_created_at ON audit_trail(created_at);
