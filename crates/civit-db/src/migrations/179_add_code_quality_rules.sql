CREATE TABLE IF NOT EXISTS code_quality_rules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    repo_id UUID NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    rule_type TEXT NOT NULL,
    severity TEXT NOT NULL DEFAULT 'warning',
    pattern TEXT,
    enabled BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_code_quality_rules_repo_id ON code_quality_rules(repo_id);
CREATE INDEX idx_code_quality_rules_enabled ON code_quality_rules(enabled);
CREATE INDEX idx_code_quality_rules_severity ON code_quality_rules(severity);
CREATE INDEX idx_code_quality_rules_repo_name ON code_quality_rules(repo_id, name);
