-- Migration 218: Code Quality Rules V3 with Enforcement
-- Adds enhanced code quality rules with enforcement configuration, threshold management, and analytics.

CREATE TABLE IF NOT EXISTS code_quality_rules_v3 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    repo_id UUID NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    rule_type TEXT NOT NULL,
    severity TEXT NOT NULL DEFAULT 'warning',
    pattern TEXT,
    auto_fix BOOLEAN NOT NULL DEFAULT false,
    fix_config JSONB NOT NULL DEFAULT '{}',
    enabled BOOLEAN NOT NULL DEFAULT true,
    version INTEGER NOT NULL DEFAULT 1,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS code_quality_rule_enforcement (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    rule_id UUID NOT NULL REFERENCES code_quality_rules_v3(id),
    enforcement_type TEXT NOT NULL DEFAULT 'warn',
    threshold INTEGER NOT NULL DEFAULT 0,
    enabled BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_code_quality_rules_v3_repo_id ON code_quality_rules_v3(repo_id);
CREATE INDEX IF NOT EXISTS idx_code_quality_rules_v3_enabled ON code_quality_rules_v3(enabled);
CREATE INDEX IF NOT EXISTS idx_code_quality_rules_v3_severity ON code_quality_rules_v3(severity);
CREATE INDEX IF NOT EXISTS idx_code_quality_rules_v3_repo_name ON code_quality_rules_v3(repo_id, name);
CREATE INDEX IF NOT EXISTS idx_code_quality_rules_v3_version ON code_quality_rules_v3(version);
CREATE INDEX IF NOT EXISTS idx_code_quality_enforcement_rule_id ON code_quality_rule_enforcement(rule_id);
CREATE INDEX IF NOT EXISTS idx_code_quality_enforcement_type ON code_quality_rule_enforcement(enforcement_type);
CREATE INDEX IF NOT EXISTS idx_code_quality_enforcement_enabled ON code_quality_rule_enforcement(enabled);
