CREATE TABLE IF NOT EXISTS test_suite_metrics_v6 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    suite_id UUID NOT NULL REFERENCES test_suites(id) ON DELETE CASCADE,
    metric_name TEXT NOT NULL,
    metric_value DOUBLE PRECISION NOT NULL,
    measured_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS test_suite_baselines_v6 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    suite_id UUID NOT NULL REFERENCES test_suites(id),
    metric_name TEXT NOT NULL,
    baseline_value DOUBLE PRECISION NOT NULL,
    threshold_percent DOUBLE PRECISION NOT NULL DEFAULT 10.0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(suite_id, metric_name)
);

CREATE INDEX IF NOT EXISTS idx_test_suite_metrics_v6_suite ON test_suite_metrics_v6(suite_id);
CREATE INDEX IF NOT EXISTS idx_test_suite_metrics_v6_metric_name ON test_suite_metrics_v6(metric_name);
CREATE INDEX IF NOT EXISTS idx_test_suite_metrics_v6_measured_at ON test_suite_metrics_v6(measured_at DESC);
CREATE INDEX IF NOT EXISTS idx_test_suite_baselines_v6_suite ON test_suite_baselines_v6(suite_id);
