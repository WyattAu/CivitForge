-- CivitForge Phase 141: Access Control Lists
-- Migration 141
-- Adds tables for fine-grained access control with permission inheritance.

CREATE TABLE IF NOT EXISTS access_control_lists (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    resource_type TEXT NOT NULL,
    resource_id UUID NOT NULL,
    principal_type TEXT NOT NULL,
    principal_id UUID NOT NULL,
    permission TEXT NOT NULL,
    granted_by UUID REFERENCES users(id),
    expires_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(resource_type, resource_id, principal_type, principal_id, permission)
);

CREATE INDEX IF NOT EXISTS idx_acl_resource ON access_control_lists(resource_type, resource_id);
CREATE INDEX IF NOT EXISTS idx_acl_principal ON access_control_lists(principal_type, principal_id);
CREATE INDEX IF NOT EXISTS idx_acl_permission ON access_control_lists(permission);
CREATE INDEX IF NOT EXISTS idx_acl_expires ON access_control_lists(expires_at) WHERE expires_at IS NOT NULL;
