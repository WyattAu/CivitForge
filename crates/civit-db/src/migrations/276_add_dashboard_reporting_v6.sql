CREATE TABLE IF NOT EXISTS dashboard_shares_v3 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    dashboard_id UUID NOT NULL REFERENCES dashboards(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id),
    permission TEXT NOT NULL DEFAULT 'view',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(dashboard_id, user_id)
);

CREATE TABLE IF NOT EXISTS report_schedules_v4 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    report_id UUID NOT NULL REFERENCES reports(id) ON DELETE CASCADE,
    cron_expression TEXT NOT NULL,
    enabled BOOLEAN NOT NULL DEFAULT true,
    last_run_at TIMESTAMPTZ,
    next_run_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_dashboard_shares_v3_dashboard ON dashboard_shares_v3(dashboard_id);
CREATE INDEX IF NOT EXISTS idx_dashboard_shares_v3_user ON dashboard_shares_v3(user_id);
CREATE INDEX IF NOT EXISTS idx_report_schedules_v4_report ON report_schedules_v4(report_id);
CREATE INDEX IF NOT EXISTS idx_report_schedules_v4_next_run ON report_schedules_v4(next_run_at);
