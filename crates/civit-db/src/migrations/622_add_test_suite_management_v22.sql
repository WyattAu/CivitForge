-- Test Suite Management v22: flaky detection v19, trends v19
CREATE TABLE IF NOT EXISTS test_suite_flaky_detection_v19 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    test_name TEXT NOT NULL,
    suite_id UUID NOT NULL REFERENCES test_suites(id) ON DELETE CASCADE,
    flaky_score DOUBLE PRECISION NOT NULL DEFAULT 0,
    total_runs INTEGER NOT NULL DEFAULT 0,
    failure_count INTEGER NOT NULL DEFAULT 0,
    last_flaky_at TIMESTAMPTZ,
    detected_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_test_suite_flaky_detection_v19_suite ON test_suite_flaky_detection_v19(suite_id);
CREATE INDEX IF NOT EXISTS idx_test_suite_flaky_detection_v19_name ON test_suite_flaky_detection_v19(test_name);
CREATE INDEX IF NOT EXISTS idx_test_suite_flaky_detection_v19_score ON test_suite_flaky_detection_v19(flaky_score DESC);
CREATE INDEX IF NOT EXISTS idx_test_suite_flaky_detection_v19_detected ON test_suite_flaky_detection_v19(detected_at DESC);

CREATE TABLE IF NOT EXISTS test_suite_trends_v19 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    suite_id UUID NOT NULL REFERENCES test_suites(id) ON DELETE CASCADE,
    metric_name TEXT NOT NULL,
    metric_value DOUBLE PRECISION NOT NULL,
    period_start TIMESTAMPTZ NOT NULL,
    period_end TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_test_suite_trends_v19_suite ON test_suite_trends_v19(suite_id);
CREATE INDEX IF NOT EXISTS idx_test_suite_trends_v19_name ON test_suite_trends_v19(metric_name);
CREATE INDEX IF NOT EXISTS idx_test_suite_trends_v19_suite_name ON test_suite_trends_v19(suite_id, metric_name);
CREATE INDEX IF NOT EXISTS idx_test_suite_trends_v19_period ON test_suite_trends_v19(period_start DESC);
