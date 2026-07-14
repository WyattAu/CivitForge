-- Migration 197: Code Quality Rules V2
-- Adds enhanced code quality rules with auto-fix support, versioning, and analytics.

CREATE TABLE IF NOT EXISTS code_quality_rules_v2 (
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
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS code_quality_rule_versions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    rule_id UUID NOT NULL REFERENCES code_quality_rules_v2(id) ON DELETE CASCADE,
    version INTEGER NOT NULL DEFAULT 1,
    config_snapshot JSONB NOT NULL,
    change_description TEXT NOT NULL DEFAULT '',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS code_quality_rule_test_results (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    rule_id UUID NOT NULL REFERENCES code_quality_rules_v2(id) ON DELETE CASCADE,
    test_file TEXT NOT NULL,
    expected_violations INTEGER NOT NULL DEFAULT 0,
    actual_violations INTEGER NOT NULL DEFAULT 0,
    passed BOOLEAN NOT NULL DEFAULT false,
    tested_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_code_quality_rules_v2_repo_id ON code_quality_rules_v2(repo_id);
CREATE INDEX IF NOT EXISTS idx_code_quality_rules_v2_enabled ON code_quality_rules_v2(enabled);
CREATE INDEX IF NOT EXISTS idx_code_quality_rules_v2_severity ON code_quality_rules_v2(severity);
CREATE INDEX IF NOT EXISTS idx_code_quality_rules_v2_repo_name ON code_quality_rules_v2(repo_id, name);
CREATE INDEX IF NOT EXISTS idx_code_quality_rule_versions_rule_id ON code_quality_rule_versions(rule_id);
CREATE INDEX IF NOT EXISTS idx_code_quality_rule_test_results_rule_id ON code_quality_rule_test_results(rule_id);
