-- Code Quality Rules v22: rules v19, rule usage v19
CREATE TABLE IF NOT EXISTS code_quality_rules_v19 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    rule_type TEXT NOT NULL,
    severity TEXT NOT NULL DEFAULT 'warning',
    enabled BOOLEAN NOT NULL DEFAULT true,
    rule_config JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_code_quality_rules_v19_name ON code_quality_rules_v19(name);
CREATE INDEX IF NOT EXISTS idx_code_quality_rules_v19_type ON code_quality_rules_v19(rule_type);
CREATE INDEX IF NOT EXISTS idx_code_quality_rules_v19_severity ON code_quality_rules_v19(severity);
CREATE INDEX IF NOT EXISTS idx_code_quality_rules_v19_enabled ON code_quality_rules_v19(enabled);

CREATE TABLE IF NOT EXISTS code_quality_rule_usage_v19 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    rule_id UUID NOT NULL REFERENCES code_quality_rules_v19(id) ON DELETE CASCADE,
    repo_id UUID NOT NULL REFERENCES repositories(id),
    trigger_count INTEGER NOT NULL DEFAULT 0,
    last_triggered_at TIMESTAMPTZ,
    UNIQUE(rule_id, repo_id)
);

CREATE INDEX IF NOT EXISTS idx_code_quality_rule_usage_v19_rule ON code_quality_rule_usage_v19(rule_id);
CREATE INDEX IF NOT EXISTS idx_code_quality_rule_usage_v19_repo ON code_quality_rule_usage_v19(repo_id);
CREATE INDEX IF NOT EXISTS idx_code_quality_rule_usage_v19_count ON code_quality_rule_usage_v19(trigger_count DESC);
