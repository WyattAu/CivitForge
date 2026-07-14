CREATE TABLE IF NOT EXISTS code_quality_metrics (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    repo_id UUID NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    metric_name TEXT NOT NULL,
    metric_value DOUBLE PRECISION NOT NULL,
    file_path TEXT,
    measured_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_code_quality_metrics_repo_id ON code_quality_metrics(repo_id);
CREATE INDEX idx_code_quality_metrics_name ON code_quality_metrics(metric_name);
CREATE INDEX idx_code_quality_metrics_measured_at ON code_quality_metrics(measured_at);
CREATE INDEX idx_code_quality_metrics_repo_name ON code_quality_metrics(repo_id, metric_name);
