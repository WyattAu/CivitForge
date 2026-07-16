-- Test Suite Management v21: metrics v18, baselines v18
CREATE TABLE IF NOT EXISTS test_suite_metrics_v18 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    suite_id UUID NOT NULL REFERENCES test_suites(id) ON DELETE CASCADE,
    metric_name TEXT NOT NULL,
    metric_value DOUBLE PRECISION NOT NULL,
    measured_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_test_suite_metrics_v18_suite ON test_suite_metrics_v18(suite_id);
CREATE INDEX IF NOT EXISTS idx_test_suite_metrics_v18_name ON test_suite_metrics_v18(metric_name);
CREATE INDEX IF NOT EXISTS idx_test_suite_metrics_v18_suite_name ON test_suite_metrics_v18(suite_id, metric_name);
CREATE INDEX IF NOT EXISTS idx_test_suite_metrics_v18_measured ON test_suite_metrics_v18(measured_at DESC);

CREATE TABLE IF NOT EXISTS test_suite_baselines_v18 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    suite_id UUID NOT NULL REFERENCES test_suites(id),
    metric_name TEXT NOT NULL,
    baseline_value DOUBLE PRECISION NOT NULL,
    threshold_percent DOUBLE PRECISION NOT NULL DEFAULT 10.0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(suite_id, metric_name)
);

CREATE INDEX IF NOT EXISTS idx_test_suite_baselines_v18_suite ON test_suite_baselines_v18(suite_id);
