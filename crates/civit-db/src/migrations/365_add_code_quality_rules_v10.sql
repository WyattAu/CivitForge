CREATE TABLE IF NOT EXISTS code_quality_metrics_v8 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    repo_id UUID NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    file_path TEXT NOT NULL,
    metric_name TEXT NOT NULL,
    metric_value DOUBLE PRECISION NOT NULL,
    measured_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS code_quality_thresholds_v7 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    repo_id UUID NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    metric_name TEXT NOT NULL,
    threshold_value DOUBLE PRECISION NOT NULL,
    enabled BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(repo_id, metric_name)
);

CREATE INDEX IF NOT EXISTS idx_code_quality_metrics_v8_repo ON code_quality_metrics_v8(repo_id);
CREATE INDEX IF NOT EXISTS idx_code_quality_metrics_v8_file_path ON code_quality_metrics_v8(file_path);
CREATE INDEX IF NOT EXISTS idx_code_quality_metrics_v8_metric_name ON code_quality_metrics_v8(metric_name);
CREATE INDEX IF NOT EXISTS idx_code_quality_metrics_v8_measured_at ON code_quality_metrics_v8(measured_at DESC);
CREATE INDEX IF NOT EXISTS idx_code_quality_thresholds_v7_repo ON code_quality_thresholds_v7(repo_id);
