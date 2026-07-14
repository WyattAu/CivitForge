-- CivitForge Phase 335: Automation Rules V10
-- Migration 335
-- Adds automation rules v10 with execution time tracking, performance analytics,
-- rule optimization, and rule recommendations.

CREATE TABLE IF NOT EXISTS automation_rules_v10 (
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

CREATE INDEX idx_automation_rules_v10_repo_id ON automation_rules_v10(repo_id);
CREATE INDEX idx_automation_rules_v10_trigger_type ON automation_rules_v10(trigger_type);
CREATE INDEX idx_automation_rules_v10_enabled ON automation_rules_v10(enabled);
CREATE INDEX idx_automation_rules_v10_priority ON automation_rules_v10(priority);
CREATE INDEX idx_automation_rules_v10_run_count ON automation_rules_v10(run_count);
CREATE INDEX idx_automation_rules_v10_success_rate ON automation_rules_v10(success_rate);
CREATE INDEX idx_automation_rules_v10_avg_execution_time_ms ON automation_rules_v10(avg_execution_time_ms);
CREATE INDEX idx_automation_rules_v10_last_run_at ON automation_rules_v10(last_run_at);
CREATE INDEX idx_automation_rules_v10_created_at ON automation_rules_v10(created_at);
