CREATE TABLE IF NOT EXISTS automation_rules_v2 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    repo_id UUID NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    trigger_type TEXT NOT NULL,
    conditions JSONB NOT NULL DEFAULT '{}',
    actions JSONB NOT NULL DEFAULT '[]',
    priority INTEGER NOT NULL DEFAULT 0,
    enabled BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS rule_execution_history (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    rule_id UUID NOT NULL REFERENCES automation_rules_v2(id) ON DELETE CASCADE,
    status TEXT NOT NULL,
    matched_conditions JSONB NOT NULL DEFAULT '[]',
    failed_conditions JSONB NOT NULL DEFAULT '[]',
    actions_executed JSONB NOT NULL DEFAULT '[]',
    error TEXT,
    executed_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
