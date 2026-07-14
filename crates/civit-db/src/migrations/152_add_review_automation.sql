-- Migration 152: Code Review Automation
-- Adds review automation rules for auto-assign, auto-label, and auto-comment.

CREATE TABLE IF NOT EXISTS review_automation_rules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    repo_id UUID NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    trigger_type TEXT NOT NULL,
    conditions JSONB NOT NULL DEFAULT '{}',
    actions JSONB NOT NULL DEFAULT '[]',
    enabled BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_review_automation_rules_repo ON review_automation_rules(repo_id);
CREATE INDEX IF NOT EXISTS idx_review_automation_rules_trigger ON review_automation_rules(trigger_type);
