CREATE TABLE IF NOT EXISTS automation_rules_v19 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    repo_id UUID NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    trigger_type TEXT NOT NULL,
    conditions JSONB NOT NULL DEFAULT '{}',
    actions JSONB NOT NULL DEFAULT '[]',
    priority INTEGER NOT NULL DEFAULT 0,
    enabled BOOLEAN NOT NULL DEFAULT true,
    last_run_at TIMESTAMPTZ,
    run_count INTEGER NOT NULL DEFAULT 0,
    success_rate DOUBLE PRECISION NOT NULL DEFAULT 100.0,
    avg_execution_time_ms INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_automation_rules_v19_repo_id ON automation_rules_v19(repo_id);
CREATE INDEX IF NOT EXISTS idx_automation_rules_v19_trigger_type ON automation_rules_v19(trigger_type);
CREATE INDEX IF NOT EXISTS idx_automation_rules_v19_priority ON automation_rules_v19(priority);
CREATE INDEX IF NOT EXISTS idx_automation_rules_v19_enabled ON automation_rules_v19(enabled);
CREATE INDEX IF NOT EXISTS idx_automation_rules_v19_last_run_at ON automation_rules_v19(last_run_at);
CREATE INDEX IF NOT EXISTS idx_automation_rules_v19_run_count ON automation_rules_v19(run_count);
CREATE INDEX IF NOT EXISTS idx_automation_rules_v19_success_rate ON automation_rules_v19(success_rate);
CREATE INDEX IF NOT EXISTS idx_automation_rules_v19_avg_execution_time_ms ON automation_rules_v19(avg_execution_time_ms);
CREATE INDEX IF NOT EXISTS idx_automation_rules_v19_created_at ON automation_rules_v19(created_at);
