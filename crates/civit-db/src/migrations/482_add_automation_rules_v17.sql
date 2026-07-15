-- Migration 482: Add automation_rules_v17

CREATE TABLE IF NOT EXISTS automation_rules_v17 (
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

CREATE INDEX idx_automation_rules_v17_repo ON automation_rules_v17(repo_id);
CREATE INDEX idx_automation_rules_v17_trigger ON automation_rules_v17(trigger_type);
CREATE INDEX idx_automation_rules_v17_priority ON automation_rules_v17(priority DESC);
CREATE INDEX idx_automation_rules_v17_enabled ON automation_rules_v17(enabled);
