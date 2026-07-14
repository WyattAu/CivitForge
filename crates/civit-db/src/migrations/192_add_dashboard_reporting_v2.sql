CREATE TABLE IF NOT EXISTS dashboard_widgets_v2 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    dashboard_id UUID NOT NULL REFERENCES dashboards(id) ON DELETE CASCADE,
    widget_type TEXT NOT NULL,
    config JSONB NOT NULL DEFAULT '{}',
    position JSONB NOT NULL DEFAULT '{}',
    size JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS report_schedules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    report_id UUID NOT NULL REFERENCES reports(id) ON DELETE CASCADE,
    cron_expression TEXT NOT NULL,
    enabled BOOLEAN NOT NULL DEFAULT true,
    last_run_at TIMESTAMPTZ,
    next_run_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_dashboard_widgets_v2_dashboard ON dashboard_widgets_v2(dashboard_id);
CREATE INDEX IF NOT EXISTS idx_dashboard_widgets_v2_type ON dashboard_widgets_v2(widget_type);
CREATE INDEX IF NOT EXISTS idx_report_schedules_report ON report_schedules(report_id);
CREATE INDEX IF NOT EXISTS idx_report_schedules_next_run ON report_schedules(next_run_at);
