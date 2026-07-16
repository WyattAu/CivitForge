-- Performance Testing v23: comparisons v20, regressions v20
CREATE TABLE IF NOT EXISTS performance_test_comparisons_v20 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    baseline_id UUID NOT NULL REFERENCES performance_baselines(id),
    comparison_id UUID NOT NULL REFERENCES performance_baselines(id),
    metric_name TEXT NOT NULL,
    baseline_value DOUBLE PRECISION NOT NULL,
    comparison_value DOUBLE PRECISION NOT NULL,
    percent_change DOUBLE PRECISION NOT NULL,
    is_regression BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_performance_test_comparisons_v20_baseline ON performance_test_comparisons_v20(baseline_id);
CREATE INDEX IF NOT EXISTS idx_performance_test_comparisons_v20_comparison ON performance_test_comparisons_v20(comparison_id);
CREATE INDEX IF NOT EXISTS idx_performance_test_comparisons_v20_metric ON performance_test_comparisons_v20(metric_name);
CREATE INDEX IF NOT EXISTS idx_performance_test_comparisons_v20_regression ON performance_test_comparisons_v20(is_regression);
CREATE INDEX IF NOT EXISTS idx_performance_test_comparisons_v20_created ON performance_test_comparisons_v20(created_at DESC);

CREATE TABLE IF NOT EXISTS performance_test_regressions_v20 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    baseline_id UUID NOT NULL REFERENCES performance_baselines(id),
    metric_name TEXT NOT NULL,
    threshold_percent DOUBLE PRECISION NOT NULL DEFAULT 10,
    enabled BOOLEAN NOT NULL DEFAULT true,
    last_detected_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_performance_test_regressions_v20_baseline ON performance_test_regressions_v20(baseline_id);
CREATE INDEX IF NOT EXISTS idx_performance_test_regressions_v20_metric ON performance_test_regressions_v20(metric_name);
CREATE INDEX IF NOT EXISTS idx_performance_test_regressions_v20_enabled ON performance_test_regressions_v20(enabled);
