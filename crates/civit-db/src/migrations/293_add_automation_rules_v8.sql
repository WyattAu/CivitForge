-- CivitForge Phase 293: Automation Rules V8
-- Migration 293
-- Adds automation rules v8 with execution time tracking, performance analytics, rule optimization, and recommendations.

CREATE TABLE IF NOT EXISTS automation_rules_v8 (
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

CREATE INDEX idx_automation_rules_v8_repo_id ON automation_rules_v8(repo_id);
CREATE INDEX idx_automation_rules_v8_trigger_type ON automation_rules_v8(trigger_type);
CREATE INDEX idx_automation_rules_v8_enabled ON automation_rules_v8(enabled);
CREATE INDEX idx_automation_rules_v8_priority ON automation_rules_v8(priority);
CREATE INDEX idx_automation_rules_v8_last_run_at ON automation_rules_v8(last_run_at);
CREATE INDEX idx_automation_rules_v8_run_count ON automation_rules_v8(run_count);
CREATE INDEX idx_automation_rules_v8_success_rate ON automation_rules_v8(success_rate);
CREATE INDEX idx_automation_rules_v8_avg_exec_time ON automation_rules_v8(avg_execution_time_ms);
CREATE INDEX idx_automation_rules_v8_repo_priority ON automation_rules_v8(repo_id, priority);
CREATE INDEX idx_automation_rules_v8_repo_enabled ON automation_rules_v8(repo_id, enabled);