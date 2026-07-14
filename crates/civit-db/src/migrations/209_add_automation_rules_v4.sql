-- CivitForge Phase 209: Automation Rules V4
-- Migration 209
-- Adds success rate tracking, performance analytics, rule optimization, and rule recommendations.

CREATE TABLE IF NOT EXISTS automation_rules_v4 (
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
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_automation_rules_v4_repo_id ON automation_rules_v4(repo_id);
CREATE INDEX idx_automation_rules_v4_trigger_type ON automation_rules_v4(trigger_type);
CREATE INDEX idx_automation_rules_v4_priority ON automation_rules_v4(priority DESC);
CREATE INDEX idx_automation_rules_v4_enabled ON automation_rules_v4(enabled);
CREATE INDEX idx_automation_rules_v4_success_rate ON automation_rules_v4(success_rate);
CREATE INDEX idx_automation_rules_v4_repo_priority ON automation_rules_v4(repo_id, priority DESC);
