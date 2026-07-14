-- CivitForge Phase 294: Scheduled Task Templates V5
-- Migration 294
-- Adds scheduled task templates v5 with ratings, analytics, recommendations, and marketplace.

CREATE TABLE IF NOT EXISTS scheduled_task_templates_v5 (
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

CREATE INDEX idx_scheduled_task_templates_v5_task_type ON scheduled_task_templates_v5(task_type);
CREATE INDEX idx_scheduled_task_templates_v5_is_public ON scheduled_task_templates_v5(is_public);
CREATE INDEX idx_scheduled_task_templates_v5_author_id ON scheduled_task_templates_v5(author_id);
CREATE INDEX idx_scheduled_task_templates_v5_usage_count ON scheduled_task_templates_v5(usage_count);
CREATE INDEX idx_scheduled_task_templates_v5_rating ON scheduled_task_templates_v5(rating);
CREATE INDEX idx_scheduled_task_templates_v5_created_at ON scheduled_task_templates_v5(created_at);