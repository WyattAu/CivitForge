-- CivitForge Phase 160: Security Scan Rules
-- Migration 160
-- Adds custom security scan rules with pattern matching, versioning, and testing support.

CREATE TABLE IF NOT EXISTS security_scan_rules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    rule_type TEXT NOT NULL,
    severity TEXT NOT NULL DEFAULT 'medium',
    pattern TEXT NOT NULL,
    enabled BOOLEAN NOT NULL DEFAULT true,
    version INTEGER NOT NULL DEFAULT 1,
    created_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_security_scan_rules_name ON security_scan_rules(name);
CREATE INDEX idx_security_scan_rules_rule_type ON security_scan_rules(rule_type);
CREATE INDEX idx_security_scan_rules_severity ON security_scan_rules(severity);
CREATE INDEX idx_security_scan_rules_enabled ON security_scan_rules(enabled);
CREATE INDEX idx_security_scan_rules_version ON security_scan_rules(version);
