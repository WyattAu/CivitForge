CREATE TABLE IF NOT EXISTS security_scan_rules_v17 (
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

CREATE TABLE IF NOT EXISTS security_scan_fixes_v15 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    scan_id UUID NOT NULL REFERENCES security_scans_v2(id),
    rule_id UUID NOT NULL REFERENCES security_scan_rules_v17(id),
    file_path TEXT NOT NULL,
    line_number INTEGER,
    fix_type TEXT NOT NULL,
    fix_content TEXT NOT NULL,
    applied BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_security_scan_rules_v17_rule_type ON security_scan_rules_v17(rule_type);
CREATE INDEX IF NOT EXISTS idx_security_scan_rules_v17_severity ON security_scan_rules_v17(severity);
CREATE INDEX IF NOT EXISTS idx_security_scan_rules_v17_enabled ON security_scan_rules_v17(enabled);
CREATE INDEX IF NOT EXISTS idx_security_scan_fixes_v15_scan_id ON security_scan_fixes_v15(scan_id);
CREATE INDEX IF NOT EXISTS idx_security_scan_fixes_v15_rule_id ON security_scan_fixes_v15(rule_id);
CREATE INDEX IF NOT EXISTS idx_security_scan_fixes_v15_applied ON security_scan_fixes_v15(applied);
