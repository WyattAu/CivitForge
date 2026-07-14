CREATE TABLE IF NOT EXISTS performance_test_configs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    test_id UUID NOT NULL REFERENCES performance_tests(id) ON DELETE CASCADE,
    config_key TEXT NOT NULL,
    config_value JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS performance_test_results (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    test_id UUID NOT NULL REFERENCES performance_tests(id) ON DELETE CASCADE,
    metric_name TEXT NOT NULL,
    metric_value DOUBLE PRECISION NOT NULL,
    percentile DOUBLE PRECISION,
    recorded_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_performance_test_configs_test_id ON performance_test_configs(test_id);
CREATE INDEX idx_performance_test_configs_key ON performance_test_configs(config_key);
CREATE INDEX idx_performance_test_results_test_id ON performance_test_results(test_id);
CREATE INDEX idx_performance_test_results_metric ON performance_test_results(metric_name);
CREATE INDEX idx_performance_test_results_percentile ON performance_test_results(percentile);
