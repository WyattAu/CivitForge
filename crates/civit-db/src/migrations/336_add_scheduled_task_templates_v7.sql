-- CivitForge Phase 336: Scheduled Task Templates V7
-- Migration 336
-- Adds scheduled task templates v7 with template ratings, analytics,
-- recommendations, and marketplace.

CREATE TABLE IF NOT EXISTS scheduled_task_templates_v7 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    task_type TEXT NOT NULL,
    config JSONB NOT NULL DEFAULT '{}',
    is_public BOOLEAN NOT NULL DEFAULT false,
    author_id UUID REFERENCES users(id),
    usage_count INTEGER NOT NULL DEFAULT 0,
    rating DOUBLE PRECISION NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_scheduled_task_templates_v7_task_type ON scheduled_task_templates_v7(task_type);
CREATE INDEX idx_scheduled_task_templates_v7_is_public ON scheduled_task_templates_v7(is_public);
CREATE INDEX idx_scheduled_task_templates_v7_author_id ON scheduled_task_templates_v7(author_id);
CREATE INDEX idx_scheduled_task_templates_v7_usage_count ON scheduled_task_templates_v7(usage_count);
CREATE INDEX idx_scheduled_task_templates_v7_rating ON scheduled_task_templates_v7(rating);
CREATE INDEX idx_scheduled_task_templates_v7_created_at ON scheduled_task_templates_v7(created_at);
