-- Migration 462: Add scheduled_task_templates_v13

CREATE TABLE IF NOT EXISTS scheduled_task_templates_v13 (
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

CREATE INDEX idx_scheduled_task_templates_v13_public ON scheduled_task_templates_v13(is_public);
CREATE INDEX idx_scheduled_task_templates_v13_type ON scheduled_task_templates_v13(task_type);
CREATE INDEX idx_scheduled_task_templates_v13_usage ON scheduled_task_templates_v13(usage_count DESC);
