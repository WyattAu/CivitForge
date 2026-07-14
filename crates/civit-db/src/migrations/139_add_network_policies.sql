-- CivitForge Phase 139: Network Policies
-- Migration 139
-- Adds tables for network policy management with ingress/egress rules.

CREATE TABLE IF NOT EXISTS network_policies (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    ingress_rules JSONB NOT NULL DEFAULT '[]',
    egress_rules JSONB NOT NULL DEFAULT '[]',
    enabled BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_network_policies_name ON network_policies(name);
CREATE INDEX IF NOT EXISTS idx_network_policies_enabled ON network_policies(enabled);
