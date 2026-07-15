-- CivitForge Phase 440: Automation Rules V15
-- Migration 440
-- Adds automation_rules_v15 with execution time tracking, performance analytics, rule optimization, and rule recommendations.

CREATE TABLE IF NOT EXISTS automation_rules_v15 (
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

CREATE INDEX idx_automation_rules_v15_repo ON automation_rules_v15(repo_id);
CREATE INDEX idx_automation_rules_v15_trigger ON automation_rules_v15(trigger_type);
CREATE INDEX idx_automation_rules_v15_priority ON automation_rules_v15(priority DESC);
CREATE INDEX idx_automation_rules_v15_enabled ON automation_rules_v15(enabled);
CREATE INDEX idx_automation_rules_v15_run_count ON automation_rules_v15(run_count DESC);
CREATE INDEX idx_automation_rules_v15_success_rate ON automation_rules_v15(success_rate);
