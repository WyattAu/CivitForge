-- Dashboard and Reporting v20: Enhanced dashboard sharing v17 and report scheduling v18

CREATE TABLE IF NOT EXISTS dashboard_shares_v17 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    dashboard_id UUID NOT NULL REFERENCES dashboards(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id),
    permission TEXT NOT NULL DEFAULT 'view',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(dashboard_id, user_id)
);

CREATE TABLE IF NOT EXISTS report_schedules_v18 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    report_id UUID NOT NULL REFERENCES reports(id) ON DELETE CASCADE,
    cron_expression TEXT NOT NULL,
    enabled BOOLEAN NOT NULL DEFAULT true,
    last_run_at TIMESTAMPTZ,
    next_run_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_dashboard_shares_v17_dashboard ON dashboard_shares_v17(dashboard_id);
CREATE INDEX IF NOT EXISTS idx_dashboard_shares_v17_user ON dashboard_shares_v17(user_id);
CREATE INDEX IF NOT EXISTS idx_report_schedules_v18_report ON report_schedules_v18(report_id);
CREATE INDEX IF NOT EXISTS idx_report_schedules_v18_next_run ON report_schedules_v18(next_run_at);
CREATE INDEX IF NOT EXISTS idx_report_schedules_v18_enabled ON report_schedules_v18(enabled);
