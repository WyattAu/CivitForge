-- CivitForge Phase 314: Automation Rules V9
-- Migration 314
-- Adds automation rules v9 with execution time tracking, performance analytics, rule optimization, and recommendations.

CREATE TABLE IF NOT EXISTS automation_rules_v9 (
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

CREATE INDEX idx_automation_rules_v9_repo_id ON automation_rules_v9(repo_id);
CREATE INDEX idx_automation_rules_v9_trigger_type ON automation_rules_v9(trigger_type);
CREATE INDEX idx_automation_rules_v9_enabled ON automation_rules_v9(enabled);
CREATE INDEX idx_automation_rules_v9_priority ON automation_rules_v9(priority);
CREATE INDEX idx_automation_rules_v9_last_run_at ON automation_rules_v9(last_run_at);
CREATE INDEX idx_automation_rules_v9_run_count ON automation_rules_v9(run_count);
CREATE INDEX idx_automation_rules_v9_success_rate ON automation_rules_v9(success_rate);
CREATE INDEX idx_automation_rules_v9_avg_exec_time ON automation_rules_v9(avg_execution_time_ms);
CREATE INDEX idx_automation_rules_v9_repo_priority ON automation_rules_v9(repo_id, priority);
CREATE INDEX idx_automation_rules_v9_repo_enabled ON automation_rules_v9(repo_id, enabled);
