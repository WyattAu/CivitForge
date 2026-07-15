-- CivitForge Phase 436: Security Scan Rules V15
-- Migration 436
-- Adds advanced security scan rules with versioning, fix tracking, compliance mapping, and fix analytics.

CREATE TABLE IF NOT EXISTS security_scan_rules_v14 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    rule_type TEXT NOT NULL,
    severity TEXT NOT NULL DEFAULT 'medium',
    pattern TEXT NOT NULL,
    enabled BOOLEAN NOT NULL DEFAULT true,
    version INTEGER NOT NULL DEFAULT 1,
    author_id UUID REFERENCES users(id),
    auto_fix BOOLEAN NOT NULL DEFAULT false,
    fix_config JSONB NOT NULL DEFAULT '{}',
    compliance_mapping JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS security_scan_fixes_v12 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    scan_id UUID NOT NULL REFERENCES security_scans_v2(id),
    rule_id UUID NOT NULL REFERENCES security_scan_rules_v14(id),
    file_path TEXT NOT NULL,
    line_number INTEGER,
    fix_type TEXT NOT NULL,
    fix_content TEXT NOT NULL,
    applied BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_security_scan_rules_v14_name ON security_scan_rules_v14(name);
CREATE INDEX idx_security_scan_rules_v14_rule_type ON security_scan_rules_v14(rule_type);
CREATE INDEX idx_security_scan_rules_v14_severity ON security_scan_rules_v14(severity);
CREATE INDEX idx_security_scan_rules_v14_enabled ON security_scan_rules_v14(enabled);
CREATE INDEX idx_security_scan_rules_v14_author_id ON security_scan_rules_v14(author_id);
CREATE INDEX idx_security_scan_fixes_v12_scan_id ON security_scan_fixes_v12(scan_id);
CREATE INDEX idx_security_scan_fixes_v12_rule_id ON security_scan_fixes_v12(rule_id);
CREATE INDEX idx_security_scan_fixes_v12_applied ON security_scan_fixes_v12(applied);
