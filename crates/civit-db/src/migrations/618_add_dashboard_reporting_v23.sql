CREATE TABLE IF NOT EXISTS dashboard_widget_library_v20 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL,
    type TEXT NOT NULL,
    category TEXT NOT NULL DEFAULT 'general',
    config JSONB NOT NULL DEFAULT '{}',
    preview_url TEXT,
    usage_count INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS report_generation_queue_v20 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    report_id UUID NOT NULL REFERENCES reports(id) ON DELETE CASCADE,
    status TEXT NOT NULL DEFAULT 'pending',
    priority INTEGER NOT NULL DEFAULT 0,
    scheduled_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    error_message TEXT
);

CREATE INDEX IF NOT EXISTS idx_dashboard_widget_library_v20_category ON dashboard_widget_library_v20(category);
CREATE INDEX IF NOT EXISTS idx_dashboard_widget_library_v20_type ON dashboard_widget_library_v20(type);
CREATE INDEX IF NOT EXISTS idx_report_generation_queue_v20_status ON report_generation_queue_v20(status);
CREATE INDEX IF NOT EXISTS idx_report_generation_queue_v20_scheduled ON report_generation_queue_v20(scheduled_at);
CREATE INDEX IF NOT EXISTS idx_report_generation_queue_v20_priority ON report_generation_queue_v20(priority DESC);
