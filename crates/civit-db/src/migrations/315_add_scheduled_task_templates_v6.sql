-- CivitForge Phase 315: Scheduled Task Templates V6
-- Migration 315
-- Adds scheduled task templates v6 with ratings, analytics, recommendations, and marketplace.

CREATE TABLE IF NOT EXISTS scheduled_task_templates_v6 (
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

CREATE INDEX idx_scheduled_task_templates_v6_task_type ON scheduled_task_templates_v6(task_type);
CREATE INDEX idx_scheduled_task_templates_v6_is_public ON scheduled_task_templates_v6(is_public);
CREATE INDEX idx_scheduled_task_templates_v6_author_id ON scheduled_task_templates_v6(author_id);
CREATE INDEX idx_scheduled_task_templates_v6_usage_count ON scheduled_task_templates_v6(usage_count);
CREATE INDEX idx_scheduled_task_templates_v6_rating ON scheduled_task_templates_v6(rating);
CREATE INDEX idx_scheduled_task_templates_v6_created_at ON scheduled_task_templates_v6(created_at);
