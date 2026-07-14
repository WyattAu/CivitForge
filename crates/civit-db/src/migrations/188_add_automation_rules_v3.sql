-- CivitForge Phase 188: Automation Rules V3
-- Migration 188
-- Enhances automation rules with run count tracking, performance analytics, and rule optimization.

CREATE TABLE IF NOT EXISTS automation_rules_v3 (
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
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_automation_rules_v3_repo_id ON automation_rules_v3(repo_id);
CREATE INDEX idx_automation_rules_v3_trigger_type ON automation_rules_v3(trigger_type);
CREATE INDEX idx_automation_rules_v3_enabled ON automation_rules_v3(enabled);
CREATE INDEX idx_automation_rules_v3_priority ON automation_rules_v3(priority);
CREATE INDEX idx_automation_rules_v3_last_run_at ON automation_rules_v3(last_run_at);
CREATE INDEX idx_automation_rules_v3_run_count ON automation_rules_v3(run_count);
CREATE INDEX idx_automation_rules_v3_repo_priority ON automation_rules_v3(repo_id, priority);
CREATE INDEX idx_automation_rules_v3_repo_enabled ON automation_rules_v3(repo_id, enabled);
