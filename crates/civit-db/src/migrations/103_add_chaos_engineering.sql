CREATE TABLE IF NOT EXISTS chaos_experiments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    experiment_type TEXT NOT NULL,
    target TEXT NOT NULL,
    parameters JSONB NOT NULL DEFAULT '{}',
    status TEXT NOT NULL DEFAULT 'pending',
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS chaos_results (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    experiment_id UUID NOT NULL REFERENCES chaos_experiments(id) ON DELETE CASCADE,
    metric_name TEXT NOT NULL,
    metric_value DOUBLE PRECISION NOT NULL,
    baseline_value DOUBLE PRECISION,
    impact TEXT NOT NULL DEFAULT 'none',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_chaos_experiments_status ON chaos_experiments(status);
CREATE INDEX idx_chaos_experiments_type ON chaos_experiments(experiment_type);
CREATE INDEX idx_chaos_results_experiment_id ON chaos_results(experiment_id);
