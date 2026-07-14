CREATE TABLE IF NOT EXISTS code_quality_metrics_v2 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    repo_id UUID NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    file_path TEXT NOT NULL,
    metric_name TEXT NOT NULL,
    metric_value DOUBLE PRECISION NOT NULL,
    measured_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS code_quality_thresholds (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    repo_id UUID NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    metric_name TEXT NOT NULL,
    threshold_value DOUBLE PRECISION NOT NULL,
    enabled BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(repo_id, metric_name)
);

CREATE INDEX IF NOT EXISTS idx_code_quality_metrics_v2_repo_id ON code_quality_metrics_v2(repo_id);
CREATE INDEX IF NOT EXISTS idx_code_quality_metrics_v2_file_path ON code_quality_metrics_v2(file_path);
CREATE INDEX IF NOT EXISTS idx_code_quality_metrics_v2_metric_name ON code_quality_metrics_v2(metric_name);
CREATE INDEX IF NOT EXISTS idx_code_quality_metrics_v2_measured_at ON code_quality_metrics_v2(measured_at DESC);
CREATE INDEX IF NOT EXISTS idx_code_quality_thresholds_repo_id ON code_quality_thresholds(repo_id);
CREATE INDEX IF NOT EXISTS idx_code_quality_thresholds_metric_name ON code_quality_thresholds(metric_name);
CREATE INDEX IF NOT EXISTS idx_code_quality_thresholds_enabled ON code_quality_thresholds(enabled);