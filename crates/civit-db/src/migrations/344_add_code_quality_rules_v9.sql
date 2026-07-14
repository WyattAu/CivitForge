CREATE TABLE IF NOT EXISTS code_quality_metrics_v7 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    repo_id UUID NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    file_path TEXT NOT NULL,
    metric_name TEXT NOT NULL,
    metric_value DOUBLE PRECISION NOT NULL,
    measured_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS code_quality_thresholds_v6 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    repo_id UUID NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    metric_name TEXT NOT NULL,
    threshold_value DOUBLE PRECISION NOT NULL,
    enabled BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(repo_id, metric_name)
);

CREATE INDEX IF NOT EXISTS idx_code_quality_metrics_v7_repo ON code_quality_metrics_v7(repo_id);
CREATE INDEX IF NOT EXISTS idx_code_quality_metrics_v7_file_path ON code_quality_metrics_v7(file_path);
CREATE INDEX IF NOT EXISTS idx_code_quality_metrics_v7_metric_name ON code_quality_metrics_v7(metric_name);
CREATE INDEX IF NOT EXISTS idx_code_quality_metrics_v7_measured_at ON code_quality_metrics_v7(measured_at DESC);
CREATE INDEX IF NOT EXISTS idx_code_quality_thresholds_v6_repo ON code_quality_thresholds_v6(repo_id);
