CREATE TABLE IF NOT EXISTS performance_test_alerts_v4 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    baseline_id UUID NOT NULL REFERENCES performance_baselines(id),
    alert_type TEXT NOT NULL,
    threshold DOUBLE PRECISION NOT NULL,
    enabled BOOLEAN NOT NULL DEFAULT true,
    last_triggered_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS performance_test_alert_history_v4 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    alert_id UUID NOT NULL REFERENCES performance_test_alerts_v4(id),
    metric_name TEXT NOT NULL,
    metric_value DOUBLE PRECISION NOT NULL,
    threshold DOUBLE PRECISION NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_perf_alerts_v4_baseline_id ON performance_test_alerts_v4(baseline_id);
CREATE INDEX IF NOT EXISTS idx_perf_alerts_v4_type ON performance_test_alerts_v4(alert_type);
CREATE INDEX IF NOT EXISTS idx_perf_alerts_v4_enabled ON performance_test_alerts_v4(enabled);
CREATE INDEX IF NOT EXISTS idx_perf_alert_history_v4_alert_id ON performance_test_alert_history_v4(alert_id);
CREATE INDEX IF NOT EXISTS idx_perf_alert_history_v4_metric ON performance_test_alert_history_v4(metric_name);
CREATE INDEX IF NOT EXISTS idx_perf_alert_history_v4_created_at ON performance_test_alert_history_v4(created_at DESC);
