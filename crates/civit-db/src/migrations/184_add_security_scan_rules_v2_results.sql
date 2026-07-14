-- CivitForge Phase 184: Security Scan Rules V2 & Results
-- Migration 184
-- Enhances security scanning with rule versioning, result tracking, and compliance mapping.

CREATE TABLE IF NOT EXISTS security_scan_rules_v2 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    rule_type TEXT NOT NULL,
    severity TEXT NOT NULL DEFAULT 'medium',
    pattern TEXT NOT NULL,
    enabled BOOLEAN NOT NULL DEFAULT true,
    version INTEGER NOT NULL DEFAULT 1,
    author_id UUID REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS security_scan_results (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    scan_id UUID NOT NULL REFERENCES security_scans_v2(id),
    rule_id UUID NOT NULL REFERENCES security_scan_rules_v2(id),
    file_path TEXT,
    line_number INTEGER,
    message TEXT NOT NULL,
    severity TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_security_scan_rules_v2_name ON security_scan_rules_v2(name);
CREATE INDEX idx_security_scan_rules_v2_rule_type ON security_scan_rules_v2(rule_type);
CREATE INDEX idx_security_scan_rules_v2_severity ON security_scan_rules_v2(severity);
CREATE INDEX idx_security_scan_rules_v2_enabled ON security_scan_rules_v2(enabled);
CREATE INDEX idx_security_scan_rules_v2_version ON security_scan_rules_v2(version);
CREATE INDEX idx_security_scan_rules_v2_author_id ON security_scan_rules_v2(author_id);
CREATE INDEX idx_security_scan_results_scan_id ON security_scan_results(scan_id);
CREATE INDEX idx_security_scan_results_rule_id ON security_scan_results(rule_id);
CREATE INDEX idx_security_scan_results_severity ON security_scan_results(severity);
CREATE INDEX idx_security_scan_results_file_path ON security_scan_results(file_path);
