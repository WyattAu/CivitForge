-- CivitForge Phase 457: Security Scanning V16
-- Migration 457
-- Enhances security scanning with rule versioning v15, fix tracking v13, compliance mapping v15, and fix analytics v15.

CREATE TABLE IF NOT EXISTS security_scan_rules_v15 (
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

CREATE TABLE IF NOT EXISTS security_scan_fixes_v13 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    scan_id UUID NOT NULL REFERENCES security_scans_v2(id),
    rule_id UUID NOT NULL REFERENCES security_scan_rules_v15(id),
    file_path TEXT NOT NULL,
    line_number INTEGER,
    fix_type TEXT NOT NULL,
    fix_content TEXT NOT NULL,
    applied BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_security_scan_rules_v15_name ON security_scan_rules_v15(name);
CREATE INDEX idx_security_scan_rules_v15_rule_type ON security_scan_rules_v15(rule_type);
CREATE INDEX idx_security_scan_rules_v15_severity ON security_scan_rules_v15(severity);
CREATE INDEX idx_security_scan_rules_v15_enabled ON security_scan_rules_v15(enabled);
CREATE INDEX idx_security_scan_rules_v15_author_id ON security_scan_rules_v15(author_id);
CREATE INDEX idx_security_scan_fixes_v13_scan_id ON security_scan_fixes_v13(scan_id);
CREATE INDEX idx_security_scan_fixes_v13_rule_id ON security_scan_fixes_v13(rule_id);
CREATE INDEX idx_security_scan_fixes_v13_applied ON security_scan_fixes_v13(applied);
