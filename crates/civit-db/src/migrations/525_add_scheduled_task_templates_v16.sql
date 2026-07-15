CREATE TABLE IF NOT EXISTS scheduled_task_templates_v16 (
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

CREATE INDEX IF NOT EXISTS idx_scheduled_task_templates_v16_name ON scheduled_task_templates_v16(name);
CREATE INDEX IF NOT EXISTS idx_scheduled_task_templates_v16_task_type ON scheduled_task_templates_v16(task_type);
CREATE INDEX IF NOT EXISTS idx_scheduled_task_templates_v16_is_public ON scheduled_task_templates_v16(is_public);
CREATE INDEX IF NOT EXISTS idx_scheduled_task_templates_v16_author_id ON scheduled_task_templates_v16(author_id);
CREATE INDEX IF NOT EXISTS idx_scheduled_task_templates_v16_usage_count ON scheduled_task_templates_v16(usage_count);
CREATE INDEX IF NOT EXISTS idx_scheduled_task_templates_v16_rating ON scheduled_task_templates_v16(rating);
CREATE INDEX IF NOT EXISTS idx_scheduled_task_templates_v16_created_at ON scheduled_task_templates_v16(created_at);
