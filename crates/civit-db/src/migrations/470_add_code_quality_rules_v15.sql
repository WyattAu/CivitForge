CREATE TABLE IF NOT EXISTS code_quality_metrics_v13 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    repo_id UUID NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    file_path TEXT NOT NULL,
    metric_name TEXT NOT NULL,
    metric_value DOUBLE PRECISION NOT NULL,
    measured_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_code_quality_metrics_v13_repo_id ON code_quality_metrics_v13(repo_id);
CREATE INDEX IF NOT EXISTS idx_code_quality_metrics_v13_metric_name ON code_quality_metrics_v13(metric_name);
CREATE INDEX IF NOT EXISTS idx_code_quality_metrics_v13_measured_at ON code_quality_metrics_v13(measured_at DESC);

CREATE TABLE IF NOT EXISTS code_quality_thresholds_v12 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    repo_id UUID NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    metric_name TEXT NOT NULL,
    threshold_value DOUBLE PRECISION NOT NULL,
    enabled BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(repo_id, metric_name)
);

CREATE INDEX IF NOT EXISTS idx_code_quality_thresholds_v12_repo_id ON code_quality_thresholds_v12(repo_id);
