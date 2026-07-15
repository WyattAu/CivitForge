-- CivitForge Phase 441: Scheduled Tasks V15
-- Migration 441
-- Adds scheduled_task_templates_v12 with template ratings, analytics, recommendations, and marketplace.

CREATE TABLE IF NOT EXISTS scheduled_task_templates_v12 (
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

CREATE INDEX idx_scheduled_task_templates_v12_type ON scheduled_task_templates_v12(task_type);
CREATE INDEX idx_scheduled_task_templates_v12_public ON scheduled_task_templates_v12(is_public);
CREATE INDEX idx_scheduled_task_templates_v12_author ON scheduled_task_templates_v12(author_id);
CREATE INDEX idx_scheduled_task_templates_v12_rating ON scheduled_task_templates_v12(rating DESC);
CREATE INDEX idx_scheduled_task_templates_v12_usage ON scheduled_task_templates_v12(usage_count DESC);
