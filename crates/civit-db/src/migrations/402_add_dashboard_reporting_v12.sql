CREATE TABLE IF NOT EXISTS dashboard_shares_v9 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    dashboard_id UUID NOT NULL REFERENCES dashboards(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id),
    permission TEXT NOT NULL DEFAULT 'view',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(dashboard_id, user_id)
);

CREATE TABLE IF NOT EXISTS report_schedules_v10 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    report_id UUID NOT NULL REFERENCES reports(id) ON DELETE CASCADE,
    cron_expression TEXT NOT NULL,
    enabled BOOLEAN NOT NULL DEFAULT true,
    last_run_at TIMESTAMPTZ,
    next_run_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
