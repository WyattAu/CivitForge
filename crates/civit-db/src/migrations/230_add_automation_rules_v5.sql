-- CivitForge Phase 230: Automation Rules V5 - Execution Time Tracking & Performance Analytics
-- Migration 230
-- Adds execution time tracking, performance analytics, rule optimization, and rule recommendations.

CREATE TABLE IF NOT EXISTS automation_rules_v5 (
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

CREATE INDEX idx_automation_rules_v5_repo_id ON automation_rules_v5(repo_id);
CREATE INDEX idx_automation_rules_v5_trigger_type ON automation_rules_v5(trigger_type);
CREATE INDEX idx_automation_rules_v5_priority ON automation_rules_v5(priority DESC);
CREATE INDEX idx_automation_rules_v5_enabled ON automation_rules_v5(enabled);
CREATE INDEX idx_automation_rules_v5_success_rate ON automation_rules_v5(success_rate);
CREATE INDEX idx_automation_rules_v5_avg_execution_time ON automation_rules_v5(avg_execution_time_ms);
CREATE INDEX idx_automation_rules_v5_repo_priority ON automation_rules_v5(repo_id, priority DESC);
CREATE INDEX idx_automation_rules_v5_repo_success ON automation_rules_v5(repo_id, success_rate DESC);
