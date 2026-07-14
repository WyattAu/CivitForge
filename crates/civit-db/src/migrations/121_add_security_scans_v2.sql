-- CivitForge Phase 121: Security Scans V2 & Security Policies
-- Migration 121
-- Adds comprehensive security scanning with SAST, DAST, container, dependency scanning and policy enforcement.

CREATE TABLE IF NOT EXISTS security_scans_v2 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    repo_id UUID NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    scan_type TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending',
    findings JSONB NOT NULL DEFAULT '[]',
    score INTEGER NOT NULL DEFAULT 0,
    started_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS security_policies (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    repo_id UUID NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    policy_name TEXT NOT NULL,
    rules JSONB NOT NULL DEFAULT '{}',
    enabled BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_security_scans_v2_repo_id ON security_scans_v2(repo_id);
CREATE INDEX idx_security_scans_v2_status ON security_scans_v2(status);
CREATE INDEX idx_security_scans_v2_scan_type ON security_scans_v2(scan_type);
CREATE INDEX idx_security_policies_repo_id ON security_policies(repo_id);
CREATE INDEX idx_security_policies_enabled ON security_policies(enabled);
