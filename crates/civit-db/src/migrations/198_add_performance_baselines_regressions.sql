-- Migration 198: Performance Baselines and Regression Detection
-- Adds baseline management, regression detection, and trend analysis for performance tests.

CREATE TABLE IF NOT EXISTS performance_baselines (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    repo_id UUID NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    metric_name TEXT NOT NULL,
    baseline_value DOUBLE PRECISION NOT NULL,
    threshold_percent DOUBLE PRECISION NOT NULL DEFAULT 10.0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS performance_regressions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    baseline_id UUID NOT NULL REFERENCES performance_baselines(id),
    test_id UUID NOT NULL REFERENCES performance_tests(id),
    regression_percent DOUBLE PRECISION NOT NULL,
    status TEXT NOT NULL DEFAULT 'open',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS performance_trend_data (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    repo_id UUID NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    metric_name TEXT NOT NULL,
    metric_value DOUBLE PRECISION NOT NULL,
    recorded_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_performance_baselines_repo_id ON performance_baselines(repo_id);
CREATE INDEX IF NOT EXISTS idx_performance_baselines_metric ON performance_baselines(metric_name);
CREATE INDEX IF NOT EXISTS idx_performance_regressions_baseline_id ON performance_regressions(baseline_id);
CREATE INDEX IF NOT EXISTS idx_performance_regressions_test_id ON performance_regressions(test_id);
CREATE INDEX IF NOT EXISTS idx_performance_regressions_status ON performance_regressions(status);
CREATE INDEX IF NOT EXISTS idx_performance_trend_data_repo_id ON performance_trend_data(repo_id);
CREATE INDEX IF NOT EXISTS idx_performance_trend_data_metric ON performance_trend_data(metric_name);
CREATE INDEX IF NOT EXISTS idx_performance_trend_data_recorded_at ON performance_trend_data(recorded_at);
