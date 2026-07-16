-- Performance Testing v22: alerts v19, alert history v19
CREATE TABLE IF NOT EXISTS performance_test_alerts_v19 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    baseline_id UUID NOT NULL REFERENCES performance_baselines(id),
    alert_type TEXT NOT NULL,
    threshold DOUBLE PRECISION NOT NULL,
    enabled BOOLEAN NOT NULL DEFAULT true,
    last_triggered_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_performance_test_alerts_v19_baseline ON performance_test_alerts_v19(baseline_id);
CREATE INDEX IF NOT EXISTS idx_performance_test_alerts_v19_enabled ON performance_test_alerts_v19(enabled);

CREATE TABLE IF NOT EXISTS performance_test_alert_history_v19 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    alert_id UUID NOT NULL REFERENCES performance_test_alerts_v19(id),
    metric_name TEXT NOT NULL,
    metric_value DOUBLE PRECISION NOT NULL,
    threshold DOUBLE PRECISION NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_performance_test_alert_history_v19_alert ON performance_test_alert_history_v19(alert_id);
CREATE INDEX IF NOT EXISTS idx_performance_test_alert_history_v19_created ON performance_test_alert_history_v19(created_at DESC);
