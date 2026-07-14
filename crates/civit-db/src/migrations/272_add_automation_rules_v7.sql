-- CivitForge Phase 272: Automation Rules V7
-- Migration 272
-- Adds automation rules v7 with execution time tracking, performance analytics, rule optimization, and recommendations.

CREATE TABLE IF NOT EXISTS automation_rules_v7 (
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

CREATE INDEX idx_automation_rules_v7_repo_id ON automation_rules_v7(repo_id);
CREATE INDEX idx_automation_rules_v7_trigger_type ON automation_rules_v7(trigger_type);
CREATE INDEX idx_automation_rules_v7_enabled ON automation_rules_v7(enabled);
CREATE INDEX idx_automation_rules_v7_priority ON automation_rules_v7(priority);
CREATE INDEX idx_automation_rules_v7_last_run_at ON automation_rules_v7(last_run_at);
CREATE INDEX idx_automation_rules_v7_run_count ON automation_rules_v7(run_count);
CREATE INDEX idx_automation_rules_v7_success_rate ON automation_rules_v7(success_rate);
CREATE INDEX idx_automation_rules_v7_avg_exec_time ON automation_rules_v7(avg_execution_time_ms);
CREATE INDEX idx_automation_rules_v7_repo_priority ON automation_rules_v7(repo_id, priority);
CREATE INDEX idx_automation_rules_v7_repo_enabled ON automation_rules_v7(repo_id, enabled);
