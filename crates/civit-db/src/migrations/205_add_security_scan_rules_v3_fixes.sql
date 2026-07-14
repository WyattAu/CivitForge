-- CivitForge Phase 205: Security Scan Rules V3 & Fixes
-- Migration 205
-- Enhances security scanning with auto-fix support, fix tracking, and fix analytics.

CREATE TABLE IF NOT EXISTS security_scan_rules_v3 (
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
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS security_scan_fixes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    scan_id UUID NOT NULL REFERENCES security_scans_v2(id),
    rule_id UUID NOT NULL REFERENCES security_scan_rules_v3(id),
    file_path TEXT NOT NULL,
    line_number INTEGER,
    fix_type TEXT NOT NULL,
    fix_content TEXT NOT NULL,
    applied BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_security_scan_rules_v3_name ON security_scan_rules_v3(name);
CREATE INDEX idx_security_scan_rules_v3_rule_type ON security_scan_rules_v3(rule_type);
CREATE INDEX idx_security_scan_rules_v3_severity ON security_scan_rules_v3(severity);
CREATE INDEX idx_security_scan_rules_v3_enabled ON security_scan_rules_v3(enabled);
CREATE INDEX idx_security_scan_rules_v3_version ON security_scan_rules_v3(version);
CREATE INDEX idx_security_scan_rules_v3_author_id ON security_scan_rules_v3(author_id);
CREATE INDEX idx_security_scan_rules_v3_auto_fix ON security_scan_rules_v3(auto_fix);
CREATE INDEX idx_security_scan_fixes_scan_id ON security_scan_fixes(scan_id);
CREATE INDEX idx_security_scan_fixes_rule_id ON security_scan_fixes(rule_id);
CREATE INDEX idx_security_scan_fixes_file_path ON security_scan_fixes(file_path);
CREATE INDEX idx_security_scan_fixes_applied ON security_scan_fixes(applied);
CREATE INDEX idx_security_scan_fixes_created_at ON security_scan_fixes(created_at);
