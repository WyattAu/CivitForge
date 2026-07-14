-- CivitForge Phase 189: Scheduled Task Templates
-- Migration 189
-- Adds scheduled task templates and usage tracking for template marketplace.

CREATE TABLE IF NOT EXISTS scheduled_task_templates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    task_type TEXT NOT NULL,
    config JSONB NOT NULL DEFAULT '{}',
    is_public BOOLEAN NOT NULL DEFAULT false,
    author_id UUID REFERENCES users(id),
    usage_count INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_scheduled_task_templates_task_type ON scheduled_task_templates(task_type);
CREATE INDEX idx_scheduled_task_templates_is_public ON scheduled_task_templates(is_public);
CREATE INDEX idx_scheduled_task_templates_author_id ON scheduled_task_templates(author_id);
CREATE INDEX idx_scheduled_task_templates_usage_count ON scheduled_task_templates(usage_count);
CREATE INDEX idx_scheduled_task_templates_created_at ON scheduled_task_templates(created_at);
