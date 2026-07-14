-- CivitForge Phase 231: Scheduled Tasks V5 - Template Ratings & Analytics
-- Migration 231
-- Adds scheduled task template v2 with ratings, analytics, recommendations, and marketplace.

CREATE TABLE IF NOT EXISTS scheduled_task_templates_v2 (
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

CREATE INDEX idx_scheduled_task_templates_v2_name ON scheduled_task_templates_v2(name);
CREATE INDEX idx_scheduled_task_templates_v2_task_type ON scheduled_task_templates_v2(task_type);
CREATE INDEX idx_scheduled_task_templates_v2_is_public ON scheduled_task_templates_v2(is_public);
CREATE INDEX idx_scheduled_task_templates_v2_author_id ON scheduled_task_templates_v2(author_id);
CREATE INDEX idx_scheduled_task_templates_v2_usage_count ON scheduled_task_templates_v2(usage_count DESC);
CREATE INDEX idx_scheduled_task_templates_v2_rating ON scheduled_task_templates_v2(rating DESC);
CREATE INDEX idx_scheduled_task_templates_v2_public_rating ON scheduled_task_templates_v2(is_public, rating DESC);
CREATE INDEX idx_scheduled_task_templates_v2_public_usage ON scheduled_task_templates_v2(is_public, usage_count DESC);
