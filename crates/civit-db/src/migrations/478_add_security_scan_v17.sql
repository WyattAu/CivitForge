-- CivitForge Phase 478: Security Scanning V17
-- Migration 478
-- Enhances security scanning with rule versioning v16, fix tracking v14, compliance mapping v16, and fix analytics v16.

CREATE TABLE IF NOT EXISTS security_scan_rules_v16 (
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

CREATE TABLE IF NOT EXISTS security_scan_fixes_v14 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    scan_id UUID NOT NULL REFERENCES security_scans_v2(id),
    rule_id UUID NOT NULL REFERENCES security_scan_rules_v16(id),
    file_path TEXT NOT NULL,
    line_number INTEGER,
    fix_type TEXT NOT NULL,
    fix_content TEXT NOT NULL,
    applied BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_security_scan_rules_v16_name ON security_scan_rules_v16(name);
CREATE INDEX idx_security_scan_rules_v16_rule_type ON security_scan_rules_v16(rule_type);
CREATE INDEX idx_security_scan_rules_v16_severity ON security_scan_rules_v16(severity);
CREATE INDEX idx_security_scan_rules_v16_enabled ON security_scan_rules_v16(enabled);
CREATE INDEX idx_security_scan_rules_v16_author_id ON security_scan_rules_v16(author_id);
CREATE INDEX idx_security_scan_fixes_v14_scan_id ON security_scan_fixes_v14(scan_id);
CREATE INDEX idx_security_scan_fixes_v14_rule_id ON security_scan_fixes_v14(rule_id);
CREATE INDEX idx_security_scan_fixes_v14_applied ON security_scan_fixes_v14(applied);
