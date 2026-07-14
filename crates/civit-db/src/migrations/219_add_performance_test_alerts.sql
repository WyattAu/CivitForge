-- Migration 219: Performance Test Alerts and Alert History
-- Adds alert configuration, alert history, notifications, and analytics for performance tests.

CREATE TABLE IF NOT EXISTS performance_test_alerts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    baseline_id UUID NOT NULL REFERENCES performance_baselines(id),
    alert_type TEXT NOT NULL,
    threshold DOUBLE PRECISION NOT NULL,
    enabled BOOLEAN NOT NULL DEFAULT true,
    last_triggered_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS performance_test_alert_history (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    alert_id UUID NOT NULL REFERENCES performance_test_alerts(id),
    metric_name TEXT NOT NULL,
    metric_value DOUBLE PRECISION NOT NULL,
    threshold DOUBLE PRECISION NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_perf_alerts_baseline_id ON performance_test_alerts(baseline_id);
CREATE INDEX IF NOT EXISTS idx_perf_alerts_type ON performance_test_alerts(alert_type);
CREATE INDEX IF NOT EXISTS idx_perf_alerts_enabled ON performance_test_alerts(enabled);
CREATE INDEX IF NOT EXISTS idx_perf_alert_history_alert_id ON performance_test_alert_history(alert_id);
CREATE INDEX IF NOT EXISTS idx_perf_alert_history_metric ON performance_test_alert_history(metric_name);
CREATE INDEX IF NOT EXISTS idx_perf_alert_history_created_at ON performance_test_alert_history(created_at);
